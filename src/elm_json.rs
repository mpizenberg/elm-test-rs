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
