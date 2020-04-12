use miniserde::{json, Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Debug)]
pub enum Config {
    Package(PackageConfig),
    Application(ApplicationConfig),
}

#[derive(Serialize, Deserialize, Debug)]
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
pub struct Dependencies {
    pub direct: HashMap<String, String>,
    pub indirect: HashMap<String, String>,
}

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
struct ProjectType {
    #[serde(rename = "type")]
    pub type_: String,
}

// Convert a package elm.json into an application elm.json
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

fn to_exact_version<T: AsRef<str>>(range: T) -> Result<String, String> {
    let low_bound = range.as_ref().split('<').next();
    low_bound
        .map(|s| s.trim().to_string())
        .ok_or(format!("Invalid package version range: {}", range.as_ref()))
}

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
