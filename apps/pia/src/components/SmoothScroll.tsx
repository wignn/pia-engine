"use client";

import { useEffect } from "react";

export function SmoothScroll() {
  useEffect(() => {
    const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)");
    const isDesktop = () => window.innerWidth > 768;
    const previousScrollBehavior = document.documentElement.style.scrollBehavior;
    document.documentElement.style.scrollBehavior = "auto";

    let target = window.scrollY;
    let frame = 0;
    let smoothing = false;

    const limit = (value: number) =>
      Math.max(0, Math.min(value, document.documentElement.scrollHeight - window.innerHeight));

    const animate = () => {
      const current = window.scrollY;
      const distance = target - current;

      if (Math.abs(distance) < 1) {
        window.scrollTo({ top: target, behavior: "auto" });
        smoothing = false;
        frame = 0;
        return;
      }

      window.scrollTo({ top: current + distance * 0.42, behavior: "auto" });
      frame = requestAnimationFrame(animate);
    };

    const onWheel = (event: WheelEvent) => {
      if (
        reduceMotion.matches ||
        !isDesktop() ||
        event.ctrlKey ||
        Math.abs(event.deltaX) > Math.abs(event.deltaY)
      ) {
        return;
      }

      event.preventDefault();
      const multiplier = event.deltaMode === 1 ? 18 : event.deltaMode === 2 ? window.innerHeight : 1.55;
      target = limit(target + event.deltaY * multiplier);
      smoothing = true;

      if (!frame) frame = requestAnimationFrame(animate);
    };

    const onScroll = () => {
      if (!smoothing) target = window.scrollY;
    };

    window.addEventListener("wheel", onWheel, { passive: false });
    window.addEventListener("scroll", onScroll, { passive: true });

    return () => {
      document.documentElement.style.scrollBehavior = previousScrollBehavior;
      window.removeEventListener("wheel", onWheel);
      window.removeEventListener("scroll", onScroll);
      if (frame) cancelAnimationFrame(frame);
    };
  }, []);

  return null;
}
