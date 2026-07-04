# Atlas

Atlas is a high-performance, intelligent media streaming proxy and aggregator, built to provide a seamless, premium cinematic experience. By acting as the unified engine between your media clients and TorBox, Atlas delivers instant playback, intelligent source selection, and device-optimized streaming.

## Features

- **Intelligent Source Ranking:** Automatically analyzes available streams and ranks them based on quality, bitrate, device compatibility, and historical reliability.
- **Lightning Fast Performance:** Engineered in Rust and Go, Atlas ensures minimal latency overhead, bringing cold startup times to under 5 seconds and source selection to under 2 seconds.
- **Unified Experience:** Replaces traditional complex configurations with a single seamless interface. Your media applications remain thin clients while Atlas handles the intelligence behind the scenes.
- **Privacy by Design:** End-to-end encrypted API keys and strict privacy controls. Your credentials are never exposed, and zero-knowledge architecture protects your streaming habits.
- **Device-Specific Optimization:** Analyzes your client hardware (e.g., Apple TV, LG OLED) to filter out incompatible codecs (like AV1 on older hardware) or prefer spatial audio formats, ensuring every stream plays flawlessly.
- **Structural Verification:** Moves beyond naive filename scraping by utilizing structural evidence (duration, hashes, release groups) to verify the authenticity and quality of media before serving it.

## Deployment

Deployment configuration and hosting details are **Coming Soon**. Atlas will be provided as a hosted service.
