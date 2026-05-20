# ATLSD Architecture Overview

ATLSD is designed as a service-oriented real-time market data platform.

## Architecture Layers
1. **Frontend Applications (`apps/`)**:
   - `portal`: React-based dashboard for SaaS tenant control.
   - `public-web`: Svelte-based public marketing site.
2. **Services (`services/`)**:
   - `core`: Ingestion pipeline, caching engine, public/private WS.
   - `control-plane`: SaaS user billing, subscription plans, JWT keys.
   - `ingestion-gateway`: Feeds market ticks into Redis.
   - `bot`: Real-time notification gateway (Discord/Telegram).
   - `analyzer`: ML-based financial tone analyzer.
