import { useEffect, useRef } from "react";

interface WaveformProps {
  level: number;
  processing?: boolean;
  className?: string;
}

const BAR_COUNT = 25;

export function Waveform({ level, processing = false, className = "w-36" }: WaveformProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const levels = useRef(Array.from({ length: BAR_COUNT }, () => 0.08));
  const target = useRef(level);

  useEffect(() => {
    target.current = level;
  }, [level]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const context = canvas.getContext("2d");
    if (!context) return;
    let frame = 0;
    let animation = 0;

    const draw = () => {
      const ratio = window.devicePixelRatio || 1;
      const width = canvas.clientWidth;
      const height = canvas.clientHeight;
      if (canvas.width !== width * ratio || canvas.height !== height * ratio) {
        canvas.width = width * ratio;
        canvas.height = height * ratio;
        context.setTransform(ratio, 0, 0, ratio, 0, 0);
      }

      frame += 1;
      const pulse = processing ? 0.25 + Math.sin(frame * 0.11) * 0.14 : target.current;
      const previous = levels.current[levels.current.length - 1] ?? 0;
      const smoothed = previous * 0.58 + Math.max(0.06, pulse) * 0.42;
      if (frame % 2 === 0) levels.current = [...levels.current.slice(1), smoothed];

      context.clearRect(0, 0, width, height);
      const gap = 3;
      const barWidth = Math.max(2, (width - gap * (BAR_COUNT - 1)) / BAR_COUNT);
      const gradient = context.createLinearGradient(0, 0, width, 0);
      gradient.addColorStop(0, "#b6abff");
      gradient.addColorStop(0.55, "#7b68ff");
      gradient.addColorStop(1, "#62ddbd");
      context.fillStyle = gradient;

      levels.current.forEach((sample, index) => {
        const shaped = Math.pow(Math.min(1, sample * 1.9), 0.72);
        const barHeight = 4 + shaped * (height - 6);
        const x = index * (barWidth + gap);
        const y = (height - barHeight) / 2;
        context.globalAlpha = 0.42 + (index / BAR_COUNT) * 0.58;
        context.beginPath();
        if (typeof context.roundRect === "function") {
          context.roundRect(x, y, barWidth, barHeight, barWidth / 2);
        } else {
          context.rect(x, y, barWidth, barHeight);
        }
        context.fill();
      });
      context.globalAlpha = 1;
      animation = requestAnimationFrame(draw);
    };
    draw();
    return () => cancelAnimationFrame(animation);
  }, [processing]);

  return <canvas ref={canvasRef} className={`h-8 ${className}`} aria-label="Live microphone level" />;
}
