"use client";

import { motion } from "framer-motion";
import { HeroArtwork } from "./HeroArtwork";
import { TerminalBox } from "./TerminalBox";

export function Hero() {
  return (
    <section className="hero" id="top">
      <div className="hero-inner">
        <motion.div className="hero-copy" initial={{ opacity: 0, y: 24 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.8, ease: [0.22, 1, 0.36, 1] }}>
          <p className="eyebrow"><span className="eyebrow-line" />FINANCIAL INFRASTRUCTURE • RUST POWERED</p>
          <h1>REALTIME FINANCIAL<br />INTELLIGENCE & MARKET</h1>

          <p className="hero-description" style={{ color: "rgba(255,255,255,0.8)", marginBottom: "32px", fontSize: "14px", lineHeight: "1.6", maxWidth: "520px" }}>
            SLV (ATLSD Engine) is a high-performance real-time financial infrastructure platform. Integrating multi-asset price feeds (Forex, Stocks, Commodities, Crypto), Options GEX analytics, Macro Intelligence, & AI-powered market movement analysis via REST, WebSocket, & Discord Bot.
          </p>

          <div className="install-desktop-section">
            <span className="section-sublabel">EXPLORE ARCHITECTURE</span>
            <a className="download-windows-btn" href="https://github.com/wignn/atlsd" target="_blank" rel="noreferrer">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path fillRule="evenodd" clipRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"/></svg>
              VIEW ON GITHUB (ATLSD)
            </a>
          </div>

          <div className="install-terminal-section">
            <span className="section-sublabel">SERVICE ENDPOINTS & CONNECTORS</span>
            <TerminalBox />
          </div>
        </motion.div>

        <motion.div className="hero-art-wrap" initial={{ opacity: 0, scale: 0.96 }} animate={{ opacity: 1, scale: 1 }} transition={{ duration: 1, delay: 0.1, ease: [0.22, 1, 0.36, 1] }}>
          <HeroArtwork />
        </motion.div>
      </div>
    </section>
  );
}