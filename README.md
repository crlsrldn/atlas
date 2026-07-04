<div align="center">
  <h1>🌍 Project Atlas</h1>
  <p><b>Premium AI-Powered Multi-Source Streaming</b></p>
  <p>
    <i>Fast, Reliable, and Privacy-Preserving Smart Play Resolution</i>
  </p>
</div>

<br />

Atlas is a next-generation hosted Stremio companion and Smart Play resolver. It intelligently ranks playable streams across your configured providers, verifies the best match using advanced structural heuristics, and seamlessly redirects to the provider's playback URL—all without proxying media bytes.

Designed for speed, reliability, and an Apple-like premium experience.

---

## ✨ Core Features

* 🧠 **Smart Play Resolution:** Intelligently picks the absolute best stream based on runtime evidence, quality, availability, and your historical playback success.
* 🛡️ **Provider Verification:** Structurally verifies media using concrete evidence (duration, hashes, release groups) rather than relying blindly on often-inaccurate filenames.
* 🔒 **Privacy First Architecture:** Atlas never stores or proxies your media bytes. Telemetry and analytics are heavily redacted and strictly anonymized.
* 🏢 **Multi-Tenant SaaS:** Supports tenant-scoped install URLs and utilizes securely encrypted provider secret handles.
* 💻 **Modern Configuration UI:** A fast, responsive, data-focused dashboard built with Deno Fresh and beautifully styled with Tailwind CSS.

### Supported Providers
* **TorBox** — Cached torrent verification and ultra-fast playback resolution.

---

## 🏗 System Architecture

Atlas operates using a highly modular microservices model, structured into specialized, provider-independent engines:

* **Compute Core (Rust):** The heavy lifter. Performs cryptographic verification, metadata parsing, hashing, and intelligent quality ranking.
* **API Gateway (Go):** The edge router. Handles incoming Stremio addon requests, robust token validation, and routes traffic efficiently at the edge.
* **Dashboard (Deno + Fresh):** The configuration UI. Handles seamless user onboarding, monetization toggles, telemetry aggregation, and provider setup.
* **Database (Supabase):** The backend-as-a-service layer for managing encrypted provider secrets, authentication, webhooks, and telemetry data.

---

## ☁️ Deployment

**Coming soon.** 

Atlas will be provided as a fully hosted and managed SaaS service. No local setup or complex configuration will be required.
