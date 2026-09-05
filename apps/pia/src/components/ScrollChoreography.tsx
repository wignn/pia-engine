"use client";

import { animate, stagger } from "animejs";
import { useEffect, useRef } from "react";

const SECTION_IDS = ["top", "features", "architecture", "techstack", "portal"];
const clamp = (value: number, min: number, max: number) => Math.min(Math.max(value, min), max);

export function ScrollChoreography() {
  const markerRef = useRef<HTMLDivElement>(null);
  const lineRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const root = document.documentElement;
    const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)");
    const lenis = (window as Window & { __lenis?: { on: (event: string, callback: () => void) => void; off: (event: string, callback: () => void) => void } }).__lenis;
    let frame = 0;
    let scheduled = false;

    const absoluteTop = (element: HTMLElement) => element.getBoundingClientRect().top + window.scrollY;

    const update = () => {
      scheduled = false;
      const maxScroll = Math.max(document.documentElement.scrollHeight - window.innerHeight, 1);
      const scrollY = window.scrollY;
      const progress = clamp(scrollY / maxScroll, 0, 1);
      const viewport = Math.max(window.innerHeight, 1);
      const heroParallax = reduceMotion.matches ? 0 : clamp(-scrollY * 0.055, -28, 0);
      const portal = document.getElementById("portal");
      const portalTop = portal ? absoluteTop(portal) : 0;
      const portalProgress = portal
        ? clamp((scrollY - portalTop + viewport * 0.7) / Math.max(portal.offsetHeight, 1), 0, 1)
        : 0;
      const portalDrift = reduceMotion.matches ? 0 : (portalProgress - 0.5) * 34;
      const activeIndex = SECTION_IDS.reduce((active, id, index) => {
        const section = document.getElementById(id);
        return section && scrollY + viewport * 0.42 >= absoluteTop(section) ? index : active;
      }, 0);

      root.style.setProperty("--page-progress", progress.toFixed(4));
      root.style.setProperty("--hero-parallax", `${heroParallax.toFixed(2)}px`);
      root.style.setProperty("--portal-drift", `${portalDrift.toFixed(2)}px`);
      root.style.setProperty("--section-index", String(activeIndex + 1).padStart(2, "0"));
      root.style.setProperty("--section-shift", `${((progress - 0.5) * -8).toFixed(2)}px`);

      if (markerRef.current) markerRef.current.textContent = `${String(activeIndex + 1).padStart(2, "0")} / ${String(SECTION_IDS.length).padStart(2, "0")}`;
    };

    const schedule = () => {
      if (scheduled) return;
      scheduled = true;
      frame = requestAnimationFrame(update);
    };
    const onLenisScroll = () => schedule();
    lenis?.on("scroll", onLenisScroll);

    const introTargets = [lineRef.current, markerRef.current].filter(
      (target): target is HTMLDivElement => target !== null
    );
    const intro = introTargets.length
      ? animate(introTargets, {
          opacity: [0, 1],
          duration: 700,
          delay: stagger(100),
          ease: "outCubic",
          autoplay: !reduceMotion.matches,
        })
      : null;

    update();
    window.addEventListener("scroll", schedule, { passive: true });
    window.addEventListener("resize", schedule);
    reduceMotion.addEventListener("change", schedule);

    return () => {
      cancelAnimationFrame(frame);
      window.removeEventListener("scroll", schedule);
      window.removeEventListener("resize", schedule);
      lenis?.off("scroll", onLenisScroll);
      reduceMotion.removeEventListener("change", schedule);
      intro?.pause();
      root.style.removeProperty("--page-progress");
      root.style.removeProperty("--hero-parallax");
      root.style.removeProperty("--portal-drift");
      root.style.removeProperty("--section-index");
      root.style.removeProperty("--section-shift");
    };
  }, []);

  return (
    <div className="scroll-choreography" aria-hidden="true">
      <div ref={lineRef} className="scroll-progress-rail">
        <span className="scroll-progress-fill" />
      </div>
      <div ref={markerRef} className="scroll-section-marker">01 / 05</div>
    </div>
  );
}
