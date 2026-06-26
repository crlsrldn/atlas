# Project Atlas
## Product Requirements Document (PRD)
**Version:** 0.1 Founding Vision  
**Status:** Internal Draft  
**Author:** Cindral Labs  
**Confidential**

---

# Executive Summary

Project Atlas is an intelligent media resolution platform designed to become the operating system between users and their media sources.

Unlike existing Stremio add-ons that simply aggregate links, Atlas analyzes every available source and automatically selects the best possible playback option based on quality, latency, reliability, compatibility, and user preferences.

Atlas is provider-agnostic.

It is not a torrent search engine.

It is not a debrid service.

It is not a streaming service.

It is the intelligence layer connecting all of them.

---

# Mission

> Eliminate every technical decision between discovering media and watching it.

The user should never think about:

- Torrents
- Magnets
- Debrid providers
- Codecs
- Containers
- Audio formats
- Subtitle sources
- Release groups
- Metadata providers

The user presses **Play**.

Atlas handles everything else.

---

# Vision

Current ecosystem:

```
User
 │
 ├── Netflix
 ├── Disney+
 ├── Prime
 ├── Plex
 ├── Jellyfin
 ├── Kodi
 └── Stremio
```

Atlas becomes the intelligence layer:

```
                User
                  │
          ┌────────────────┐
          │     Atlas      │
          └────────────────┘
          │      │      │
     Metadata  Sources  Playback
          │      │      │
    TMDB IMDb   TorBox  VLC
    TVDB Trakt  RD      MPV
                PM      ExoPlayer
                NAS
                HTTP
```

---

# Product Philosophy

## Principle 1

One click.

No technical knowledge required.

---

## Principle 2

Never expose complexity unless requested.

---

## Principle 3

Every recommendation is explainable.

---

## Principle 4

Fast is a feature.

---

## Principle 5

Privacy first.

---

# Product Pillars

## Intelligence

Atlas should make better decisions than the user.

---

## Speed

Everything should feel instant.

Target startup:

< 2 seconds

---

## Trust

Never recommend broken media.

Every recommendation is verified.

---

## Simplicity

The UI should feel closer to Apple TV than qBittorrent.

---

# User Personas

## Casual Viewer

Goals:

- Press Play
- Watch immediately

Never sees:

- Hashes
- Magnets
- Seeders
- Codecs

---

## Power User

Can inspect:

- Video codec
- Audio codec
- Bitrate
- HDR
- Atmos
- Release group
- Runtime verification
- CDN latency

---

## Home Theater

Preferences:

- Dolby Vision
- HDR10+
- Atmos
- TrueHD
- NAS priority
- Remux support

---

## Mobile User

Preferences:

- Maximum file size
- Mobile-friendly codecs
- Low startup latency
- Data saver mode

---

## Family

Supports:

- Kids profile
- Language preference
- Content filtering
- Subtitle defaults

---

# Core Architecture

```
                    Atlas Core

 ┌────────────────────────────────────────────┐
 │ Identity Engine                            │
 ├────────────────────────────────────────────┤
 │ Metadata Engine                            │
 ├────────────────────────────────────────────┤
 │ Source Engine                              │
 ├────────────────────────────────────────────┤
 │ Verification Engine                        │
 ├────────────────────────────────────────────┤
 │ Quality Ranking Engine                     │
 ├────────────────────────────────────────────┤
 │ Playback Engine                            │
 ├────────────────────────────────────────────┤
 │ Learning Engine                            │
 ├────────────────────────────────────────────┤
 │ AI Decision Engine                         │
 ├────────────────────────────────────────────┤
 │ Analytics                                  │
 └────────────────────────────────────────────┘
```

---

# Module Specifications

## Identity Engine

Purpose:

Normalize every provider into one internal media identifier.

Supports:

- IMDb
- TMDB
- TVDB
- Trakt
- AniDB

Output:

```
AtlasID
```

Every internal system references AtlasID only.

---

## Metadata Engine

Responsibilities:

Collect:

- Runtime
- Genres
- Posters
- Ratings
- Release dates
- Audio languages
- Subtitle languages
- HDR availability
- Dolby Vision support
- Cast
- Crew

Normalize all metadata.

---

## Source Engine

Provider Plugins:

- TorBox
- Real Debrid
- Premiumize
- EasyNews
- Local NAS
- Plex
- Jellyfin
- HTTP
- FTP
- Future providers

Plugin interface:

```
Search()

Resolve()

Health()

Capabilities()

Priority()
```

---

## Verification Engine

Problem:

Current addons trust filenames.

Atlas trusts evidence.

