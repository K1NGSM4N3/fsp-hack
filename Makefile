SHELL := /bin/bash
DATABASE_URL ?= postgres://$(USER)@localhost:5432/fsp_hack
PG_ADMIN_URL ?= postgres://postgres@localhost:5432
DOCKER_COMPOSE := $(shell docker compose version >/dev/null 2>&1 && echo 'docker compose' || echo docker-compose)
COMPOSE := $(DOCKER_COMPOSE) -f deploy/compose.yml
# The black-box stack: same images and ports as `make up`, its own project and volumes, a fresh seed
# on every run. Keycloak and observability are left out: no test goes through them.
# `--profile full` lets `down` reach the profiled mocks too; `up` starts only BLACKBOX_SERVICES.
BLACKBOX := $(DOCKER_COMPOSE) -p fsp-hack-blackbox -f deploy/compose.yml -f deploy/compose.blackbox.yml --profile full
BLACKBOX_SERVICES := caddy web api embedder postgres valkey mailpit fsp-mock ats-mock
MODEL ?= multilingual-e5-small

.PHONY: db db-prepare api fsp-mock ats-mock embedder model gen-api web test blackbox blackbox-degraded blackbox-down lint validate docs up down logs

db:
	@psql "$(PG_ADMIN_URL)/postgres" -tAc "select 1 from pg_database where datname='fsp_hack'" | grep -q 1 \
		|| psql "$(PG_ADMIN_URL)/postgres" -c "create database fsp_hack owner \"$(USER)\""

db-prepare:
	sqlx migrate run --source db/migrations --database-url "$(DATABASE_URL)"
	cargo sqlx prepare --workspace --database-url "$(DATABASE_URL)" -- --all-targets

api:
	DATABASE_URL="$(DATABASE_URL)" PUBLIC_URL=$${PUBLIC_URL:-http://localhost:3000} FSP_DEV_LINK=$${FSP_DEV_LINK:-true} \
		EMBEDDER_URL=$${EMBEDDER_URL:-http://localhost:8093} cargo run -p api

# Embedding model weights at the pinned revision, sha256-checked (models/, gitignored).
model:
	sh crates/embedder/fetch-model.sh $(MODEL) models

embedder: model
	EMBEDDER_BIND=$${EMBEDDER_BIND:-127.0.0.1:8093} EMBEDDER_MODEL=$(MODEL) EMBEDDER_MODEL_DIR=models/$(MODEL) \
		cargo run --release -p embedder

fsp-mock:
	cargo run -p fsp-mock

ats-mock:
	ATS_MOCK_BIND=$${ATS_MOCK_BIND:-127.0.0.1:8082} ATS_API_URL=$${ATS_API_URL:-http://localhost:8000} \
		ATS_DEMO_WEBHOOK_SECRET=$${ATS_DEMO_WEBHOOK_SECRET:-whsec_demo_ats_secret_do_not_use_in_production} \
		cargo run -p ats-mock

gen-api:
	cargo run -q -p api -- openapi > web/openapi.json
	cd web && npx openapi-typescript openapi.json -o src/api/schema.d.ts

web:
	cd web && npm run dev

test:
	cargo test --workspace
	cd web && npx tsc --noEmit

blackbox:
	test -f .env || cp .env.example .env
	$(COMPOSE) --profile full stop
	$(BLACKBOX) down -v --remove-orphans
	$(BLACKBOX) up -d --build --wait $(BLACKBOX_SERVICES)
	cd tests/blackbox && npm ci --no-audit --no-fund && npx playwright install chromium && npx tsc --noEmit
	cd tests/blackbox && npx playwright test $(ARGS); status=$$?; \
		if [ "$(DOWN)" = "1" ]; then cd ../.. && $(BLACKBOX) down -v --remove-orphans; fi; exit $$status

# After `make blackbox`, on its stack: the embedder is stopped and the app must keep working on
# keywords (tests/blackbox/degraded). The container is stopped here, not by the test.
blackbox-degraded:
	$(BLACKBOX) stop embedder
	cd tests/blackbox && BLACKBOX_DEGRADED=1 npx playwright test --project=degraded; status=$$?; \
		cd ../.. && $(BLACKBOX) start embedder; exit $$status

blackbox-down:
	$(BLACKBOX) down -v --remove-orphans

lint:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets -- -D warnings
	cd web && npm run lint

validate: model
	cargo run --release -p validate -- --out docs/validation

docs:
	pandoc docs/solution.md -o docs/solution.pdf --pdf-engine=typst --pdf-engine-opt=--font-path=crates/api/assets/fonts --metadata-file=docs/pdf-meta.yaml --toc --toc-depth=2
	pandoc docs/solution.md -o docs/solution.docx --metadata-file=docs/pdf-meta.yaml --toc --toc-depth=2

up:
	$(COMPOSE) --profile full up -d --build

down:
	$(COMPOSE) --profile full down

logs:
	$(COMPOSE) logs -f api
