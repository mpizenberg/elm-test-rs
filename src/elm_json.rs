//! Module dealing with the elm.json file.
//! It handles serialization and deserialization with miniserde
//! instead of serde to avoid all the dependencies and compile time.

use miniserde::{json, Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Debug)]
/// Project configuration in an elm.json.
/// It either is a package or an application.
/// Both have different sets of fields.
pub enum Config {
    Package(PackageConfig),
    Application(ApplicationConfig),
}

#[derive(Serialize, Deserialize, Debug)]
/// Struct representing a package elm.json.
pub struct PackageConfig {
    #[serde(rename = "type")]
    pub type_: String,
    pub name: String,
    pub summary: String,
    pub license: String,
    pub version: String,
    #[serde(rename = "exposed-modules")]
    pub exposed_modules: Vec<String>,
    #[serde(rename = "elm-version")]
    pub elm_version: String,
    pub dependencies: HashMap<String, String>,
    #[serde(rename = "test-dependencies")]
    pub test_dependencies: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
/// Struct representing an application elm.json.
pub struct ApplicationConfig {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "source-directories")]
    pub source_directories: Vec<String>,
    #[serde(rename = "elm-version")]
    pub elm_version: String,
    pub dependencies: Dependencies,
    #[serde(rename = "test-dependencies")]
    pub test_dependencies: Dependencies,
}

#[derive(Serialize, Deserialize, Debug)]
/// Application dependencies have both direct and indirect dependencies.
pub struct Dependencies {
    pub direct: HashMap<String, String>,
    pub indirect: HashMap<String, String>,
}

/// Convert a string (read from file) into a Config with miniserde.
impl TryFrom<&str> for Config {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let project_type: ProjectType =
            json::from_str(value).map_err(|_| "Field type is missing".to_string())?;
        match project_type.type_.as_ref() {
            "package" => json::from_str(value)
                .map(|package| Config::Package(package))
                .map_err(|_| "Invalid elm.json for a package".into()),
            "application" => json::from_str(value)
                .map(|app| Config::Application(app))
                .map_err(|_| "Invalid elm.json for an application".into()),
            type_ => Err(format!("Invalid type: {}", type_)),
        }
    }
}

#[derive(Deserialize, Debug)]
/// Temporary struct just to determine if an elm.json
/// is for a package or an application since miniserde
/// does not handles enums.
struct ProjectType {
    #[serde(rename = "type")]
    pub type_: String,
}

/// Convert a package elm.json into an application elm.json
/// This is useful to generate the tests elm.json starting from
/// the elm.json of the project tested if it is a package.
/// Basically takes the lower bounds of ranges.
impl TryFrom<&PackageConfig> for ApplicationConfig {
    type Error = String;
    fn try_from(package: &PackageConfig) -> Result<Self, Self::Error> {
        // Helper closure to map on the HashMap
        let to_exact =
            |(name, range): (&String, _)| Ok((name.to_owned(), to_exact_version(range)?));
        // Convert package dependencies into app direct dependencies
        let direct_dependencies: Result<HashMap<String, String>, Self::Error> =
            package.dependencies.iter().map(to_exact).collect();
        let direct_test_dependencies: Result<HashMap<String, String>, Self::Error> =
            package.test_dependencies.iter().map(to_exact).collect();
        // Return converted config
        Ok(ApplicationConfig {
            type_: "application".into(),
            source_directories: vec!["src".into()],
            elm_version: "0.19.1".into(),
            dependencies: Dependencies {
                direct: direct_dependencies?,
                indirect: HashMap::new(),
            },
            test_dependencies: Dependencies {
                direct: direct_test_dependencies?,
                indirect: HashMap::new(),
            },
        })
    }
}

/// Take the lower bound of a version range.
fn to_exact_version<T: AsRef<str>>(range: T) -> Result<String, String> {
    let low_bound = range.as_ref().split('<').next();
    low_bound
        .map(|s| s.trim().to_string())
        .ok_or(format!("Invalid package version range: {}", range.as_ref()))
}

/// Move test dependencies into normal dependencies.
/// This is applied to convert the tested project elm.json into the elm.json
/// used to compile all tests.
impl ApplicationConfig {
    pub fn promote_test_dependencies(&mut self) {
        let direct_test = self.test_dependencies.direct.clone();
        let indirect_test = self.test_dependencies.indirect.clone();
        self.dependencies.direct.extend(direct_test);
        self.dependencies.indirect.extend(indirect_test);
        self.test_dependencies.direct = HashMap::new();
        self.test_dependencies.indirect = HashMap::new();
    }
}
