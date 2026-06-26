use serde::{Deserialize, Serialize};
use serde_json::json;
use reqwest::Client;
use crate::api::config::current_preferences;
use once_cell::sync::Lazy;

static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

#[derive(Serialize, Deserialize, Debug)]
pub struct Recommendation {
    pub imdb_id: String,
    pub title: String,
    pub reason: String,
}

pub async fn get_movie_recommendations() -> Vec<Recommendation> {
    let prefs = current_preferences();
    let api_key = prefs.gemini_api_key;
    
    if api_key.is_empty() {
        tracing::warn!("No Gemini API Key found in preferences. Returning empty AI recommendations.");
        return vec![];
    }

    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}", api_key);
    
    let prompt = "You are a media recommendation engine. The user wants 3 highly-rated sci-fi or thriller movie recommendations that are available on digital release. Respond ONLY with a JSON array of objects. Do not wrap it in markdown block quotes. Each object must have an 'imdb_id' (e.g. tt1234567), a 'title', and a short 'reason' string explaining why they will like it.";

    let payload = json!({
        "contents": [{
            "parts": [{"text": prompt}]
        }],
        "generationConfig": {
            "temperature": 0.7,
            "responseMimeType": "application/json"
        }
    });

    let res = HTTP_CLIENT.post(&url)
        .json(&payload)
        .send()
        .await;

    if let Ok(response) = res {
        if let Ok(text) = response.text().await {
            if let Ok(json_res) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(text_val) = json_res["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                    if let Ok(recommendations) = serde_json::from_str::<Vec<Recommendation>>(text_val) {
                        return recommendations;
                    }
                }
            }
        }
    }

    vec![]
}
