use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
  pub packages: HashMap<String, Package>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
  pub metadata: PackageMetadata,
  pub versions: HashMap<String, PackageVersion>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageMetadata {
  pub license: Option<String>,
  #[serde(rename = "sourceCode")]
  pub repo: Option<String>,
  #[serde(rename = "authorName", default = "author_def")]
  pub author: String,
  pub name: HashMap<String, String>,
  pub summary: Option<HashMap<String, String>>,
  pub description: Option<HashMap<String, String>>,
  pub icon: Option<HashMap<String, Icon>>
}

fn author_def() -> String {
  "Unknown".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageVersion {
  pub file: PackageFile,
  pub manifest: Manifest,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
  #[serde(rename = "versionCode")]
  pub version: u64,
  #[serde(rename = "usesSdk")]
  pub info: Option<ManifestInfo>,
  #[serde(rename = "releaseChannels")]
  pub channels: Option<Vec<String>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestInfo {
  #[serde(rename = "minSdkVersion")]
  pub min: Option<u16>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageFile {
  pub name: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icon {
  pub name: Option<String>,
}