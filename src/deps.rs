use pubgrub::error::PubGrubError;
use pubgrub::range::Range;
use pubgrub::report::{DefaultStringReporter, Reporter};
use pubgrub::solver::resolve;
use pubgrub::type_aliases::Map;
use pubgrub::version::SemanticVersion as SemVer;
use std::path::Path;
use std::{collections::BTreeMap, error::Error};

use pubgrub_dependency_provider_elm::dependency_provider::{
    ElmPackageProviderOffline, ElmPackageProviderOnline, ProjectAdapter, VersionStrategy,
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
            let normal_deps = app_config.dependencies.direct.iter();
            let deps: Map<String, Range<SemVer>> = normal_deps
                .chain(app_config.test_dependencies.direct.iter())
                .map(|(p, v)| (p.clone(), Range::exact(v.clone())))
                .collect();
            solve_helper(src_dirs, &"root".to_string(), deps)
        }
        ProjectConfig::Package(pkg_config) => {
            let normal_deps = pkg_config.dependencies.iter();
            let deps: Map<String, Range<SemVer>> = normal_deps
                .chain(pkg_config.test_dependencies.iter())
                .map(|(p, c)| (p.clone(), c.0.clone()))
                .collect();
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
    let offline_provider = ElmPackageProviderOffline::new(crate::utils::elm_home(), "0.19.1");
    let deps_provider = ProjectAdapter::new(pkg_id.clone(), version, &deps, &offline_provider);

    // Solve dependencies.
    let resolution = resolve(&deps_provider, pkg_id.clone(), version).or_else(|_| {
        eprintln!("Solving offline failed, switching to online");
        let online_provider = ElmPackageProviderOnline::new(
            crate::utils::elm_home(),
            "0.19.1",
            "https://package.elm-lang.org",
            crate::utils::http_fetch,
            VersionStrategy::Newest,
        )
        .unwrap();
        let deps_provider = ProjectAdapter::new(pkg_id.clone(), version, &deps, &online_provider);
        resolve(&deps_provider, pkg_id.clone(), version)
    });
    let mut solution = match resolution {
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
