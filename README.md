# ATLSD Platform

ATLSD is a real-time market intelligence and data distribution platform for financial news, economic calendar events, market signals, and tenant-aware data delivery. The platform aggregates multiple financial information sources, enriches incoming content with Natural Language Processing (NLP), and distributes structured market intelligence through REST APIs, WebSockets, dashboards, and bot integrations.


The system is designed around four main goals:

* **Low-latency market intelligence delivery** for dashboards, bots, and API consumers.
* **Reliable multi-source ingestion** from news feeds, market calendars, equity feeds, and social sources.
* **AI-assisted sentiment enrichment** using a dedicated NLP analyzer service.
* **SaaS-ready tenant governance** with plans, limits, API keys, quotas, and entitlement checks.

---

## 1. Platform Architecture

The architecture follows a service-oriented model. The Rust Core Engine acts as the primary ingestion, processing, API, and WebSocket gateway. The Control Plane manages tenants, subscriptions, plans, API keys, and entitlement rules. The Python NLP Analyzer is isolated as a dedicated AI inference service so that model execution can scale independently from the real-time data plane.

```mermaid
flowchart LR
    %% =====================================================
    %% External Clients
    %% =====================================================
    subgraph Clients["Client Channels"]
        Web["SvelteKit Web App\nDashboard + NLP Sandbox"]
        Bots["Telegram / Discord Bots\nReal-time Alerts"]
        API["External API Consumers\nREST + WebSocket Clients"]
    end

    %% =====================================================
    %% External Data Sources
    %% =====================================================
    subgraph Sources["External Market Data Sources"]
        Forex["Forex & Global News Feeds"]
        Stocks["Stock & Equity News Feeds"]
        Calendar["Economic Calendar Sources"]
        Social["X / Twitter via RSSHub"]
    end

    %% =====================================================
    %% Application Layer
    %% =====================================================
    subgraph App["Application Services - Docker Compose"]
        Core["Rust Core Engine\nAxum + Tokio\nIngestion, APIs, WebSockets"]
        Control["Rust Control Plane\nTenant Admin, Plans, API Keys"]
        Analyzer["Python NLP Analyzer\nFastAPI + FinBERT"]
        BotSvc["Bot Notification Service\nTelegram / Discord Delivery"]
    end

    %% =====================================================
    %% Data Layer
    %% =====================================================
    subgraph Data["Persistence, Cache & Event Backbone"]
        PG[("PostgreSQL\nTenants, Articles, Events, Sentiment")]
        Redis[("Redis\nStreams, Pub/Sub, Rate Limits")]
    end

    %% =====================================================
    %% Ingestion Flow
    %% =====================================================
    Forex -->|Poll / scrape feeds| Core
    Stocks -->|Poll market news| Core
    Calendar -->|Poll calendar events| Core
    Social -->|RSSHub polling| Core

    %% =====================================================
    %% Processing Flow
    %% =====================================================
    Core -->|Normalize + deduplicate| Core
    Core -->|Send title/content for inference| Analyzer
    Analyzer -->|Sentiment, confidence, highlights| Core
    Core -->|Persist normalized records| PG
    Core -->|Publish processed events| Redis

    %% =====================================================
    %% Governance Flow
    %% =====================================================
    Control -->|Manage tenants, plans, keys| PG
    Control -->|Sync quotas and entitlement cache| Redis
    Core -->|Validate API key/JWT + quota| Redis
    Core -->|Read tenant entitlements| PG

    %% =====================================================
    %% Distribution Flow
    %% =====================================================
    Web -->|REST + WebSocket| Core
    API -->|REST + WebSocket| Core
    Bots -->|Webhook / WebSocket subscription| Core
    Redis -->|Event fan-out| Core
    Redis -->|Alert stream| BotSvc
    BotSvc -->|Push notifications| Bots

    %% =====================================================
    %% Styling
    %% =====================================================
    classDef source fill:#FFF7ED,stroke:#FB923C,color:#7C2D12,stroke-width:1px;
    classDef client fill:#EFF6FF,stroke:#2563EB,color:#1E3A8A,stroke-width:1px;
    classDef service fill:#F8FAFC,stroke:#334155,color:#0F172A,stroke-width:1.5px;
    classDef ai fill:#F5F3FF,stroke:#7C3AED,color:#3B0764,stroke-width:1.5px;
    classDef data fill:#ECFDF5,stroke:#059669,color:#064E3B,stroke-width:1.5px;

    class Forex,Stocks,Calendar,Social source;
    class Web,Bots,API client;
    class Core,Control,BotSvc service;
    class Analyzer ai;
    class PG,Redis data;
```

