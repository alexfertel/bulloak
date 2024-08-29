"use client";
import React, { useEffect, useRef } from "react";

interface Tree {
  x: number;
  y: number;
  angle: number;
  maxLevels: number;
  color: string;
  growth: number;
  age: number;
}

const AsciiTreeAnimation: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const resizeCanvas = () => {
      canvas.width = window.innerWidth;
      canvas.height = Math.min(400, window.innerHeight * 0.4);
    };

    resizeCanvas();
    window.addEventListener('resize', resizeCanvas);

    const colors = ["#4CAF50", "#8BC34A", "#CDDC39", "#FFC107", "#FF9800"];
    let trees: Tree[] = [];

    const generateTree = (): Tree => ({
      x: Math.random() * canvas.width,
      y: canvas.height,
      angle: 15 + Math.random() * 30,
      maxLevels: 3 + Math.floor(Math.random() * 4),
      color: colors[Math.floor(Math.random() * colors.length)],
      growth: 0,
      age: 0,
    });

    const branch = (h: number, level: number, angle: number, maxLevels: number, growth: number) => {
      h *= 0.66;

      if (level < maxLevels * growth) {
        ctx.save();
        ctx.rotate((angle * Math.PI) / 180);
        ctx.beginPath();
        ctx.moveTo(0, 0);
        ctx.lineTo(0, -h);
        ctx.stroke();
        ctx.translate(0, -h);
        branch(h, level + 1, angle, maxLevels, growth);
        ctx.restore();

        ctx.save();
        ctx.rotate((-angle * Math.PI) / 180);
        ctx.beginPath();
        ctx.moveTo(0, 0);
        ctx.lineTo(0, -h);
        ctx.stroke();
        ctx.translate(0, -h);
        branch(h, level + 1, angle, maxLevels, growth);
        ctx.restore();
      }
    };

    const drawTree = (tree: Tree) => {
      ctx.save();
      ctx.translate(tree.x, tree.y);
      ctx.strokeStyle = tree.color;
      ctx.globalAlpha = Math.max(0, 1 - (tree.age - 5000) / 2000);
      ctx.lineWidth = 1;

      // Draw initial trunk
      ctx.beginPath();
      ctx.moveTo(0, 0);
      ctx.lineTo(0, -40 * tree.growth);
      ctx.stroke();

      ctx.translate(0, -40 * tree.growth);
      branch(40 * tree.growth, 0, tree.angle, tree.maxLevels, tree.growth);

      ctx.restore();
    };

    const drawAsciiOverlay = () => {
      const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
      ctx.fillStyle = "rgba(0, 0, 0, 0.8)";
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      const chars = [" ", ".", ",", "-", "~", "+", "o", "*", "%", "#", "@"];
      ctx.font = "10px monospace";
      ctx.fillStyle = "#4CAF50";

      for (let y = 0; y < canvas.height; y += 8) {
        for (let x = 0; x < canvas.width; x += 5) {
          const i = (y * canvas.width + x) * 4;
          const brightness = (imageData.data[i] + imageData.data[i + 1] + imageData.data[i + 2]) / 3;
          const charIndex = Math.floor((brightness / 255) * (chars.length - 1));
          ctx.fillText(chars[charIndex], x, y);
        }
      }
    };

    const updateAndDraw = (deltaTime: number) => {
      ctx.fillStyle = "black";
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      if (trees.length < 5 && Math.random() < 0.02) {
        trees.push(generateTree());
      }

      trees = trees.filter(tree => {
        tree.growth = Math.min(tree.growth + 0.0002 * deltaTime, 1);
        tree.age += deltaTime;
        drawTree(tree);
        return tree.age < 7000; // Remove trees after 7 seconds
      });

      drawAsciiOverlay();
    };

    let lastTime = 0;
    const animate = (currentTime: number) => {
      const deltaTime = currentTime - lastTime;
      lastTime = currentTime;

      updateAndDraw(deltaTime);
      requestAnimationFrame(animate);
    };

    requestAnimationFrame(animate);

    return () => {
      window.removeEventListener('resize', resizeCanvas);
    };
  }, []);

  return (
    <div className="absolute bottom-0 left-0 right-0">
      <canvas ref={canvasRef} />
    </div>
  );
};

export default AsciiTreeAnimation;
