use crate::config::NetworkSettings;
use anyhow::{bail, Context, Result};
use std::time::{Duration, Instant};
use serde::Serialize;

#[derive(Clone)]
pub struct NetworkService { client: reqwest::Client }
#[derive(Debug, Serialize)]
pub struct ConnectionTest { pub connected: bool, pub latency_ms: u128, pub message: String }
impl NetworkService {
    pub fn new(settings: &NetworkSettings) -> Result<Self> {
        let mut builder = reqwest::Client::builder().user_agent(concat!("SunshineLibrarySync/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(8)).timeout(Duration::from_secs(20))
            .redirect(reqwest::redirect::Policy::limited(3));
        builder = match settings.proxy_mode.as_str() {
            "direct" => builder.no_proxy(),
            "custom" => {
                let url = url::Url::parse(&settings.proxy_url).context("Invalid proxy URL")?;
                if !["http", "https", "socks5", "socks5h"].contains(&url.scheme()) { bail!("Unsupported proxy scheme"); }
                builder.no_proxy().proxy(reqwest::Proxy::all(url).map_err(|_| anyhow::anyhow!("Invalid proxy URL"))?)
            },
            // reqwest system-proxy: environment (including NO_PROXY), then OS.
            "system" => builder,
            _ => bail!("Invalid proxy mode"),
        };
        Ok(Self { client: builder.build().context("Unable to initialize network client")? })
    }
    async fn request(&self, url: &str, key: Option<&str>) -> Result<reqwest::Response> {
        for attempt in 0..2 {
            let mut request = self.client.get(url);
            if let Some(key) = key { request = request.bearer_auth(key); }
            match request.send().await {
                Ok(response) if response.status().is_success() => return Ok(response),
                Ok(response) if !response.status().is_server_error() => bail!("Remote server returned HTTP {}", response.status().as_u16()),
                _ if attempt == 0 => tokio::time::sleep(Duration::from_millis(400)).await,
                _ => bail!("Network request failed; check connection and proxy settings"),
            }
        } unreachable!()
    }
    pub async fn bytes(&self, url: &str, key: Option<&str>, limit: usize) -> Result<Vec<u8>> {
        let mut response = self.request(url, key).await?;
        if response.content_length().is_some_and(|n| n > limit as u64) { bail!("Download exceeds size limit"); }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| anyhow::anyhow!("Download interrupted"))? {
            if bytes.len() + chunk.len() > limit { bail!("Download exceeds size limit"); } bytes.extend_from_slice(&chunk);
        }
        Ok(bytes)
    }
    pub async fn json(&self, url: &str, key: &str) -> Result<serde_json::Value> { Ok(serde_json::from_slice(&self.bytes(url, Some(key), 2 * 1024 * 1024).await?)?) }
    pub async fn test(&self) -> ConnectionTest {
        let start = Instant::now();
        match self.bytes("https://www.gstatic.com/generate_204", None, 4096).await {
            Ok(_) => ConnectionTest { connected: true, latency_ms: start.elapsed().as_millis(), message: "Connected".into() },
            Err(e) => ConnectionTest { connected: false, latency_ms: start.elapsed().as_millis(), message: e.to_string() },
        }
    }
}
