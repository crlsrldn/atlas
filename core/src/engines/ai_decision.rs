use crate::api::config::UserPreferences;

pub fn infer_capabilities(user_agent: &str, mut prefs: UserPreferences) -> UserPreferences {
    let ua = user_agent.to_lowercase();
    
    // Rule 1: Apple Devices struggle with AV1 depending on age. 
    // Safest bet for AppleTV / Safari without a native app is to exclude AV1.
    if ua.contains("appletv") || ua.contains("mac os x") || ua.contains("iphone") || ua.contains("ipad") {
        tracing::info!("AI Decision: Apple device detected, forcing exclude_av1 = true");
        prefs.exclude_av1 = true;
    }

    // Rule 2: Mobile devices should default to lower resolution to save bandwidth
    if ua.contains("mobile") || ua.contains("android") || ua.contains("iphone") {
        if prefs.max_resolution == "4K" {
            tracing::info!("AI Decision: Mobile device detected, lowering max_resolution to 1080p");
            prefs.max_resolution = "1080p".to_string();
        }
    }

    // Rule 3: Web Browsers (Chrome/Firefox/Safari) generally cannot play HEVC natively
    // We check for typical browser signatures that aren't native apps (like ExoPlayer, mpv, VLC)
    let is_native_player = ua.contains("exoplayer") || ua.contains("vlc") || ua.contains("mpv") || ua.contains("stremio");
    if !is_native_player && (ua.contains("mozilla") || ua.contains("chrome") || ua.contains("safari")) {
        tracing::info!("AI Decision: Web Browser detected, forcing exclude_hevc = true");
        prefs.exclude_hevc = true;
    }

    prefs
}
