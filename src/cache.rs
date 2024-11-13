use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    accessed: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CacheManager {
    cache_dir: PathBuf,
    ttl: Duration,
}

impl CacheManager {
    pub fn new(cache_dir: PathBuf, ttl_days: u64) -> Self {
        std::fs::create_dir_all(&cache_dir).unwrap_or_default();
        Self {
            cache_dir,
            ttl: Duration::days(ttl_days as i64),
        }
    }

    async fn get_game_path(&self, game_name: &str) -> PathBuf {
        let path = self.cache_dir.join(game_name);
        fs::create_dir_all(&path).await.ok();
        path
    }

    pub async fn get(&self, game_name: &str, filename: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let game_path = self.get_game_path(game_name).await;
        let file_path = game_path.join(filename);
        let meta_path = file_path.with_extension("meta.bin");

        if file_path.exists() && meta_path.exists() {
            let content = fs::read(&meta_path).await?;
            let entry: CacheEntry = bincode::deserialize(&content)?;

            if Utc::now() - entry.accessed < self.ttl {
                let entry = CacheEntry {
                    accessed: Utc::now(),
                };
                fs::write(&meta_path, bincode::serialize(&entry)?).await?;
                return Ok(Some(fs::read(&file_path).await?));
            }

            fs::remove_file(&file_path).await.ok();
            fs::remove_file(&meta_path).await.ok();
        }

        Ok(None)
    }

    pub async fn put(
        &self,
        game_name: &str,
        filename: &str,
        data: &[u8],
        _sha1: Option<&str>,
    ) -> anyhow::Result<()> {
        let game_path = self.get_game_path(game_name).await;
        let file_path = game_path.join(filename);
        let meta_path = file_path.with_extension("meta.bin");

        fs::write(&file_path, data).await?;

        let entry = CacheEntry {
            accessed: Utc::now(),
        };

        fs::write(&meta_path, bincode::serialize(&entry)?).await?;

        Ok(())
    }

    pub async fn cleanup(&self) -> anyhow::Result<()> {
        let now = Utc::now();
        let mut entries = fs::read_dir(&self.cache_dir).await?;

        while let Some(game_dir) = entries.next_entry().await? {
            if !game_dir.file_type().await?.is_dir() {
                continue;
            }

            let mut files = fs::read_dir(game_dir.path()).await?;
            while let Some(entry) = files.next_entry().await? {
                if entry.file_name().to_string_lossy().ends_with(".meta.bin") {
                    if let Ok(content) = fs::read(entry.path()).await {
                        if let Ok(cache_entry) = bincode::deserialize::<CacheEntry>(&content) {
                            if now - cache_entry.accessed > self.ttl {
                                let base_path = entry.path().with_extension("");
                                fs::remove_file(&base_path).await.ok();
                                fs::remove_file(entry.path()).await.ok();
                            }
                        }
                    }
                }
            }

            if fs::read_dir(game_dir.path())
                .await?
                .next_entry()
                .await?
                .is_none()
            {
                fs::remove_dir(game_dir.path()).await.ok();
            }
        }

        Ok(())
    }
}
