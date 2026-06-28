#!/bin/bash
set -e

sed -i '' 's/-> Redirect {/-> axum::response::Response {/g' core/src/api/resolve.rs
sed -i '' 's/use axum::{extract::Path, response::Redirect, routing::post, Router};/use axum::{extract::Path, response::{IntoResponse, Response}, routing::post, Router};\nuse http::StatusCode;/g' core/src/api/resolve.rs

# Replace Redirect::temporary(...) with (StatusCode::FOUND, [("Location", ...)]).into_response()
sed -i '' 's/return Redirect::temporary("https:\/\/torbox.app");/return (StatusCode::FOUND, [("Location", "https:\/\/torbox.app")]).into_response();/g' core/src/api/resolve.rs
sed -i '' 's/return Redirect::temporary("https:\/\/real-debrid.com");/return (StatusCode::FOUND, [("Location", "https:\/\/real-debrid.com")]).into_response();/g' core/src/api/resolve.rs
sed -i '' 's/Redirect::temporary("https:\/\/real-debrid.com")/return (StatusCode::FOUND, [("Location", "https:\/\/real-debrid.com")]).into_response();/g' core/src/api/resolve.rs
sed -i '' 's/return Redirect::temporary(&dl_url);/return (StatusCode::FOUND, [("Location", dl_url)]).into_response();/g' core/src/api/resolve.rs
sed -i '' 's/return Redirect::temporary(&download);/return (StatusCode::FOUND, [("Location", download)]).into_response();/g' core/src/api/resolve.rs
sed -i '' 's/return Redirect::temporary(&candidate.url);/return (StatusCode::FOUND, [("Location", candidate.url.clone())]).into_response();/g' core/src/api/resolve.rs
sed -i '' 's/Redirect::temporary(provider_home)/(StatusCode::FOUND, [("Location", provider_home)]).into_response()/g' core/src/api/resolve.rs

