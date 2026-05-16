import { useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useAuth } from "@/contexts/AuthContext";
import {
  getGoalSheet,
  addGoal,
  updateGoal,
  deleteGoal,
  submitSheet,
  listThrustAreas,
} from "@/lib/api";
import type {
  GoalSheetResponse,
  GoalResponse,
  ThrustArea,
  UomType,
  CreateGoalPayload,
  UpdateGoalPayload,
} from "@/lib/types";
import { UOM_LABELS, UOM_COLORS } from "@/lib/types";
import { DashboardLayout } from "@/components/DashboardLayout";
import { StatusBadge } from "@/components/StatusBadge";
import { WeightageBar } from "@/components/WeightageBar";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Slider } from "@/components/ui/slider";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { cn } from "@/lib/utils";
import {
  Plus,
  Loader2,
  AlertCircle,
  ArrowLeft,
  Send,
  Edit,
  Trash2,
  Lock,
  Share2,
} from "lucide-react";

const UOM_OPTIONS: { value: UomType; label: string }[] = [
  { value: "min_numeric", label: UOM_LABELS.min_numeric },
  { value: "max_numeric", label: UOM_LABELS.max_numeric },
  { value: "min_percent", label: UOM_LABELS.min_percent },
  { value: "max_percent", label: UOM_LABELS.max_percent },
  { value: "timeline", label: UOM_LABELS.timeline },
  { value: "zero", label: UOM_LABELS.zero },
];

