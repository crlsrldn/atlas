use crate::api::config::UserPreferences;
use crate::engines::cache::{get_json, scoped_key, set_json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use reqwest;

#[derive(Serialize, Deserialize, Debug)]
struct AiConstraints {
    exclude_hevc: Option<bool>,
    exclude_av1: Option<bool>,
    max_resolution: Option<String>,
    prefer_hdr: Option<bool>,
}

pub async fn evaluate_device_profile(mut prefs: UserPreferences) -> UserPreferences {
    if prefs.device_profile.is_empty() || prefs.gemini_api_key.is_empty() || !prefs.is_premium {
        return prefs;
    }

    let cache_key = scoped_key("ai_decision", "profile", &prefs.device_profile);
    
    let constraints: AiConstraints = if let Some(cached) = get_json(&cache_key) {
        if let Ok(c) = serde_json::from_value(cached) {
            c
        } else {
            return prefs;
        }
    } else {
        let client = reqwest::Client::new();
        let prompt = format!(
            "You are an AI configuring video streaming settings. Hardware: '{}'. Output valid JSON only with these fields: {{ \"exclude_hevc\": bool, \"exclude_av1\": bool, \"max_resolution\": \"4K\" | \"1080p\" | \"720p\", \"prefer_hdr\": bool }}. Do not output markdown code blocks.",
            prefs.device_profile
        );

        let body = json!({
            "contents": [{
                "parts": [{"text": prompt}]
            }],
            "generationConfig": {
                "responseMimeType": "application/json"
            }
        });

        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}", prefs.gemini_api_key);
        
        let res = client.post(&url).json(&body).send().await;
        
        if let Ok(r) = res {
            if let Ok(json_res) = r.json::<serde_json::Value>().await {
                if let Some(text) = json_res["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                    if let Ok(parsed) = serde_json::from_str::<AiConstraints>(text) {
                        set_json(&cache_key, serde_json::to_value(&parsed).unwrap(), Duration::from_secs(86400 * 7));
                        parsed
                    } else {
                        return prefs;
                    }
                } else {
                    return prefs;
                }
            } else {
                return prefs;
            }
        } else {
            return prefs;
        }
    };

    if let Some(eh) = constraints.exclude_hevc {
        prefs.exclude_hevc = eh;
    }
    if let Some(ea) = constraints.exclude_av1 {
        prefs.exclude_av1 = ea;
    }
    if let Some(mr) = constraints.max_resolution {
        prefs.max_resolution = mr;
    }
    if let Some(ph) = constraints.prefer_hdr {
        prefs.prefer_hdr = ph;
    }

    tracing::info!("AI Decision applied device profile constraints");
    prefs
}
pub fn infer_capabilities(user_agent: &str, mut prefs: UserPreferences) -> UserPreferences {
    let ua = user_agent.to_lowercase();

    // Rule 1: Apple Devices struggle with AV1 depending on age.
    // Safest bet for AppleTV / Safari without a native app is to exclude AV1.
    if ua.contains("appletv")
        || ua.contains("mac os x")
        || ua.contains("iphone")
        || ua.contains("ipad")
    {
        tracing::info!("AI Decision: Apple device detected, forcing exclude_av1 = true");
        prefs.exclude_av1 = true;
    }

    // Rule 2: Mobile devices should default to lower resolution to save bandwidth
    if (ua.contains("mobile") || ua.contains("android") || ua.contains("iphone"))
        && prefs.max_resolution == "4K"
    {
        tracing::info!("AI Decision: Mobile device detected, lowering max_resolution to 1080p");
        prefs.max_resolution = "1080p".to_string();
    }

    // Rule 3: Web Browsers (Chrome/Firefox/Safari) generally cannot play HEVC natively
    // We check for typical browser signatures that aren't native apps (like ExoPlayer, mpv, VLC)
    let is_native_player = ua.contains("exoplayer")
        || ua.contains("vlc")
        || ua.contains("mpv")
        || ua.contains("stremio");
    if !is_native_player
        && (ua.contains("mozilla") || ua.contains("chrome") || ua.contains("safari"))
    {
        tracing::info!("AI Decision: Web Browser detected, forcing exclude_hevc = true");
        prefs.exclude_hevc = true;
    }

    prefs
}
