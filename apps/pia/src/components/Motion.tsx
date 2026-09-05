"use client";

import { motion } from "framer-motion";

export function Portal() {
  return (
    <section className="relative isolate min-h-screen overflow-hidden text-white flex flex-col justify-between" id="portal">

      <div className="absolute inset-0 z-0 bg-gradient-to-b from-blue-700 via-blue-900 to-black opacity-40 pointer-events-none" />

      <h1 className="
        absolute
        top-[35%]
        left-1/2
        -translate-x-1/2
        text-[260px]
        font-serif
        tracking-widest
        text-white/10
        select-none
        pointer-events-none
      ">
        SLV
      </h1>

      <h1 className="
        absolute
        top-[52%]
        left-1/2
        -translate-x-1/2
        text-[150px]
        font-serif
        tracking-widest
        text-white/10
        pointer-events-none
      ">
        PORTAL
      </h1>


      {/* top */}
      <div className="
        relative
        z-50
        flex
        flex-col
        items-center
        pt-8
        pointer-events-auto
      ">

        <p className="
          text-xs
          tracking-[0.5em]
          font-mono
        ">
          FREE · DEVELOPER · PRO · ENTERPRISE
        </p>


        <h2 className="
          mt-8
          font-serif
          text-7xl
          tracking-wide
          uppercase
        ">
          SLV
        </h2>


        <p className="
          mt-5
          max-w-xl
          text-center
          uppercase
          font-mono
          text-sm
          leading-6
          text-white/80
        ">
          ACCESS HIGH-PERFORMANCE REALTIME MARKET STREAMING, QUANT & GEX ANALYTICS, MACRO INTELLIGENCE, AND INTEGRATED RUST DISCORD BOT SERVICES.
        </p>


        <a
          href="/portal"
          target="_blank"
          rel="noreferrer"
          style={{ backgroundColor: "#ffffff", color: "#0000ff" }}
          className="
            relative
            z-50
            mt-8
            cursor-pointer
            pointer-events-auto
            px-8
            py-3.5
            font-mono
            text-xs
            font-extrabold
            tracking-widest
            shadow-lg
            hover:scale-105
            hover:bg-blue-50
            transition-all
            inline-block
            border
            border-blue-600
          "
        >
          GET STARTED
        </a>
      </div>

      <div className="relative z-10 w-full h-[70vh] max-h-[850px] flex items-center justify-center my-4 pointer-events-none">
        <motion.video
          src="/char.webm"
          autoPlay
          loop
          muted
          playsInline
          aria-label="SLV character animation"
          initial={{ opacity: 0, y: 50 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 1 }}
          className="
            h-full
            max-h-[600px]
            w-auto
            object-contain
            relative
            z-0
            pointer-events-none
            mix-blend-screen
            scale-110
            md:scale-125
          "
        />
      </div>

      {/* footer bar */}
      {/* <div className="
        relative
        z-30
        w-full
        px-10
        flex
        justify-between
        items-end
        font-mono
        text-xs
        tracking-widest
      ">
        <div>
          ATLSD ENGINE V1.0.0
        </div>

        <div className="text-right">
          wignn/atlsd<br/>
          MIT LICENSE · 2026
        </div>
      </div> */}

    </section>
  );
}