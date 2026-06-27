# Project Atlas Implementation Plan

## Phase 0: Stabilize the MVP

- [x] Restrict local backend exposure and remove plaintext secret reads from public API responses.
- [x] Make provider status honest: no optimistic cached streams when cache checks fail.
- [x] Support Stremio episode IDs without treating series requests as movie requests.
- [x] Return no streams instead of invalid empty playback URLs.
- [x] Add a backend health endpoint and clearer frontend offline state.
- [x] Add focused tests around identity parsing, ranking exclusions, and provider cache parsing.
- [x] Add provider connection tests in Settings for configured services.
- [x] Replace local JSON secret storage with an encrypted/OS-native store.

## Phase 1: Provider Correctness

- [x] Verify the Stremio `manifest.json` route remains provisioned by the backend.
- [x] Finish TorBox resolution by selecting the largest playable file instead of assuming `file_id=1`.
- [x] Finish Real Debrid integration with cache verification, file selection, unrestriction, and telemetry.
- [x] Add provider health checks that feed ranking rather than static placeholder values.
- [x] Normalize provider errors into structured internal statuses.

## Phase 2: Metadata and Verification

- [x] Replace Torrentio-derived placeholder metadata with normalized metadata from TMDB/IMDb-compatible sources.
- [x] Introduce a real verification engine that scores runtime, episode match, release group, language, file structure, and known hash history.
- [x] Preserve season/episode context throughout identity, metadata, source search, ranking, and playback.
- [x] Add explainable confidence output for each stream.

## Phase 3: Ranking and Playback Intelligence

- [x] Expand ranking inputs to include bitrate, codec, audio format, HDR/DV, subtitle availability, latency, reliability, provider health, and user device constraints.
- [x] Add automatic retry/fallback behavior when a selected stream fails.
- [x] Track playback success/failure locally and feed it back into ranking.
- [x] Add an advanced stream inspection view for power users.

## Phase 4: Product Surface

- [x] Make the first screen a real Smart Play control surface rather than an overview page.
- [x] Add settings validation, provider connection tests, and clear provider health states.
- [x] Move telemetry behind the backend so Appwrite permissions and schema are not exposed directly to the frontend.
- [x] Add profile-level preferences for mobile, home theater, family, language, and subtitles.

## Phase 5: Privacy and Operations

- [x] Move secrets to an OS keychain or encrypted local store for desktop builds.
- [x] Add structured logs without API keys, hashes where avoidable, or personally identifying playback context.
- [x] Create repeatable local development setup, CI checks, and release packaging.
- [x] Document supported providers, configuration, and Stremio install flow.
