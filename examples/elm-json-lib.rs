// Ok so I've been diving a little bit in elm-json code and there are both quite interesting and nasty things. Generally, it isn't as well documented as I hoped, reading the code was a bit harder than expected. Among the annoying things, elm-json has a lot of dependencies and the fact that it is a CLI-first tool permeates most of its API. For example, there is a logger struct passed to almost all functions, which is a bit annoying. The interface is also not well adapted to our case.
//
// On the bright side, I've found out that the heart of the tool, the version constraints solver is a quite recent algorithm. It originates from an algorithm called PubGrub, which was introduced as the version solver for the Dart language not so long ago. Here is the introductary blog post about it: https://medium.com/@nex3/pubgrub-2fb6470504f. It's an interesting read.
//
// The PubGrub rust implementation inside elm-json is actually a set of files extracted from elba (https://github.com/elba/elba), a package manager for the Idris language. The downside is that there too, it's extracted from a CLI tool and you can feel it. It's also not very well documented but you could understand it from the PubGrub doc I guess.
//
// I've looked a bit and didn't find another implementation of PubGrub in Rust, or it is well hidden. I think elm-json would not be usable as-is, and requires quite some work if we want to strip it from all the unwanted. On the other hand, writing an independent library implementation of PubGrub in rust would be quite a cool project I believe. So I think the way forward is probably to make a PubGrub solver crate. Then we'll be able to use it directly (as well as elm-json if Ilias wishes to).

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
