use anyhow::Context;
use notify_debouncer_mini::new_debouncer;
use notify_debouncer_mini::notify::RecursiveMode;
use pubgrub::version::SemanticVersion as SemVer;
use pubgrub_dependency_provider_elm::project_config::ProjectConfig;
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Debug)]
pub struct Project {
    pub config: ProjectConfig,
    pub src_and_test_dirs: BTreeSet<PathBuf>,
    pub root_directory: PathBuf,
}

impl Project {
    /// The Elm version to solve dependencies and generate the tests runner for.
    ///
    /// An application pins an exact version in its elm.json, so that is
    /// authoritative. A package only declares a version constraint, so we ask
    /// the compiler which version is actually installed.
    pub fn elm_version(config: &ProjectConfig, compiler: &str) -> anyhow::Result<SemVer> {
        match config {
            ProjectConfig::Application(app_config) => Ok(app_config.elm_version),
            ProjectConfig::Package(_) => crate::utils::elm_version_from_compiler(compiler),
        }
    }

    pub fn from_dir<P: AsRef<Path>>(root_directory: P) -> anyhow::Result<Project> {
        let root_directory = crate::utils::absolute_path(root_directory)?;

        // Read project elm.json
        let elm_json_str = std::fs::read_to_string(root_directory.join("elm.json"))
            .context("Unable to read elm.json")?;
        let config: ProjectConfig =
            serde_json::from_str(&elm_json_str).context("Invalid elm.json")?;

        // Retrieve source directories from the project config.
        let default_src_dir = ["src".to_string()];
        let src_dirs = match &config {
            ProjectConfig::Application(app) => app.source_directories.as_slice(),
            ProjectConfig::Package(_) => &default_src_dir,
        };

        // Transform source directories to absolute paths.
        let mut src_and_test_dirs: BTreeSet<PathBuf> = src_dirs
            .iter()
            .map(|src| crate::utils::absolute_path(root_directory.join(src)))
            .collect::<Result<_, _>>()
            .context("It seems source directories do not all exist")?;

        // Add tests/ to the list of source directories if it exists.
        let tests_dir = root_directory.join("tests");
        if tests_dir.exists() {
            src_and_test_dirs.insert(tests_dir);
        }

        Ok(Project {
            config,
            src_and_test_dirs,
            root_directory,
        })
    }

    pub fn watch(&mut self, call_back: impl Fn(&Self) -> anyhow::Result<()>) -> anyhow::Result<()> {
        // Create a channel to receive the events.
        let (tx, rx) = channel();
        // Create a debounced watcher, delivering batches of settled events.
        // See discussion here for the debouncing duration.
        // https://users.rust-lang.org/t/how-to-make-good-usage-of-the-notify-crate-for-responsive-events/55891
        let mut debouncer =
            new_debouncer(Duration::from_millis(100), tx).context("Failed to start watcher")?;
        let recursive = RecursiveMode::Recursive;

        // Watch the elm.json and the content of source directories.
        let elm_json_path = self.root_directory.join("elm.json");
        debouncer
            .watcher()
            .watch(&elm_json_path, recursive)
            .context(format!("Failed to watch {}", elm_json_path.display()))?;
        for path in &self.src_and_test_dirs {
            debouncer
                .watcher()
                .watch(path, recursive)
                .context(format!("Failed to watch {}", path.display()))?;
        }

        // Call the function to execute passed as argument.
        call_back(self).context("Initial run in watch mode")?;

        // We only process an event if it is of interest to us, meaning the path
        // is an elm file or elm.json or a directory.
        let is_of_interest = |p: &Path| {
            p.extension() == Some(OsStr::new("elm")) // this is an elm file
                || p.ends_with("elm.json") // elm.json changed
                || p.is_dir() // a directory changed
        };

        // Enter the watch loop.
        loop {
            // Each message is a debounced batch of events (or a set of errors).
            let events = match rx.recv().context("Error watching files")? {
                Ok(events) => events,
                Err(errors) => {
                    log::debug!("Watch errors: {errors:?}");
                    continue;
                }
            };
            log::debug!("{events:?}");

            // Find the first changed path that is of interest to us, if any.
            let changed_path = events
                .iter()
                .map(|event| event.path.as_path())
                .find(|p| is_of_interest(p));
            let changed_path = match changed_path {
                Some(p) => p.to_path_buf(),
                None => continue,
            };

            // drain event queue
            for _ in rx.try_iter() {}

            // Load the potential updated elm.json.
            let new_project = Project::from_dir(&self.root_directory)?;

            // Update watched directories if they changed.
            let old_src_dirs = &self.src_and_test_dirs;
            let new_src_dirs = &new_project.src_and_test_dirs;
            if old_src_dirs != new_src_dirs {
                for path in old_src_dirs.difference(new_src_dirs) {
                    debouncer
                        .watcher()
                        .unwatch(path)
                        .context(format!("Failed to unwatch {}", path.display()))?;
                }
                for path in new_src_dirs.difference(old_src_dirs) {
                    debouncer
                        .watcher()
                        .watch(path, recursive)
                        .context(format!("Failed to watch {}", path.display()))?;
                }
            }

            // Update the current project since dependencies or source directories may have change.
            *self = new_project;

            // Log to stderr that a change was detected.
            let relative_path = pathdiff::diff_paths(&changed_path, &self.root_directory).context(
                format!(
                    "Could not get path {} relative to path {}",
                    changed_path.display(),
                    self.root_directory.display()
                ),
            )?;
            let detection_msg = format!("Change detected in {}", relative_path.display());
            log::error!(
                "\n\n\n\n{}\n{}\n\n\n\n",
                detection_msg,
                "=".repeat(detection_msg.len())
            );

            // Call the function to execute passed as argument.
            call_back(self).context("Subsequent run in watch mode")?;
        }
    }
}
