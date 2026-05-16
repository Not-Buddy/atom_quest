import { useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useAuth } from "@/contexts/AuthContext";
import { getGoalSheet, addCheckinComment } from "@/lib/api";
import type { GoalSheetResponse, GoalResponse, AchievementResponse, CheckinCommentResponse, Quarter, QuarterLabel } from "@/lib/types";
import { QUARTER_LABELS, UOM_LABELS, UOM_COLORS } from "@/lib/types";
import { DashboardLayout } from "@/components/DashboardLayout";
import { StatusBadge } from "@/components/StatusBadge";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Skeleton } from "@/components/ui/skeleton";
import {
  ArrowLeft,
  Send,
  Loader2,
  AlertCircle,
  MessageSquare,
  CheckCircle2,
  Clock,
  Circle,
} from "lucide-react";

const QUARTER_TO_LOWER: Record<QuarterLabel, Quarter> = {
  Q1: "q1",
  Q2: "q2",
  Q3: "q3",
  Q4: "q4",
};

const ACHIEVEMENT_STATUS_ICONS: Record<string, React.ElementType> = {
  completed: CheckCircle2,
  on_track: Clock,
  not_started: Circle,
};

const ACHIEVEMENT_STATUS_COLORS: Record<string, string> = {
  completed: "text-emerald-400",
  on_track: "text-amber-400",
  not_started: "text-slate-500",
};

function scoreColor(score: number | null): string {
  if (score === null) return "text-slate-500";
  if (score >= 80) return "text-emerald-400";
  if (score >= 50) return "text-amber-400";
  return "text-red-400";
}

function formatDate(dateStr: string | null): string {
  if (!dateStr) return "—";
  try {
    return new Date(dateStr).toLocaleDateString("en-US", { month: "short", day: "numeric" });
  } catch {
    return "—";
  }
}

