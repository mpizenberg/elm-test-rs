use dirs;
use pubgrub::error::PubGrubError;
use pubgrub::range::Range;
use pubgrub::report::{DefaultStringReporter, Reporter};
use pubgrub::solver::resolve;
use pubgrub::type_aliases::Map;
use pubgrub::version::SemanticVersion as SemVer;
use std::path::{Path, PathBuf};
use std::{collections::BTreeMap, error::Error};
use ureq;

use pubgrub_dependency_provider_elm::dependency_provider::{
    ElmPackageProviderOffline, ProjectAdapter,
};
use pubgrub_dependency_provider_elm::project_config::{
    AppDependencies, ApplicationConfig, ProjectConfig,
};

pub fn solve<P: AsRef<Path>>(
    config: &ProjectConfig,
    src_dirs: &[P],
) -> Result<ApplicationConfig, Box<dyn Error>> {
    match config {
        ProjectConfig::Application(app_config) => {
            // Extract direct dependencies.
            let mut direct_deps = app_config.dependencies.direct.clone();
            direct_deps.extend(app_config.test_dependencies.direct.clone());
            let deps: Map<String, Range<SemVer>> = direct_deps
                .iter()
                .map(|(p, v)| (p.clone(), Range::exact(v.clone())))
                .collect();
            solve_helper(src_dirs, &"root".to_string(), deps)
        }
        ProjectConfig::Package(pkg_config) => {
            // Extract dependencies.
            let mut normal_deps = pkg_config.dependencies.clone();
            normal_deps.extend(pkg_config.test_dependencies.clone());
            let deps: Map<String, Range<SemVer>> =
                normal_deps.into_iter().map(|(p, c)| (p, c.0)).collect();
            solve_helper(src_dirs, &pkg_config.name, deps)
        }
    }
}

fn solve_helper<P: AsRef<Path>>(
    src_dirs: &[P],
    pkg_id: &String,
    deps: Map<String, Range<SemVer>>,
) -> Result<ApplicationConfig, Box<dyn Error>> {
    // TODO: there might be an issue if that was already in the dependencies.
    let mut deps = deps;
    deps.insert(
        "mpizenberg/elm-placeholder-pkg".to_string(),
        Range::exact(SemVer::one()),
    );
    let version = SemVer::new(0, 0, 0);
    let offline_provider = ElmPackageProviderOffline::new(elm_home(), "0.19.1");
    let deps_provider =
        ProjectAdapter::new(pkg_id.clone(), version.clone(), deps, &offline_provider);

    // Solve dependencies.
    let mut solution = match resolve(&deps_provider, pkg_id.clone(), version) {
        Ok(sol) => sol,
        Err(PubGrubError::NoSolution(tree)) => {
            return Err(DefaultStringReporter::report(&tree).into())
        }
        Err(err) => return Err(err.into()),
    };
    solution.remove(pkg_id);

    // TODO: Split solution into direct and indirect deps.
    let dependencies = AppDependencies {
        direct: solution.into_iter().collect(),
        indirect: BTreeMap::new(),
    };
    let test_dependencies = AppDependencies {
        direct: BTreeMap::new(),
        indirect: BTreeMap::new(),
    };
    let source_directories: Vec<String> = src_dirs
        .iter()
        .map(|p| p.as_ref().to_str().unwrap().to_string())
        .collect();
    Ok(ApplicationConfig {
        source_directories,
        // TODO: might have to change that
        elm_version: SemVer::new(0, 19, 1),
        dependencies,
        test_dependencies,
    })
}

fn elm_home() -> PathBuf {
    match std::env::var_os("ELM_HOME") {
        None => default_elm_home(),
        Some(os_string) => os_string.into(),
    }
}

#[cfg(target_family = "unix")]
fn default_elm_home() -> PathBuf {
    dirs::home_dir()
        .expect("Unknown home directory")
        .join(".elm")
}

#[cfg(target_family = "windows")]
fn default_elm_home() -> PathBuf {
    dirs::data_dir()
        .expect("Unknown data directory")
        .join("elm")
}

fn http_fetch(url: &str) -> Result<String, Box<dyn Error>> {
    ureq::get(url)
        .timeout_connect(10_000)
        .call()
        .into_string()
        .map_err(|e| e.into())
}
