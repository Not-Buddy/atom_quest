import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Edit, Trash2 } from "lucide-react";
import { cn } from "@/lib/utils";
import { Goal, UOM_LABELS, UOM_COLORS } from "@/lib/types";

interface GoalCardProps {
  goal: Goal;
  readOnly?: boolean;
  onEdit?: (goal: Goal) => void;
  onDelete?: (goalId: string) => void;
  actualValue?: number;
  score?: number;
  className?: string;
}

export function GoalCard({
  goal,
  readOnly = false,
  onEdit,
  onDelete,
  actualValue,
  score,
  className,
}: GoalCardProps) {
  const uomBadgeColor = UOM_COLORS[goal.uom_type] || "bg-slate-800 text-slate-400";

  return (
    <Card className={cn("bg-slate-900/80 backdrop-blur-sm border border-slate-800", className)}>
      <CardHeader className="pb-2">
        <div className="flex items-start justify-between gap-2">
          <div className="flex items-center gap-2 min-w-0">
            <CardTitle className="text-base font-semibold text-slate-100 truncate">
              {goal.title}
            </CardTitle>
            <Badge className={cn("shrink-0 text-xs border", uomBadgeColor)} variant="outline">
              {UOM_LABELS[goal.uom_type]}
            </Badge>
          </div>
          {!readOnly && onEdit && (
            <div className="flex items-center gap-1 shrink-0">
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7 text-slate-400 hover:text-indigo-400"
                onClick={() => onEdit(goal)}
              >
                <Edit className="h-3.5 w-3.5" />
              </Button>
              {onDelete && (
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-7 w-7 text-slate-400 hover:text-red-400"
                  onClick={() => onDelete(goal.id)}
                >
                  <Trash2 className="h-3.5 w-3.5" />
                </Button>
              )}
            </div>
          )}
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        {goal.description && (
          <p className="text-sm text-slate-400">{goal.description}</p>
        )}

        <div className="grid grid-cols-2 gap-3 text-sm">
          <div>
            <span className="text-slate-500">Target</span>
            <p className="font-semibold text-slate-200">{goal.target_value}</p>
          </div>
          <div>
            <span className="text-slate-500">Weightage</span>
            <p className="font-semibold text-indigo-400">{goal.weightage}%</p>
          </div>
        </div>

        <div className="text-sm">
          <span className="text-slate-500">Thrust Area</span>
          <p className="font-medium text-slate-300">{goal.thrust_area}</p>
        </div>

        {actualValue !== undefined && (
          <div className="mt-2 pt-2 border-t border-slate-800">
            <div className="flex items-center justify-between text-sm">
              <span className="text-slate-500">Actual</span>
              <span className="font-semibold text-slate-200">{actualValue}</span>
            </div>
            {score !== undefined && (
              <div className="flex items-center justify-between text-sm mt-1">
                <span className="text-slate-500">Score</span>
                <span
                  className={cn(
                    "font-semibold",
                    score >= 80 ? "text-emerald-400" : score >= 50 ? "text-amber-400" : "text-red-400"
                  )}
                >
                  {Math.round(score)}%
                </span>
              </div>
            )}
            <div className="mt-2 h-1.5 w-full rounded-full bg-slate-800 overflow-hidden">
              <div
                className={cn(
                  "h-full rounded-full transition-all",
                  score !== undefined
                    ? score >= 80
                      ? "bg-emerald-500"
                      : score >= 50
                        ? "bg-amber-500"
                        : "bg-red-500"
                    : "bg-indigo-500"
                )}
                style={{ width: `${Math.min(score ?? 0, 100)}%` }}
              />
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
