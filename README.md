# Project Atlas 🌍

Atlas is a hosted Stremio companion and Smart Play resolver. It intelligently ranks playable streams across configured providers, verifies and explains the best match, and seamlessly redirects to provider playback URLs without proxying media bytes. 

The architecture is designed to be highly modular, privacy-preserving, and incredibly fast. It operates using a microservices model split across Rust (Compute), Go (API Gateway), and Deno (Frontend Dashboard).

---

## ✨ Features

- **Smart Play Resolution:** Intelligently picks the best stream based on runtime evidence, quality, availability, and historical success.
- **Provider Verification:** Structurally verifies media using evidence (duration, hashes, release groups) rather than relying blindly on filenames.
- **Privacy First:** Atlas never stores or proxies media bytes. Telemetry and analytics are heavily redacted and anonymized. 
- **Multi-Tenant SaaS:** Supports tenant-scoped install URLs and securely encrypted provider secret handles.
- **Modern UI:** A fast, responsive, data-focused configuration dashboard built with Deno Fresh and styled with Tailwind CSS.

### Supported Providers
- **TorBox** - Cached torrent verification and playback resolution.


## 🏗 Architecture

The system is structured into specialized, provider-independent engines:

- **Compute Core (Rust):** The heavy lifter. Performs cryptographic verification, metadata parsing, hashing, and quality ranking.
- **API Gateway (Go):** The edge router. Handles incoming Stremio addon requests, token validation, and routes traffic efficiently.
- **Dashboard (Deno + Fresh):** The configuration UI. Handles user onboarding, monetization toggles, telemetry aggregation, and provider setup.
- **Database (Supabase):** Backend-as-a-service for managing encrypted provider secrets, authentication, webhooks, and telemetry data.

## 🚀 Local Setup

### Requirements

To run Atlas locally, you will need the following installed:
- [Rust](https://www.rust-lang.org/tools/install) (Stable)
- [Go](https://go.dev/doc/install) (1.20+)
- [Deno](https://deno.land/manual/getting_started/installation) (1.37+)

### Installation

1. **Clone the repository and install dependencies:**
   ```sh
   make setup
   ```

2. **Run the services (in separate terminals):**

   *Start the Rust Compute Core:*
   ```sh
   make core-dev
   ```

   *Start the Go API Gateway:*
   ```sh
   make gateway-dev
   ```

   *Start the Deno Dashboard:*
   ```sh
   make dashboard-dev
   ```

### Local Development URLs
- **Compute Core (Backend):** `http://127.0.0.1:3000`
- **Dashboard (Frontend):** `http://127.0.0.1:8000` (Default Deno port)
- **API Gateway:** *(Check Go service output for port binding)*

## ⚙️ Configuration

Atlas is configured via environment variables. Create a `.env` file or export the following in your shell:

```sh
# Database / Backend-as-a-Service
PUBLIC_SUPABASE_URL=your-supabase-url
PUBLIC_SUPABASE_ANON_KEY=your-anon-key
SUPABASE_SERVICE_ROLE_KEY=your-service-role-key

# Atlas Environment
ATLAS_ENV=local
ATLAS_BIND_ADDR=127.0.0.1:3000
ATLAS_PUBLIC_BASE_URL=http://127.0.0.1:3000
ATLAS_VAULT_MASTER_KEY=change-this-before-production

# Billing (Optional)
STRIPE_CHECKOUT_URL=
```

Provider keys are meant to be entered securely through the **Platform Settings** tab in the dashboard. They are encrypted before being stored in the database.

## 🛠 Testing & Validation

The project includes an automated test suite across all services:

```sh
# Run all tests and type checks
make check

# Run a live smoke test against local or remote environments
make smoke
ATLAS_SMOKE_URL=https://staging.yourdomain.com make smoke
```

## ☁️ Deployment

Atlas is designed for Fly.io deployments. The API Gateway (Go) and Dashboard (Deno) should be exposed publicly, while the Compute Core (Rust) can be isolated on a 6PN private network for enhanced security.

Deployment configurations are defined in `fly.toml` files, and continuous integration is managed via GitHub Actions.

---

*Designed for speed, reliability, and an Apple-like premium experience.*
