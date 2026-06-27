#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${1:-http://127.0.0.1:3000}"
BASE_URL="${BASE_URL%/}"

echo "Smoke testing ${BASE_URL}"

curl -fsS "${BASE_URL}/health" >/tmp/atlas-health.json
grep -q '"status":"ok"' /tmp/atlas-health.json

curl -fsS "${BASE_URL}/stremio/demo-install-token/manifest.json" >/tmp/atlas-manifest.json
grep -q '"Atlas Cloud"' /tmp/atlas-manifest.json

invalid_status="$(curl -s -o /tmp/atlas-invalid-token.json -w "%{http_code}" "${BASE_URL}/stremio/bad-token/manifest.json")"
test "${invalid_status}" = "404"

curl -fsS "${BASE_URL}/stremio/demo-install-token/stream/movie/not-an-id.json" >/tmp/atlas-streams.json
grep -q '"streams"' /tmp/atlas-streams.json

echo "Smoke tests passed"
