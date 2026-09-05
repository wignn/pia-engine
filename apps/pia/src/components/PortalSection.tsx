"use client";

import Image from "next/image";

export function PortalSection() {
  return (
    <section className="portal-section" id="portal">
      <div className="portal-content">
        <p className="eyebrow-tiers">FREE • DEVELOPER • PRO • ENTERPRISE</p>
        <h2 className="portal-title">SLV PORTAL</h2>
        <p className="portal-desc">
          ACCESS HIGH-PERFORMANCE REALTIME MARKET STREAMING, QUANT & GEX ANALYTICS, MACRO INTELLIGENCE, AND INTEGRATED RUST DISCORD BOT SERVICES.
        </p>
        <a className="portal-btn" href="https://github.com/wignn/atlsd" target="_blank" rel="noreferrer">
          VIEW ALL OUR PLANS
        </a>
      </div>

      <div className="portal-art-wrap">
        <div className="portal-watermark-stacked" aria-hidden="true">
          <div>SLV</div>
          <div>PORTAL</div>
        </div>
        <div className="portal-illustration">
          <Image
            src="/footer.png"
            alt="SLV Portal Illustration"
            width={900}
            height={900}
            className="portal-center-img"
            priority
          />
        </div>
      </div>

      <div className="portal-bottom-bar">
        <div className="portal-meta-left">ATLSD ENGINE V1.0.0</div>
        <div className="portal-meta-right">
          <div className="portal-brand-box">
            <Image src="/logo.png" alt="SLV Logo" width={24} height={24} className="object-contain" />
            <span className="brand-name">SLV</span>
          </div>
          <div className="portal-copy-right">
            <span>wignn/atlsd</span>
            <span>MIT LICENSE • 2026</span>
          </div>
        </div>
      </div>
    </section>
  );
}