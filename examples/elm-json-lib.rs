/// usage: cargo run --example elm-json-lib
pub fn main() {
    let elm_json_path = "examples/elm-json/elm.json";
    match read_elm_json(elm_json_path).as_str() {
        "Application" => solve::application(),
        "Package" => solve::package(),
        _ => println!("to be removed"),
    }
}

/// CF elm_json::util::read_elm_json(&matches)
fn read_elm_json(_path: &str) -> String {
    "Application".into()
}

/// CF elm_json::solve module
mod solve {
    pub fn package() {
        println!("solving package elm.json");
    }

    /// CF elm_json::solve::solve_application(&matches, &logger, info)
    pub fn application() {
        println!("solving application elm.json");

        let deps = "&info.dependencies(&semver::Strictness::Exact)";
        let elm_version = "info.elm_version()";

        // let mut retriever: Retriever =
        //     Retriever::new(&logger, &elm_version.into()).context(ErrorKind::Unknown)?;
        // let extras = util::add_extra_deps(&matches, &mut retriever);

        let extra_deps: Vec<String> = vec![
            "elm/core".into(),
            "elm/json".into(),
            "elm/time".into(),
            "elm/random".into(),
        ];

        // CF elm_json::package::retriever::Retriever::add_dep
        let mut retriever = "Retriever::new(...)";
        retriever = "retriever.add_deps(&extra_deps)";

        // retriever.add_preferred_versions(
        //     info.dependencies
        //         .indirect
        //         .iter()
        //         .filter(|&(k, _)| !extras.contains(&k.clone()))
        //         .map(|(k, v)| (k.clone().into(), *v)),
        // );
        //
        // retriever.add_deps(deps.iter().filter(|(k, _)| !extras.contains(k)));
        //
        // if matches.is_present("test") {
        //     retriever.add_deps(
        //         info.test_dependencies(&semver::Strictness::Exact)
        //             .iter()
        //             .filter(|(k, _)| !extras.contains(k)),
        //     );
        //
        //     retriever.add_preferred_versions(
        //         info.test_dependencies
        //             .indirect
        //             .iter()
        //             .filter(|&(k, _)| !extras.contains(&k.clone()))
        //             .map(|(k, v)| (k.clone().into(), *v)),
        //     )
        // }
        //
        // Resolver::new(&logger, &mut retriever)
        //     .solve()
        //     .context(ErrorKind::NoResolution)
        //     .and_then(|x| serde_json::to_string(&AppDependencies::from(x)).context(ErrorKind::Unknown))
        //     .map(|v| println!("{}", v))?;

        let resolver = "Resolver::new(retriever)";
        // resolver.solve().to_json_string().println()
    }
}
