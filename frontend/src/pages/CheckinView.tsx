import { useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useAuth } from "@/contexts/AuthContext";
import { fetchSheet, fetchCheckin, saveCheckin } from "@/lib/api";
import { Quarter, QUARTERS, AchievementStatus } from "@/lib/types";
import { DashboardLayout } from "@/components/DashboardLayout";
import { StatusBadge } from "@/components/StatusBadge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Card, CardContent } from "@/components/ui/card";
import { Alert, AlertDescription } from "@/components/ui/alert";
import {
  ArrowLeft,
  Save,
  Loader2,
  AlertCircle,
} from "lucide-react";

const SCORE_COLORS = (score: number) =>
  score >= 80 ? "text-emerald-400" : score >= 50 ? "text-amber-400" : "text-red-400";

export default function CheckinView() {
  const { sheetId } = useParams<{ sheetId: string }>();
  const { token } = useAuth();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const [activeQuarter, setActiveQuarter] = useState<Quarter>("Q1");
  const [comments, setComments] = useState<Record<string, string>>({});
  const [error, setError] = useState("");

  const { data: sheetData, isLoading: sheetLoading } = useQuery({
    queryKey: ["sheet", sheetId],
    queryFn: () => fetchSheet(token!, sheetId!),
    enabled: !!token && !!sheetId,
  });

  const { data: checkinData, isLoading: checkinLoading } = useQuery({
    queryKey: ["checkin", sheetId],
    queryFn: () => fetchCheckin(token!, sheetId!),
    enabled: !!token && !!sheetId,
  });

  const saveMutation = useMutation({
    mutationFn: (entries: {
      goal_id: string;
      q1_actual: number;
      q2_actual: number;
      q3_actual: number;
      q4_actual: number;
      q1_status: AchievementStatus;
      q2_status: AchievementStatus;
      q3_status: AchievementStatus;
      q4_status: AchievementStatus;
      q1_score: number;
      q2_score: number;
      q3_score: number;
      q4_score: number;
      comment?: string;
    }[]) => saveCheckin(token!, sheetId!, entries),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["checkin", sheetId] });
      setError("");
    },
    onError: (err: Error) => setError(err.message),
  });

  const sheet = sheetData?.sheet;
  const goals = sheet?.goals || [];
  const checkinEntries = checkinData?.checkin || [];

  const handleSave = () => {
    const entries = goals.map((g) => {
      const existing = checkinEntries.find((c) => c.goal_id === g.id) || {};
      return {
        goal_id: g.id,
        q1_actual: Number((document.getElementById(`q1_${g.id}`) as HTMLInputElement)?.value) || existing.q1_actual || 0,
        q2_actual: Number((document.getElementById(`q2_${g.id}`) as HTMLInputElement)?.value) || existing.q2_actual || 0,
        q3_actual: Number((document.getElementById(`q3_${g.id}`) as HTMLInputElement)?.value) || existing.q3_actual || 0,
        q4_actual: Number((document.getElementById(`q4_${g.id}`) as HTMLInputElement)?.value) || existing.q4_actual || 0,
        q1_status: "not_started" as AchievementStatus,
        q2_status: "not_started" as AchievementStatus,
        q3_status: "not_started" as AchievementStatus,
        q4_status: "not_started" as AchievementStatus,
        q1_score: computeScore(g, Number((document.getElementById(`q1_${g.id}`) as HTMLInputElement)?.value) || 0),
        q2_score: computeScore(g, Number((document.getElementById(`q2_${g.id}`) as HTMLInputElement)?.value) || 0),
        q3_score: computeScore(g, Number((document.getElementById(`q3_${g.id}`) as HTMLInputElement)?.value) || 0),
        q4_score: computeScore(g, Number((document.getElementById(`q4_${g.id}`) as HTMLInputElement)?.value) || 0),
        comment: comments[g.id] || existing.comment || "",
      };
    });
    saveMutation.mutate(entries);
  };

  const computeScore = (goal: typeof goals[0], actual: number) => {
    if (!actual || !goal.target_value) return 0;
    if (goal.uom_type === "min_numeric" || goal.uom_type === "zero") {
      return Math.round(Math.max(0, Math.min(100, (goal.target_value / Math.max(actual, 1)) * 100)));
    }
    return Math.round(Math.max(0, Math.min(100, (actual / goal.target_value) * 100)));
  };

  if (sheetLoading) {
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
          <Button variant="outline" onClick={() => navigate("/manager")}>
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
              <p className="text-sm text-slate-400">
                {sheet.cycle_name} &middot; {sheet.user_department}
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
        <Tabs value={activeQuarter} onValueChange={(v) => setActiveQuarter(v as Quarter)}>
          <TabsList className="bg-slate-900 border border-slate-800">
            {QUARTERS.map((q) => (
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

        {/* Checkin Table */}
        {checkinLoading ? (
          <div className="flex justify-center py-12">
            <Loader2 className="h-6 w-6 animate-spin text-indigo-500" />
          </div>
        ) : goals.length === 0 ? (
          <div className="text-center py-12 text-slate-500">No goals in this sheet</div>
        ) : (
          <div className="rounded-xl border border-slate-800 overflow-auto">
            <Table>
              <TableHeader>
                <TableRow className="border-slate-800 hover:bg-transparent">
                  <TableHead className="text-slate-400 font-medium">Goal</TableHead>
                  <TableHead className="text-slate-400 font-medium text-right">Target</TableHead>
                  <TableHead className="text-slate-400 font-medium text-right">Q1 Actual</TableHead>
                  <TableHead className="text-slate-400 font-medium text-right">Q2 Actual</TableHead>
                  <TableHead className="text-slate-400 font-medium text-right">Q3 Actual</TableHead>
                  <TableHead className="text-slate-400 font-medium text-right">Q4 Actual</TableHead>
                  <TableHead className="text-slate-400 font-medium text-right">Q1 Score</TableHead>
                  <TableHead className="text-slate-400 font-medium text-right">Q2 Score</TableHead>
                  <TableHead className="text-slate-400 font-medium text-right">Q3 Score</TableHead>
                  <TableHead className="text-slate-400 font-medium text-right">Q4 Score</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {goals.map((goal) => {
                  const existing = checkinEntries.find((c) => c.goal_id === goal.id) || ({} as any);
                  return (
                    <TableRow key={goal.id} className="border-slate-800">
                      <TableCell className="font-medium text-slate-200 max-w-[200px] truncate">
                        {goal.title}
                      </TableCell>
                      <TableCell className="text-right text-slate-300">{goal.target_value}</TableCell>
                      {(["q1", "q2", "q3", "q4"] as const).map((q) => (
                        <TableCell key={q} className="text-right">
                          <Input
                            id={`${q}_${goal.id}`}
                            type="number"
                            defaultValue={existing[`${q}_actual`] || ""}
                            className="w-20 ml-auto bg-slate-800/50 border-slate-700 text-slate-100 text-right"
                            placeholder="—"
                          />
                        </TableCell>
                      ))}
                      {(["q1", "q2", "q3", "q4"] as const).map((q) => (
                        <TableCell key={`score_${q}`} className="text-right">
                          <span className={SCORE_COLORS(existing[`${q}_score`] || 0)}>
                            {existing[`${q}_score`] || 0}%
                          </span>
                        </TableCell>
                      ))}
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </div>
        )}

        {/* Comments section */}
        {goals.length > 0 && (
          <div className="space-y-3">
            <h3 className="text-sm font-medium text-slate-400">Check-in Comments</h3>
            {goals.map((goal) => {
              const existing = checkinEntries.find((c) => c.goal_id === goal.id);
              return (
                <Card key={goal.id} className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
                  <CardContent className="py-3">
                    <p className="text-sm font-medium text-slate-300 mb-2">{goal.title}</p>
                    <Textarea
                      defaultValue={existing?.comment || ""}
                      onChange={(e) =>
                        setComments((prev) => ({ ...prev, [goal.id]: e.target.value }))
                      }
                      placeholder="Add comment..."
                      className="bg-slate-800/50 border-slate-700 text-slate-100 placeholder:text-slate-600"
                      rows={2}
                    />
                  </CardContent>
                </Card>
              );
            })}
          </div>
        )}

        {/* Save */}
        {goals.length > 0 && (
          <div className="border-t border-slate-800 pt-4">
            <Button
              onClick={handleSave}
              disabled={saveMutation.isPending}
              className="bg-indigo-600 hover:bg-indigo-500 text-white"
            >
              {saveMutation.isPending ? (
                <>
                  <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
                  Saving...
                </>
              ) : (
                <>
                  <Save className="mr-2 h-3.5 w-3.5" />
                  Save Check-in
                </>
              )}
            </Button>
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}
