SHELL := /bin/bash
DATABASE_URL ?= postgres://$(USER)@localhost:5432/fsp_hack
PG_ADMIN_URL ?= postgres://postgres@localhost:5432
# sqlx::test creates a throwaway database per test on this server; the role needs CREATEDB.
TEST_DATABASE_URL ?= $(PG_ADMIN_URL)/postgres
COMPOSE := $(shell docker compose version >/dev/null 2>&1 && echo 'docker compose' || echo docker-compose) -f deploy/compose.yml

.PHONY: db db-prepare api fsp-mock ats-mock gen-api web test lint validate docs up down logs

db:
	@psql "$(PG_ADMIN_URL)/postgres" -tAc "select 1 from pg_database where datname='fsp_hack'" | grep -q 1 \
		|| psql "$(PG_ADMIN_URL)/postgres" -c "create database fsp_hack owner \"$(USER)\""

db-prepare:
	sqlx migrate run --source db/migrations --database-url "$(DATABASE_URL)"
	cargo sqlx prepare --workspace --database-url "$(DATABASE_URL)" -- --all-targets

api:
	DATABASE_URL="$(DATABASE_URL)" PUBLIC_URL=$${PUBLIC_URL:-http://localhost:3000} FSP_DEV_LINK=$${FSP_DEV_LINK:-true} \
		cargo run -p api

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
	DATABASE_URL="$(TEST_DATABASE_URL)" cargo test --workspace
	cd web && npx tsc --noEmit

lint:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets -- -D warnings
	cd web && npm run lint

validate:
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
