use crate::{
    config::Settings,
    models::{ArtworkCandidate, Game},
    network::NetworkService,
    providers, storage,
};
use anyhow::{bail, Result};
use std::{
    future::Future,
    io::Cursor,
    path::{Path, PathBuf},
    pin::Pin,
};

pub trait ArtworkProvider: Send + Sync {
    fn candidates<'a>(
        &'a self,
        game: &'a Game,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ArtworkCandidate>>> + Send + 'a>>;
}
pub struct SteamGridDb {
    pub network: NetworkService,
    pub api_key: String,
}
impl ArtworkProvider for SteamGridDb {
    fn candidates<'a>(
        &'a self,
        game: &'a Game,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ArtworkCandidate>>> + Send + 'a>> {
        Box::pin(async move {
            if self.api_key.is_empty() {
                return Ok(vec![]);
            }
            let name = providers::epic::encode_component(&game.name);
            let result = self
                .network
                .json(
                    &format!("https://www.steamgriddb.com/api/v2/search/autocomplete/{name}"),
                    &self.api_key,
                )
                .await?;
            let id = result["data"]
                .as_array()
                .and_then(|a| {
                    a.iter().find(|g| {
                        g["name"]
                            .as_str()
                            .is_some_and(|s| s.eq_ignore_ascii_case(&game.name))
                    })
                })
                .and_then(|g| g["id"].as_u64());
            let Some(id) = id else {
                return Ok(vec![]);
            };
            let result = self.network.json(&format!("https://www.steamgriddb.com/api/v2/grids/game/{id}?dimensions=600x900&types=static&nsfw=false"), &self.api_key).await?;
            Ok(result["data"]
                .as_array()
                .into_iter()
                .flatten()
                .take(3)
                .filter_map(|v| v["url"].as_str())
                .filter(|u| u.starts_with("https://"))
                .map(|u| ArtworkCandidate::Remote(u.into()))
                .collect())
        })
    }
}
pub fn cache_path(root: &Path, game: &Game) -> PathBuf {
    root.join("artwork").join(format!(
        "{}.png",
        storage::hash(game.key.encoded().as_bytes())
    ))
}
pub fn cached(root: &Path, game: &Game) -> Option<PathBuf> {
    let p = cache_path(root, game);
    p.is_file().then_some(p)
}
fn normalize(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut reader = image::ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let image = reader.decode()?.thumbnail(600, 900);
    let mut png = Cursor::new(Vec::new());
    image.write_to(&mut png, image::ImageFormat::Png)?;
    Ok(png.into_inner())
}
pub async fn fetch(root: &Path, settings: &Settings, game: &Game) -> Result<Option<PathBuf>> {
    if let Some(path) = cached(root, game) {
        return Ok(Some(path));
    }
    let mut candidates = providers::registry(settings)
        .iter()
        .find(|p| p.id() == game.key.provider_id)
        .map(|p| p.artwork_candidates(game))
        .unwrap_or_default();
    let source = settings.artwork.provider.as_str();
    candidates.retain(|c| match c {
        ArtworkCandidate::Local(_) => true,
        ArtworkCandidate::Remote(_) => source == "auto" || source == game.key.provider_id,
    });
    let network = if source == "local" {
        None
    } else {
        Some(NetworkService::new(&settings.network)?)
    };
    let path = cache_path(root, game);
    let mut attempted_remote = false;
    // Try local and platform candidates first; a remote metadata lookup is only
    // a fallback and never runs on the local scan/sync path.
    for candidate in candidates {
        attempted_remote |= matches!(candidate, ArtworkCandidate::Remote(_));
        if let Ok(bytes) = read_candidate(candidate, network.as_ref()).await {
            if let Ok(png) = normalize(&bytes) {
                storage::atomic_write(&path, &png)?;
                return Ok(Some(path));
            }
        }
    }
    if ["auto", "steamgriddb"].contains(&source) && !settings.artwork.steamgriddb_api_key.is_empty()
    {
        let service = SteamGridDb {
            network: network.clone().unwrap(),
            api_key: settings.artwork.steamgriddb_api_key.clone(),
        };
        for candidate in service.candidates(game).await? {
            attempted_remote = true;
            if let Ok(bytes) = read_candidate(candidate, network.as_ref()).await {
                if let Ok(png) = normalize(&bytes) {
                    storage::atomic_write(&path, &png)?;
                    return Ok(Some(path));
                }
            }
        }
    }
    if attempted_remote {
        bail!("Artwork download unavailable");
    }
    Ok(None)
}
async fn read_candidate(
    candidate: ArtworkCandidate,
    network: Option<&NetworkService>,
) -> Result<Vec<u8>> {
    match candidate {
        ArtworkCandidate::Local(path) => {
            if tokio::fs::metadata(&path).await?.len() > 12 * 1024 * 1024 {
                bail!("Artwork exceeds size limit");
            }
            Ok(tokio::fs::read(path).await?)
        }
        ArtworkCandidate::Remote(url) => match network {
            Some(n) => n.bytes(&url, None, 12 * 1024 * 1024).await,
            None => bail!("Online artwork disabled"),
        },
    }
}