export default function CheckinView() {
  const { sheetId } = useParams<{ sheetId: string }>();
  const { token } = useAuth();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const numericId = Number(sheetId);

  const [activeQuarter, setActiveQuarter] = useState<QuarterLabel>("Q1");
  const [commentText, setCommentText] = useState("");
  const [error, setError] = useState("");

  const { data: sheet, isLoading } = useQuery<GoalSheetResponse>({
    queryKey: ["sheet", numericId],
    queryFn: () => getGoalSheet(token!, numericId),
    enabled: !!token && !Number.isNaN(numericId),
  });

  const saveCommentMutation = useMutation({
    mutationFn: () =>
      addCheckinComment(token!, numericId, {
        quarter: activeQuarter,
        comment: commentText,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["sheet", numericId] });
      setCommentText("");
      setError("");
    },
    onError: (err: Error) => setError(err.message),
  });

  const goals = sheet?.goals || [];
  const checkins = sheet?.checkins || [];
  const activeQuarterLower = QUARTER_TO_LOWER[activeQuarter];

  const filteredCheckins = checkins.filter(
    (c: CheckinCommentResponse) => c.quarter === activeQuarterLower
  );

  if (isLoading) {
    return (
      <DashboardLayout>
        <div className="flex items-center justify-center py-20">
          <Loader2 className="h-8 w-8 animate-spin text-indigo-500" />
        </div>
      </DashboardLayout>
    );
  }

  if (!sheet) {
    return (
      <DashboardLayout>
        <div className="flex flex-col items-center justify-center py-20 gap-4">
          <AlertCircle className="h-8 w-8 text-red-400" />
          <p className="text-slate-400">Goal sheet not found</p>
          <Button variant="outline" className="border-slate-700" onClick={() => navigate("/manager")}>
            <ArrowLeft className="mr-2 h-3.5 w-3.5" />
            Back
          </Button>
        </div>
      </DashboardLayout>
    );
  }

  return (
    <DashboardLayout>
      <div className="max-w-6xl mx-auto space-y-6">
        {/* Header */}
        <div>
          <button
            onClick={() => navigate("/manager")}
            className="text-sm text-slate-500 hover:text-slate-300 flex items-center gap-1 mb-2"
          >
            <ArrowLeft className="h-3.5 w-3.5" />
            Manager Dashboard
          </button>
          <div className="flex items-start justify-between gap-4 flex-wrap">
            <div>
              <h1 className="text-2xl font-bold text-slate-100">
                Check-in &mdash; {sheet.user_name || "Employee"}
              </h1>
              <p className="text-sm text-slate-400 mt-1">
                {sheet.cycle_name || "Unknown Cycle"}
              </p>
            </div>
            <StatusBadge status={sheet.status} className="px-3 py-1 text-sm" />
          </div>
        </div>

        {error && (
          <Alert variant="destructive" className="bg-red-950/50 border-red-800 text-red-400">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {/* Quarter Tabs */}
        <Tabs value={activeQuarter} onValueChange={(v) => setActiveQuarter(v as QuarterLabel)}>
          <TabsList className="bg-slate-900 border border-slate-800">
            {QUARTER_LABELS.map((q) => (
              <TabsTrigger
                key={q}
                value={q}
                className="data-[state=active]:bg-indigo-600 data-[state=active]:text-white"
              >
                {q}
              </TabsTrigger>
            ))}
          </TabsList>
        </Tabs>

        {/* Achievement Table */}
        <div className="rounded-xl border border-slate-800 overflow-auto">
          <Table>
            <TableHeader>
              <TableRow className="border-slate-800 hover:bg-transparent">
                <TableHead className="text-slate-400 font-medium">Goal Title</TableHead>
                <TableHead className="text-slate-400 font-medium">UoM</TableHead>
                <TableHead className="text-slate-400 font-medium text-right">Target</TableHead>
                <TableHead className="text-slate-400 font-medium text-right">Actual Value</TableHead>
                <TableHead className="text-slate-400 font-medium">Date</TableHead>
                <TableHead className="text-slate-400 font-medium">Status</TableHead>
                <TableHead className="text-slate-400 font-medium text-right">Score</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {goals.length === 0 ? (
                <TableRow className="border-slate-800">
                  <TableCell colSpan={7} className="text-center py-8 text-slate-500">
                    No goals in this sheet
                  </TableCell>
                </TableRow>
              ) : (
                goals.map((goal: GoalResponse) => {
                  const achievement = goal.achievements.find(
                    (a: AchievementResponse) => a.quarter === activeQuarterLower
                  );

                  const StatusIcon = achievement
                    ? ACHIEVEMENT_STATUS_ICONS[achievement.status] || Circle
                    : Circle;

                  const statusColor = achievement
                    ? ACHIEVEMENT_STATUS_COLORS[achievement.status] || "text-slate-500"
                    : "text-slate-500";

                  return (
                    <TableRow key={goal.id} className="border-slate-800">
                      <TableCell className="font-medium text-slate-200 max-w-[200px] truncate">
                        {goal.title}
                      </TableCell>
                      <TableCell>
                        {goal.uom_type ? (
                          <Badge
                            variant="outline"
                            className={`text-[10px] px-1.5 py-0 border ${UOM_COLORS[goal.uom_type]}`}
                          >
                            {UOM_LABELS[goal.uom_type] || goal.uom_type}
                          </Badge>
                        ) : (
                          <span className="text-slate-500 text-xs">—</span>
                        )}
                      </TableCell>
                      <TableCell className="text-right text-slate-300">
                        {goal.target_value}
                      </TableCell>
                      <TableCell className="text-right">
                        <span className={achievement?.actual_value != null ? "text-slate-200 font-medium" : "text-slate-600"}>
                          {achievement?.actual_value ?? "—"}
                        </span>
                      </TableCell>
                      <TableCell className="text-slate-400 text-xs">
                        {formatDate(achievement?.actual_date ?? null)}
                      </TableCell>
                      <TableCell>
                        <div className={`flex items-center gap-1.5 ${statusColor}`}>
                          <StatusIcon className="h-3.5 w-3.5" />
                          <span className="text-xs capitalize">
                            {achievement?.status?.replace("_", " ") || "not started"}
                          </span>
                        </div>
                      </TableCell>
                      <TableCell className="text-right">
                        <span className={`font-semibold ${scoreColor(achievement?.computed_score ?? null)}`}>
                          {achievement?.computed_score != null ? `${achievement.computed_score}%` : "—"}
                        </span>
                      </TableCell>
                    </TableRow>
                  );
                })
              )}
            </TableBody>
          </Table>
        </div>

        {/* Check-in Comments Section */}
        <div className="space-y-4">
          <div className="flex items-center gap-2">
            <MessageSquare className="h-4 w-4 text-slate-400" />
            <h3 className="text-sm font-semibold text-slate-300">Check-in Comments for {activeQuarter}</h3>
          </div>

          {/* Existing comments */}
          {filteredCheckins.length > 0 ? (
            <div className="space-y-2">
              {filteredCheckins.map((c: CheckinCommentResponse) => (
                <Card key={c.id} className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
                  <CardContent className="py-3">
                    <div className="flex items-center justify-between mb-1">
                      <span className="text-xs font-medium text-indigo-400">
                        {c.manager_name || "Manager"}
                      </span>
                      <span className="text-[10px] text-slate-600">
                        {c.created_at ? new Date(c.created_at).toLocaleString() : ""}
                      </span>
                    </div>
                    <p className="text-sm text-slate-300">{c.comment}</p>
                  </CardContent>
                </Card>
              ))}
            </div>
          ) : (
            <p className="text-xs text-slate-600">No comments yet for {activeQuarter}</p>
          )}

          {/* Add comment form */}
          <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
            <CardContent className="py-4 space-y-3">
              <Textarea
                value={commentText}
                onChange={(e) => setCommentText(e.target.value)}
                placeholder={`Add a check-in comment for ${activeQuarter}...`}
                className="bg-slate-800/50 border-slate-700 text-slate-100 placeholder:text-slate-600"
                rows={3}
              />
              <div className="flex justify-end">
                <Button
                  className="bg-indigo-600 hover:bg-indigo-500 text-white"
                  onClick={() => saveCommentMutation.mutate()}
                  disabled={saveCommentMutation.isPending || !commentText.trim()}
                >
                  {saveCommentMutation.isPending ? (
                    <>
                      <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
                      Saving...
                    </>
                  ) : (
                    <>
                      <Send className="mr-2 h-3.5 w-3.5" />
                      Save Comment
                    </>
                  )}
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </DashboardLayout>
  );
}
