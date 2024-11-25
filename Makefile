DB_USER ?= $(or $(DATABASE_USER),postgres)
DB_NAME ?= $(or $(DATABASE_NAME),postgres)
DB_HOST ?= $(or $(DATABASE_HOST),localhost)
DB_PORT ?= $(or $(DATABASE_PORT),5432)
DB_PASSWORD ?= $(or $(DATABASE_PASSWORD), postgres)

CONTAINER_NAME ?= database

.PHONY: install start-database dump-schema load-schema setup-db migrate up down test-integration
# Directory for SQL files
SQL_DIR := ./db

install:
	brew install podman-desktop
	brew install podman-compose


start-database:
	@echo "Waiting for database to be healthy..."
	podman-compose up -d database 
	@sleep 10

stop-database:
	podman-compose down --volumes wait database

run-migrate: start-database migrate-database dump-schema

migrate-database:
	diesel migration run --migration-dir=./db/migrations

dump-schema:
	@mkdir -p $(SQL_DIR)
	@echo "Dumping schema to $(SQL_DIR)/schema.sql"
	@podman-compose exec -T $(CONTAINER_NAME) pg_dump -U $(DB_USER) -d $(DB_NAME) --schema-only > $(SQL_DIR)/schema.sql

load-schema:
	@echo "Loading schema from $(SQL_DIR)/schema.sql"
	@podman-compose exec -T $(CONTAINER_NAME) psql -U $(DB_USER) -d $(DB_NAME) < $(SQL_DIR)/schema.sql

start-backend:
	@echo "Waiting for backend service to run"
	podman-compose up -d backend 

up: start-database load-schema 

down:
	podman pod rm -f pod_newsletter_service

test-lib:
	cargo nextest run --lib

test-integration:
	cargo nextest run --test '*'
