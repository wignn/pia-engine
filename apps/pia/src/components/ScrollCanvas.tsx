"use client";

import { useEffect, useRef } from "react";
import * as THREE from "three";

export function ScrollCanvas() {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(45, window.innerWidth / window.innerHeight, 0.1, 100);
    const cameraTarget = new THREE.Vector3(0, 0, 20);
    camera.position.copy(cameraTarget);

    const renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true, powerPreference: "high-performance" });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 1.5));
    renderer.setSize(window.innerWidth, window.innerHeight);
    renderer.domElement.style.willChange = "transform";
    container.appendChild(renderer.domElement);

    const particleCount = 420;
    const particleGeometry = new THREE.BufferGeometry();
    const particlePositions = new Float32Array(particleCount * 3);
    for (let i = 0; i < particleCount; i++) {
      const index = i * 3;
      particlePositions[index] = (Math.random() - 0.5) * 42;
      particlePositions[index + 1] = (Math.random() - 0.5) * 100;
      particlePositions[index + 2] = (Math.random() - 0.5) * 18 - 3;
    }
    particleGeometry.setAttribute("position", new THREE.BufferAttribute(particlePositions, 3));
    const particleMaterial = new THREE.PointsMaterial({
      color: 0x7070ff,
      size: 0.08,
      transparent: true,
      opacity: 0.3,
      depthWrite: false,
      blending: THREE.AdditiveBlending,
    });
    const particles = new THREE.Points(particleGeometry, particleMaterial);
    scene.add(particles);

    const orbit = new THREE.Group();
    const ringMeshes: THREE.Mesh[] = [];
    const rings = [
      { radius: 5.5, color: 0xffffff, opacity: 0.22, tilt: 0.34 },
      { radius: 7.8, color: 0x6969ff, opacity: 0.16, tilt: -0.48 },
      { radius: 10.5, color: 0x00d9ff, opacity: 0.1, tilt: 0.18 },
    ];

    for (const ring of rings) {
      const geometry = new THREE.TorusGeometry(ring.radius, 0.018, 12, 96);
      const material = new THREE.MeshBasicMaterial({
        color: ring.color,
        transparent: true,
        opacity: ring.opacity,
        depthWrite: false,
      });
      const mesh = new THREE.Mesh(geometry, material);
      mesh.rotation.x = ring.tilt;
      ringMeshes.push(mesh);
      orbit.add(mesh);
    }
    orbit.position.set(4, 0, -5);
    scene.add(orbit);

    // Minimal celestial symbol: a quiet moon/sun disk with orbital halos.
    const celestial = new THREE.Group();
    const celestialGeometry = new THREE.CircleGeometry(3.2, 64);
    const celestialMaterial = new THREE.MeshBasicMaterial({
      color: 0x8fa4ff,
      transparent: true,
      opacity: 0.13,
      depthWrite: false,
    });
    const celestialDisk = new THREE.Mesh(celestialGeometry, celestialMaterial);
    celestial.add(celestialDisk);

    const celestialRingGeometry = new THREE.TorusGeometry(3.7, 0.025, 12, 96);
    const celestialRingMaterial = new THREE.MeshBasicMaterial({
      color: 0xb8c2ff,
      transparent: true,
      opacity: 0.3,
      depthWrite: false,
    });
    const celestialRing = new THREE.Mesh(celestialRingGeometry, celestialRingMaterial);
    celestial.add(celestialRing);

    const celestialHaloGeometry = new THREE.TorusGeometry(5.4, 0.012, 12, 96);
    const celestialHaloMaterial = new THREE.MeshBasicMaterial({
      color: 0x5268ff,
      transparent: true,
      opacity: 0.16,
      depthWrite: false,
    });
    const celestialHalo = new THREE.Mesh(celestialHaloGeometry, celestialHaloMaterial);
    celestialHalo.rotation.x = Math.PI / 3;
    celestial.add(celestialHalo);
    celestial.position.set(-5, 2, -8);
    scene.add(celestial);

    const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)");
    const damp = (current: number, target: number, speed: number, delta: number) =>
      THREE.MathUtils.lerp(current, target, 1 - Math.exp(-speed * delta));

    let scrollProgress = window.scrollY / Math.max(document.documentElement.scrollHeight - window.innerHeight, 1);
    let targetScrollProgress = scrollProgress;
    let animationId = 0;
    let lastTime = performance.now();
    let running = true;

    const onScroll = () => {
      targetScrollProgress = window.scrollY / Math.max(document.documentElement.scrollHeight - window.innerHeight, 1);
    };
    const onResize = () => {
      camera.aspect = window.innerWidth / window.innerHeight;
      camera.updateProjectionMatrix();
      renderer.setSize(window.innerWidth, window.innerHeight);
    };
    const render = (now: number) => {
      if (!running) return;
      const delta = Math.min((now - lastTime) / 1000, 0.05);
      lastTime = now;
      const elapsed = now / 1000;

      scrollProgress = damp(scrollProgress, targetScrollProgress, 3.8, delta);
      const motion = reduceMotion.matches ? 0 : 1;
      const float = Math.sin(elapsed * 0.32) * 0.18 * motion;
      const scrollY = -scrollProgress * 5;
      const celestialY = 2 - scrollProgress * 10;

      orbit.position.y = damp(orbit.position.y, 2.5 + scrollY + float, 5.5, delta);
      orbit.position.x = damp(orbit.position.x, 4 + Math.sin(scrollProgress * Math.PI * 2) * 1.2, 2.6, delta);
      orbit.position.z = damp(orbit.position.z, -5 - scrollProgress * 1.5, 2.4, delta);
      orbit.rotation.z = damp(orbit.rotation.z, elapsed * 0.018 + scrollProgress * Math.PI * 0.4, 2.2, delta);
      orbit.rotation.y = damp(orbit.rotation.y, Math.sin(elapsed * 0.12) * 0.08, 2.2, delta);
      const breathing = 1 + Math.sin(elapsed * 0.42) * 0.012 * motion;
      orbit.scale.setScalar(damp(orbit.scale.x, breathing, 2.4, delta));

      celestial.position.y = damp(celestial.position.y, celestialY + float * 0.35, 2.6, delta);
      celestial.position.x = damp(celestial.position.x, -5 + Math.sin(scrollProgress * Math.PI) * 2, 2.2, delta);
      celestial.rotation.z = elapsed * 0.012 + scrollProgress * Math.PI * 0.18;
      celestialRing.rotation.z = elapsed * 0.02;
      celestialHalo.rotation.z = -elapsed * 0.012;
      const celestialScale = 1 + Math.sin(elapsed * 0.28) * 0.015 * motion;
      celestial.scale.setScalar(damp(celestial.scale.x, celestialScale, 2.2, delta));

      particles.rotation.y = elapsed * 0.008;
      particles.position.y = damp(particles.position.y, scrollProgress * -4, 2.5, delta);

      cameraTarget.y = damp(cameraTarget.y, Math.sin(scrollProgress * Math.PI) * -0.45, 2.2, delta);
      cameraTarget.x = damp(cameraTarget.x, Math.sin(scrollProgress * Math.PI * 2) * 0.12, 2.2, delta);
      cameraTarget.z = damp(cameraTarget.z, 20 - scrollProgress * 0.8, 2.2, delta);
      camera.position.lerp(cameraTarget, 1 - Math.exp(-3.2 * delta));
      camera.lookAt(0, camera.position.y * 0.12, -2);

      renderer.render(scene, camera);
      animationId = requestAnimationFrame(render);
    };

    const onVisibility = () => {
      running = document.visibilityState === "visible";
      if (running) {
        lastTime = performance.now();
        animationId = requestAnimationFrame(render);
      } else {
        cancelAnimationFrame(animationId);
      }
    };

    window.addEventListener("scroll", onScroll, { passive: true });
    window.addEventListener("resize", onResize);
    document.addEventListener("visibilitychange", onVisibility);
    animationId = requestAnimationFrame(render);

    return () => {
      cancelAnimationFrame(animationId);
      window.removeEventListener("scroll", onScroll);
      window.removeEventListener("resize", onResize);
      document.removeEventListener("visibilitychange", onVisibility);
      if (container.contains(renderer.domElement)) container.removeChild(renderer.domElement);

      particleGeometry.dispose();
      particleMaterial.dispose();
      celestialGeometry.dispose();
      celestialMaterial.dispose();
      celestialRingGeometry.dispose();
      celestialRingMaterial.dispose();
      celestialHaloGeometry.dispose();
      celestialHaloMaterial.dispose();
      ringMeshes.forEach((mesh) => {
        mesh.geometry.dispose();
        (mesh.material as THREE.Material).dispose();
      });
      renderer.dispose();
    };
  }, []);

  return <div ref={containerRef} className="fixed inset-0 pointer-events-none z-0 overflow-hidden three-canvas" aria-hidden="true" />;
}
