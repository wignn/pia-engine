"use client";

import Image from "next/image";
import { useEffect, useRef, useState } from "react";
import * as THREE from "three";

export function HeroArtwork() {
  const containerRef = useRef<HTMLDivElement>(null);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const width = container.clientWidth || 500;
    const height = container.clientHeight || 500;
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(45, width / height, 0.1, 1000);
    const cameraTarget = new THREE.Vector3(0, 0, 14);
    camera.position.copy(cameraTarget);

    const renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true, powerPreference: "high-performance" });
    renderer.setSize(width, height);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 1.5));
    container.appendChild(renderer.domElement);

    const bgGroup = new THREE.Group();
    bgGroup.position.z = -3;
    scene.add(bgGroup);

    const ringGeo = new THREE.TorusGeometry(4.2, 0.025, 16, 100);
    const ringMat = new THREE.MeshBasicMaterial({ color: 0xffffff, wireframe: true, transparent: true, opacity: 0.4 });
    const ring = new THREE.Mesh(ringGeo, ringMat);
    bgGroup.add(ring);

    const ringGeo2 = new THREE.TorusGeometry(5.2, 0.015, 16, 100);
    const ringMat2 = new THREE.MeshBasicMaterial({ color: 0xffffff, wireframe: true, transparent: true, opacity: 0.25 });
    const ring2 = new THREE.Mesh(ringGeo2, ringMat2);
    ring2.rotation.x = Math.PI / 4;
    bgGroup.add(ring2);

    const icoGeo = new THREE.IcosahedronGeometry(6.0, 1);
    const icoMat = new THREE.MeshBasicMaterial({ color: 0xffffff, wireframe: true, transparent: true, opacity: 0.18 });
    const ico = new THREE.Mesh(icoGeo, icoMat);
    bgGroup.add(ico);

    const raysGroup = new THREE.Group();
    const rayLines: THREE.Line[] = [];
    for (let i = 0; i < 44; i++) {
      const angle = (i / 44) * Math.PI * 2;
      const points = [
        new THREE.Vector3(Math.cos(angle) * 2.6, Math.sin(angle) * 2.6, -1),
        new THREE.Vector3(Math.cos(angle) * 8.5, Math.sin(angle) * 8.5, (Math.random() - 0.5) * 3 - 2),
      ];
      const lineGeo = new THREE.BufferGeometry().setFromPoints(points);
      const lineMat = new THREE.LineBasicMaterial({ color: 0xffffff, transparent: true, opacity: 0.3 });
      const line = new THREE.Line(lineGeo, lineMat);
      rayLines.push(line);
      raysGroup.add(line);
    }
    bgGroup.add(raysGroup);

    const particleCount = 80;
    const particleGeo = new THREE.BufferGeometry();
    const positions = new Float32Array(particleCount * 3);
    for (let i = 0; i < particleCount * 3; i += 3) {
      positions[i] = (Math.random() - 0.5) * 16;
      positions[i + 1] = (Math.random() - 0.5) * 16;
      positions[i + 2] = -Math.random() * 6 - 2;
    }
    particleGeo.setAttribute("position", new THREE.BufferAttribute(positions, 3));
    const particleMat = new THREE.PointsMaterial({ size: 0.06, color: 0xffffff, transparent: true, opacity: 0.45 });
    const particles = new THREE.Points(particleGeo, particleMat);
    bgGroup.add(particles);

    const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)");
    const damp = (current: number, target: number, speed: number, delta: number) =>
      THREE.MathUtils.lerp(current, target, 1 - Math.exp(-speed * delta));
    let targetX = 0;
    let targetY = 0;
    let currentX = 0;
    let currentY = 0;
    let animationId = 0;
    let lastTime = performance.now();
    let running = true;

    const onMouseMove = (event: MouseEvent) => {
      targetX = ((event.clientX / window.innerWidth) - 0.5) * 0.18;
      targetY = ((event.clientY / window.innerHeight) - 0.5) * 0.14;
    };
    const onResize = () => {
      const newWidth = container.clientWidth;
      const newHeight = container.clientHeight;
      camera.aspect = newWidth / newHeight;
      camera.updateProjectionMatrix();
      renderer.setSize(newWidth, newHeight);
    };
    const render = (now: number) => {
      if (!running) return;
      const delta = Math.min((now - lastTime) / 1000, 0.05);
      lastTime = now;
      const elapsed = now / 1000;
      const motion = reduceMotion.matches ? 0 : 1;

      currentX = damp(currentX, targetX * motion, 3.4, delta);
      currentY = damp(currentY, targetY * motion, 3.4, delta);
      bgGroup.rotation.y = damp(bgGroup.rotation.y, currentX, 3, delta);
      bgGroup.rotation.x = damp(bgGroup.rotation.x, currentY, 3, delta);
      bgGroup.position.y = damp(bgGroup.position.y, Math.sin(elapsed * 0.45) * 0.1 * motion, 2.5, delta);
      bgGroup.scale.setScalar(damp(bgGroup.scale.x, 1 + Math.sin(elapsed * 0.5) * 0.008 * motion, 2.5, delta));

      ring.rotation.z = elapsed * 0.025;
      ring2.rotation.z = -elapsed * 0.018;
      ico.rotation.y = elapsed * 0.018;
      ico.rotation.x = -elapsed * 0.01;
      raysGroup.rotation.z = -elapsed * 0.012;
      particles.rotation.y = elapsed * 0.008;

      cameraTarget.x = damp(cameraTarget.x, currentX * 0.35, 2.5, delta);
      cameraTarget.y = damp(cameraTarget.y, currentY * 0.25, 2.5, delta);
      camera.position.lerp(cameraTarget, 1 - Math.exp(-2.8 * delta));
      camera.lookAt(0, 0, -2);

      renderer.render(scene, camera);
      animationId = requestAnimationFrame(render);
    };
    const onVisibility = () => {
      running = document.visibilityState === "visible";
      if (running) {
        lastTime = performance.now();
        animationId = requestAnimationFrame(render);
      } else cancelAnimationFrame(animationId);
    };

    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("resize", onResize);
    document.addEventListener("visibilitychange", onVisibility);
    setReady(true);
    animationId = requestAnimationFrame(render);

    return () => {
      cancelAnimationFrame(animationId);
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("resize", onResize);
      document.removeEventListener("visibilitychange", onVisibility);
      if (container.contains(renderer.domElement)) container.removeChild(renderer.domElement);

      ringGeo.dispose();
      ringMat.dispose();
      ringGeo2.dispose();
      ringMat2.dispose();
      icoGeo.dispose();
      icoMat.dispose();
      particleGeo.dispose();
      particleMat.dispose();
      rayLines.forEach((line) => {
        line.geometry.dispose();
        (line.material as THREE.Material).dispose();
      });
      renderer.dispose();
    };
  }, []);

  return (
    <div className={`artwork-container ${ready ? "is-ready" : ""}`} aria-label="SLV logo with background 3D animation" role="img">
      <div ref={containerRef} className="art-three-canvas-bg" />
      <div className="art-logo-front">
        <Image src="/logo.png" alt="SLV Logo" width={650} height={650} className="hero-logo-img" priority />
      </div>
    </div>
  );
}
