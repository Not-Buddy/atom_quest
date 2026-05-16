import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Edit, Trash2, Share2 } from "lucide-react";
import { cn } from "@/lib/utils";
import type { GoalResponse, UomType } from "@/lib/types";
import { UOM_LABELS, UOM_COLORS } from "@/lib/types";

interface GoalCardProps {
  goal: GoalResponse;
  readOnly?: boolean;
  onEdit?: (g: GoalResponse) => void;
  onDelete?: (id: number) => void;
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
  const uomType: UomType = goal.uom_type as UomType;
  const uomBadgeColor = UOM_COLORS[uomType] || "bg-slate-800 text-slate-400 border-slate-700";

  return (
    <Card
      className={cn(
        "bg-slate-900/80 backdrop-blur-sm border border-slate-800",
        className,
      )}
    >
      <CardHeader className="pb-2">
        <div className="flex items-start justify-between gap-2">
          <div className="flex items-center gap-2 min-w-0 flex-wrap">
            <CardTitle className="text-base font-semibold text-slate-100 truncate">
              {goal.title}
            </CardTitle>
            <Badge
              className={cn("shrink-0 text-xs border", uomBadgeColor)}
              variant="outline"
            >
              {UOM_LABELS[uomType] || goal.uom_type}
            </Badge>
            {goal.is_shared && (
              <Badge
                variant="outline"
                className="shrink-0 text-xs border bg-teal-500/10 text-teal-400 border-teal-500/30"
              >
                <Share2 className="h-3 w-3 mr-1" />
                Shared
              </Badge>
            )}
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

        {goal.thrust_area_name && (
          <div className="text-sm">
            <span className="text-slate-500">Thrust Area</span>
            <p className="font-medium text-slate-300">{goal.thrust_area_name}</p>
          </div>
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

        {(actualValue !== undefined || score !== undefined) && (
          <div className="mt-2 pt-2 border-t border-slate-800">
            {actualValue !== undefined && (
              <div className="flex items-center justify-between text-sm">
                <span className="text-slate-500">Actual</span>
                <span className="font-semibold text-slate-200">{actualValue}</span>
              </div>
            )}
            {score !== undefined && (
              <div className="flex items-center justify-between text-sm mt-1">
                <span className="text-slate-500">Score</span>
                <span
                  className={cn(
                    "font-semibold",
                    score >= 80
                      ? "text-emerald-400"
                      : score >= 50
                        ? "text-amber-400"
                        : "text-red-400",
                  )}
                >
                  {Math.round(score)}%
                </span>
              </div>
            )}
            {score !== undefined && (
              <div className="mt-2 h-1.5 w-full rounded-full bg-slate-800 overflow-hidden">
                <div
                  className={cn(
                    "h-full rounded-full transition-all",
                    score >= 80
                      ? "bg-emerald-500"
                      : score >= 50
                        ? "bg-amber-500"
                        : "bg-red-500",
                  )}
                  style={{ width: `${Math.min(score, 100)}%` }}
                />
              </div>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
