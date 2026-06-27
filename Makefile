.PHONY: setup check backend-test frontend-check backend-dev frontend-dev frontend-build tauri-build

setup:
	cd frontend && npm ci

check: backend-test frontend-check

backend-test:
	cd backend && cargo test

frontend-check:
	cd frontend && npm run check

backend-dev:
	cd backend && cargo run

frontend-dev:
	cd frontend && npm run dev -- --host 127.0.0.1 --port 1420

frontend-build:
	cd frontend && npm run build

tauri-build:
	cd frontend && npm run tauri build
