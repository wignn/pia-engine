"use client";

import { motion, useReducedMotion } from "framer-motion";

export function Portal() {
  const shouldReduceMotion = useReducedMotion();

  return (
    <section
      className="relative isolate min-h-screen overflow-hidden text-white flex flex-col justify-between py-12 md:py-16 px-6 md:px-12"
      id="portal"
      aria-labelledby="portal-heading"
    >
      <div className="absolute inset-0 z-0 bg-gradient-to-b from-blue-700 via-blue-900 to-black opacity-40 pointer-events-none" />

      {/* Decorative background watermark */}
      <div
        aria-hidden="true"
        className="
          absolute
          top-[35%]
          left-1/2
          -translate-x-1/2
          text-[clamp(6rem,18vw,260px)]
          font-serif
          tracking-widest
          text-white/10
          select-none
          pointer-events-none
          leading-none
        "
      >
        PIA
      </div>

      <div
        aria-hidden="true"
        className="
          absolute
          top-[52%]
          left-1/2
          -translate-x-1/2
          text-[clamp(4rem,12vw,150px)]
          font-serif
          tracking-widest
          text-white/10
          pointer-events-none
          select-none
          leading-none
        "
      >
        PORTAL
      </div>

      {/* top */}
      <div className="
        relative
        z-20
        flex
        flex-col
        items-center
        pt-4 md:pt-8
        pointer-events-auto
        text-center
      ">
        <p className="
          text-[10px] md:text-xs
          tracking-[0.3em] md:tracking-[0.5em]
          font-mono
          text-white/80
        ">
          FREE · DEVELOPER · PRO · ENTERPRISE
        </p>

        <h2
          id="portal-heading"
          className="
            mt-6 md:mt-8
            font-serif
            text-5xl md:text-7xl
            tracking-wide
            uppercase
          "
        >
          PIA PORTAL
        </h2>

        <p className="
          mt-4 md:mt-5
          max-w-xl
          text-center
          uppercase
          font-mono
          text-xs md:text-sm
          leading-relaxed
          text-white/80
        ">
          ACCESS HIGH-PERFORMANCE REALTIME MARKET STREAMING, QUANT & GEX ANALYTICS, MACRO INTELLIGENCE, AND INTEGRATED RUST DISCORD BOT SERVICES.
        </p>

        <a
          href="/portal"
          style={{ backgroundColor: "#ffffff", color: "#0000ff" }}
          className="
            relative
            z-20
            mt-6 md:mt-8
            cursor-pointer
            pointer-events-auto
            px-7 md:px-8
            py-3 md:py-3.5
            font-mono
            text-xs
            font-extrabold
            tracking-widest
            shadow-lg
            hover:scale-105
            hover:bg-blue-50
            transition-transform
            inline-flex
            items-center
            gap-2
            border
            border-blue-600
          "
        >
          GET STARTED <span aria-hidden="true">→</span>
        </a>
      </div>

      <div className="relative z-10 w-full h-[40vh] md:h-[60vh] max-h-[700px] flex items-center justify-center my-4 pointer-events-none">
        <motion.video
          src="/char.webm"
          autoPlay={!shouldReduceMotion}
          loop
          muted
          playsInline
          aria-hidden="true"
          initial={shouldReduceMotion ? false : { opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8 }}
          className="
            h-full
            max-h-[550px]
            w-auto
            object-contain
            relative
            z-0
            pointer-events-none
            mix-blend-screen
            scale-105 md:scale-120
          "
        />
      </div>

      {/* footer bar */}
      <div className="
        relative
        z-20
        w-full
        flex
        flex-col sm:flex-row
        justify-between
        items-center sm:items-end
        gap-4
        font-mono
        text-[10px] md:text-xs
        tracking-widest
        text-white/70
        pt-4
        border-t border-white/10
      ">
        <div>
          ATLSD ENGINE V1.0.0
        </div>

        <div className="text-center sm:text-right">
          wignn/atlsd<br/>
          MIT LICENSE · 2026
        </div>
      </div>
    </section>
  );
}