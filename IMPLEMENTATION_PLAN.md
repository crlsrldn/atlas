# Project Atlas Implementation Plan

## Phase 0: Stabilize the MVP

- Restrict local backend exposure and remove plaintext secret reads from public API responses.
- Make provider status honest: no optimistic cached streams when cache checks fail.
- Support Stremio episode IDs without treating series requests as movie requests.
- Return no streams instead of invalid empty playback URLs.
- Add focused tests around identity parsing, ranking exclusions, and provider failure behavior.

## Phase 1: Provider Correctness

- Finish TorBox resolution by selecting the largest playable file instead of assuming `file_id=1`.
- Finish Real Debrid integration with cache verification, file selection, unrestriction, and telemetry.
- Add provider health checks that feed ranking rather than static placeholder values.
- Normalize provider errors into structured internal statuses.

## Phase 2: Metadata and Verification

- Replace Torrentio-derived placeholder metadata with normalized metadata from TMDB/IMDb-compatible sources.
- Introduce a real verification engine that scores runtime, episode match, release group, language, file structure, and known hash history.
- Preserve season/episode context throughout identity, metadata, source search, ranking, and playback.
- Add explainable confidence output for each stream.

## Phase 3: Ranking and Playback Intelligence

- Expand ranking inputs to include bitrate, codec, audio format, HDR/DV, subtitle availability, latency, reliability, provider health, and user device constraints.
- Add automatic retry/fallback behavior when a selected stream fails.
- Track playback success/failure locally and feed it back into ranking.
- Add an advanced stream inspection view for power users.

## Phase 4: Product Surface

- Make the first screen a real Smart Play control surface rather than an overview page.
- Add settings validation, provider connection tests, and clear provider health states.
- Move telemetry behind the backend so Appwrite permissions and schema are not exposed directly to the frontend.
- Add profile-level preferences for mobile, home theater, family, language, and subtitles.

## Phase 5: Privacy and Operations

- Move secrets to an OS keychain or encrypted local store for desktop builds.
- Add structured logs without API keys, hashes where avoidable, or personally identifying playback context.
- Create repeatable local development setup, CI checks, and release packaging.
- Document supported providers, configuration, and Stremio install flow.