---

## 2. End-to-End Data Flow

The platform processes market information through six coordinated stages: ingestion, normalization, enrichment, persistence, publication, and distribution.

```mermaid
sequenceDiagram
    autonumber
    participant Source as External Feed
    participant Core as Rust Core Engine
    participant NLP as NLP Analyzer
    participant DB as PostgreSQL
    participant Cache as Redis Streams
    participant Client as Web / Bot / API Client

    Source->>Core: New article, calendar event, or social update
    Core->>Core: Parse, sanitize, normalize, deduplicate
    Core->>NLP: Analyze title and content
    NLP-->>Core: Sentiment label, confidence scores, highlights
    Core->>DB: Store event, metadata, sentiment, entities
    Core->>Cache: Publish processed market event
    Client->>Core: Connect with JWT or API key
    Core->>Cache: Validate quota and connection limits
    Core-->>Client: Stream real-time updates via WebSocket
```

---

## 3. Key Platform Capabilities

### 3.1 Multi-Source Ingestion

The Core Engine runs scheduled workers that collect and normalize market information from multiple sources:

* Forex and global market news feeds.
* Regional stock and equity news feeds.
* Economic calendar sources.
* X / Twitter accounts aggregated through RSSHub.
* Tenant-specific social watchlists merged with global source configuration.

Each ingested item is normalized into a consistent internal event format before further processing.

### 3.2 NLP Sentiment Pipeline

When a news article or social post is fetched, the Core Engine sends sanitized text to the Python NLP Analyzer. The analyzer uses the FinBERT tone model to classify financial sentiment as **positive**, **negative**, or **neutral**.

The NLP output includes:

* Sentiment label.
* Confidence score.
* Class probability distribution.
* Sentence-level highlights.
* Extracted financial entities such as currencies, tickers, or instruments.

### 3.3 Real-Time Distribution

Processed events are published into Redis Streams and delivered to connected clients through WebSockets. This allows dashboards, bots, and external API consumers to receive market updates with low latency.

Typical distribution channels include:

* SvelteKit dashboard live feed.
* Telegram and Discord alert bots.
* REST API consumers.
* WebSocket subscribers.

### 3.4 Tenant Governance

The Control Plane manages SaaS administration and authorization logic. It stores tenant profiles, plan definitions, API keys, subscription state, and entitlement rules in PostgreSQL.

The Core Engine validates access at runtime using JWTs or hashed API keys. Usage counters and connection limits are cached in Redis to reduce database load and support high-frequency checks.

Governed resources include:

* Daily REST API request limits.
* Maximum concurrent WebSocket connections.
* Access to restricted symbols, tickers, or asset classes.
* Plan-specific feature availability.
* Tenant-specific source subscriptions.

---

## 4. Core Components

| Component     | Technology                        | Responsibility                                                                         |
| ------------- | --------------------------------- | -------------------------------------------------------------------------------------- |
| Core Engine   | Rust, Axum, Tokio                 | Ingestion, API gateway, WebSocket hub, event normalization, tenant runtime enforcement |
| Control Plane | Rust, Axum                        | Tenant management, plans, subscriptions, API keys, entitlements                        |
| NLP Analyzer  | Python, FastAPI, PyTorch, FinBERT | Financial sentiment inference, confidence scoring, text highlights, entity extraction  |
| Web App       | SvelteKit, Tailwind CSS, Vercel   | Market intelligence dashboard, live feed, NLP sandbox                                  |
| Bot Service   | Telegram / Discord integrations   | Push alerts and user-facing notifications                                              |
| PostgreSQL    | SQL database                      | Durable storage for tenants, plans, articles, events, sentiment results, audit data    |
| Redis         | Streams, Pub/Sub, counters        | Event fan-out, rate limiting, quota tracking, real-time cache                          |

---

## 5. Suggested Database Model

The following logical schema supports the core SaaS and market intelligence features.

