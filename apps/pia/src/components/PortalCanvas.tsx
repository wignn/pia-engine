"use client";

import { useEffect, useRef } from "react";
import * as THREE from "three";

export function PortalCanvas() {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(32, 1, 0.1, 100);
    camera.position.z = 15;
    const renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true, powerPreference: "high-performance" });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 1.5));
    container.appendChild(renderer.domElement);

    // Particle field
    const fieldCount = 420;
    const positions = new Float32Array(fieldCount * 3);
    for (let i = 0; i < fieldCount; i += 1) {
      const index = i * 3;
      positions[index] = (Math.random() - 0.5) * 24;
      positions[index + 1] = (Math.random() - 0.5) * 14;
      positions[index + 2] = (Math.random() - 0.5) * 8 - 2;
    }
    const fieldGeometry = new THREE.BufferGeometry();
    fieldGeometry.setAttribute("position", new THREE.BufferAttribute(positions, 3));
    const fieldMaterial = new THREE.PointsMaterial({ color: 0x5b64f1, size: 0.05, transparent: true, opacity: 0.45, depthWrite: false });
    const field = new THREE.Points(fieldGeometry, fieldMaterial);
    scene.add(field);

    // Orbital network group
    const orbital = new THREE.Group();
    orbital.position.set(3.6, -0.1, -1);
    scene.add(orbital);

    const ringGeometries: THREE.TorusGeometry[] = [];
    const ringMaterials: THREE.MeshBasicMaterial[] = [];
    [
      { radius: 3.2, tilt: 0.25, opacity: 0.32 },
      { radius: 4.4, tilt: -0.5, opacity: 0.22 },
      { radius: 5.6, tilt: 0.6, opacity: 0.14 },
    ].forEach(({ radius, tilt, opacity }) => {
      const geometry = new THREE.TorusGeometry(radius, 0.022, 12, 96);
      const material = new THREE.MeshBasicMaterial({ color: 0x4650e5, transparent: true, opacity, depthWrite: false });
      const ring = new THREE.Mesh(geometry, material);
      ring.rotation.x = tilt;
      orbital.add(ring);
      ringGeometries.push(geometry);
      ringMaterials.push(material);
    });

    const globeGeometry = new THREE.IcosahedronGeometry(1.9, 2);
    const globeMaterial = new THREE.MeshBasicMaterial({ color: 0x515be8, wireframe: true, transparent: true, opacity: 0.22, depthWrite: false });
    const globe = new THREE.Mesh(globeGeometry, globeMaterial);
    orbital.add(globe);

    const coreGeometry = new THREE.SphereGeometry(0.36, 20, 20);
    const coreMaterial = new THREE.MeshBasicMaterial({ color: 0x8a92ff, transparent: true, opacity: 0.75, depthWrite: false });
    const core = new THREE.Mesh(coreGeometry, coreMaterial);
    orbital.add(core);

    const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)");
    let visible = false;
    let running = document.visibilityState === "visible";
    let animationId = 0;
    let lastTime = performance.now();
    let targetX = 0;
    let targetY = 0;
    let currentX = 0;
    let currentY = 0;
    const damp = (current: number, target: number, speed: number, delta: number) => THREE.MathUtils.lerp(current, target, 1 - Math.exp(-speed * delta));

    const resize = () => {
      const width = container.clientWidth || 1;
      const height = container.clientHeight || 1;
      camera.aspect = width / height;
      camera.updateProjectionMatrix();
      renderer.setSize(width, height);
    };

    const onPointerMove = (event: PointerEvent) => {
      targetX = THREE.MathUtils.clamp((event.clientX / window.innerWidth - 0.5) * 0.35, -0.18, 0.18);
      targetY = THREE.MathUtils.clamp((event.clientY / window.innerHeight - 0.5) * 0.25, -0.14, 0.14);
    };

    const render = (now: number) => {
      animationId = 0;
      if (!running || !visible) return;
      const delta = Math.min((now - lastTime) / 1000, 0.05);
      lastTime = now;
      const motion = reduceMotion.matches ? 0 : 1;

      currentX = damp(currentX, targetX * motion, 2.5, delta);
      currentY = damp(currentY, targetY * motion, 2.5, delta);

      orbital.rotation.y += delta * 0.04 * motion;
      orbital.rotation.x = damp(orbital.rotation.x, currentY, 2, delta);
      orbital.rotation.z = damp(orbital.rotation.z, currentX, 2, delta);
      globe.rotation.x += delta * 0.07 * motion;
      globe.rotation.z -= delta * 0.03 * motion;
      field.rotation.y += delta * 0.008 * motion;
      core.scale.setScalar(1 + Math.sin(now / 650) * 0.09 * motion);

      renderer.render(scene, camera);
      animationId = requestAnimationFrame(render);
    };

    const start = () => {
      if (visible && running && !animationId) {
        lastTime = performance.now();
        animationId = requestAnimationFrame(render);
      }
    };
    const stop = () => {
      cancelAnimationFrame(animationId);
      animationId = 0;
    };
    const onVisibility = () => {
      running = document.visibilityState === "visible";
      running ? start() : stop();
    };

    const observer = new IntersectionObserver(([entry]) => {
      visible = entry.isIntersecting;
      visible ? start() : stop();
    }, { rootMargin: "18% 0px 18%" });

    resize();
    observer.observe(container);
    window.addEventListener("resize", resize);
    window.addEventListener("pointermove", onPointerMove, { passive: true });
    document.addEventListener("visibilitychange", onVisibility);

    return () => {
      stop();
      observer.disconnect();
      window.removeEventListener("resize", resize);
      window.removeEventListener("pointermove", onPointerMove);
      document.removeEventListener("visibilitychange", onVisibility);
      if (container.contains(renderer.domElement)) container.removeChild(renderer.domElement);
      fieldGeometry.dispose();
      fieldMaterial.dispose();
      ringGeometries.forEach((geometry) => geometry.dispose());
      ringMaterials.forEach((material) => material.dispose());
      globeGeometry.dispose();
      globeMaterial.dispose();
      coreGeometry.dispose();
      coreMaterial.dispose();
      renderer.dispose();
    };
  }, []);

  return <div ref={containerRef} className="portal-canvas" aria-hidden="true" />;
}