Verification inputs:

- Runtime
- Episode duration
- Hash history
- Metadata
- Audio language
- Subtitle language
- Release group
- File structure

Output:

```
Confidence Score

99.7%
```

---

## Quality Ranking Engine

Every source receives a score.

Inputs:

- Cached status
- Startup latency
- Bitrate
- Codec
- HDR
- Audio
- Resolution
- Reliability
- Device compatibility
- Historical playback success

Example:

```
Overall Score

94.2
```

---

## Playback Engine

Responsibilities:

Choose:

- Best source
- Best server
- Best CDN
- Compatible codec
- Subtitle source

Automatically retry failures.

---

## Learning Engine

Learns:

- Preferred codecs
- Preferred resolutions
- Preferred release groups
- Language
- Subtitle usage
- Device capability
- Internet speed
- Watch history
- Failure history

Privacy preserving.

---

## AI Decision Engine

Purpose:

Reason about playback.

Examples:

User:

"I only have 30 minutes."

Atlas:

Suggest shorter content.

---

User:

"My TV cannot decode AV1."

Atlas:

Automatically exclude AV1.

---

User:

"I hate buffering."

Atlas:

Favor cached lower bitrate releases.

---

# Ranking Algorithm

```
Overall Score

=

Availability

×

Quality

×

Reliability

×

Compatibility

×

Startup Speed

×

User Preference

×

Historical Success

×

Provider Health
```

Every recommendation is explainable.

---

# User Interface

Default:

```
▶ Play
```

Advanced:

```
▼ Advanced

4K HDR

1080 HEVC

720 Mobile

Remux

Alternative Audio

Manual Source Selection
```

Most users never open Advanced.

---

# Smart Play

Workflow:

```
User

↓

Atlas

↓

Search providers

↓

Verify sources

↓

Rank

↓

Resolve playback

↓

Play
```

---

# Device Intelligence

Atlas detects:

- Phone
- Tablet
- Browser
- Apple TV
- Android TV
- Fire TV
- Chromecast
- Shield
- Desktop

Automatically adjusts recommendations.

---

# Health Dashboard

Displays:

```
Metadata

Healthy

Sources

Healthy

TorBox

42 ms

Cache

99%

Subtitle Engine

Healthy

Average Startup

1.1 sec
```

---

# Performance Targets

Metadata:

< 30 ms

Search:

< 200 ms

Ranking:

< 75 ms

Playback decision:

< 50 ms

Cold startup:

< 5 sec

Warm startup:

< 2 sec

---

# Security

API Keys:

Encrypted

Secrets:

Never logged

Plugins:

Sandboxed

Rate limiting:

Enabled

Authentication:

OAuth ready

---

# Extensibility

Everything is a plugin.

Plugin types:

- Metadata Provider
- Source Provider
- Subtitle Provider
- Playback Provider
- AI Provider
- Analytics Provider

Atlas Core remains provider-independent.

---

# API Design

```
GET

/media/search

/media/details

/media/play

/provider/status

/provider/search

/provider/cache

/user/preferences

/user/devices

/analytics
```

---

# Storage & Infrastructure

Primary Platform:

Appwrite (Database, Auth, Storage, Functions)

Search:

Meilisearch

Telemetry:

ClickHouse

---

# Technology Stack

Core Platform:

Appwrite

Backend / Serverless Functions:

Rust (Axum, Tokio)

Frontend (Web, Desktop, Mobile Config Dashboard):

SvelteKit + Tauri v2

Queue (if external needed):

NATS

Observability:

OpenTelemetry

Deployment:

Docker

Cloudflare

---

# Roadmap

## Phase 1

- Stremio integration
- TorBox support
- Smart ranking
- Metadata normalization
- Cached-only playback
- Smart Play

---

## Phase 2

- Multiple providers
- AI recommendations
- Local NAS
- Jellyfin integration
- Plex integration
- Analytics dashboard

---

## Phase 3

- Semantic search
- Cross-device synchronization
- Plugin marketplace
- Community verification network
- Federated reputation system

---

# Long-Term Vision

Atlas becomes the operating system for media playback.

Applications become thin clients.

Providers become interchangeable plugins.

Users never think about infrastructure.

Only content.

---

# North Star Metric

**Time from selecting media to playback.**

Goal:

```
< 2 seconds

99.9% successful playback

Zero manual source selection
```

---

# Product Principles

- Hide complexity.
- Optimize relentlessly.
- Make intelligent decisions automatically.
- Every recommendation must be explainable.
- Privacy is non-negotiable.
- Performance is a feature.
- Design for the next decade, not the next release.