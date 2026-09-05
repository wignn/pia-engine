"use client";

const components = [
  {
    num: "01",
    name: "Ingestion & Event Backbone",
    tech: "Rust • NATS JetStream • Redis",
    desc: "Ingestion Gateway connects vendor feeds (Tiingo, Finnhub, SEC Filings, & Central Banks) to NATS JetStream event-bus without overhead on REST APIs.",
  },
  {
    num: "02",
    name: "Core Domain Services",
    tech: "Rust • Tokio • Axum • SQLx",
    desc: "Independent microservices manage real-time tick data, macro news/document aggregation, quantitative calculations, and multi-tenant SaaS Control Plane.",
  },
  {
    num: "03",
    name: "Dedicated Gateways",
    tech: "Axum • Tower • WebSocket",
    desc: "API Gateway with JWT authentication & tiered rate-limiting, coupled with Realtime WebSocket Gateway for live price ticks & instant alerts.",
  },
  {
    num: "04",
    name: "AI & NLP Analyzer Runtime",
    tech: "Python • FastAPI • FinBERT",
    desc: "Dedicated NLP service for financial text sentiment scoring, entity recognition, and instant price movement attribution ('Why Did It Move').",
  },
  {
    num: "05",
    name: "Multi-Database Data Layer",
    tech: "ClickHouse • PostgreSQL • Redis",
    desc: "ClickHouse for high-throughput time-series ticks & candlestick history, PostgreSQL for SaaS state & news archive, and Redis for hot-caching.",
  },
  {
    num: "06",
    name: "Multi-Channel Distribution",
    tech: "Web Dashboard • Rust Discord Bot",
    desc: "Seamless data delivery via Public Web Dashboard, Admin Monitoring Web, and integrated Discord Bot powered by Serenity & Poise.",
  },
];

export function ArchitectureSection() {
  return (
    <section className="arch-section" id="architecture">

      <div className="arch-container">
        <div className="arch-header">
          <p className="eyebrow"><span className="eyebrow-line" />SYSTEM TOPOLOGY</p>
          <h2>HIGH-PERFORMANCE<br /><em>MICROSERVICES ARCHITECTURE</em></h2>
          <p className="arch-sub">
            Architected with event-driven principles and clear layer separation to guarantee minimal latency, multi-tenant scalability, and data durability.
          </p>
        </div>

        <div className="arch-grid">
          {components.map((item) => (
            <div className="arch-card" key={item.num}>
              <div className="arch-card-top">
                <span className="arch-num">{item.num}</span>
                <span className="arch-tech">{item.tech}</span>
              </div>
              <h3>{item.name}</h3>
              <p>{item.desc}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}