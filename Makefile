INFRA_DIR ?= infra/compose
ENGINE ?= podman
COMPOSE = $(ENGINE) compose -f local.yml

.PHONY: help up down restart logs ps build pull run-podman down-podman up-docker down-docker

help:
	@echo "Available ATLSD commands:"
	@echo "  make up           - Start services in background (default: podman)"
	@echo "  make down         - Stop services and remove volumes"
	@echo "  make restart      - Recreate full stack"
	@echo "  make logs         - Follow service logs"
	@echo "  make ps           - Show running services"
	@echo "  make build        - Build images"
	@echo "  make pull         - Pull latest images"
	@echo "  make up ENGINE=docker   - Use docker compose"
	@echo "  make down ENGINE=docker - Use docker compose"

up:
	cd $(INFRA_DIR) && $(COMPOSE) up -d

down:
	cd $(INFRA_DIR) && $(COMPOSE) down -v

restart: down up

logs:
	cd $(INFRA_DIR) && $(COMPOSE) logs -f --tail=200

ps:
	cd $(INFRA_DIR) && $(COMPOSE) ps

build:
	cd $(INFRA_DIR) && $(COMPOSE) build

up-build:
	cd $(INFRA_DIR) && $(COMPOSE) up --build

pull:
	cd $(INFRA_DIR) && $(COMPOSE) pull

# Backward-compatible aliases
run-podman: up
down-podman: down

up-docker: ENGINE=docker
up-docker: up

down-docker: ENGINE=docker
down-docker: down