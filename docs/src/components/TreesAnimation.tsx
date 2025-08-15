"use client";
import React, { useEffect, useRef } from "react";

interface Tree {
  x: number;
  y: number;
  angle: number;
  maxLevels: number;
  color: string;
  currentLevel: number;
  age: number;
  height: number;
  branches: Branch[];
}

interface Branch {
  startX: number;
  startY: number;
  endX: number;
  endY: number;
  level: number;
}

const AsciiTreeAnimation: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    let trees: Tree[] = [];
    let maxTrees: number;

    const colors = [
      "#F9D5E5", // Soft Pink
      "#EDE7B1", // Pale Yellow
      "#A9D7DA", // Light Sky Blue
      "#B8E0D2", // Mint Green
      "#D6A2AD", // Dusty Rose
      "#F1E0C5", // Sand
      "#C7CEEA", // Periwinkle
      "#F1C0B9", // Peach
      "#A2D2FF", // Baby Blue
      "#FFD8BE", // Apricot
      "#E8D0B3", // Wheat
      "#B5D8CC", // Sea Foam
    ];

    const resizeCanvas = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
      maxTrees = Math.max(3, Math.floor(canvas.width / 150));
      trees = trees.slice(0, maxTrees);
    };

    resizeCanvas();
    window.addEventListener("resize", resizeCanvas);

    const generateTree = (): Tree => {
      const maxLevels = 13;
      const height = canvas.height * (0.2 + Math.random() * 0.3);
      const x = Math.random() * canvas.width;
      const y = canvas.height;
      return {
        x,
        y,
        angle: 10 + Math.random() * 30,
        maxLevels,
        color: colors[Math.floor(Math.random() * colors.length)],
        currentLevel: 0,
        age: 0,
        height,
        branches: [
          {
            startX: x,
            startY: y,
            endX: x + Math.random() * 100 - 50,
            endY: y - height * 0.3,
            level: 0,
          },
        ],
      };
    };

    const branchGrow = (tree: Tree, startX: number, startY: number, h: number, angle: number, level: number) => {
      if (level >= tree.currentLevel) return;

      const endX = startX + Math.sin(angle) * h;
      const endY = startY - Math.cos(angle) * h;

      tree.branches.push({ startX, startY, endX, endY, level });

      const newH = h * 0.6 * (1 + Math.random() * 0.6);
      const newLevel = level + 1;

      const rangleSign = Math.random() > 0.5 ? 1 : -1;
      const langleSign = Math.random() > 0.5 ? 1 : -1;
      const rangleDelta = ((tree.angle * Math.PI) / 180) * (0.5 + Math.random() * 0.7);
      const langleDelta = ((tree.angle * Math.PI) / 180) * (0.5 + Math.random() * 0.7);
      const rangle = angle + rangleSign * rangleDelta;
      const langle = angle + langleSign * langleDelta;

      const growRightBranch = Math.random() > 0.2;
      const growLeftBranch = Math.random() > 0.2;
      if (growRightBranch) {
        branchGrow(tree, endX, endY, newH, rangle, newLevel);
      }
      if (growLeftBranch) {
        branchGrow(tree, endX, endY, newH, langle, newLevel);
      }
    };

    const drawTree = (tree: Tree) => {
      ctx.strokeStyle = tree.color;
      ctx.globalAlpha = 0.05;
      ctx.lineWidth = 1;

      tree.branches.forEach((branch) => {
        if (branch.level <= tree.currentLevel) {
          ctx.beginPath();
          ctx.moveTo(branch.startX, branch.startY);
          ctx.lineTo(branch.endX, branch.endY);
          ctx.stroke();
        }
      });
    };

    const updateAndDraw = (deltaTime: number) => {
      if (trees.length < maxTrees && Math.random() < 0.03) {
        trees.push(generateTree());
      }

      trees = trees.filter((tree) => {
        tree.age += deltaTime;
        if (tree.age > 500 && tree.currentLevel < tree.maxLevels) {
          tree.currentLevel++;
          tree.age = 0;
          const trunk = tree.branches[0];
          branchGrow(tree, trunk.endX, trunk.endY, tree.height * 0.3 * 0.8, 0, 1);
        }
        drawTree(tree);
        return tree.age < 20000 || tree.currentLevel < tree.maxLevels;
      });
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
      window.removeEventListener("resize", resizeCanvas);
    };
  }, []);

  return (
    <div className="absolute inset-0">
      <canvas ref={canvasRef} />
    </div>
  );
};

export default AsciiTreeAnimation;
