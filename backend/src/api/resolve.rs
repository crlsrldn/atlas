use axum::{
    extract::Path,
    routing::get,
    Router,
    response::Redirect,
};
use reqwest;
use serde::Deserialize;
use crate::api::config::current_preferences;

#[derive(Deserialize)]
struct TorboxCreateResponse {
    success: bool,
    data: Option<TorboxTorrentData>,
}

#[derive(Deserialize)]
struct TorboxTorrentData {
    torrent_id: u64,
}

pub fn router() -> Router {
    Router::new()
        .route("/resolve/torbox/:hash", get(resolve_torbox))
        .route("/resolve/realdebrid/:hash", get(resolve_realdebrid))
}

async fn resolve_torbox(Path(hash): Path<String>) -> Redirect {
    let prefs = current_preferences();
    let api_key = prefs.torbox_api_key;

    if api_key.is_empty() {
        return Redirect::temporary("https://torbox.app");
    }

    let client = reqwest::Client::new();
    let magnet = format!("magnet:?xt=urn:btih:{}", hash);
    
    // 1. Create/Add the torrent
    let create_res = client.post("https://api.torbox.app/v1/api/torrents/createtorrent")
        .bearer_auth(&api_key)
        .form(&[("magnet", magnet.as_str())])
        .send()
        .await;

    if let Ok(res) = create_res {
        if let Ok(json) = res.json::<TorboxCreateResponse>().await {
            if json.success {
                if let Some(data) = json.data {
                    let torrent_id = data.torrent_id;
                    // 2. Redirect to the permalink to automatically start streaming
                    // For TorBox, file_id=1 is usually the largest file (the movie itself).
                    // In a more robust implementation, we would query the torrent info to find the largest file ID.
                    let dl_url = format!("https://api.torbox.app/v1/api/torrents/requestdl?token={}&torrent_id={}&file_id=1&redirect=true", api_key, torrent_id);
                    
                    crate::engines::telemetry::log_event("playback_started", serde_json::json!({
                        "provider": "torbox",
                        "hash": hash,
                        "success": true
                    }));

                    return Redirect::temporary(&dl_url);
                }
            }
        }
    }

    crate::engines::telemetry::log_event("playback_started", serde_json::json!({
        "provider": "torbox",
        "hash": hash,
        "success": false
    }));

    Redirect::temporary("https://torbox.app")
}

#[derive(Deserialize)]
struct RDAddMagnetResponse {
    id: String,
}

#[derive(Deserialize)]
struct RDInfoResponse {
    links: Vec<String>,
}

#[derive(Deserialize)]
struct RDUnrestrictResponse {
    download: String,
}

async fn resolve_realdebrid(Path(hash): Path<String>) -> Redirect {
    let prefs = current_preferences();
    let api_key = prefs.real_debrid_api_key;

    if api_key.is_empty() {
        return Redirect::temporary("https://real-debrid.com");
    }

    let client = reqwest::Client::new();
    let magnet = format!("magnet:?xt=urn:btih:{}", hash);

    // 1. Add Magnet
    let add_res = client.post("https://api.real-debrid.com/rest/1.0/torrents/addMagnet")
        .bearer_auth(&api_key)
        .form(&[("magnet", magnet.as_str())])
        .send()
        .await;

    if let Ok(res) = add_res {
        if let Ok(json) = res.json::<RDAddMagnetResponse>().await {
            let id = json.id;

            // 2. Select all files
            let _ = client.post(&format!("https://api.real-debrid.com/rest/1.0/torrents/selectFiles/{}", id))
                .bearer_auth(&api_key)
                .form(&[("files", "all")])
                .send()
                .await;

            // 3. Get Info (to get the generated CDN links)
            let info_res = client.get(&format!("https://api.real-debrid.com/rest/1.0/torrents/info/{}", id))
                .bearer_auth(&api_key)
                .send()
                .await;

            if let Ok(res2) = info_res {
                if let Ok(json2) = res2.json::<RDInfoResponse>().await {
                    if let Some(link) = json2.links.first() {
                        // 4. Unrestrict link
                        let unrestrict_res = client.post("https://api.real-debrid.com/rest/1.0/unrestrict/link")
                            .bearer_auth(&api_key)
                            .form(&[("link", link.as_str())])
                            .send()
                            .await;

                        if let Ok(res3) = unrestrict_res {
                            if let Ok(json3) = res3.json::<RDUnrestrictResponse>().await {
                                return Redirect::temporary(&json3.download);
                            }
                        }
                    }
                }
            }
        }
    }

    Redirect::temporary("https://real-debrid.com")
}
