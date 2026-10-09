import Link from "next/link";

const apiBase = "https://api-engine.wign.dev";

const pathways = [
  {
    number: "01",
    title: "Explore the product",
    copy: "Use Public Web to explore market data and the consumer-facing market experience.",
    href: "https://atlsd.wign.dev/",
    link: "Open Public Web",
    label: "PUBLIC WEB",
  },
  {
    number: "02",
    title: "Build with the API",
    copy: "Connect your app with REST endpoints or subscribe to realtime updates through the official SDKs.",
    href: "/portal/docs",
    link: "Read the developer docs",
    label: "DEVELOPER PLATFORM",
  },
  {
    number: "03",
    title: "Manage your workspace",
    copy: "Create an account to manage API credentials, review usage, and see available plans.",
    href: "/portal/account",
    link: "Open SaaS account",
    label: "PIA SAAS",
  },
];

export default function Home() {
  return (
    <main className="pia-home" id="main-content">
      <header className="pia-home-nav">
        <Link className="pia-home-brand" href="/" aria-label="PIA home">
          <span className="pia-brand-mark">P</span>
          <span>PIA <small>MARKET DATA PLATFORM</small></span>
        </Link>
        <nav aria-label="Main navigation">
          <a href="https://atlsd.wign.dev/">Public Web</a>
          <a href="/portal/docs">Developers</a>
          <a href="/portal/account">SaaS account</a>
        </nav>
        <a className="pia-nav-cta" href="/portal/account">Get started <span aria-hidden="true">↗</span></a>
      </header>

      <section className="pia-home-intro">
        <div className="pia-home-copy">
          <p className="pia-home-eyebrow"><span /> MARKET DATA, BUILT FOR PRODUCTS</p>
          <h1>Market infrastructure.<br /><em>Ready to integrate.</em></h1>
          <p className="pia-home-lede">
            PIA brings market data and realtime access together behind one API platform.
            Explore the product, build with an SDK, then manage your integration from one account.
          </p>
          <div className="pia-home-actions">
            <a className="pia-button-primary" href="/portal/docs">Start building <span aria-hidden="true">→</span></a>
            <a className="pia-button-secondary" href="https://atlsd.wign.dev/">Explore Public Web <span aria-hidden="true">↗</span></a>
          </div>
          <p className="pia-home-base">API BASE <code>{apiBase}</code></p>
        </div>
        <aside className="pia-home-diagram" aria-label="How PIA connects products and developers">
          <div className="pia-diagram-top"><span>PLATFORM MAP</span><span>01 — 03</span></div>
          <div className="pia-diagram-source"><span className="pia-diagram-dot" /> PIA MARKET DATA API</div>
          <div className="pia-diagram-connector" />
          <div className="pia-diagram-branches">
            <a href="https://atlsd.wign.dev/"><b>01</b><strong>PUBLIC WEB</strong><small>Explore market experience ↗</small></a>
            <a href="/portal/docs"><b>02</b><strong>SDK + API</strong><small>Integrate REST and realtime →</small></a>
            <a href="/portal/account"><b>03</b><strong>SAAS ACCOUNT</strong><small>Keys, usage, plans →</small></a>
          </div>
          <p className="pia-diagram-foot">ONE PLATFORM · THREE CLEAR PATHS</p>
        </aside>
      </section>

      <section className="pia-home-paths" aria-labelledby="pia-paths-title">
        <div className="pia-section-heading">
          <div><p className="pia-home-eyebrow">CHOOSE YOUR PATH</p><h2 id="pia-paths-title">From discovery to production.</h2></div>
          <p>Public Web is the product experience. PIA is the developer and SaaS layer that connects your own application.</p>
        </div>
        <div className="pia-path-grid">
          {pathways.map((path) => (
            <article className="pia-path" key={path.number}>
              <div className="pia-path-meta"><span>{path.number}</span><span>{path.label}</span></div>
              <h3>{path.title}</h3>
              <p>{path.copy}</p>
              <a href={path.href}>{path.link} <span aria-hidden="true">→</span></a>
            </article>
          ))}
        </div>
      </section>

      <section className="pia-home-quickstart" aria-labelledby="pia-quickstart-title">
        <div className="pia-quickstart-copy">
          <p className="pia-home-eyebrow">QUICKSTART · TYPESCRIPT</p>
          <h2 id="pia-quickstart-title">Your first market request.</h2>
          <p>Install the SDK, create an API key in your account, and use it from your server-side application. Keep the secret out of browser code.</p>
          <a href="/portal/docs">SDK guides and API reference <span aria-hidden="true">→</span></a>
        </div>
        <pre className="pia-code"><code><span className="pia-code-muted">{"// Install: npm install @piaa/sdk"}</span>{"\n"}<span className="pia-code-keyword">import</span> {"{ PiaClient }"} <span className="pia-code-keyword">from</span> <span className="pia-code-string">&quot;@piaa/sdk&quot;</span>{"\n\n"}<span className="pia-code-keyword">const</span> client = <span className="pia-code-keyword">new</span> PiaClient({"{ "}<span className="pia-code-prop">apiKey</span>: process.env.PIA_API_KEY{" }"}){"\n"}<span className="pia-code-keyword">const</span> prices = <span className="pia-code-keyword">await</span> client.market.getPrices(){"\n"}console.log(prices)</code></pre>
      </section>

      <footer className="pia-home-footer">
        <Link className="pia-home-brand" href="/"><span className="pia-brand-mark">P</span><span>PIA <small>MARKET DATA PLATFORM</small></span></Link>
        <span>API · SDK · SAAS</span>
        <a href="/portal/docs">Documentation <span aria-hidden="true">↗</span></a>
      </footer>
    </main>
  );
}
