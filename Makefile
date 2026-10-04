SHELL := /bin/bash
DATABASE_URL ?= postgres://$(USER)@localhost:5432/fsp_hack
PG_ADMIN_URL ?= postgres://postgres@localhost:5432
MIGRATIONS := $(sort $(wildcard db/migrations/*.sql))
COMPOSE := docker compose -f deploy/compose.yml

.PHONY: db gen-db api fsp-mock gen-api web test lint validate up down logs

db:
	@psql "$(PG_ADMIN_URL)/postgres" -tAc "select 1 from pg_database where datname='fsp_hack'" | grep -q 1 \
		|| psql "$(PG_ADMIN_URL)/postgres" -c "create database fsp_hack owner \"$(USER)\""

gen-db:
	clorinde fresh -u "$(PG_ADMIN_URL)" $(MIGRATIONS)

api:
	DATABASE_URL="$(DATABASE_URL)" PUBLIC_URL=$${PUBLIC_URL:-http://localhost:3000} FSP_DEV_LINK=$${FSP_DEV_LINK:-true} \
		cargo run -p api

fsp-mock:
	cargo run -p fsp-mock

gen-api:
	cargo run -q -p api -- openapi > web/openapi.json
	cd web && npx openapi-typescript openapi.json -o src/api/schema.d.ts

web:
	cd web && npm run dev

test:
	cargo test --workspace
	cd web && npx tsc --noEmit

lint:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets -- -D warnings
	cd web && npm run lint

validate:
	cargo run --release -p validate -- --out docs/validation

up:
	$(COMPOSE) --profile full up -d --build

down:
	$(COMPOSE) --profile full down

logs:
	$(COMPOSE) logs -f api
