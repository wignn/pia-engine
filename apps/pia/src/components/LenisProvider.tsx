"use client";

import { useEffect } from "react";
import Lenis from "lenis";

export function LenisProvider() {
  useEffect(() => {
    const lenis = new Lenis({
      autoRaf: false,
      anchors: true,
      duration: 0.6,
      lerp: 0.16,
      stopInertiaOnNavigate: true,
      respectReducedMotion: true,
    });

    (window as Window & { __lenis?: Lenis }).__lenis = lenis;

    let frame = 0;
    const raf = (time: number) => {
      lenis.raf(time);
      frame = requestAnimationFrame(raf);
    };

    frame = requestAnimationFrame(raf);
    return () => {
      cancelAnimationFrame(frame);
      delete (window as Window & { __lenis?: Lenis }).__lenis;
      lenis.destroy();
    };
  }, []);

  return null;
}
