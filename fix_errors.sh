#!/bin/bash
set -e

# Fix http::StatusCode in resolve.rs
sed -i '' 's/use http::StatusCode;/use axum::http::StatusCode;/g' core/src/api/resolve.rs

# Re-add get to axum routing in resolve.rs
sed -i '' 's/routing::post/routing::get/g' core/src/api/resolve.rs

# Change return type in internal.rs
sed -i '' 's/-> axum::response::Redirect {/-> axum::response::Response {/g' core/src/api/internal.rs

# Also change the fallback redirect in internal.rs to 302 Found
sed -i '' 's/axum::response::Redirect::temporary("https:\/\/github.com\/cindral\/atlas")/(axum::http::StatusCode::FOUND, \[("Location", "https:\/\/github.com\/cindral\/atlas")]).into_response()/g' core/src/api/internal.rs

# Add IntoResponse to internal.rs
sed -i '' 's/use axum::{/use axum::{response::IntoResponse, /g' core/src/api/internal.rs