export default function GoalSheetEditor() {
  const { sheetId } = useParams<{ sheetId: string }>();
  const { token } = useAuth();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const numSheetId = sheetId ? parseInt(sheetId) : 0;

  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingGoal, setEditingGoal] = useState<GoalResponse | null>(null);
  const [error, setError] = useState("");

  // Goal form state
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [uomType, setUomType] = useState<UomType>("max_numeric");
  const [targetValue, setTargetValue] = useState("");
  const [targetDate, setTargetDate] = useState("");
  const [weightage, setWeightage] = useState(20);
  const [thrustAreaId, setThrustAreaId] = useState<number | null>(null);

  const { data: sheet, isLoading, isError } = useQuery({
    queryKey: ["sheet", numSheetId],
    queryFn: () => getGoalSheet(token!, numSheetId),
    enabled: !!token && !!numSheetId,
  });

  const { data: thrustAreas = [] } = useQuery({
    queryKey: ["thrust-areas"],
    queryFn: () => listThrustAreas(token!),
    enabled: !!token,
  });

  const submitMutation = useMutation({
    mutationFn: () => submitSheet(token!, numSheetId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["sheet", numSheetId] });
      queryClient.invalidateQueries({ queryKey: ["my-sheets"] });
    },
    onError: (err: Error) => setError(err.message),
  });

  const createGoalMutation = useMutation({
    mutationFn: (payload: CreateGoalPayload) =>
      addGoal(token!, numSheetId, payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["sheet", numSheetId] });
      closeDialog();
    },
    onError: (err: Error) => setError(err.message),
  });

  const updateGoalMutation = useMutation({
    mutationFn: ({
      goalId,
      payload,
    }: {
      goalId: number;
      payload: UpdateGoalPayload;
    }) => updateGoal(token!, goalId, payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["sheet", numSheetId] });
      closeDialog();
    },
    onError: (err: Error) => setError(err.message),
  });

  const deleteGoalMutation = useMutation({
    mutationFn: (goalId: number) => deleteGoal(token!, goalId),
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: ["sheet", numSheetId] }),
  });

  const goals: GoalResponse[] = sheet?.goals ?? [];
  const isDraft = sheet?.status === "draft";
  const isReturned = sheet?.status === "returned";
  const totalWeightage = goals.reduce((sum, g) => sum + g.weightage, 0);
  const goalsCount = goals.length;
  const maxGoals = 8;

  const resetForm = () => {
    setTitle("");
    setDescription("");
    setUomType("max_numeric");
    setTargetValue("");
    setTargetDate("");
    setWeightage(20);
    setThrustAreaId(null);
    setError("");
  };

  const openAddDialog = () => {
    setEditingGoal(null);
    resetForm();
    setDialogOpen(true);
  };

  const openEditDialog = (goal: GoalResponse) => {
    setEditingGoal(goal);
    setTitle(goal.title);
    setDescription(goal.description || "");
    setUomType(goal.uom_type);
    setTargetValue(String(goal.target_value));
    setTargetDate(goal.target_date || "");
    setWeightage(goal.weightage);
    setThrustAreaId(goal.thrust_area_id);
    setError("");
    setDialogOpen(true);
  };

  const closeDialog = () => {
    setDialogOpen(false);
    setEditingGoal(null);
    setError("");
  };

  const handleSaveGoal = () => {
    setError("");

    if (!title.trim()) {
      setError("Title is required");
      return;
    }
    if (!targetValue || isNaN(Number(targetValue))) {
      setError("Valid target value is required");
      return;
    }

    const newWeightage = editingGoal
      ? totalWeightage - editingGoal.weightage + weightage
      : totalWeightage + weightage;

    if (newWeightage > 100 && !editingGoal) {
      setError(
        `Adding this goal would exceed 100% weightage (currently at ${totalWeightage}%)`
      );
      return;
    }

    if (!editingGoal && goalsCount >= maxGoals) {
      setError(`Maximum ${maxGoals} goals allowed per sheet`);
      return;
    }

    const payload: CreateGoalPayload | UpdateGoalPayload = {
      thrust_area_id: thrustAreaId,
      title: title.trim(),
      description: description.trim() || null,
      uom_type: uomType,
      target_value: Number(targetValue),
      target_date: targetDate || null,
      weightage,
    };

    if (editingGoal) {
      updateGoalMutation.mutate({
        goalId: editingGoal.id,
        payload: payload as UpdateGoalPayload,
      });
    } else {
      createGoalMutation.mutate(payload as CreateGoalPayload);
    }
  };

  const handleDeleteGoal = (goalId: number) => {
    if (window.confirm("Delete this goal?")) {
      deleteGoalMutation.mutate(goalId);
    }
  };

  const isSaving = createGoalMutation.isPending || updateGoalMutation.isPending;

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
          <p className="text-slate-400">Failed to load goal sheet</p>
          <Button
            variant="outline"
            onClick={() => navigate("/employee")}
            className="border-slate-700 text-slate-300 hover:bg-slate-800"
          >
            <ArrowLeft className="mr-2 h-3.5 w-3.5" />
            Back to Dashboard
          </Button>
        </div>
      </DashboardLayout>
    );
  }

  return (
    <DashboardLayout>
      <div className="max-w-4xl mx-auto space-y-6">
        {/* Header */}
        <div className="flex items-start justify-between gap-4 flex-wrap">
          <div>
            <button
              onClick={() => navigate("/employee")}
              className="text-sm text-slate-500 hover:text-slate-300 flex items-center gap-1 mb-2"
            >
              <ArrowLeft className="h-3.5 w-3.5" />
              Dashboard
            </button>
            <h1 className="text-2xl font-bold text-slate-100">
              {sheet.cycle_name || "Goal Sheet"}
            </h1>
            <p className="text-sm text-slate-400 mt-1">
              {goalsCount} goal{goalsCount !== 1 ? "s" : ""} &middot;{" "}
              {totalWeightage}% weightage
            </p>
          </div>
          <div className="flex items-center gap-2">
            {!isDraft && (
              <Badge
                variant="secondary"
                className="bg-slate-800 text-slate-400 border-slate-700"
              >
                <Lock className="mr-1 h-3 w-3" />
                Read-only
              </Badge>
            )}
            <StatusBadge status={sheet.status} className="px-3 py-1 text-sm" />
          </div>
        </div>

        {/* Returned reason */}
        {isReturned && sheet.returned_reason && (
          <Alert className="bg-amber-950/30 border-amber-800 text-amber-300">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{sheet.returned_reason}</AlertDescription>
          </Alert>
        )}

        {/* Error alert */}
        {error && (
          <Alert
            variant="destructive"
            className="bg-red-950/50 border-red-800 text-red-400"
          >
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}
        {submitMutation.isError && (
          <Alert
            variant="destructive"
            className="bg-red-950/50 border-red-800 text-red-400"
          >
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              Failed to submit:{" "}
              {submitMutation.error?.message || "Please try again."}
            </AlertDescription>
          </Alert>
        )}

        {/* Weightage Bar */}
        <WeightageBar total={totalWeightage} />

        {/* Actions */}
        {isDraft && (
          <div className="flex items-center gap-2 flex-wrap">
            <Button
              onClick={openAddDialog}
              disabled={goalsCount >= maxGoals}
              className="bg-indigo-600 hover:bg-indigo-500 text-white"
            >
              <Plus className="mr-2 h-3.5 w-3.5" />
              Add Goal
            </Button>
            {totalWeightage === 100 && goalsCount > 0 && (
              <Button
                className="bg-emerald-600 hover:bg-emerald-500"
                onClick={() => submitMutation.mutate()}
                disabled={submitMutation.isPending}
              >
                {submitMutation.isPending ? (
                  <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
                ) : (
                  <Send className="mr-2 h-3.5 w-3.5" />
                )}
                Submit for Approval
              </Button>
            )}
          </div>
        )}

        {goalsCount === 0 && (
          <div className="text-center py-12 border border-dashed border-slate-700 rounded-xl">
            <p className="text-slate-500">
              No goals added yet. Click "Add Goal" to start.
            </p>
          </div>
        )}

        {/* Goals Grid */}
        {goalsCount > 0 && (
          <div className="grid gap-3 sm:grid-cols-2">
            {goals.map((goal) => {
              const isShared = goal.is_shared && goal.shared_from_goal_id;
              const uomBadgeColor =
                UOM_COLORS[goal.uom_type] || "bg-slate-800 text-slate-400";

              return (
                <Card
                  key={goal.id}
                  className="bg-slate-900/80 backdrop-blur-sm border border-slate-800"
                >
                  <CardContent className="py-4 space-y-3">
                    <div className="flex items-start justify-between gap-2">
                      <div className="flex items-center gap-2 min-w-0">
                        <p className="text-base font-semibold text-slate-100 truncate">
                          {goal.title}
                        </p>
                        <Badge
                          className={cn("shrink-0 text-xs border", uomBadgeColor)}
                          variant="outline"
                        >
                          {UOM_LABELS[goal.uom_type]}
                        </Badge>
                        {isShared && (
                          <Badge
                            variant="secondary"
                            className="shrink-0 text-xs bg-purple-500/20 text-purple-400 border-purple-500/30"
                          >
                            <Share2 className="mr-0.5 h-3 w-3" />
                            Shared
                          </Badge>
                        )}
                      </div>
                      {isDraft && (
                        <div className="flex items-center gap-1 shrink-0">
                          <Button
                            variant="ghost"
                            size="icon"
                            className="h-7 w-7 text-slate-400 hover:text-indigo-400"
                            onClick={() => openEditDialog(goal)}
                          >
                            <Edit className="h-3.5 w-3.5" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="h-7 w-7 text-slate-400 hover:text-red-400"
                            onClick={() => handleDeleteGoal(goal.id)}
                          >
                            <Trash2 className="h-3.5 w-3.5" />
                          </Button>
                        </div>
                      )}
                    </div>

                    {goal.description && (
                      <p className="text-sm text-slate-400">
                        {goal.description}
                      </p>
                    )}

                    <div className="grid grid-cols-2 gap-3 text-sm">
                      <div>
                        <span className="text-slate-500">Target</span>
                        <p className="font-semibold text-slate-200">
                          {goal.target_value}
                        </p>
                      </div>
                      <div>
                        <span className="text-slate-500">Weightage</span>
                        <p className="font-semibold text-indigo-400">
                          {goal.weightage}%
                        </p>
                      </div>
                    </div>

                    {goal.thrust_area_name && (
                      <div className="text-sm">
                        <span className="text-slate-500">Thrust Area</span>
                        <p className="font-medium text-slate-300">
                          {goal.thrust_area_name}
                        </p>
                      </div>
                    )}
                  </CardContent>
                </Card>
              );
            })}
          </div>
        )}

        {/* Goal Form Dialog */}
        <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
          <DialogContent className="bg-slate-900 border border-slate-800 text-slate-100 max-w-lg max-h-[85vh] overflow-y-auto">
            <DialogHeader>
              <DialogTitle className="text-slate-100">
                {editingGoal ? "Edit Goal" : "Add New Goal"}
              </DialogTitle>
              <DialogDescription className="text-slate-400">
                {editingGoal
                  ? "Update the goal details below."
                  : "Define a measurable goal with weightage."}
              </DialogDescription>
            </DialogHeader>

            <div className="space-y-4">
              {error && (
                <Alert
                  variant="destructive"
                  className="bg-red-950/50 border-red-800 text-red-400"
                >
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{error}</AlertDescription>
                </Alert>
              )}

              <div className="space-y-2">
                <Label className="text-slate-300">Title</Label>
                <Input
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  placeholder="e.g., Reduce defect rate"
                  disabled={
                    !!(
                      editingGoal?.is_shared &&
                      editingGoal?.shared_from_goal_id
                    )
                  }
                  className="bg-slate-800/50 border-slate-700 text-slate-100 placeholder:text-slate-600 disabled:opacity-50"
                />
              </div>

              <div className="space-y-2">
                <Label className="text-slate-300">Description (optional)</Label>
                <Textarea
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  placeholder="Brief description of the goal..."
                  className="bg-slate-800/50 border-slate-700 text-slate-100 placeholder:text-slate-600"
                  rows={2}
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label className="text-slate-300">UOM Type</Label>
                  <Select
                    value={uomType}
                    onValueChange={(v) => setUomType(v as UomType)}
                    disabled={
                      !!(
                        editingGoal?.is_shared &&
                        editingGoal?.shared_from_goal_id
                      )
                    }
                  >
                    <SelectTrigger className="bg-slate-800/50 border-slate-700 text-slate-100 disabled:opacity-50">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent className="bg-slate-900 border-slate-700 text-slate-100">
                      {UOM_OPTIONS.map((opt) => (
                        <SelectItem key={opt.value} value={opt.value}>
                          {opt.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-2">
                  <Label className="text-slate-300">Target Value</Label>
                  <Input
                    type="number"
                    value={targetValue}
                    onChange={(e) => setTargetValue(e.target.value)}
                    placeholder="e.g., 95"
                    disabled={
                      !!(
                        editingGoal?.is_shared &&
                        editingGoal?.shared_from_goal_id
                      )
                    }
                    className="bg-slate-800/50 border-slate-700 text-slate-100 placeholder:text-slate-600 disabled:opacity-50"
                  />
                </div>
              </div>

              <div className="space-y-2">
                <Label className="text-slate-300">
                  Target Date (optional)
                </Label>
                <Input
                  type="date"
                  value={targetDate}
                  min={new Date().toISOString().split("T")[0]}
                  onChange={(e) => setTargetDate(e.target.value)}
                  className="bg-slate-800/50 border-slate-700 text-slate-100"
                />
              </div>

              <div className="space-y-2">
                <Label className="text-slate-300">Thrust Area</Label>
                <Select
                  value={thrustAreaId != null ? String(thrustAreaId) : "none"}
                  onValueChange={(v) =>
                    setThrustAreaId(v === "none" ? null : Number(v))
                  }
                >
                  <SelectTrigger className="bg-slate-800/50 border-slate-700 text-slate-100">
                    <SelectValue placeholder="Select a thrust area..." />
                  </SelectTrigger>
                  <SelectContent className="bg-slate-900 border-slate-700 text-slate-100">
                    <SelectItem value="none">None</SelectItem>
                    {thrustAreas.map((area: ThrustArea) => (
                      <SelectItem key={area.id} value={String(area.id)}>
                        {area.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <Label className="text-slate-300">Weightage</Label>
                  <span className="text-sm font-semibold text-indigo-400">
                    {weightage}%
                  </span>
                </div>
                <Slider
                  value={[weightage]}
                  onValueChange={([v]) => setWeightage(v)}
                  min={10}
                  max={100}
                  step={5}
                />
                <p className="text-xs text-slate-500">
                  Total allocated:{" "}
                  {totalWeightage -
                    (editingGoal?.weightage || 0)}
                  % + {weightage}% ={" "}
                  {totalWeightage -
                    (editingGoal?.weightage || 0) +
                    weightage}
                  %
                </p>
              </div>
            </div>

            <DialogFooter>
              <Button
                variant="outline"
                onClick={closeDialog}
                className="border-slate-700 text-slate-300"
              >
                Cancel
              </Button>
              <Button
                onClick={handleSaveGoal}
                disabled={isSaving}
                className="bg-indigo-600 hover:bg-indigo-500 text-white"
              >
                {isSaving ? (
                  <>
                    <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
                    Saving...
                  </>
                ) : editingGoal ? (
                  "Update Goal"
                ) : (
                  "Add Goal"
                )}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>
    </DashboardLayout>
  );
}
