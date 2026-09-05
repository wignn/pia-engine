"use client";

import { animate, stagger } from "animejs";
import { motion, useReducedMotion } from "framer-motion";
import type { ReactNode } from "react";
import { useEffect, useRef } from "react";

type SectionTransitionProps = {
  children: ReactNode;
  className?: string;
};

export function SectionTransition({ children, className }: SectionTransitionProps) {
  const reducedMotion = useReducedMotion();
  const rootRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const root = rootRef.current;
    if (!root || reducedMotion) return;

    const targets = root.querySelectorAll<HTMLElement>(
      ".section-intro, .arch-header, .tech-header, .portal-content, .feature-card, .arch-card, .tech-tag"
    );
    if (!targets.length) return;

    const animation = animate(targets, {
      opacity: [0, 1],
      translateY: [14, 0],
      delay: stagger(45),
      duration: 650,
      ease: "outCubic",
      autoplay: false,
    });

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (!entry.isIntersecting) return;
        animation.play();
        observer.disconnect();
      },
      { threshold: 0.16, rootMargin: "0px 0px -12%" }
    );
    observer.observe(root);

    return () => {
      observer.disconnect();
      animation.pause();
    };
  }, [reducedMotion]);

  return (
    <motion.div
      ref={rootRef}
      className={`section-transition${className ? ` ${className}` : ""}`}
      initial={reducedMotion ? false : { opacity: 0, y: 16, filter: "blur(4px)" }}
      whileInView={reducedMotion ? undefined : { opacity: 1, y: 0, filter: "blur(0px)" }}
      viewport={{ once: true, amount: 0.18, margin: "0px 0px -12%" }}
      transition={{ duration: 0.9, ease: [0.22, 1, 0.36, 1] }}
    >
      {children}
    </motion.div>
  );
}
