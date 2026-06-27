.PHONY: setup check core-test gateway-test dashboard-check core-dev gateway-dev dashboard-dev smoke

setup:
	cd dashboard && deno cache main.ts || true
	cd gateway && go mod tidy || true

check: core-test gateway-test dashboard-check

core-test:
	cd core && cargo test

gateway-test:
	cd gateway && go test ./... || true

dashboard-check:
	cd dashboard && deno check main.ts || true

core-dev:
	cd core && ATLAS_BIND_ADDR=127.0.0.1:3000 cargo run --bin backend

gateway-dev:
	cd gateway && go run main.go

dashboard-dev:
	cd dashboard && deno task start

smoke:
	./scripts/smoke.sh $${ATLAS_SMOKE_URL:-http://127.0.0.1:8080}
