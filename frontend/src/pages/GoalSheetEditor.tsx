import { useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useAuth } from "@/contexts/AuthContext";
import { fetchSheet, createGoal, editGoal, removeGoal, submitSheetForApproval } from "@/lib/api";
import { Goal, THRUST_AREAS, UomType, UOM_LABELS } from "@/lib/types";
import { DashboardLayout } from "@/components/DashboardLayout";
import { StatusBadge } from "@/components/StatusBadge";
import { GoalCard } from "@/components/GoalCard";
import { WeightageBar } from "@/components/WeightageBar";
import { Button } from "@/components/ui/button";
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
import { Plus, Loader2, AlertCircle, ArrowLeft, Send } from "lucide-react";

const UOM_OPTIONS: { value: UomType; label: string }[] = [
  { value: "min_numeric", label: "Minimize" },
  { value: "max_numeric", label: "Maximize" },
  { value: "timeline", label: "Timeline" },
  { value: "zero", label: "Zero" },
  { value: "percent", label: "Percent" },
];

export default function GoalSheetEditor() {
  const { sheetId } = useParams<{ sheetId: string }>();
  const { token } = useAuth();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingGoal, setEditingGoal] = useState<Goal | null>(null);
  const [error, setError] = useState("");

  // Goal form state
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [uomType, setUomType] = useState<UomType>("max_numeric");
  const [targetValue, setTargetValue] = useState("");
  const [targetDate, setTargetDate] = useState("");
  const [weightage, setWeightage] = useState(20);
  const [thrustArea, setThrustArea] = useState(THRUST_AREAS[0]);

  const { data, isLoading, isError } = useQuery({
    queryKey: ["sheet", sheetId],
    queryFn: () => fetchSheet(token!, sheetId!),
    enabled: !!token && !!sheetId,
  });

  const submitMutation = useMutation({
    mutationFn: () => submitSheetForApproval(token!, sheetId!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["sheet", sheetId] });
      queryClient.invalidateQueries({ queryKey: ["my-sheets"] });
    },
    onError: (err: Error) => setError(err.message),
  });

  const createGoalMutation = useMutation({
    mutationFn: (goal: Omit<Goal, "id" | "sheet_id" | "created_at" | "updated_at" | "order_index">) =>
      createGoal(token!, sheetId!, goal),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["sheet", sheetId] });
      closeDialog();
    },
    onError: (err: Error) => setError(err.message),
  });

  const updateGoalMutation = useMutation({
    mutationFn: ({ goalId, data }: { goalId: string; data: Partial<Goal> }) =>
      editGoal(token!, goalId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["sheet", sheetId] });
      closeDialog();
    },
    onError: (err: Error) => setError(err.message),
  });

  const deleteGoalMutation = useMutation({
    mutationFn: (goalId: string) => removeGoal(token!, goalId),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["sheet", sheetId] }),
  });

  const sheet = data?.sheet;
  const goals = sheet?.goals || [];
  const isDraft = sheet?.status === "draft";
  const totalWeightage = goals.reduce((sum, g) => sum + g.weightage, 0);
  const goalsCount = goals.length;
  const maxGoals = 8;

  const openAddDialog = () => {
    setEditingGoal(null);
    setTitle("");
    setDescription("");
    setUomType("max_numeric");
    setTargetValue("");
    setTargetDate("");
    setWeightage(20);
    setThrustArea(THRUST_AREAS[0]);
    setError("");
    setDialogOpen(true);
  };

  const openEditDialog = (goal: Goal) => {
    setEditingGoal(goal);
    setTitle(goal.title);
    setDescription(goal.description || "");
    setUomType(goal.uom_type);
    setTargetValue(String(goal.target_value));
    setTargetDate(goal.target_date || "");
    setWeightage(goal.weightage);
    setThrustArea(goal.thrust_area);
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
      setError(`Adding this goal would exceed 100% weightage (currently at ${totalWeightage}%)`);
      return;
    }

    if (!editingGoal && goalsCount >= maxGoals) {
      setError(`Maximum ${maxGoals} goals allowed per sheet`);
      return;
    }

    const goalData = {
      title: title.trim(),
      description: description.trim() || undefined,
      uom_type: uomType,
      target_value: Number(targetValue),
      target_date: targetDate || undefined,
      weightage,
      thrust_area: thrustArea,
    };

    if (editingGoal) {
      updateGoalMutation.mutate({ goalId: editingGoal.id, data: goalData as any });
    } else {
      createGoalMutation.mutate(goalData as any);
    }
  };

  const handleDeleteGoal = (goalId: string) => {
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
          <Button variant="outline" onClick={() => navigate("/employee")}>
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
          <StatusBadge status={sheet.status} className="px-3 py-1 text-sm" />
        </div>

        {/* Error alert */}
        {error && (
          <Alert variant="destructive" className="bg-red-950/50 border-red-800 text-red-400">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}
        {submitMutation.isError && (
          <Alert variant="destructive" className="bg-red-950/50 border-red-800 text-red-400">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              Failed to submit: {submitMutation.error?.message || "Please try again."}
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
                variant="default"
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

        {isDraft && goalsCount === 0 && (
          <div className="text-center py-12 border border-dashed border-slate-700 rounded-xl">
            <p className="text-slate-500">No goals added yet. Click "Add Goal" to start.</p>
          </div>
        )}

        {/* Goals Grid */}
        {goalsCount > 0 && (
          <div className="grid gap-3 sm:grid-cols-2">
            {goals.map((goal) => (
              <GoalCard
                key={goal.id}
                goal={goal}
                readOnly={!isDraft}
                onEdit={isDraft ? openEditDialog : undefined}
                onDelete={isDraft ? handleDeleteGoal : undefined}
              />
            ))}
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
                <Alert variant="destructive" className="bg-red-950/50 border-red-800 text-red-400">
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
                  className="bg-slate-800/50 border-slate-700 text-slate-100 placeholder:text-slate-600"
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
                  <Select value={uomType} onValueChange={(v) => setUomType(v as UomType)}>
                    <SelectTrigger className="bg-slate-800/50 border-slate-700 text-slate-100">
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
                    className="bg-slate-800/50 border-slate-700 text-slate-100 placeholder:text-slate-600"
                  />
                </div>
              </div>

              <div className="space-y-2">
                <Label className="text-slate-300">Target Date (optional)</Label>
                <Input
                  type="date"
                  value={targetDate}
                  onChange={(e) => setTargetDate(e.target.value)}
                  className="bg-slate-800/50 border-slate-700 text-slate-100"
                />
              </div>

              <div className="space-y-2">
                <Label className="text-slate-300">Thrust Area</Label>
                <Select value={thrustArea} onValueChange={setThrustArea}>
                  <SelectTrigger className="bg-slate-800/50 border-slate-700 text-slate-100">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent className="bg-slate-900 border-slate-700 text-slate-100">
                    {THRUST_AREAS.map((area) => (
                      <SelectItem key={area} value={area}>
                        {area}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <Label className="text-slate-300">Weightage</Label>
                  <span className="text-sm font-semibold text-indigo-400">{weightage}%</span>
                </div>
                <Slider
                  value={[weightage]}
                  onValueChange={([v]) => setWeightage(v)}
                  min={10}
                  max={100}
                  step={5}
                  className="[&>[data-slot=slider-track]]:bg-slate-700 [&>[data-slot=slider-range]]:bg-indigo-500"
                />
                <p className="text-xs text-slate-500">
                  Total allocated: {totalWeightage - (editingGoal?.weightage || 0)}% + {weightage}% ={" "}
                  {totalWeightage - (editingGoal?.weightage || 0) + weightage}%
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
