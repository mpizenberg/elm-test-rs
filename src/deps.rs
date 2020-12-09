use dirs;
use pubgrub::error::PubGrubError;
use pubgrub::range::Range;
use pubgrub::report::{DefaultStringReporter, Reporter};
use pubgrub::solver::{resolve, DependencyProvider};
use pubgrub::type_aliases::Map;
use pubgrub::version::SemanticVersion as SemVer;
use std::path::{Path, PathBuf};
use std::{collections::BTreeMap, error::Error};
use ureq;

use pubgrub_dependency_provider_elm::dependency_provider::{
    ElmPackageProviderOffline, ElmPackageProviderOnline, ProjectAdapter, VersionStrategy,
};
use pubgrub_dependency_provider_elm::pkg_version::PkgVersion;
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
            let mut deps: Map<String, Range<SemVer>> = direct_deps
                .iter()
                // Convert exact versions to sem-ver compatible ranges
                // TODO: try to find a solution avoiding doing that
                .map(|(p, v)| (p.clone(), Range::between(v.clone(), v.bump_major())))
                .collect();
            // TODO: there might be an issue if that was already in the dependencies.
            // TODO: maybe we should vendor all this instead.
            deps.insert(
                "mpizenberg/elm-placeholder-pkg".to_string(),
                Range::exact(SemVer::one()),
            );

            // Initialize a dependency provider.
            let pkg_id = "root".to_string();
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
            solution.remove(&pkg_id);

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
                elm_version: app_config.elm_version.clone(),
                dependencies,
                test_dependencies,
            })
        }
        ProjectConfig::Package(pkg_config) => {
            // Extract dependencies.
            let mut normal_deps = pkg_config.dependencies.clone();
            normal_deps.extend(pkg_config.test_dependencies.clone());
            let mut deps: Map<String, Range<SemVer>> =
                normal_deps.into_iter().map(|(p, c)| (p, c.0)).collect();
            // TODO: there might be an issue if that was already in the dependencies.
            deps.insert(
                "mpizenberg/elm-placeholder-pkg".to_string(),
                Range::exact(SemVer::one()),
            );

            // Initialize a dependency provider.
            let pkg_id = pkg_config.name.clone();
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
            solution.remove(&pkg_id);

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
    }
}

fn run(pkg_version: Option<PkgVersion>, offline: bool, online_strat: Option<VersionStrategy>) {
    match (offline, online_strat) {
        (true, _) => {
            eprintln!("Solving offline");
            let deps_provider = ElmPackageProviderOffline::new(elm_home(), "0.19.1");
            solve_deps(&deps_provider, &pkg_version);
        }
        (false, None) => {
            eprintln!("Solving offline");
            let deps_provider = ElmPackageProviderOffline::new(elm_home(), "0.19.1");
            if !solve_deps(&deps_provider, &pkg_version) {
                eprintln!("Offline solving failed, switching to online");
                run(pkg_version, false, Some(VersionStrategy::Newest));
            }
        }
        (false, Some(strat)) => {
            eprintln!("Solving online with strategy {:?}", &strat);
            let deps_provider = ElmPackageProviderOnline::new(
                elm_home(),
                "0.19.1",
                "https://package.elm-lang.org",
                http_fetch,
                strat,
            )
            .expect("Error initializing the online dependency provider");
            solve_deps(&deps_provider, &pkg_version);
            // Save the versions cache
            deps_provider.save_cache().unwrap();
        }
    };
}

fn solve_deps<DP: DependencyProvider<String, SemVer>>(
    deps_provider: &DP,
    pkg_version: &Option<PkgVersion>,
) -> bool {
    match pkg_version {
        // No package in CLI arguments so we solve deps of the elm project in the current directory
        None => {
            let version = SemVer::new(0, 0, 0);
            let elm_json_str = std::fs::read_to_string("elm.json")
                .expect("Are you in an elm project? there was an issue loading the elm.json");
            let project: ProjectConfig = serde_json::from_str(&elm_json_str).unwrap();
            match project {
                ProjectConfig::Application(app_config) => {
                    let pkg_id = "root".to_string();
                    let direct_deps: Map<String, Range<SemVer>> = app_config
                        .dependencies
                        .direct
                        .into_iter()
                        .map(|(p, v)| (p, Range::exact(v)))
                        .collect();
                    let deps_provider = ProjectAdapter::new(
                        pkg_id.clone(),
                        version.clone(),
                        direct_deps,
                        deps_provider,
                    );
                    resolve_helper(pkg_id, version, &deps_provider)
                }
                ProjectConfig::Package(pkg_config) => {
                    let pkg_id = pkg_config.name.clone();
                    let direct_deps: Map<String, Range<SemVer>> = pkg_config
                        .dependencies
                        .into_iter()
                        .map(|(p, c)| (p, c.0))
                        .collect();
                    let deps_provider = ProjectAdapter::new(
                        pkg_id.clone(),
                        version.clone(),
                        direct_deps,
                        deps_provider,
                    );
                    resolve_helper(pkg_id, version, &deps_provider)
                }
            }
        }
        // A published package was directly provided as CLI argument
        Some(pkg_v) => {
            let author = &pkg_v.author_pkg.author;
            let pkg = &pkg_v.author_pkg.pkg;
            let pkg_id = format!("{}/{}", author, pkg);
            resolve_helper(pkg_id, pkg_v.version, deps_provider)
        }
    }
}

fn resolve_helper<DP: DependencyProvider<String, SemVer>>(
    pkg_id: String,
    version: SemVer,
    deps_provider: &DP,
) -> bool {
    match resolve(deps_provider, pkg_id, version) {
        Ok(all_deps) => {
            let mut all_deps_formatted: Vec<_> = all_deps
                .iter()
                .map(|(p, v)| format!("{}@{}", p, v))
                .collect();
            all_deps_formatted.sort();
            eprintln!("{:#?}", all_deps_formatted);
            true
        }
        Err(err) => {
            eprintln!("{:?}", err);
            false
        }
    }
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
