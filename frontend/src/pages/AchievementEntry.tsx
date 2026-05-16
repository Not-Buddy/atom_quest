import { useState, useMemo } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useAuth } from "@/contexts/AuthContext";
import { fetchSheet, fetchAchievements, saveAchievements } from "@/lib/api";
import { Quarter, QUARTERS, Achievement, AchievementStatus } from "@/lib/types";
import { DashboardLayout } from "@/components/DashboardLayout";
import { StatusBadge } from "@/components/StatusBadge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Alert, AlertDescription } from "@/components/ui/alert";
import {
  ArrowLeft,
  Save,
  Loader2,
  AlertCircle,
  CheckCircle,
  Clock,
  XCircle,
  TrendingUp,
} from "lucide-react";

const STATUS_ICONS: Record<AchievementStatus, React.ElementType> = {
  not_started: XCircle,
  on_track: Clock,
  completed: CheckCircle,
};

const STATUS_COLORS: Record<AchievementStatus, string> = {
  not_started: "text-slate-500",
  on_track: "text-amber-400",
  completed: "text-emerald-400",
};

export default function AchievementEntry() {
  const { sheetId } = useParams<{ sheetId: string }>();
  const { token } = useAuth();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const [activeQuarter, setActiveQuarter] = useState<Quarter>("Q1");
  const [entries, setEntries] = useState<Record<string, { actual: string; status: AchievementStatus }>>({});
  const [saveError, setSaveError] = useState("");

  const { data: sheetData, isLoading: sheetLoading } = useQuery({
    queryKey: ["sheet", sheetId],
    queryFn: () => fetchSheet(token!, sheetId!),
    enabled: !!token && !!sheetId,
  });

  const { data: achData, isLoading: achLoading } = useQuery({
    queryKey: ["achievements", sheetId, activeQuarter],
    queryFn: () => fetchAchievements(token!, sheetId!, activeQuarter),
    enabled: !!token && !!sheetId,
  });

  const saveMutation = useMutation({
    mutationFn: (payload: { goal_id: string; actual_value: number; status: AchievementStatus; comment?: string }[]) =>
      saveAchievements(token!, sheetId!, activeQuarter, payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["achievements", sheetId] });
      setSaveError("");
    },
    onError: (err: Error) => setSaveError(err.message),
  });

  const sheet = sheetData?.sheet;
  const goals = sheet?.goals || [];
  const achievements = achData?.achievements || [];

  // Seed entries from existing achievements
  useMemo(() => {
    const seeded: Record<string, { actual: string; status: AchievementStatus }> = { ...entries };
    achievements.forEach((a) => {
      if (!seeded[a.goal_id]) {
        seeded[a.goal_id] = { actual: String(a.actual_value), status: a.status };
      }
    });
    return seeded;
  }, [achievements]);

  const handleActualChange = (goalId: string, value: string) => {
    setEntries((prev) => ({
      ...prev,
      [goalId]: { ...prev[goalId], actual: value },
    }));
  };

  const handleStatusChange = (goalId: string, status: AchievementStatus) => {
    setEntries((prev) => ({
      ...prev,
      [goalId]: { ...prev[goalId], status },
    }));
  };

  const handleSave = () => {
    const payload = goals
      .filter((g) => entries[g.id]?.actual)
      .map((g) => ({
        goal_id: g.id,
        actual_value: Number(entries[g.id].actual),
        status: entries[g.id].status || "not_started",
      }));
    if (payload.length === 0) {
      setSaveError("Enter at least one actual value");
      return;
    }
    saveMutation.mutate(payload);
  };

  const computedScore = (goalId: string) => {
    const entry = entries[goalId];
    const existing = achievements.find((a) => a.goal_id === goalId);
    const actual = entry?.actual ? Number(entry.actual) : existing?.actual_value || 0;
    const goal = goals.find((g) => g.id === goalId);
    if (!goal || !actual) return 0;

    if (goal.uom_type === "min_numeric" || goal.uom_type === "zero") {
      return Math.max(0, Math.min(100, (goal.target_value / Math.max(actual, 1)) * 100));
    }
    return Math.max(0, Math.min(100, (actual / goal.target_value) * 100));
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
          <Button variant="outline" onClick={() => navigate("/employee")}>
            <ArrowLeft className="mr-2 h-3.5 w-3.5" />
            Back
          </Button>
        </div>
      </DashboardLayout>
    );
  }

  return (
    <DashboardLayout>
      <div className="max-w-5xl mx-auto space-y-6">
        {/* Header */}
        <div className="flex items-center gap-3">
          <button
            onClick={() => navigate("/employee")}
            className="text-slate-500 hover:text-slate-300"
          >
            <ArrowLeft className="h-4 w-4" />
          </button>
          <div>
            <h1 className="text-2xl font-bold text-slate-100">{sheet.cycle_name || "Goal Sheet"}</h1>
            <p className="text-sm text-slate-400">Achievement Entry</p>
          </div>
        </div>

        <StatusBadge status={sheet.status} />

        {/* Quarter Tabs */}
        <Tabs
          value={activeQuarter}
          onValueChange={(v) => setActiveQuarter(v as Quarter)}
          className="w-full"
        >
          <TabsList className="bg-slate-900 border border-slate-800 w-fit">
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

        {/* Error */}
        {saveError && (
          <Alert variant="destructive" className="bg-red-950/50 border-red-800 text-red-400">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{saveError}</AlertDescription>
          </Alert>
        )}

        {/* Table */}
        {achLoading ? (
          <div className="flex justify-center py-12">
            <Loader2 className="h-6 w-6 animate-spin text-indigo-500" />
          </div>
        ) : goals.length === 0 ? (
          <div className="text-center py-12 text-slate-500">No goals in this sheet</div>
        ) : (
          <div className="rounded-xl border border-slate-800 overflow-hidden">
            <Table>
              <TableHeader>
                <TableRow className="border-slate-800 hover:bg-transparent">
                  <TableHead className="text-slate-400 font-medium">Goal</TableHead>
                  <TableHead className="text-slate-400 font-medium">UOM</TableHead>
                  <TableHead className="text-slate-400 font-medium text-right">Target</TableHead>
                  <TableHead className="text-slate-400 font-medium text-right">Actual</TableHead>
                  <TableHead className="text-slate-400 font-medium">Status</TableHead>
                  <TableHead className="text-slate-400 font-medium text-right">Score</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {goals.map((goal) => {
                  const entry = entries[goal.id] || {};
                  const existing = achievements.find((a) => a.goal_id === goal.id);
                  const currentActual = entry.actual || existing?.actual_value?.toString() || "";
                  const currentStatus = entry.status || existing?.status || "not_started";
                  const score = computedScore(goal.id);
                  const StatusIcon = STATUS_ICONS[currentStatus];

                  return (
                    <TableRow key={goal.id} className="border-slate-800">
                      <TableCell className="font-medium text-slate-200">
                        {goal.title}
                      </TableCell>
                      <TableCell className="text-slate-400 text-xs uppercase">
                        {goal.uom_type.replace("_", " ")}
                      </TableCell>
                      <TableCell className="text-right text-slate-300">
                        {goal.target_value}
                      </TableCell>
                      <TableCell className="text-right">
                        <Input
                          type="number"
                          value={currentActual}
                          onChange={(e) => handleActualChange(goal.id, e.target.value)}
                          className="w-24 ml-auto bg-slate-800/50 border-slate-700 text-slate-100 text-right"
                          placeholder="—"
                        />
                      </TableCell>
                      <TableCell>
                        <Select
                          value={currentStatus}
                          onValueChange={(v) => handleStatusChange(goal.id, v as AchievementStatus)}
                        >
                          <SelectTrigger className="w-36 bg-slate-800/50 border-slate-700 text-slate-200">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent className="bg-slate-900 border-slate-700 text-slate-100">
                            <SelectItem value="not_started">Not Started</SelectItem>
                            <SelectItem value="on_track">On Track</SelectItem>
                            <SelectItem value="completed">Completed</SelectItem>
                          </SelectContent>
                        </Select>
                      </TableCell>
                      <TableCell className="text-right">
                        <span
                          className={
                            score >= 80
                              ? "text-emerald-400"
                              : score >= 50
                                ? "text-amber-400"
                                : "text-red-400"
                          }
                        >
                          {Math.round(score)}%
                        </span>
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </div>
        )}

        {/* Progress indicator */}
        {goals.length > 0 && (
          <div className="border-t border-slate-800 pt-4">
            <div className="flex items-center justify-between text-sm">
              <div className="flex items-center gap-4">
                {(["not_started", "on_track", "completed"] as AchievementStatus[]).map((st) => {
                  const count = Object.values(entries).filter(
                    (e) => e.status === st
                  ).length;
                  const Icon = STATUS_ICONS[st];
                  return (
                    <div key={st} className={STATUS_COLORS[st]}>
                      <Icon className="h-3.5 w-3.5 inline mr-1" />
                      {count} {st.replace("_", " ")}
                    </div>
                  );
                })}
              </div>
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
                    Save {activeQuarter} Achievements
                  </>
                )}
              </Button>
            </div>
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}
