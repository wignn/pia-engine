# Tenancy Architecture

API keys are validated by the api-gateway (REST) and realtime-gateway (WS)
against the tenant registry (60s cache; revocation via reload). Plans gate
quotas, stream subscriptions, and per-key WS connection limits (enforced with
Redis slot counters). Data filtering is per-connection at the realtime
gateway: every gateway process sees all market/news data and filters by the
client's subscribed streams — per-tenant data partitioning is planned for
Fase 5 of `REMEDIATION_PLAN.md` (database decomposition per domain).

Internal service-to-service calls use the shared `INTERNAL_API_KEY` header
(constant-time compare); public endpoints are only api-gateway, realtime-gateway
(WS), control-plane (admin), and public-web.
