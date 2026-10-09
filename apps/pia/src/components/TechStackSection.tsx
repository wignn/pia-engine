"use client";

const tags = [
  "Rust",
  "Tokio",
  "Serenity / Poise",
  "Axum",
  "REST & WebSocket API",
  "ClickHouse",
  "PostgreSQL / SQLx",
  "NATS JetStream",
  "Redis",
  "Python FastAPI",
  "FinBERT NLP",
  "Docker & CI/CD",
];

export function TechStackSection() {
  return (
    <section className="tech-section" id="techstack" aria-labelledby="tech-heading">
      <div className="tech-container">
        <div className="tech-header">
          <p className="eyebrow"><span className="eyebrow-line" />INFRASTRUCTURE & ECOSYSTEM</p>
          <h2 id="tech-heading">BUILT WITH THE BEST<br /><em>TECH STACK</em></h2>
        </div>

        <div className="tech-tags" role="list">
          {tags.map((tag) => (
            <span className="tech-tag" key={tag} role="listitem">
              <span className="dot" aria-hidden="true" />
              {tag}
            </span>
          ))}
        </div>
      </div>
    </section>
  );
}