import { useState, useEffect, useCallback, useMemo } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useAuth } from "@/contexts/AuthContext";
import { getGoalSheet, updateAchievement } from "@/lib/api";
import type {
  GoalResponse,
  AchievementResponse,
  AchievementStatus,
  QuarterLabel,
  Quarter,
  AchievementUpdatePayload,
} from "@/lib/types";
import { QUARTER_LABELS, UOM_LABELS } from "@/lib/types";
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
import { cn } from "@/lib/utils";
import {
  ArrowLeft,
  Save,
  Loader2,
  AlertCircle,
  CheckCircle,
  Clock,
  XCircle,
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

type EntryMap = Record<
  number,
  { actualValue: string; actualDate: string; status: AchievementStatus }
>;

function quarterLabelToQuarter(label: QuarterLabel): Quarter {
  return label.toLowerCase() as Quarter;
}

export default function AchievementEntry() {
  const { sheetId } = useParams<{ sheetId: string }>();
  const { token } = useAuth();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const numSheetId = sheetId ? parseInt(sheetId) : 0;

  const [activeTab, setActiveTab] = useState<QuarterLabel>("Q1");
  const [entries, setEntries] = useState<EntryMap>({});
  const [saveError, setSaveError] = useState("");
  const [saveSuccess, setSaveSuccess] = useState("");

  const { data: sheet, isLoading, isError } = useQuery({
    queryKey: ["sheet", numSheetId],
    queryFn: () => getGoalSheet(token!, numSheetId),
    enabled: !!token && !!numSheetId,
  });

  const goals: GoalResponse[] = useMemo(() => sheet?.goals ?? [], [sheet?.goals]);
  const currentQuarter = quarterLabelToQuarter(activeTab);

  // Seed entries from existing achievements when quarter changes
  const seedEntries = useCallback(() => {
    const currentQuarterLabel = quarterLabelToQuarter(activeTab);
    const seeded: EntryMap = {};
    for (const goal of goals) {
      const ach = goal.achievements.find(
        (a) => a.quarter === currentQuarterLabel
      );
      seeded[goal.id] = {
        actualValue: ach?.actual_value != null ? String(ach.actual_value) : "",
        actualDate: ach?.actual_date ?? "",
        status: ach?.status ?? "not_started",
      };
    }
    setEntries(seeded);
    setSaveError("");
    setSaveSuccess("");
  }, [goals, activeTab]);

  useEffect(() => {
    seedEntries();
  }, [seedEntries]);

  const saveMutation = useMutation({
    mutationFn: async () => {
      const quarter = quarterLabelToQuarter(activeTab);
      const promises = goals.map((goal) => {
        const entry = entries[goal.id];
        if (!entry) return Promise.resolve(null);
        const payload: AchievementUpdatePayload = {
          actual_value: entry.actualValue
            ? Number(entry.actualValue)
            : null,
          actual_date: entry.actualDate || null,
          status: entry.status,
        };
        return updateAchievement(token!, goal.id, quarter, payload);
      });
      await Promise.all(promises);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["sheet", numSheetId] });
      setSaveError("");
      setSaveSuccess(`${activeTab} achievements saved`);
      setTimeout(() => setSaveSuccess(""), 3000);
    },
    onError: (err: Error) => {
      setSaveError(err.message);
      setSaveSuccess("");
    },
  });

  const updateEntry = (
    goalId: number,
    field: "actualValue" | "actualDate" | "status",
    value: string
  ) => {
    setEntries((prev) => ({
      ...prev,
      [goalId]: {
        ...prev[goalId],
        [field]: field === "status" ? (value as AchievementStatus) : value,
      },
    }));
  };

  // Get computed score from the achievement data for current quarter
  const getComputedScore = (goal: GoalResponse): number | null => {
    const ach = goal.achievements.find((a) => a.quarter === currentQuarter);
    return ach?.computed_score ?? null;
  };

  // Count statuses for the progress bar
  const statusCounts = {
    not_started: 0,
    on_track: 0,
    completed: 0,
  };
  for (const goal of goals) {
    const entry = entries[goal.id];
    if (entry) {
      statusCounts[entry.status]++;
    }
  }

  if (isLoading) {
    return (
      <DashboardLayout>
        <div className="flex items-center justify-center py-20">
          <Loader2 className="h-8 w-8 animate-spin text-indigo-500" />
        </div>
      </DashboardLayout>
    );
  }

  if (isError || !sheet) {
    return (
      <DashboardLayout>
        <div className="flex flex-col items-center justify-center py-20 gap-4">
          <AlertCircle className="h-8 w-8 text-red-400" />
          <p className="text-slate-400">Goal sheet not found</p>
          <Button
            variant="outline"
            onClick={() => navigate("/employee")}
            className="border-slate-700 text-slate-300 hover:bg-slate-800"
          >
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
            <h1 className="text-2xl font-bold text-slate-100">
              {sheet.cycle_name || "Goal Sheet"}
            </h1>
            <p className="text-sm text-slate-400">Achievement Entry</p>
          </div>
          <StatusBadge status={sheet.status} />
        </div>

        {/* Success / Error alerts */}
        {saveError && (
          <Alert className="bg-red-950/50 border-red-800 text-red-400">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{saveError}</AlertDescription>
          </Alert>
        )}
        {saveSuccess && (
          <Alert className="bg-emerald-950/30 border-emerald-800 text-emerald-300">
            <CheckCircle className="h-4 w-4" />
            <AlertDescription>{saveSuccess}</AlertDescription>
          </Alert>
        )}

        {/* Quarter Tabs */}
        <Tabs
          value={activeTab}
          onValueChange={(v) => setActiveTab(v as QuarterLabel)}
          className="w-full"
        >
          <TabsList className="bg-slate-900 border border-slate-800 w-fit">
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

          <TabsContent value={activeTab}>
            {goals.length === 0 ? (
              <div className="text-center py-12 text-slate-500">
                No goals in this sheet
              </div>
            ) : (
              <div className="rounded-xl border border-slate-800 overflow-hidden">
                <Table>
                  <TableHeader>
                    <TableRow className="border-slate-800 hover:bg-transparent">
                      <TableHead className="text-slate-400 font-medium">
                        Goal
                      </TableHead>
                      <TableHead className="text-slate-400 font-medium">
                        UOM
                      </TableHead>
                      <TableHead className="text-slate-400 font-medium text-right">
                        Target
                      </TableHead>
                      <TableHead className="text-slate-400 font-medium text-right">
                        Actual
                      </TableHead>
                      <TableHead className="text-slate-400 font-medium">
                        Status
                      </TableHead>
                      <TableHead className="text-slate-400 font-medium text-right">
                        Score
                      </TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {goals.map((goal) => {
                      const entry = entries[goal.id] ?? {
                        actualValue: "",
                        actualDate: "",
                        status: "not_started" as AchievementStatus,
                      };
                      const score = getComputedScore(goal);
                      const StatusIcon = STATUS_ICONS[entry.status];

                      return (
                        <>
                          <TableRow className="border-slate-800">
                            <TableCell className="font-medium text-slate-200">
                              {goal.title}
                            </TableCell>
                            <TableCell className="text-slate-400 text-xs uppercase">
                              {UOM_LABELS[goal.uom_type]}
                            </TableCell>
                            <TableCell className="text-right text-slate-300">
                              {goal.target_value}
                              {goal.uom_type === "min_percent" ||
                              goal.uom_type === "max_percent"
                                ? "%"
                                : ""}
                            </TableCell>
                            <TableCell className="text-right">
                              {goal.uom_type === "timeline" ? (
                                <Input
                                  type="date"
                                  value={entry.actualDate}
                                  onChange={(e) =>
                                    updateEntry(
                                      goal.id,
                                      "actualDate",
                                      e.target.value
                                    )
                                  }
                                  className="w-40 ml-auto bg-slate-800/50 border-slate-700 text-slate-100"
                                />
                              ) : (
                                <Input
                                  type="number"
                                  value={entry.actualValue}
                                  onChange={(e) =>
                                    updateEntry(
                                      goal.id,
                                      "actualValue",
                                      e.target.value
                                    )
                                  }
                                  className="w-24 ml-auto bg-slate-800/50 border-slate-700 text-slate-100 text-right"
                                  placeholder="—"
                                />
                              )}
                            </TableCell>
                            <TableCell>
                              <Select
                                value={entry.status}
                                onValueChange={(v) =>
                                  updateEntry(goal.id, "status", v)
                                }
                              >
                                <SelectTrigger className="w-36 bg-slate-800/50 border-slate-700 text-slate-200">
                                  <SelectValue />
                                </SelectTrigger>
                                <SelectContent className="bg-slate-900 border-slate-700 text-slate-100">
                                  <SelectItem value="not_started">
                                    Not Started
                                  </SelectItem>
                                  <SelectItem value="on_track">
                                    On Track
                                  </SelectItem>
                                  <SelectItem value="completed">
                                    Completed
                                  </SelectItem>
                                </SelectContent>
                              </Select>
                            </TableCell>
                            <TableCell className="text-right">
                              {score != null ? (
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
                              ) : (
                                <span className="text-slate-600">—</span>
                              )}
                            </TableCell>
                          </TableRow>
                          {/* Actual Date row for non-timeline goals with a date */}
                          {goal.uom_type !== "timeline" &&
                            entry.actualDate && (
                              <TableRow className="border-slate-800">
                                <TableCell
                                  colSpan={3}
                                  className="text-xs text-slate-500"
                                >
                                  Actual Date
                                </TableCell>
                                <TableCell colSpan={3} className="text-xs text-slate-400">
                                  {entry.actualDate}
                                </TableCell>
                              </TableRow>
                            )}
                        </>
                      );
                    })}
                  </TableBody>
                </Table>
              </div>
            )}
          </TabsContent>
        </Tabs>

        {/* Progress indicator + Save */}
        {goals.length > 0 && (
          <div className="border-t border-slate-800 pt-4">
            <div className="flex items-center justify-between text-sm">
              <div className="flex items-center gap-4">
                {(
                  ["not_started", "on_track", "completed"] as AchievementStatus[]
                ).map((st) => {
                  const count = statusCounts[st];
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
                onClick={() => saveMutation.mutate()}
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
                    Save {activeTab} Achievements
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

