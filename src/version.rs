use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct VersionData {
    versions: Vec<String>,
    language: String,
    maintainer: MaintainerData,
    project: ProjectData,
}

impl Default for VersionData {
    fn default() -> Self {
        Self {
            versions: vec!["v2".into()],
            language: "Rust".into(),
            maintainer: MaintainerData::default(),
            project: ProjectData::default(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct MaintainerData {
    email: String,
    website: String,
}

impl Default for MaintainerData {
    fn default() -> Self {
        Self {
            email: "nebuladev@undefinedbhvr.com".into(),
            website: "nebulaproxy.io".into(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct ProjectData {
    name: String,
    description: String,
    email: String,
    website: String,
    repository: String,
    version: String,
}

impl Default for ProjectData {
    fn default() -> Self {
        Self {
            name: "Nebula TOMPHTTP Server".into(),
            description: "Clean implementation of the TOMPHttp Specification in Rust.".into(),
            email: "".into(),
            website: "nebulaproxy.io".into(),
            repository: "".into(),
            version: "1.0.0".into(),
        }
    }
}
