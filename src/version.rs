//! This module defines the data structures for the version information of the
//! Nebula TOMPHTTP Server project.

// Import the serde crate to enable serialization of the structs
use serde::Serialize;

/// A struct that represents the version data of the project
#[derive(Serialize, Debug)]
pub struct VersionData {
    /// A vector of strings that contains the supported versions of the TOMPHTTP
    /// specification
    versions: Vec<String>,
    /// A string that indicates the programming language used for the project
    language: String,
    /// A MaintainerData struct that contains the information of the project
    /// maintainer
    maintainer: MaintainerData,
    /// A ProjectData struct that contains the general information of the
    /// project
    project: ProjectData,
}

// Implement the Default trait for VersionData to provide a default value
impl Default for VersionData {
    fn default() -> Self {
        Self {
            // Currently the project supports only version 2 of the [`TOMPHTTP`](https://github.com/tomphttp/specifications/blob/master/BareServerV2.md) specification
            versions: vec!["v2".into()],
            // The project is written in Rust
            language: "Rust".into(),
            // Use the default value for MaintainerData
            maintainer: MaintainerData::default(),
            // Use the default value for ProjectData
            project: ProjectData::default(),
        }
    }
}

/// A struct that represents the maintainer data of the project
#[derive(Serialize, Debug)]
pub struct MaintainerData {
    /// A string that contains the email address of the maintainer
    email: String,
    /// A string that contains the website of the maintainer
    website: String,
}

// Implement the Default trait for MaintainerData to provide a default value
impl Default for MaintainerData {
    fn default() -> Self {
        Self {
            email: "nebuladev@undefinedbhvr.com".into(),
            website: "https://github.com/NebulaServices".into(),
        }
    }
}

/// A struct that represents the project data of the project
#[derive(Serialize, Debug)]
pub struct ProjectData {
    /// A string that contains the name of the project
    name: String,
    /// A string that contains a brief description of the project
    description: String,
    /// A string that contains the email address of the project contact
    email: String,
    /// A string that contains the website of the project
    website: String,
    /// A string that contains the repository URL of the project
    repository: String,
    /// A string that contains the current version of the project
    version: String,
}

// Implement the Default trait for ProjectData to provide a default value
impl Default for ProjectData {
    fn default() -> Self {
        Self {
            name: "Nebula TOMPHTTP Server".into(),
            description: "Clean implementation of the TOMPHttp Specification in Rust.".into(),
            email: "".into(),
            website: "https://github.com/NebulaServices/bare-server-rust".into(),
            repository: "https://github.com/NebulaServices/bare-server-rust".into(),
            version: "1.0.0".into(),
        }
    }
}