```mermaid
erDiagram
    TENANTS ||--o{ API_KEYS : owns
    TENANTS ||--o{ SUBSCRIPTIONS : has
    PLANS ||--o{ SUBSCRIPTIONS : defines
    TENANTS ||--o{ TENANT_SOURCES : configures
    MARKET_EVENTS ||--o| SENTIMENT_RESULTS : enriched_by
    MARKET_EVENTS ||--o{ EVENT_ENTITIES : contains
    TENANTS ||--o{ USAGE_COUNTERS : consumes

    TENANTS {
        uuid id PK
        string name
        string status
        timestamp created_at
        timestamp updated_at
    }

    PLANS {
        uuid id PK
        string name
        int daily_request_limit
        int max_ws_connections
        jsonb feature_flags
        jsonb restricted_symbols
    }

    SUBSCRIPTIONS {
        uuid id PK
        uuid tenant_id FK
        uuid plan_id FK
        string status
        timestamp current_period_start
        timestamp current_period_end
    }

    API_KEYS {
        uuid id PK
        uuid tenant_id FK
        string key_hash
        string label
        timestamp last_used_at
        timestamp revoked_at
    }

    TENANT_SOURCES {
        uuid id PK
        uuid tenant_id FK
        string source_type
        string source_ref
        bool enabled
        jsonb config
    }

    MARKET_EVENTS {
        uuid id PK
        string source_type
        string source_url
        string title
        text content
        timestamp published_at
        timestamp ingested_at
        string dedupe_hash
    }

    SENTIMENT_RESULTS {
        uuid id PK
        uuid market_event_id FK
        string label
        float confidence
        jsonb probabilities
        jsonb highlights
        string model_version
    }

    EVENT_ENTITIES {
        uuid id PK
        uuid market_event_id FK
        string entity_type
        string symbol
        string display_name
    }

    USAGE_COUNTERS {
        uuid id PK
        uuid tenant_id FK
        string counter_type
        int counter_value
        timestamp window_start
        timestamp window_end
    }
```

---

## 6. Repository Structure

```text
├── apps/
│   └── public-web/              # SvelteKit dashboard application
│
├── services/
│   ├── core/                    # Rust core engine: ingestion, APIs, WebSockets
│   ├── control-plane/           # Rust SaaS admin: tenants, plans, API keys
│   ├── analyzer/                # Python FastAPI service: FinBERT NLP model
│   ├── bot/                     # Telegram and Discord notification agent
│   └── ingestion-gateway/       # Optional high-speed ingestion adapter
│
├── db/
│   └── migrations/              # SQLx migrations and schema definitions
│
└── infra/
    ├── compose/                 # Docker Compose environments
    ├── docker/                  # Component-specific Dockerfiles
    └── env/                     # Environment variable templates
```

---

## 7. Local Development

### Prerequisites

* Docker and Docker Compose.
* Node.js and Bun for frontend development.
* Rust toolchain for backend development.
* Python runtime for the NLP analyzer if running outside Docker.

### Run the Local Stack

```bash
cp infra/env/.env.core.example infra/env/.env.core
# Fill in the required environment variables.

docker compose -f infra/compose/local.yml up --build
```

Local service endpoints:

| Service           | URL                     |
| ----------------- | ----------------------- |
| Core Service API  | `http://localhost:8090` |
| Control Plane API | `http://localhost:8081` |
| NLP Analyzer API  | `http://localhost:5000` |

### Run the Frontend Locally

```bash
cd apps/public-web
bun install
bun run dev
```

Open the dashboard at:

```text
http://localhost:5173
```

---

## 8. Production Notes

For production, the platform should be deployed with clear separation between the real-time data plane and the administrative control plane.

Recommended production considerations:

* Run Core Engine replicas behind a load balancer.
* Use managed PostgreSQL with automated backups and point-in-time recovery.
* Use managed Redis or a highly available Redis-compatible service.
* Isolate the NLP Analyzer with independent CPU/GPU scaling.
* Add structured logging, tracing, and metrics across all services.
* Protect APIs with JWT validation, API key hashing, tenant-level rate limits, and audit logging.
* Define Redis Stream retention policies to prevent unbounded memory growth.
* Add dead-letter handling for failed ingestion or NLP processing jobs.

---

## 9. Observability Checklist

| Area            | Recommended Metric / Signal                                              |
| --------------- | ------------------------------------------------------------------------ |
| Ingestion       | Feed polling latency, failed fetch count, deduplication rate             |
| NLP             | Inference latency, model error rate, queue depth, sentiment distribution |
| API             | Request latency, status codes, tenant-level usage                        |
| WebSocket       | Active connections, messages delivered, dropped connections              |
| Redis           | Stream length, consumer lag, memory usage                                |
| PostgreSQL      | Query latency, connection pool saturation, storage growth                |
| SaaS Governance | Quota violations, rejected API keys, plan entitlement failures           |
