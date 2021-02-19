use anyhow::Context;
use notify::{watcher, RecursiveMode, Watcher};
use pubgrub_dependency_provider_elm::project_config::ProjectConfig;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Debug)]
pub struct Project {
    pub config: ProjectConfig,
    pub src_and_test_dirs: BTreeSet<PathBuf>,
    pub elm_project_root: PathBuf,
}

impl Project {
    pub fn from_dir<P: AsRef<Path>>(elm_project_root: P) -> anyhow::Result<Project> {
        let elm_project_root = elm_project_root.as_ref();

        // Read project elm.json
        let elm_json_str = std::fs::read_to_string(elm_project_root.join("elm.json"))
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
            .map(|src| elm_project_root.join(src).canonicalize())
            .collect::<Result<_, _>>()
            .context("It seems source directories do not all exist")?;

        // Add tests/ to the list of source directories if it exists.
        if let Ok(path) = elm_project_root.join("tests").canonicalize() {
            src_and_test_dirs.insert(path);
        }

        Ok(Project {
            config,
            src_and_test_dirs,
            elm_project_root: elm_project_root.into(),
        })
    }

    pub fn watch(&mut self, f: impl Fn(&Self) -> anyhow::Result<()>) -> anyhow::Result<()> {
        dbg!(&self);
        // Create a channel to receive the events.
        let (tx, rx) = channel();
        // Create a watcher object, delivering debounced events.
        let mut watcher = watcher(tx, Duration::from_secs(1)).context("Failed to start watcher")?;
        let recursive = RecursiveMode::Recursive;

        // Watch the elm.json and the content of source directories.
        let elm_json_path = self.elm_project_root.join("elm.json");
        watcher
            .watch(&elm_json_path, recursive)
            .context(format!("Failed to watch {}", elm_json_path.display()))?;
        for path in &self.src_and_test_dirs {
            watcher
                .watch(path, recursive)
                .context(format!("Failed to watch {}", path.display()))?;
        }

        // Call the function to execute passed as argument.
        f(self).context("Initial run in watch mode")?;

        // Enter the watch loop.
        loop {
            match rx.recv().context("Error watching files")? {
                notify::DebouncedEvent::NoticeWrite(_) => {}
                notify::DebouncedEvent::NoticeRemove(_) => {}
                _event => {
                    // drain event queue
                    for _ in rx.try_iter() {}

                    // Load the potential updated elm.json.
                    let new_project = Project::from_dir(&self.elm_project_root)?;

                    // Update watched directories if they changed.
                    // TODO: Improve by computing the sets of new and removed directories.
                    if self.src_and_test_dirs != new_project.src_and_test_dirs {
                        dbg!(&new_project);
                        for path in &self.src_and_test_dirs {
                            watcher
                                .unwatch(path)
                                .context(format!("Failed to unwatch {}", path.display()))?;
                        }
                        for path in &new_project.src_and_test_dirs {
                            watcher
                                .watch(path, recursive)
                                .context(format!("Failed to watch {}", path.display()))?;
                        }
                    }

                    // Update the current project since dependencies or source directories may have change.
                    *self = new_project;

                    // Call the function to execute passed as argument.
                    f(&self).context("Subsequent run in watch mode")?;
                }
            }
        }
    }
}
