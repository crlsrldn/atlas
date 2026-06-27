.PHONY: setup check backend-test frontend-check backend-dev frontend-dev frontend-build smoke deploy-dev deploy-staging deploy-production

setup:
	cd frontend && npm ci

check: backend-test frontend-check

backend-test:
	cd backend && cargo test

frontend-check:
	cd frontend && npm run check

backend-dev:
	cd backend && cargo run --bin backend

frontend-dev:
	cd frontend && npm run dev -- --host 127.0.0.1 --port 1420

frontend-build:
	cd frontend && npm run build

smoke:
	./scripts/smoke.sh $${ATLAS_SMOKE_URL:-http://127.0.0.1:3000}

deploy-dev:
	flyctl deploy -c fly.toml --remote-only

deploy-staging:
	flyctl deploy -c fly.staging.toml --remote-only

deploy-production:
	flyctl deploy -c fly.production.toml --remote-only
