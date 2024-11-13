use anyhow::Result;
use reqwest::Client;
use serde::de::DeserializeOwned;

#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl HttpClient {
    pub async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let res = self.client.get(url).send().await?;
        Ok(res.json::<T>().await?)
    }

    pub fn get(&self, url: impl AsRef<str>) -> reqwest::RequestBuilder {
        self.client.get(url.as_ref())
    }
}
