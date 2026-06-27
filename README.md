# Project Atlas

Project Atlas is a hosted Stremio companion and Smart Play resolver that ranks playable streams across configured providers, explains the best match, and redirects to provider playback URLs without proxying media bytes.

Atlas still supports the local desktop workflow, but the primary direction is Atlas Cloud: a Rust Axum service deployed on Fly.io with tenant-scoped Stremio install URLs, encrypted provider secret handles, monthly quotas, and Stripe-compatible billing hooks.

## Supported Providers

Atlas currently supports:

- TorBox for cached torrent verification and playback resolution.
- Real Debrid for cached torrent verification and playback resolution.
- Gemini for optional AI catalog recommendations.
- Appwrite for backend-owned preferences and telemetry storage.

## Atlas Cloud APIs

Hosted SaaS routes are available alongside the legacy local endpoints:

- `POST /auth/session` creates or loads a lightweight tenant session.
- `GET /v1/account` returns plan, quota, install token, and redacted preferences.
- `GET /v1/preferences` and `POST /v1/preferences` manage tenant preferences.
- `GET /v1/providers/status` tests provider keys from the tenant vault.
- `POST /v1/billing/checkout` returns a Stripe Checkout URL placeholder.
- `POST /v1/billing/webhook` updates subscription state for Stripe-style events.
- `GET /stremio/:install_token/manifest.json` exposes a tenant-scoped Stremio manifest.
- `GET /stremio/:install_token/stream/:type/:id.json` resolves tenant-scoped streams.

Atlas Cloud caches metadata and resolver decisions only. It never stores or relays media bytes.

Provider API keys are never returned by the public preferences API. On macOS desktop builds, Atlas stores provider secrets in Keychain under `com.cindrallabs.atlas` and persists only non-secret preferences to disk or Appwrite.

## Local Setup

Requirements:

- Rust stable
- Node.js 20
- npm
- macOS with Xcode Command Line Tools for Tauri desktop packaging

Install and verify:

```sh
make setup
make check
```

Run the backend:

```sh
make backend-dev
```

Run the frontend in another terminal:

```sh
make frontend-dev
```

Default local URLs:

- Backend: `http://127.0.0.1:3000`
- Frontend: `http://127.0.0.1:1420`
- Stremio manifest: `http://127.0.0.1:3000/manifest.json`
- Hosted demo manifest: `http://127.0.0.1:3000/stremio/demo-install-token/manifest.json`

Useful checks:

```sh
curl http://127.0.0.1:3000/manifest.json
curl http://127.0.0.1:3000/stream/movie/tt0133093.json
curl http://127.0.0.1:3000/inspect/movie/tt0133093.json
make smoke
```

## Configuration

Atlas reads optional Appwrite settings from environment variables:

```sh
APPWRITE_ENDPOINT=
APPWRITE_PROJECT_ID=
APPWRITE_API_KEY=
ATLAS_ENV=local
ATLAS_BIND_ADDR=127.0.0.1:3000
ATLAS_PUBLIC_BASE_URL=http://127.0.0.1:3000
ATLAS_VAULT_MASTER_KEY=change-this-before-production
STRIPE_CHECKOUT_URL=
```

The backend binds to `127.0.0.1:3000` by default. Override it with:

```sh
ATLAS_BIND_ADDR=127.0.0.1:3001
```

Provider keys can be entered through the Settings page. The backend migrates legacy local secrets from `preferences.json` into Keychain when possible, then writes redacted preferences back to local/cloud storage.

## Stremio Install Flow

1. Start the backend with `make backend-dev`.
2. Open `http://127.0.0.1:3000/manifest.json` and confirm the manifest returns JSON.
3. In Stremio, install the local addon using `http://127.0.0.1:3000/manifest.json`.
4. Configure providers in the Atlas Settings page.
5. Use Smart Play or request streams through Stremio.

## Cloud Deployment

Build and run the hosted backend container:

```sh
docker build -t atlas-backend .
docker run --rm -p 3000:3000 \
  -e ATLAS_BIND_ADDR=0.0.0.0:3000 \
  -e ATLAS_PUBLIC_BASE_URL=http://127.0.0.1:3000 \
  -e ATLAS_VAULT_MASTER_KEY=change-this-before-production \
  atlas-backend
```

Atlas ships with three Fly configs:

- `fly.toml` for development: `cindral-atlas-api-dev`
- `fly.staging.toml` for staging: `cindral-atlas-api-staging`
- `fly.production.toml` for production: `cindral-atlas-api`

Deploy to Fly.io from the CLI:

```sh
make deploy-dev
make deploy-staging
make deploy-production
```

Run smoke checks against any deployed environment:

```sh
ATLAS_SMOKE_URL=https://cindral-atlas-api-dev.fly.dev make smoke
ATLAS_SMOKE_URL=https://cindral-atlas-api-staging.fly.dev make smoke
ATLAS_SMOKE_URL=https://cindral-atlas-api.fly.dev make smoke
```

GitHub Actions CI/CD expects:

```sh
FLY_API_TOKEN              # repository secret
FLY_DEV_APP                # optional repository variable, defaults to cindral-atlas-api-dev
FLY_STAGING_APP            # optional repository variable, defaults to cindral-atlas-api-staging
FLY_PRODUCTION_APP         # optional repository variable, defaults to cindral-atlas-api
```

Deployment flow:

```text
develop or codex/* branch -> development
main branch               -> staging
v* tag                    -> production
workflow_dispatch         -> selected environment
```

## Privacy and Operations

Runtime logs use structured `tracing` fields and can be filtered with `RUST_LOG`, for example:

```sh
RUST_LOG=backend=debug make backend-dev
```

Logs and telemetry avoid API keys, download URLs, magnets, torrent hashes, and raw Stremio playback identifiers. Local playback history stays in `backend/playback_history.json`, which is ignored by Git.

CI runs Rust format, clippy, backend tests, frontend checks, frontend builds, container builds, local API smoke checks, and environment deploy smoke checks. Release packaging is handled by the Tauri release workflow on tags matching `v*`.
