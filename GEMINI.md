# Project Atlas - AI Agent Rules

## Core Principles
1. **One Click:** Design all user-facing interactions to require no technical knowledge. The user presses play, and Atlas handles everything else.
2. **Hide Complexity:** Never expose technical details (hashes, magnets, seeders, codecs, etc.) to the user unless explicitly requested in an "Advanced" view.
3. **Explainability:** Ensure every recommendation made by the system is explainable and verifiable.
4. **Performance:** Fast is a feature. Optimize relentlessly. Target cold startup < 5 seconds, warm startup < 2 seconds, and media selection to playback in < 2 seconds.
5. **Privacy First:** Ensure all analytics and learning features preserve user privacy and keep secrets out of logs.
6. **Trust & Reliability:** Never recommend broken media. Every source recommendation must be verified via evidence (runtime, hashes, metadata), not just filenames.

## Technical Stack Guidelines
When writing code or suggesting architectural changes, adhere to the approved technology stack:
- **Core Platform:** Appwrite (Database, Authentication, Encrypted Provider Secrets, Webhooks)
- **API Gateway / Router:** Go (Handles incoming Stremio addon requests, token validation, edge routing)
- **Compute Core (Backend):** Rust (Cryptographic verification, metadata parsing, hashing, quality ranking)
- **Frontend (Configuration Dashboard):** Deno + Fresh (Edge-rendered Island architecture)
- **Search:** Meilisearch
- **Message Queue:** NATS (if needed outside Appwrite)
- **Telemetry:** ClickHouse, OpenTelemetry
- **Deployment:** Fly.io (Go and Deno exposed publicly, Rust isolated on 6PN private network)

## Architecture & Modules
Structure the system into distinct, provider-independent engines:
- **Identity Engine:** Normalize all provider identifiers (TMDB, IMDb, TVDB, Trakt, AniDB) into a single internal `AtlasID`. Every internal system must reference `AtlasID` only.
- **Metadata Engine:** Collect and normalize media metadata (runtime, HDR availability, cast, ratings, etc.).
- **Source Engine:** Implement provider plugins (TorBox, Real Debrid, Local NAS, Plex, Jellyfin, etc.) via a standard interface (`Search()`, `Resolve()`, `Health()`, `Capabilities()`, `Priority()`).
- **Verification Engine:** Verify media using structural evidence (duration, hashes, release groups) instead of blind trust in filenames. Output a confidence score.
- **Quality Ranking Engine:** Rank sources based on availability, quality, reliability, compatibility, speed, user preference, and historical success.
- **Playback Engine:** Handle seamless source routing, CDN selection, subtitle fetching, and automatic failure retries.
- **Learning & AI Decision Engine:** Personalize choices while preserving privacy, and infer optimal playback strategies dynamically (e.g., automatically excluding incompatible codecs based on device intelligence).

## Design Aesthetic
- Emulate the simplicity and premium feel of Apple TV rather than a traditional torrent client like qBittorrent.
- Ensure a unified, provider-agnostic experience. Applications should be thin clients while the Atlas Core handles all intelligence.
