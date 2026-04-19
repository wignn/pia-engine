# ATLSD Platform

ATLSD is a market information aggregation platform that combines multi-source data collection, content processing, real-time distribution, and tenant-based access control within a unified architecture. This repository is designed as the foundation for a data product that can evolve into a commercially operated subscription service.

## Product Overview

The platform separates responsibilities into three primary layers:

- **Core engine** for market data ingestion, normalization, processing, and distribution.
- **Control plane** for user identity, API key management, tenant configuration, service plans, and usage metrics.
- **Web portal** as the interface for account and service configuration management.

This separation allows each layer to evolve independently without compromising domain consistency or inter-service API contracts.

## Architectural Characteristics

ATLSD follows a service-oriented architecture with clear context boundaries:

- **Compute layer (Rust, Axum, Tokio):** handles HTTP endpoints, WebSocket channels, schedulers, and asynchronous pipelines.
- **Persistence layer (PostgreSQL):** stores news content, tenant data, API keys, service plans, and usage logs.
- **Realtime coordination (Redis, optional):** synchronizes tenant changes and enforces low-latency daily quota control.
- **External data ingress:** RSS feeds, article scraping, X feeds via RSSHub, economic calendar events, and TradingView market streams.

This design prioritizes throughput, tenant isolation, and operational resilience under polling workloads and real-time event fanout.

## Core Components

### 1. Core Service

The core service is the primary data engine and runs multiple parallel pipelines:

- **News pipeline** to consume RSS feeds, scrape article content, clean HTML, and publish processed content to real-time channels.
- **Stock pipeline** to collect equity-related news and deliver timely updates.
- **Calendar pipeline** to monitor scheduled economic events.
- **Twitter/X pipeline** powered by RSSHub with username aggregation from both global and tenant configurations.
- **Price stream pipeline** to consume TradingView streams with volatility spike detection.

The core also exposes public HTTP endpoints, private key-protected endpoints, and multiple WebSocket channels for low-latency event distribution.

### 2. Control Plane

The control plane acts as the SaaS administration center:

- Registration, login, account verification, and JWT-based identity.
- Support for external OAuth providers.
- API key creation, update, and revocation.
- Per-user tenant configuration management.
- Service plan catalog and plan upgrade flows.
- Usage summary and API consumption history.

This component is the authoritative source of tenant entitlements, which are synchronized to the core service.

### 3. Portal

The portal is built with React and Vite as the product-facing interface, communicating with the control plane via the v1 API. It focuses on account management, API credentials, tenant configuration, and visibility into plans and usage.

## Multi-Tenant Model

ATLSD applies a tenant-aware model at the request level:

- Access validation can use either **JWT Bearer** tokens or **API keys**.
- Tenant context includes user identity, active plan, and service limits.
- Plan constraints include daily request quotas, WebSocket connection limits, per-minute rate limits, and feature capabilities.
- Tenant configuration (such as X usernames and TradingView symbols) is loaded from centralized storage and cached in memory for fast access.

With this approach, policy enforcement remains consistent across both API and real-time distribution layers.

## Data Domain and Schema

The database schema reflects two primary domains:

- **Market content domain:** news sources, articles, news analyses, and stock news data.
- **SaaS domain:** users, api_keys, tenant_configs, usage_logs, plans, and OAuth account relations.

This modeling strategy cleanly separates editorial and market data concerns from business and access-control concerns, without duplicating control logic.

## Security and Access Control

The platform applies layered controls:

- SHA-256 API key hashing and hash-only storage.
- Middleware-based endpoint authorization for public, optional-auth, and strict-auth modes.
- JWT token support for portal user sessions.
- Tenant entitlement isolation based on active plan and account status.
- Quota protection through daily counters in Redis (fail-open behavior if Redis is unavailable).

This implementation balances practical security, performance, and service availability.

## Observability and Operations

ATLSD adopts production-oriented observability patterns:

- Structured JSON logging for Rust services.
- Scheduled background tasks for pipelines and tenant registry synchronization.
- Batched usage log inserts for database I/O efficiency.
- Graceful HTTP server shutdown to preserve state consistency during termination.

These patterns support incident investigation, performance analysis, and stable operation under increasing traffic.

## Infrastructure Integration

The infrastructure layer composes containers for key components:

- PostgreSQL as the primary datastore.
- Core service and control plane as separate backend services.
- Portal as a containerized frontend application.
- RSSHub as the social feed source.
- Cloudflare WARP as a networking component in specific deployment scenarios.

This composition provides a consistent deployment foundation across development and production environments.

## Delivery and Repository Standards

The repository is managed as a multi-crate Rust workspace with a separate frontend application. CI/CD pipelines publish container images per major component, enabling service version traceability by commit, tag, and stable release channels.

Code organization emphasizes clear domain boundaries, auditability of changes, and long-term product maintainability.

## Repository Structure

- `core/` - data aggregation engine, public API, access middleware, pipelines, WebSocket, and tenant registry.
- `control-plane/` - identity domain, SaaS management APIs, service plans, and tenant configuration synchronization.
- `portal/` - React frontend application for management interfaces.
- `infrastructure/` - container composition, service Dockerfiles, and environment configuration.
- `.github/workflows/` - automated build and container publishing pipelines.

## Platform Positioning

ATLSD is positioned as a foundational real-time data platform that combines **data ingestion**, **tenant-aware API governance**, and an **operational SaaS product model** in a single integrated codebase. Its design focus is domain accuracy, pipeline scalability, and readiness for plan-based service commercialization.