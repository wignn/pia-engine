"use client";

import { animate, stagger } from "animejs";
import Image from "next/image";
import { useEffect, useRef, useState } from "react";
import { PortalCanvas } from "./PortalCanvas";

const navigation = [
  ["OVERVIEW", "overview"],
  ["ENGINE", "engine"],
  ["ANALYTICS", "analytics"],
  ["PLANS", "plans"],
];

const reasons = [
  ["01", "REALTIME STREAMS", "Low-latency market data across Forex, Stocks, Commodities, and Crypto."],
  ["02", "QUANT INTELLIGENCE", "GEX, options chains, macro events, and movement attribution in one system."],
  ["03", "ONE API SURFACE", "REST, WebSocket, and Discord delivery backed by Rust microservices."],
  ["04", "ALWAYS IN CONTEXT", "News, filings, sentiment, and signals organized around the market."],
];

export function PortalPage() {
  const [menuOpen, setMenuOpen] = useState(false);
  const cardsRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const cards = cardsRef.current?.querySelectorAll<HTMLElement>(".portal-feature-card");
    if (!cards?.length) return;
    const animation = animate(cards, {
      opacity: [0, 1],
      translateY: [18, 0],
      delay: stagger(65),
      duration: 700,
      ease: "outCubic",
      autoplay: false,
    });
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (!entry.isIntersecting) return;
        animation.play();
        observer.disconnect();
      },
      { threshold: 0.12 }
    );
    const cardsRoot = cardsRef.current;
    if (!cardsRoot) {
      animation.pause();
      observer.disconnect();
      return;
    }
    observer.observe(cardsRoot);
    return () => {
      observer.disconnect();
      animation.pause();
    };
  }, []);

  return (
    <main className="portal-page">
      <aside className={`portal-sidebar${menuOpen ? " is-open" : ""}`}>
        <div className="portal-corner portal-corner-top" />
        <div className="portal-corner portal-corner-bottom" />
        <div className="portal-brand">
          <div className="portal-brand-mark">
            <Image src="/logo.png" alt="SLV logo" width={64} height={64} priority />
          </div>
          <div>
            <strong>SLV<br />PORTAL</strong>
            <small>ATLSD ENGINE / V1.0</small>
          </div>
        </div>

        <div className="portal-nav-group">
          <span className="portal-nav-label">EXPLORE</span>
          {navigation.map(([label, id], index) => (
            <a href={`#${id}`} key={id} onClick={() => setMenuOpen(false)}>
              <span>✣</span>
              {label}
              <b>[ {index + 1} ]</b>
            </a>
          ))}
        </div>

        <div className="portal-nav-group">
          <span className="portal-nav-label">RESOURCES</span>
          <a href="#api">
            <span>▣</span>API DOCS<b>[ 01 ]</b>
          </a>
          <a href="/portal/account">
            <span>◫</span>ACCOUNT<b>[ 02 ]</b>
          </a>
          <a href="https://github.com/wignn/atlsd" target="_blank" rel="noreferrer">
            <span>?</span>GITHUB<b>[ 03 ]</b>
          </a>
          <a
            href="https://discord.com/oauth2/authorize?client_id=1410191834241601556&permissions=8&integration_type=0&scope=bot"
            target="_blank"
            rel="noreferrer"
          >
            <span>◈</span>DISCORD BOT<b>[ 04 ]</b>
          </a>
        </div>

        <a
          className="portal-signin"
          href="https://discord.com/oauth2/authorize?client_id=1410191834241601556&permissions=8&integration_type=0&scope=bot"
          target="_blank"
          rel="noreferrer"
        >
          ↪ &nbsp; OPEN SLV ACCOUNT
        </a>
      </aside>

      <button
        className="portal-mobile-toggle"
        onClick={() => setMenuOpen((open) => !open)}
        aria-label="Toggle portal navigation"
        aria-expanded={menuOpen}
      >
        ☰
      </button>

      <section className="portal-main">
        <div className="portal-hero" id="overview">
          <PortalCanvas />
          <div className="portal-hero-copy">
            <span className="portal-kicker">✧ &nbsp; SLV PORTAL</span>
            <h1>
              EVERYTHING TO<br />
              POWER YOUR<br />
              <em>MARKET</em>
            </h1>
            <p className="portal-lead">SLV Portal is the operating layer for realtime financial intelligence.</p>
            <p className="portal-description">
              Connect market feeds, options GEX analytics, macro intelligence, and Discord delivery through one high-performance ATLSD engine.
            </p>
            <div className="portal-actions">
              <a
                className="portal-primary-button"
                href="https://discord.com/oauth2/authorize?client_id=1410191834241601556&permissions=8&integration_type=0&scope=bot"
                target="_blank"
                rel="noreferrer"
              >
                ADD BOT TO DISCORD
              </a>
              <a className="portal-secondary-button" href="#engine">
                EXPLORE ENGINE
              </a>
            </div>
          </div>
          <div className="portal-hero-code">
            SLV / ATLSD / V1.0.0<br />
            STATUS: <span>OPERATIONAL</span>
          </div>
        </div>

        <section className="portal-section-block" id="engine">
          <div className="portal-section-heading">
            <span>01 / ENGINE</span>
            <h2>WHY SLV PORTAL?</h2>
          </div>
          <div className="portal-feature-grid" ref={cardsRef}>
            {reasons.map(([number, title, body]) => (
              <article className="portal-feature-card" key={number}>
                <small>#{number}</small>
                <h3>{title}</h3>
                <p>{body}</p>
              </article>
            ))}
          </div>
        </section>

        <section className="portal-data-section" id="analytics">
          <div>
            <span className="portal-section-label">02 / SIGNAL LAYER</span>
            <h2>
              FROM TICK<br />
              <em>TO THESIS.</em>
            </h2>
          </div>
          <div className="portal-data-copy">
            <p>Every service is designed around the moments where information becomes conviction.</p>
            <div className="portal-metric-row">
              <span>
                DATA FEEDS <b>24 / 7</b>
              </span>
              <span>
                LATENCY <b>&lt; 50MS</b>
              </span>
              <span>
                DELIVERY <b>REST · WS · DC</b>
              </span>
            </div>
          </div>
        </section>

        <section className="portal-api-section" id="api">
          <div>
            <span className="portal-section-label">03 / DEVELOPER SURFACE</span>
            <h2>
              ONE ENGINE.<br />
              <em>EVERY OUTPUT.</em>
            </h2>
          </div>
          <div className="portal-api-list">
            <div>
              <b>REST API</b>
              <span>Query structured market and intelligence data.</span>
            </div>
            <div>
              <b>WEBSOCKET</b>
              <span>Subscribe to realtime prices and event streams.</span>
            </div>
            <div>
              <b>DISCORD BOT</b>
              <span>Route alerts directly to your community server.</span>
            </div>
          </div>
        </section>

        <section className="portal-cta-section" id="plans">
          <span className="portal-section-label">04 / NEXT ACCESS</span>
          <h2>
            BUILD YOUR<br />
            <em>MARKET EDGE.</em>
          </h2>
          <p>Start with the public engine. Expand into the Portal when your workflow is ready.</p>
          <a
            className="portal-primary-button"
            href="https://discord.com/oauth2/authorize?client_id=1410191834241601556&permissions=8&integration_type=0&scope=bot"
            target="_blank"
            rel="noreferrer"
          >
            GET STARTED →
          </a>
        </section>

        <footer className="portal-footer">
          <span>SLV PORTAL / ATLSD ENGINE</span>
          <span>WIGNN / 2026 / MIT LICENSE</span>
        </footer>
      </section>
    </main>
  );
}
