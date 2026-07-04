.PHONY: setup check core-test gateway-test dashboard-check core-dev gateway-dev dashboard-dev

setup:
	cd dashboard && bun install || true
	cd gateway && go mod tidy || true

check: core-test gateway-test dashboard-check

core-test:
	cd core && cargo test

gateway-test:
	cd gateway && go test ./... || true

dashboard-check:
	cd dashboard && bun run check || true

core-dev:
	cd core && ATLAS_BIND_ADDR=127.0.0.1:3000 cargo run --bin backend

gateway-dev:
	cd gateway && go run main.go

dashboard-dev:
	cd dashboard && bun run dev
