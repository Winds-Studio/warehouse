use crate::game::{Build, GameLoader, Version};
use crate::games::common::HttpClient;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum VersionType {
    #[serde(rename = "release")]
    Release,
    #[serde(rename = "snapshot")]
    Snapshot,
    #[serde(rename = "old_beta")]
    OldBeta,
    #[serde(rename = "old_alpha")]
    OldAlpha,
}

impl std::fmt::Display for VersionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Release => write!(f, "release"),
            Self::Snapshot => write!(f, "snapshot"),
            Self::OldBeta => write!(f, "old_beta"),
            Self::OldAlpha => write!(f, "old_alpha"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct VanillaLoader {
    client: HttpClient,
}

#[derive(Deserialize)]
struct VersionManifest {
    versions: Vec<VersionManifestEntry>,
}

#[derive(Deserialize)]
struct VersionManifestEntry {
    id: String,
    #[serde(rename = "type")]
    version_type: String,
    url: String,
}

#[derive(Deserialize)]
struct VersionMetadata {
    downloads: VersionDownloads,
}

#[derive(Deserialize)]
struct VersionDownloads {
    server: DownloadInfo,
}

#[derive(Deserialize)]
struct DownloadInfo {
    url: String,
}

impl VanillaLoader {
    fn parse_version_type(type_str: &str) -> VersionType {
        match type_str {
            "release" => VersionType::Release,
            "snapshot" => VersionType::Snapshot,
            "old_beta" => VersionType::OldBeta,
            "old_alpha" => VersionType::OldAlpha,
            _ => VersionType::Snapshot,
        }
    }
}

#[async_trait]
impl GameLoader for VanillaLoader {
    fn name(&self) -> &str {
        "vanilla"
    }

    fn website(&self) -> Option<&str> {
        Some("https://www.minecraft.net")
    }

    fn supports_version_type(&self, _version_type: &str) -> bool {
        true
    }

    async fn fetch_versions(&self) -> anyhow::Result<Vec<Version>> {
        let manifest: VersionManifest = self
            .client
            .get_json("https://launchermeta.mojang.com/mc/game/version_manifest_v2.json")
            .await?;

        Ok(manifest
            .versions
            .into_iter()
            .map(|entry| {
                let version_type = Self::parse_version_type(&entry.version_type);
                Version::new(
                    entry.id,
                    version_type.to_string(),
                    version_type == VersionType::Release,
                )
            })
            .collect())
    }

    async fn fetch_builds(&self, version: &Version) -> anyhow::Result<Vec<Build>> {
        let manifest: VersionManifest = self
            .client
            .get("https://launchermeta.mojang.com/mc/game/version_manifest_v2.json")
            .send()
            .await?
            .json()
            .await?;

        let version_entry = manifest
            .versions
            .into_iter()
            .find(|v| v.id == version.id())
            .ok_or_else(|| anyhow::anyhow!("Version not found"))?;

        let metadata: VersionMetadata = self
            .client
            .get(version_entry.url)
            .send()
            .await?
            .json()
            .await?;

        Ok(vec![Build::new(
            version_entry.id,
            version.clone(),
            Some(metadata.downloads.server.url),
        )])
    }
}
