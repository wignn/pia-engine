"use client";

import { useEffect, useRef } from "react";
import * as THREE from "three";

const damp = (current: number, target: number, speed: number, delta: number) =>
  THREE.MathUtils.lerp(current, target, 1 - Math.exp(-speed * delta));

export function ArchitectureCanvas() {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(34, 1, 0.1, 100);
    camera.position.z = 18;

    const renderer = new THREE.WebGLRenderer({
      alpha: true,
      antialias: true,
      powerPreference: "high-performance",
    });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 1.5));
    container.appendChild(renderer.domElement);

    const system = new THREE.Group();
    system.rotation.x = -0.18;
    scene.add(system);

    const coreGeometry = new THREE.IcosahedronGeometry(1.15, 2);
    const coreMaterial = new THREE.MeshBasicMaterial({
      color: 0x7f8cff,
      transparent: true,
      opacity: 0.2,
      wireframe: true,
      depthWrite: false,
    });
    const core = new THREE.Mesh(coreGeometry, coreMaterial);
    system.add(core);

    const glowGeometry = new THREE.SphereGeometry(0.42, 24, 24);
    const glowMaterial = new THREE.MeshBasicMaterial({
      color: 0xdce2ff,
      transparent: true,
      opacity: 0.75,
      depthWrite: false,
    });
    const glow = new THREE.Mesh(glowGeometry, glowMaterial);
    system.add(glow);

    const ringGeometries: THREE.TorusGeometry[] = [];
    const ringMaterials: THREE.MeshBasicMaterial[] = [];
    const rings = [
      { radius: 2.6, tilt: 0.2, opacity: 0.34, color: 0xbcc5ff },
      { radius: 3.8, tilt: -0.42, opacity: 0.2, color: 0x5f73ff },
      { radius: 5.1, tilt: 0.62, opacity: 0.13, color: 0x24d9ff },
    ];

    rings.forEach(({ radius, tilt, opacity, color }) => {
      const geometry = new THREE.TorusGeometry(radius, 0.018, 10, 128);
      const material = new THREE.MeshBasicMaterial({
        color,
        transparent: true,
        opacity,
        depthWrite: false,
      });
      const ring = new THREE.Mesh(geometry, material);
      ring.rotation.x = tilt;
      ring.rotation.y = tilt * 0.7;
      system.add(ring);
      ringGeometries.push(geometry);
      ringMaterials.push(material);
    });

    const nodePositions = [
      [-2.4, 1.8, 0.2],
      [2.6, 1.4, -0.4],
      [3.3, -1.2, 0.5],
      [-2.8, -1.7, -0.3],
      [0.5, 3, -0.8],
      [-0.5, -3, 0.6],
    ].map(([x, y, z]) => new THREE.Vector3(x, y, z));
    const nodeGeometry = new THREE.BufferGeometry().setFromPoints(nodePositions);
    const nodeMaterial = new THREE.PointsMaterial({
      color: 0xe9edff,
      size: 0.12,
      transparent: true,
      opacity: 0.8,
      depthWrite: false,
      blending: THREE.AdditiveBlending,
    });
    const nodes = new THREE.Points(nodeGeometry, nodeMaterial);
    system.add(nodes);

    const linePositions: number[] = [];
    nodePositions.forEach((node, index) => {
      const next = nodePositions[(index + 1) % nodePositions.length];
      linePositions.push(node.x, node.y, node.z, next.x, next.y, next.z);
      linePositions.push(node.x, node.y, node.z, 0, 0, 0);
    });
    const lineGeometry = new THREE.BufferGeometry();
    lineGeometry.setAttribute("position", new THREE.Float32BufferAttribute(linePositions, 3));
    const lineMaterial = new THREE.LineBasicMaterial({
      color: 0x9ba8ff,
      transparent: true,
      opacity: 0.22,
      depthWrite: false,
    });
    const network = new THREE.LineSegments(lineGeometry, lineMaterial);
    system.add(network);

    const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)");
    let visible = false;
    let running = document.visibilityState === "visible";
    let animationId = 0;
    let lastTime = performance.now();
    let scrollOffset = 0;
    let targetScrollOffset = 0;

    const resize = () => {
      const width = container.clientWidth || 1;
      const height = container.clientHeight || 1;
      camera.aspect = width / height;
      camera.updateProjectionMatrix();
      renderer.setSize(width, height);
    };

    const render = (now: number) => {
      animationId = 0;
      if (!running || !visible) return;
      const delta = Math.min((now - lastTime) / 1000, 0.05);
      lastTime = now;
      const elapsed = now / 1000;
      const motion = reduceMotion.matches ? 0 : 1;

      scrollOffset = damp(scrollOffset, targetScrollOffset, 3, delta);
      system.rotation.y += delta * 0.045 * motion;
      system.rotation.z = damp(system.rotation.z, scrollOffset * 0.04, 2, delta);
      core.rotation.x += delta * 0.08 * motion;
      core.rotation.z -= delta * 0.05 * motion;
      nodes.rotation.y = -system.rotation.y * 1.5;
      const breath = 1 + Math.sin(elapsed * 0.8) * 0.025 * motion;
      system.scale.setScalar(damp(system.scale.x, breath, 3, delta));
      glow.scale.setScalar(1 + Math.sin(elapsed * 1.4) * 0.08 * motion);

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
    const onScroll = () => {
      const rect = container.getBoundingClientRect();
      targetScrollOffset = THREE.MathUtils.clamp((window.innerHeight * 0.5 - (rect.top + rect.height * 0.5)) / window.innerHeight, -1, 1);
    };
    const onVisibility = () => {
      running = document.visibilityState === "visible";
      if (running) start(); else stop();
    };

    const onMotionPreferenceChange = () => {
      if (!reduceMotion.matches) start();
    };
    const observer = new IntersectionObserver(
      ([entry]) => {
        visible = entry.isIntersecting;
        if (visible) start(); else stop();
      },
      { rootMargin: "20% 0px 20%" }
    );

    resize();
    onScroll();
    observer.observe(container);
    window.addEventListener("resize", resize);
    window.addEventListener("scroll", onScroll, { passive: true });
    document.addEventListener("visibilitychange", onVisibility);
    reduceMotion.addEventListener("change", onMotionPreferenceChange);

    return () => {
      stop();
      observer.disconnect();
      window.removeEventListener("resize", resize);
      window.removeEventListener("scroll", onScroll);
      document.removeEventListener("visibilitychange", onVisibility);
      reduceMotion.removeEventListener("change", onMotionPreferenceChange);
      if (container.contains(renderer.domElement)) container.removeChild(renderer.domElement);

      coreGeometry.dispose();
      coreMaterial.dispose();
      glowGeometry.dispose();
      glowMaterial.dispose();
      ringGeometries.forEach((geometry) => geometry.dispose());
      ringMaterials.forEach((material) => material.dispose());
      nodeGeometry.dispose();
      nodeMaterial.dispose();
      lineGeometry.dispose();
      lineMaterial.dispose();
      renderer.dispose();
    };
  }, []);

  return <div ref={containerRef} className="architecture-canvas" aria-hidden="true" />;
}
