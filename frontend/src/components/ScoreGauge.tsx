import { cn } from "@/lib/utils";

interface ScoreGaugeProps {
  score: number;
  max?: number;
  size?: number;
  strokeWidth?: number;
  className?: string;
}

export function ScoreGauge({
  score,
  max = 100,
  size = 64,
  strokeWidth = 5,
  className,
}: ScoreGaugeProps) {
  const radius = (size - strokeWidth) / 2;
  const circumference = 2 * Math.PI * radius;
  const clamped = Math.min(Math.max(score, 0), max);
  const progress = (clamped / max) * circumference;

  const color =
    score >= 80 ? "text-emerald-400" : score >= 50 ? "text-amber-400" : "text-red-400";

  return (
    <div className={cn("relative inline-flex items-center justify-center", className)}>
      <svg width={size} height={size} className="-rotate-90">
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke="currentColor"
          strokeWidth={strokeWidth}
          className="text-slate-800"
        />
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke="currentColor"
          strokeWidth={strokeWidth}
          strokeDasharray={circumference}
          strokeDashoffset={circumference - progress}
          strokeLinecap="round"
          className={cn("transition-all duration-500", color)}
        />
      </svg>
      <span className={cn("absolute text-xs font-bold", color)}>
        {Math.round(score)}%
      </span>
    </div>
  );
}
