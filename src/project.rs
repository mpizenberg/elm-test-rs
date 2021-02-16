use anyhow::Context;
use notify::watcher;
use notify::RecursiveMode;
use notify::Watcher;
use pubgrub_dependency_provider_elm::project_config::ProjectConfig;
use std::collections::BTreeSet;
use std::iter;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Debug)]
pub struct Project {
    pub config: ProjectConfig,
    pub src_and_test_dirs: BTreeSet<PathBuf>,
    pub elm_project_root: PathBuf,
}

impl Project {
    pub fn from_dir(elm_project_root: PathBuf) -> anyhow::Result<Project> {
        // Read project elm.json
        let elm_json_str = std::fs::read_to_string(elm_project_root.join("elm.json"))
            .context("Unable to read elm.json")?;
        let config: ProjectConfig =
            serde_json::from_str(&elm_json_str).context("Invalid elm.json")?;

        let source_directories: BTreeSet<PathBuf> = match &config {
            ProjectConfig::Application(app_config) => app_config
                .source_directories
                .iter()
                .map(PathBuf::from)
                .collect(),
            ProjectConfig::Package(_) => iter::once("src".into()).collect(),
        };

        // Get absolute paths for test directories
        let mut source_or_test_directories: BTreeSet<PathBuf> = source_directories
            .iter()
            .map(|path| elm_project_root.join(path).canonicalize())
            .collect::<Result<_, _>>()?;
        // Add tests/ to the list of source directories
        if let Ok(path) = elm_project_root.join("tests").canonicalize() {
            source_or_test_directories.insert(path);
        }
        Ok(Project {
            config,
            src_and_test_dirs: source_or_test_directories,
            elm_project_root,
        })
    }

    pub fn watch(&mut self, f: impl Fn(&Self) -> anyhow::Result<()>) -> anyhow::Result<()> {
        dbg!(&self);
        // Create a channel to receive the events.
        let (tx, rx) = channel();
        // Create a watcher object, delivering debounced events.
        let mut watcher = watcher(tx, Duration::from_secs(1)).context("Failed to start watcher")?;
        let recursive = RecursiveMode::Recursive;

        let elm_json_path = self.elm_project_root.join("elm.json");

        // Watch the elm.json and the content of source directories.
        watcher
            .watch(&elm_json_path, recursive)
            .context(format!("Failed to watch {}", elm_json_path.display()))?;
        for path in &self.src_and_test_dirs {
            watcher
                .watch(path, recursive)
                .context(format!("Failed to watch {}", path.display()))?;
        }

        f(self).context("Initial run in watch mode")?;
        loop {
            match rx.recv().context("Error watching files")? {
                notify::DebouncedEvent::NoticeWrite(_) => {}
                notify::DebouncedEvent::NoticeRemove(_) => {}
                _event => {
                    // drain event queue
                    for _ in rx.try_iter() {}

                    // watch action
                    let new_project = Project::from_dir(self.elm_project_root.to_path_buf())?;
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
                        *self = new_project;
                    }
                    f(&self).context("Subsequent run in watch mode")?;
                }
            }
        }
    }
}
