use crate::cache::CacheManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Version {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    #[serde(default)]
    pub is_stable: bool,
}

impl Version {
    pub fn new(id: String, version_type: String, is_stable: bool) -> Self {
        Self {
            id,
            version_type,
            is_stable,
        }
    }

    pub fn new_standard(id: String, version_type: String) -> Self {
        Self {
            id,
            version_type: version_type.clone(),
            is_stable: version_type == "release",
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn version_type(&self) -> &str {
        &self.version_type
    }
    pub fn is_stable(&self) -> bool {
        self.is_stable
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Build {
    id: String,
    version: Version,
    download_url: Option<String>,
}

impl Build {
    pub fn new(id: String, version: Version, download_url: Option<String>) -> Self {
        Self {
            id,
            version,
            download_url,
        }
    }

    pub fn new_standard(id: String, version: Version, download_url: Option<String>) -> Self {
        Self {
            id,
            version,
            download_url,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn version(&self) -> &Version {
        &self.version
    }
    pub fn download_url(&self) -> Option<&str> {
        self.download_url.as_deref()
    }
    pub fn filename(&self) -> String {
        format!("{}-{}.jar", self.version.id, self.id)
    }
}

#[async_trait::async_trait]
pub trait GameLoader: Send + Sync + std::fmt::Debug + 'static {
    fn name(&self) -> &str;
    fn website(&self) -> Option<&str>;

    fn supports_version_type(&self, _version_type: &str) -> bool {
        true
    }

    async fn fetch_versions(&self) -> anyhow::Result<Vec<Version>>;
    async fn fetch_builds(&self, version: &Version) -> anyhow::Result<Vec<Build>>;

    async fn get_latest_stable(&self) -> anyhow::Result<Option<Version>> {
        let versions = self.fetch_versions().await?;
        Ok(versions.into_iter().find(|v| v.is_stable()))
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    id: String,
    loaders: HashMap<String, Arc<dyn GameLoader>>,
}

impl Game {
    pub fn new(id: String) -> Self {
        Self {
            id,
            loaders: HashMap::new(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn add_loader<L: GameLoader>(&mut self, loader: L) {
        self.loaders
            .insert(loader.name().to_string(), Arc::new(loader));
    }

    pub fn get_loader(&self, name: &str) -> Option<Arc<dyn GameLoader>> {
        self.loaders.get(name).cloned()
    }

    pub fn list_loaders(&self) -> Vec<Arc<dyn GameLoader>> {
        self.loaders.values().cloned().collect()
    }
}

#[derive(Clone, Debug)]
pub struct GameProvider {
    games: Arc<RwLock<HashMap<String, Game>>>,
    pub cache: Arc<CacheManager>,
}

impl GameProvider {
    pub fn from_settings(settings: &crate::config::Settings) -> Self {
        Self {
            games: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(CacheManager::new(
                PathBuf::from(settings.storage_path.clone()),
                settings.cache_ttl,
            )),
        }
    }

    pub async fn register_game(&self, game: Game) {
        let mut games = self.games.write().await;
        games.insert(game.id().to_string(), game);
    }

    pub async fn get_game(&self, name: &str) -> Option<Game> {
        self.games.read().await.get(name).cloned()
    }

    pub async fn get_loader(&self, game_id: &str, loader_id: &str) -> Option<Arc<dyn GameLoader>> {
        self.get_game(game_id).await?.get_loader(loader_id)
    }

    pub async fn download_build(&self, game_name: &str, build: &Build) -> anyhow::Result<Vec<u8>> {
        let filename = build.filename();

        if let Some(data) = self.cache.get(game_name, &filename).await? {
            return Ok(data);
        }

        let url = build
            .download_url()
            .ok_or_else(|| anyhow::anyhow!("Download URL not available"))?;

        let response = reqwest::get(url).await?;
        let data = response.bytes().await?.to_vec();

        self.cache.put(game_name, &filename, &data, None).await?;

        Ok(data)
    }

    pub async fn cleanup_cache(&self) -> anyhow::Result<()> {
        self.cache.cleanup().await
    }

    pub async fn list_games(&self) -> Vec<Game> {
        self.games.read().await.values().cloned().collect()
    }
}
