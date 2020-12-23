use pubgrub::error::PubGrubError;
use pubgrub::range::Range;
use pubgrub::report::{DefaultStringReporter, Reporter};
use pubgrub::solver::resolve;
use pubgrub::type_aliases::Map;
use pubgrub::version::SemanticVersion as SemVer;
use std::path::Path;
use std::str::FromStr;
use std::{collections::BTreeMap, error::Error};

use pubgrub_dependency_provider_elm::constraint::Constraint;
use pubgrub_dependency_provider_elm::dependency_provider::{
    ElmPackageProviderOffline, ElmPackageProviderOnline, ProjectAdapter, VersionStrategy,
};
use pubgrub_dependency_provider_elm::project_config::{
    AppDependencies, ApplicationConfig, PackageConfig, ProjectConfig,
};

#[derive(Debug)]
pub enum ConnectivityStrategy {
    Progressive,
    Offline,
    Online(VersionStrategy),
}

impl FromStr for ConnectivityStrategy {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "progressive" => Ok(Self::Progressive),
            "offline" => Ok(Self::Offline),
            "online-newest" => Ok(Self::Online(VersionStrategy::Newest)),
            "online-oldest" => Ok(Self::Online(VersionStrategy::Oldest)),
            _ => Err(format!("Invalid connectivity option: {}", s)),
        }
    }
}

/// Install elm-explorations/test to the tests dependencies.
pub fn init(config: ProjectConfig) -> Result<ProjectConfig, Box<dyn Error>> {
    match config {
        ProjectConfig::Application(app_config) => {
            Ok(ProjectConfig::Application(init_app(app_config)?))
        }
        ProjectConfig::Package(pkg_config) => Ok(ProjectConfig::Package(init_pkg(pkg_config)?)),
    }
}

fn init_app(mut app_config: ApplicationConfig) -> Result<ApplicationConfig, Box<dyn Error>> {
    // Retrieve all direct and indirect dependencies
    let indirect_test_deps = app_config.test_dependencies.indirect.iter();
    let mut all_deps: Map<String, Range<SemVer>> = indirect_test_deps
        .chain(app_config.dependencies.indirect.iter())
        .chain(app_config.test_dependencies.direct.iter())
        .chain(app_config.dependencies.direct.iter())
        .map(|(p, v)| (p.clone(), Range::exact(*v)))
        .collect();

    // Check that those dependencies are correct
    solve_check(&all_deps, true)?;

    // Check if elm-explorations/test is already in the dependencies.
    let test_pkg = "elm-explorations/test".to_string();
    if all_deps.contains_key(&test_pkg) {
        if app_config
            .test_dependencies
            .indirect
            .contains_key(&test_pkg)
        {
            eprintln!("elm-explorations/test is already in your indirect test dependencies,");
            eprintln!("so we just upgrade it to a direct test dependency.");
            let v = app_config
                .test_dependencies
                .indirect
                .remove(&test_pkg)
                .unwrap();
            app_config.test_dependencies.direct.insert(test_pkg, v);
        } else {
            eprintln!("elm-explorations/test is already in your dependencies.");
        }
        return Ok(app_config);
    }

    // Add elm-explorations/test to the dependencies
    all_deps.insert(test_pkg.clone(), Range::between((1, 0, 0), (2, 0, 0)));

    // Solve dependencies
    let solution = solve_deps(
        &ConnectivityStrategy::Progressive,
        &all_deps,
        "root".to_string(),
        SemVer::zero(),
    )?;

    // Add the selected elm-explorations/test version to direct tests deps
    let test_version = solution.get(&test_pkg).unwrap();
    app_config
        .test_dependencies
        .direct
        .insert(test_pkg, *test_version);

    // Add all other new deps to indirect tests deps
    for (p, v) in solution.into_iter() {
        if !all_deps.contains_key(&p) && &p != "root" {
            app_config.test_dependencies.indirect.insert(p, v);
        }
    }
    Ok(app_config)
}

fn init_pkg(mut pkg_config: PackageConfig) -> Result<PackageConfig, Box<dyn Error>> {
    // Retrieve all dependencies
    let test_deps = pkg_config.test_dependencies.iter();
    let mut all_deps: Map<String, Range<SemVer>> = test_deps
        .chain(pkg_config.dependencies.iter())
        .map(|(p, c)| (p.clone(), c.0.clone()))
        .collect();

    // Check that those dependencies are correct
    solve_check(&all_deps, false)?;

    // Check if elm-explorations/test is already in the dependencies.
    let test_pkg = "elm-explorations/test".to_string();
    if all_deps.contains_key(&test_pkg) {
        eprintln!("elm-explorations/test is already in your dependencies.");
        return Ok(pkg_config);
    }

    // Add elm-explorations/test to the dependencies
    let test_range = Range::between((1, 0, 0), (2, 0, 0));
    all_deps.insert(test_pkg.clone(), test_range.clone());

    // Solve dependencies to check that elm-explorations/test is compatible
    solve_deps(
        &ConnectivityStrategy::Progressive,
        &all_deps,
        pkg_config.name.clone(),
        SemVer::zero(),
    )?;

    // Add elm-explorations/test to tests deps
    pkg_config
        .test_dependencies
        .insert(test_pkg, Constraint(test_range));
    Ok(pkg_config)
}

