import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import { SheetStatus, STATUS_CONFIG } from "@/lib/types";

interface StatusBadgeProps {
  status: SheetStatus;
  className?: string;
}

export function StatusBadge({ status, className }: StatusBadgeProps) {
  const config = STATUS_CONFIG[status];

  return (
    <Badge
      variant={config.variant}
      className={cn("text-xs font-medium", className)}
    >
      {config.label}
    </Badge>
  );
}
