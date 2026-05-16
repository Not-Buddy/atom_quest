import { cn } from "@/lib/utils";

interface ProgressBarProps {
  value: number;
  max: number;
  className?: string;
  showLabel?: boolean;
}

export function ProgressBar({ value, max, className, showLabel = true }: ProgressBarProps) {
  const percentage = Math.min(100, Math.max(0, (value / max) * 100));
  
  const getColor = () => {
    if (percentage >= 100) return "bg-success";
    if (percentage >= 70) return "bg-accent";
    if (percentage >= 40) return "bg-warning";
    return "bg-destructive";
  };

  return (
    <div className={cn("flex items-center gap-3", className)}>
      <div className="flex-1 h-2.5 bg-muted rounded-full overflow-hidden">
        <div
          className={cn("h-full rounded-full transition-all duration-500", getColor())}
          style={{ width: `${percentage}%` }}
        />
      </div>
      {showLabel && (
        <span className="text-sm font-medium text-muted-foreground min-w-[48px] text-right">
          {percentage.toFixed(0)}%
        </span>
      )}
    </div>
  );
}
