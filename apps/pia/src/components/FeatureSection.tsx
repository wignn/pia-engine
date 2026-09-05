const features = [
  ["01", "Realtime Market Data Pipeline", "Low-latency Gateway REST API & WebSocket pipeline aggregating global price feeds (SPX, Gold, FX, Crypto)."],
  ["02", "Options Data Pack & Quant Analytics", "Gamma Exposure (GEX) calculation, Put/Call ratios, and Options Chain analytics for dealer support/resistance detection."],
  ["03", "Macro & News Intelligence", "Automated aggregator for Economic Calendar events, SEC Filings (10-K, 10-Q), Central Bank releases, and geopolitical signals."],
  ["04", "AI-Powered Financial Insights", "AI-driven NLP engine for financial sentiment analysis and instant price movement attribution ('Why Did It Move')."],
  ["05", "Multi-Channel Distribution", "Service integration utilizing Rust microservices, WebSocket event-bus, and integrated Discord bot for real-time alerts."],
  ["06", "Rust Microservices Architecture", "High-performance backend built with Rust, Tokio, Serenity/Poise, ClickHouse, NATS JetStream, and PostgreSQL."],
];

export function FeatureSection() {
  return (
    <section className="features relative overflow-hidden" id="features">
      <div className="section-intro relative z-10">
        <p className="eyebrow"><span className="eyebrow-line" />SYSTEM CAPABILITIES</p>
        <h2>ENGINE ARCHITECTURE &<br /><em>FINANCIAL INTELLIGENCE</em></h2>
      </div>
      <div className="feature-list relative z-10 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
        {features.map(([number, title, body]) => (
          <article className="feature-card" key={number}>
            <span className="feature-number">{number}</span>
            <h3>{title}</h3>
            <p>{body}</p>
            <span className="feature-arrow" aria-hidden="true">↗</span>
          </article>
        ))}
      </div>
    </section>
  );
}