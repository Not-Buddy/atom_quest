import { cn } from "@/lib/utils";

interface WeightageBarProps {
  total: number;
  max?: number;
  className?: string;
}

export function WeightageBar({ total, max = 100, className }: WeightageBarProps) {
  const percentage = Math.min(total, 150);
  const colorClass =
    total < max
      ? "bg-emerald-500"
      : total === max
        ? "bg-amber-500"
        : "bg-red-500";

  const textColor =
    total < max
      ? "text-emerald-400"
      : total === max
        ? "text-amber-400"
        : "text-red-400";

  return (
    <div className={cn("w-full space-y-1", className)}>
      <div className="flex items-center justify-between text-xs">
        <span className="text-slate-400">Weightage</span>
        <span className={cn("font-semibold", textColor)}>
          {total}% allocated
        </span>
      </div>
      <div className="h-2 w-full rounded-full bg-slate-800 overflow-hidden">
        <div
          className={cn("h-full rounded-full transition-all duration-300", colorClass)}
          style={{ width: `${Math.min(percentage, 100)}%` }}
        />
      </div>
      {total > max && (
        <p className="text-xs text-red-400">Weightage exceeds {max}% limit</p>
      )}
    </div>
  );
}
