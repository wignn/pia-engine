INFRA_DIR ?= infra/compose
ENV_DIR   ?= infra/env
ENGINE    ?= docker
COLOR     ?= $(shell cat $(ENV_DIR)/active_target 2>/dev/null || echo blue)

COMPOSE = $(ENGINE) compose

APP_ARGS  = -p atlsd-$(COLOR)  -f $(INFRA_DIR)/prod.app.yml  --env-file $(ENV_DIR)/.env.shared --env-file $(ENV_DIR)/.env.$(COLOR)
EDGE_ARGS = -p atlsd-edge      -f $(INFRA_DIR)/prod.edge.yml --env-file $(ENV_DIR)/.env.shared
INFRA_ARGS= -p atlsd-infra     -f $(INFRA_DIR)/prod.infra.yml

.PHONY: help up up-infra up-edge up-app down down-all ps logs pull deploy status color help-colors

help:
	@echo "ATLSD Blue/Green commands (active color: $(COLOR)):"
	@echo "  make deploy      - full blue/green deployment (recommended)"
	@echo "  make up          - ensure infra + edge + app stack ($(COLOR))"
	@echo "  make up-infra    - start shared datastores (postgres/ch/redis/nats)"
	@echo "  make up-edge     - start edge singletons (control-plane/analyzer/bot/router)"
	@echo "  make up-app      - start colored app stack ($(COLOR))"
	@echo "  make down        - stop colored app stack ($(COLOR))"
	@echo "  make down-all    - stop everything incl. infra (DATA STAYS)"
	@echo "  make ps          - status of all three stacks"
	@echo "  make logs S=api-gateway   - follow logs of an app-stack service"
	@echo "  make pull        - pull latest images for app+edge"
	@echo "  make color COLOR=green  - override target color for one command"

deploy:
	bash infra/scripts/deploy-blue-green.sh

up: up-infra up-edge up-app

up-infra:
	cd $(INFRA_DIR) && $(COMPOSE) $(INFRA_ARGS) up -d

up-edge:
	cd $(INFRA_DIR) && $(COMPOSE) $(EDGE_ARGS) up -d

up-app:
	cd $(INFRA_DIR) && $(COMPOSE) $(APP_ARGS) up -d

down:
	cd $(INFRA_DIR) && $(COMPOSE) $(APP_ARGS) down --remove-orphans

down-all: down
	cd $(INFRA_DIR) && $(COMPOSE) $(EDGE_ARGS) down --remove-orphans
	cd $(INFRA_DIR) && $(COMPOSE) $(INFRA_ARGS) down --remove-orphans

ps:
	cd $(INFRA_DIR) && $(COMPOSE) $(INFRA_ARGS) ps
	cd $(INFRA_DIR) && $(COMPOSE) $(EDGE_ARGS) ps
	cd $(INFRA_DIR) && $(COMPOSE) $(APP_ARGS) ps

logs:
ifeq ($(S),)
	cd $(INFRA_DIR) && $(COMPOSE) $(APP_ARGS) logs -f --tail=200
else
	cd $(INFRA_DIR) && $(COMPOSE) $(APP_ARGS) logs -f --tail=200 $(S)
endif

pull:
	cd $(INFRA_DIR) && $(COMPOSE) $(APP_ARGS) pull --ignore-buildable
	cd $(INFRA_DIR) && $(COMPOSE) $(EDGE_ARGS) pull --ignore-buildable

status:
	@echo "Active color: $(COLOR)"
	@docker ps --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}'

color:
	@echo "$(COLOR)"
