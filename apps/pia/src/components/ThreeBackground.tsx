"use client";

import { useEffect, useRef } from "react";
import * as THREE from "three";

export function ThreeBackground() {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const width = container.clientWidth || window.innerWidth;
    const height = container.clientHeight || 400;

    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(50, width / height, 0.1, 1000);
    camera.position.z = 15;

    const renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true });
    renderer.setSize(width, height);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    container.appendChild(renderer.domElement);

    // Torus knot geometric mesh
    const geometry = new THREE.TorusKnotGeometry(3.8, 0.8, 120, 16);
    const material = new THREE.MeshBasicMaterial({
      color: 0x0909ee,
      wireframe: true,
      transparent: true,
      opacity: 0.25,
    });
    const mesh = new THREE.Mesh(geometry, material);
    scene.add(mesh);

    // Floating particle field
    const pCount = 120;
    const pGeo = new THREE.BufferGeometry();
    const pPos = new Float32Array(pCount * 3);
    for (let i = 0; i < pCount * 3; i += 3) {
      pPos[i] = (Math.random() - 0.5) * 30;
      pPos[i + 1] = (Math.random() - 0.5) * 20;
      pPos[i + 2] = (Math.random() - 0.5) * 10;
    }
    pGeo.setAttribute("position", new THREE.BufferAttribute(pPos, 3));
    const pMat = new THREE.PointsMaterial({ size: 0.08, color: 0x0909ee, transparent: true, opacity: 0.35 });
    const particles = new THREE.Points(pGeo, pMat);
    scene.add(particles);

    let animationId: number;
    const animate = () => {
      mesh.rotation.x += 0.003;
      mesh.rotation.y += 0.005;
      particles.rotation.y -= 0.001;

      renderer.render(scene, camera);
      animationId = requestAnimationFrame(animate);
    };
    animate();

    const onResize = () => {
      if (!container) return;
      const w = container.clientWidth;
      const h = container.clientHeight;
      camera.aspect = w / h;
      camera.updateProjectionMatrix();
      renderer.setSize(w, h);
    };
    window.addEventListener("resize", onResize);

    return () => {
      cancelAnimationFrame(animationId);
      window.removeEventListener("resize", onResize);
      if (container.contains(renderer.domElement)) {
        container.removeChild(renderer.domElement);
      }
      geometry.dispose();
      material.dispose();
      pGeo.dispose();
      pMat.dispose();
      renderer.dispose();
    };
  }, []);

  return <div ref={containerRef} className="three-section-bg" aria-hidden="true" />;
}