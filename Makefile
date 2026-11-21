.DEFAULT_GOAL := help
SHELL := /bin/sh

DATA_DIR ?= .data
PORT ?= 8099
PROXY_PORT ?= 18100
IMAGE ?= memory-card:dev
# Home Assistant user ID that the dev proxies send, made a manager by `make run`.
DEV_USER ?= dev

# Database the sqlx query macros check against while preparing server/.sqlx.
SQLX_DB_URL = sqlite://$(CURDIR)/target/sqlx/dev.sqlite

RUN_ENV = MEMORY_CARD_DATA_DIR=$(DATA_DIR) \
	MEMORY_CARD_TMP_DIR=$(DATA_DIR)/tmp \
	MEMORY_CARD_LISTEN=127.0.0.1:$(PORT) \
	MEMORY_CARD_TRUSTED_PEER=127.0.0.1 \
	MEMORY_CARD_MANAGERS=$(DEV_USER)

.PHONY: help
help: ## List targets
	@grep -E '^[a-z-]+:.*## ' $(MAKEFILE_LIST) | awk -F':.*## ' '{printf "  %-12s %s\n", $$1, $$2}'

node_modules: package.json package-lock.json web/package.json e2e/package.json
	npm install
	@touch node_modules

.PHONY: install
install: node_modules ## Install npm dependencies

.PHONY: web
web: node_modules ## Build the web app into web/dist
	npm run build

.PHONY: run
run: web ## Run the server locally (override PORT, DATA_DIR)
	$(RUN_ENV) cargo run -- serve

.PHONY: web-dev
web-dev: node_modules ## Vite dev server with /api proxied to the server
	DEV_USER=$(DEV_USER) npm run dev -w web

.PHONY: proxy
proxy: ## Ingress stand-in at /api/hassio_ingress/e2e-token/ on PROXY_PORT
	UPSTREAM=http://127.0.0.1:$(PORT) PORT=$(PROXY_PORT) DEFAULT_USER=$(DEV_USER) node e2e/ingress-proxy.ts

.PHONY: test
test: node_modules ## Rust and web unit tests
	cargo test --locked
	npm test

.PHONY: e2e
e2e: web ## Playwright tests behind the ingress stand-in
	cargo build --locked
	npx playwright install chromium
	npm run e2e

.PHONY: fmt
fmt: ## Format Rust code
	cargo fmt

.PHONY: lint
lint: node_modules ## Rust and web lint, typecheck
	cargo fmt --check
	cargo clippy --locked --all-targets -- -D warnings
	npm run lint
	npm run typecheck

.PHONY: licenses
licenses: node_modules ## Dependency license and advisory checks
	cargo deny check
	npm run licenses

.PHONY: api
api: node_modules ## Regenerate web/openapi.json and the TypeScript API types
	./scripts/gen-api.sh

.PHONY: api-check
api-check: node_modules ## Fail if the generated API files are out of date
	./scripts/gen-api.sh --check

.PHONY: sqlx
sqlx: ## Regenerate server/.sqlx after adding or changing a query
	mkdir -p target/sqlx
	rm -f target/sqlx/dev.sqlite*
	DATABASE_URL=$(SQLX_DB_URL) sqlx database create
	DATABASE_URL=$(SQLX_DB_URL) sqlx migrate run --source server/migrations
	cd server && DATABASE_URL=$(SQLX_DB_URL) cargo sqlx prepare -- --all-targets

.PHONY: sqlx-check
sqlx-check: ## Fail if server/.sqlx is out of date
	mkdir -p target/sqlx
	rm -f target/sqlx/dev.sqlite*
	DATABASE_URL=$(SQLX_DB_URL) sqlx database create
	DATABASE_URL=$(SQLX_DB_URL) sqlx migrate run --source server/migrations
	cd server && DATABASE_URL=$(SQLX_DB_URL) cargo sqlx prepare --check -- --all-targets

.PHONY: check
check: lint test licenses api-check sqlx-check e2e ## Everything CI runs except the image

.PHONY: binaries
binaries: web ## Static musl binaries into dist/ (needs zig and cargo-zigbuild)
	@command -v zig >/dev/null || { echo "zig not found: install zig 0.15.2" >&2; exit 1; }
	@command -v cargo-zigbuild >/dev/null || { echo "cargo-zigbuild not found: cargo install cargo-zigbuild --locked" >&2; exit 1; }
	./scripts/build-binaries.sh

.PHONY: image
image: binaries ## Build the amd64 image (override IMAGE)
	docker buildx build --platform linux/amd64 -f memory-card/Dockerfile -t $(IMAGE) --load .

.PHONY: smoke
smoke: image ## Run the image like Home Assistant and check start, UI, API and stop
	./scripts/smoke-image.sh $(IMAGE)

.PHONY: reset-db
reset-db: ## Delete the local database and covers, keeping options.json
	rm -rf $(DATA_DIR)/db $(DATA_DIR)/covers $(DATA_DIR)/tmp

.PHONY: clean
clean: ## Remove build output and local data
	cargo clean
	rm -rf web/dist dist $(DATA_DIR) e2e/test-results e2e/playwright-report