/// Solve dependencies needed to run the tests.
pub fn solve<P: AsRef<Path>>(
    connectivity: &ConnectivityStrategy,
    config: &ProjectConfig,
    src_dirs: &[P],
) -> Result<ApplicationConfig, Box<dyn Error>> {
    match config {
        ProjectConfig::Application(app_config) => {
            let normal_deps = app_config.dependencies.direct.iter();
            let deps: Map<String, Range<SemVer>> = normal_deps
                .chain(app_config.test_dependencies.direct.iter())
                .map(|(p, v)| (p.clone(), Range::exact(*v)))
                .collect();
            solve_helper(
                connectivity,
                src_dirs,
                &"root".to_string(),
                SemVer::zero(),
                deps,
            )
        }
        ProjectConfig::Package(pkg_config) => {
            let normal_deps = pkg_config.dependencies.iter();
            let deps: Map<String, Range<SemVer>> = normal_deps
                .chain(pkg_config.test_dependencies.iter())
                .map(|(p, c)| (p.clone(), c.0.clone()))
                .collect();
            solve_helper(
                connectivity,
                src_dirs,
                &pkg_config.name,
                pkg_config.version,
                deps,
            )
        }
    }
}

#[allow(clippy::ptr_arg)]
fn solve_helper<P: AsRef<Path>>(
    connectivity: &ConnectivityStrategy,
    src_dirs: &[P],
    pkg_id: &String,
    version: SemVer,
    deps: Map<String, Range<SemVer>>,
) -> Result<ApplicationConfig, Box<dyn Error>> {
    // TODO: there might be an issue if that was already in the dependencies.
    let mut deps = deps;
    deps.insert(
        "mpizenberg/elm-test-runner".to_string(),
        Range::exact((3, 0, 0)),
    );
    let mut solution = solve_deps(connectivity, &deps, pkg_id.clone(), version)?;
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

/// Check that those dependencies are correct.
/// Use progressive connectivity mode.
fn solve_check(deps: &Map<String, Range<SemVer>>, is_app: bool) -> Result<(), Box<dyn Error>> {
    let pkg_id = "root".to_string();
    let version = SemVer::zero();
    let mut solution = solve_deps(
        &ConnectivityStrategy::Progressive,
        deps,
        pkg_id.clone(),
        version,
    )?;
    // Check that indirect deps are correct if this is for an application.
    // All packages in the solution must exist in the original dependencies.
    if is_app {
        solution.remove(&pkg_id);
        for p in solution.keys() {
            if !deps.contains_key(p) {
                return Err(format!("{} is missing in the indirect dependencies", p).into());
            }
        }
    }
    Ok(())
}

/// Solve project dependencies.
fn solve_deps(
    connectivity: &ConnectivityStrategy,
    deps: &Map<String, Range<SemVer>>,
    pkg_id: String,
    version: SemVer,
) -> Result<Map<String, SemVer>, Box<dyn Error>> {
    let solution = |resolution| match resolution {
        Ok(sol) => Ok(sol),
        Err(PubGrubError::NoSolution(tree)) => Err(DefaultStringReporter::report(&tree).into()),
        Err(err) => Err(err.into()),
    };
    match connectivity {
        ConnectivityStrategy::Offline => {
            let offline_provider =
                ElmPackageProviderOffline::new(crate::utils::elm_home(), "0.19.1");
            let deps_provider =
                ProjectAdapter::new(pkg_id.clone(), version, deps, &offline_provider);
            solution(resolve(&deps_provider, pkg_id, version))
        }
        ConnectivityStrategy::Online(strategy) => {
            let online_provider = ElmPackageProviderOnline::new(
                crate::utils::elm_home(),
                "0.19.1",
                "https://package.elm-lang.org",
                crate::utils::http_fetch,
                strategy.clone(),
            )
            .unwrap();
            let deps_provider =
                ProjectAdapter::new(pkg_id.clone(), version, deps, &online_provider);
            solution(resolve(&deps_provider, pkg_id, version))
        }
        ConnectivityStrategy::Progressive => solve_deps(
            &ConnectivityStrategy::Offline,
            deps,
            pkg_id.clone(),
            version,
        )
        .or_else(|_| {
            solve_deps(
                &ConnectivityStrategy::Online(VersionStrategy::Newest),
                deps,
                pkg_id,
                version,
            )
        }),
    }
}
