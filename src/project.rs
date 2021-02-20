use anyhow::Context;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
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
    pub fn from_dir<P: AsRef<Path>>(root_directory: P) -> anyhow::Result<Project> {
        let root_directory = root_directory.as_ref();

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
            .map(|src| root_directory.join(src).canonicalize())
            .collect::<Result<_, _>>()
            .context("It seems source directories do not all exist")?;

        // Add tests/ to the list of source directories if it exists.
        if let Ok(path) = root_directory.join("tests").canonicalize() {
            src_and_test_dirs.insert(path);
        }

        Ok(Project {
            config,
            src_and_test_dirs,
            root_directory: root_directory.into(),
        })
    }

    pub fn watch(&mut self, call_back: impl Fn(&Self) -> anyhow::Result<()>) -> anyhow::Result<()> {
        // Create a channel to receive the events.
        let (tx, rx) = channel();
        // Create a watcher object, delivering debounced events.
        // See discussion here for the debouncing duration.
        // https://users.rust-lang.org/t/how-to-make-good-usage-of-the-notify-crate-for-responsive-events/55891
        let mut watcher =
            watcher(tx, Duration::from_millis(100)).context("Failed to start watcher")?;
        let recursive = RecursiveMode::Recursive;

        // Watch the elm.json and the content of source directories.
        let elm_json_path = self.root_directory.join("elm.json");
        watcher
            .watch(&elm_json_path, recursive)
            .context(format!("Failed to watch {}", elm_json_path.display()))?;
        for path in &self.src_and_test_dirs {
            watcher
                .watch(path, recursive)
                .context(format!("Failed to watch {}", path.display()))?;
        }

        // Call the function to execute passed as argument.
        call_back(self).context("Initial run in watch mode")?;

        // Enter the watch loop.
        loop {
            match rx.recv().context("Error watching files")? {
                DebouncedEvent::NoticeWrite(_) => {}
                DebouncedEvent::NoticeRemove(_) => {}
                event => {
                    log::debug!("{:?}", event);
                    // Get the path of the file that triggered the event.
                    let path = match &event {
                        DebouncedEvent::Create(p) => Some(p.as_path()),
                        DebouncedEvent::Write(p) => Some(p.as_path()),
                        DebouncedEvent::Chmod(p) => Some(p.as_path()),
                        DebouncedEvent::Remove(p) => Some(p.as_path()),
                        DebouncedEvent::Rename(_p1, p2) => Some(p2.as_path()), // TODO: Improve that
                        _ => None,
                    };

                    // We only process that event if it is of interest to us,
                    // meaning the path is and elm file or elm.json or a directory.
                    // If there was no path associated to the event we also might have
                    // to process it so we that's what we do.
                    let is_of_interest = |p: &Path| {
                        p.extension() == Some(&OsStr::new("elm")) // this is an elm file
                            || p.ends_with("elm.json") // elm.json changed
                            || p.is_dir() // a directory changed
                    };
                    match path {
                        Some(p) if !is_of_interest(p) => continue,
                        _ => (),
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
                            watcher
                                .unwatch(path)
                                .context(format!("Failed to unwatch {}", path.display()))?;
                        }
                        for path in new_src_dirs.difference(old_src_dirs) {
                            watcher
                                .watch(path, recursive)
                                .context(format!("Failed to watch {}", path.display()))?;
                        }
                    }

                    // Update the current project since dependencies or source directories may have change.
                    *self = new_project;

                    // Call the function to execute passed as argument.
                    call_back(&self).context("Subsequent run in watch mode")?;
                }
            }
        }
    }
}
