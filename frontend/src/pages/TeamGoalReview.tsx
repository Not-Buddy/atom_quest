import { useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useAuth } from "@/contexts/AuthContext";
import { getGoalSheet, approveSheet, returnSheet, managerEditGoal } from "@/lib/api";
import type { GoalSheetResponse, GoalResponse, UomType, SheetStatus } from "@/lib/types";
import { UOM_LABELS, UOM_COLORS } from "@/lib/types";
import { DashboardLayout } from "@/components/DashboardLayout";
import { StatusBadge } from "@/components/StatusBadge";
import { WeightageBar } from "@/components/WeightageBar";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Alert, AlertDescription } from "@/components/ui/alert";
import {
  ArrowLeft,
  CheckCircle,
  Loader2,
  AlertCircle,
  Edit,
  Save,
  Undo2,
} from "lucide-react";

export default function TeamGoalReview() {
  const { sheetId } = useParams<{ sheetId: string }>();
  const { token } = useAuth();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const numericId = Number(sheetId);

  const [returnDialogOpen, setReturnDialogOpen] = useState(false);
  const [returnReason, setReturnReason] = useState("");
  const [error, setError] = useState("");
  const [editingGoalId, setEditingGoalId] = useState<number | null>(null);
  const [editTarget, setEditTarget] = useState("");
  const [editWeightage, setEditWeightage] = useState("");

  const { data: sheet, isLoading } = useQuery<GoalSheetResponse>({
    queryKey: ["sheet", numericId],
    queryFn: () => getGoalSheet(token!, numericId),
    enabled: !!token && !Number.isNaN(numericId),
  });

  const approveMutation = useMutation({
    mutationFn: () => approveSheet(token!, numericId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["sheet", numericId] });
      queryClient.invalidateQueries({ queryKey: ["team-sheets"] });
    },
    onError: (err: Error) => setError(err.message),
  });

  const returnMutation = useMutation({
    mutationFn: () => returnSheet(token!, numericId, { reason: returnReason }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["sheet", numericId] });
      queryClient.invalidateQueries({ queryKey: ["team-sheets"] });
      setReturnDialogOpen(false);
      setReturnReason("");
    },
    onError: (err: Error) => setError(err.message),
  });

  const updateGoalMutation = useMutation({
    mutationFn: ({ goalId, payload }: { goalId: number; payload: { target_value: number; weightage: number } }) =>
      managerEditGoal(token!, numericId, goalId, payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["sheet", numericId] });
      setEditingGoalId(null);
    },
    onError: (err: Error) => setError(err.message),
  });

  const goals = sheet?.goals || [];
  const totalWeightage = sheet?.total_weightage ?? goals.reduce((sum, g) => sum + g.weightage, 0);
  const canApprove = sheet?.status === "submitted" || sheet?.status === "returned";
  const canReturn = sheet?.status === "submitted";

  const startEditing = (goal: GoalResponse) => {
    setEditingGoalId(goal.id);
    setEditTarget(String(goal.target_value));
    setEditWeightage(String(goal.weightage));
  };

  const saveEditing = (goalId: number) => {
    updateGoalMutation.mutate({
      goalId,
      payload: {
        target_value: Number(editTarget),
        weightage: Number(editWeightage),
      },
    });
  };

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
      <div className="max-w-4xl mx-auto space-y-6">
        {/* Header */}
        <div>
          <button
            onClick={() => navigate("/manager")}
            className="text-sm text-slate-500 hover:text-slate-300 flex items-center gap-1 mb-2"
          >
            <ArrowLeft className="h-3.5 w-3.5" />
            Team Dashboard
          </button>
          <div className="flex items-start justify-between gap-4 flex-wrap">
            <div>
              <h1 className="text-2xl font-bold text-slate-100">
                {sheet.user_name || "Employee"} &mdash; {sheet.cycle_name || "Goal Sheet"}
              </h1>
              <p className="text-sm text-slate-400 mt-1">
                {goals.length} goals &middot; Status: {sheet.status}
              </p>
            </div>
            <StatusBadge status={sheet.status} className="px-3 py-1 text-sm" />
          </div>
        </div>

        {/* Return reason display */}
        {sheet.status === "returned" && sheet.returned_reason && (
          <Alert className="bg-amber-950/30 border-amber-800 text-amber-300">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              Return reason: {sheet.returned_reason}
            </AlertDescription>
          </Alert>
        )}

        {error && (
          <Alert variant="destructive" className="bg-red-950/50 border-red-800 text-red-400">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {/* Weightage bar */}
        <WeightageBar total={totalWeightage} />

        {/* Actions */}
        <div className="flex items-center gap-2 flex-wrap">
          {canApprove && (
            <Button
              className="bg-emerald-600 hover:bg-emerald-500 text-white"
              onClick={() => approveMutation.mutate()}
              disabled={approveMutation.isPending}
            >
              {approveMutation.isPending ? (
                <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
              ) : (
                <CheckCircle className="mr-2 h-3.5 w-3.5" />
              )}
              Approve
            </Button>
          )}
          {canReturn && (
            <Button
              variant="outline"
              className="border-amber-700 text-amber-400 hover:bg-amber-950/30"
              onClick={() => setReturnDialogOpen(true)}
            >
              <Undo2 className="mr-2 h-3.5 w-3.5" />
              Return with Reason
            </Button>
          )}
        </div>

        {/* Goals list */}
        <div className="grid gap-3 sm:grid-cols-2">
          {goals.map((goal: GoalResponse) => (
            <Card key={goal.id} className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
              <CardHeader className="pb-2">
                <div className="flex items-start justify-between gap-2">
                  <CardTitle className="text-base text-slate-100">{goal.title}</CardTitle>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-7 w-7 text-slate-400 hover:text-indigo-400"
                    onClick={() => startEditing(goal)}
                  >
                    <Edit className="h-3.5 w-3.5" />
                  </Button>
                </div>
              </CardHeader>
              <CardContent className="space-y-3">
                {editingGoalId === goal.id ? (
                  <div className="space-y-3">
                    <div>
                      <Label className="text-xs text-slate-400">Target Value</Label>
                      <Input
                        type="number"
                        value={editTarget}
                        onChange={(e) => setEditTarget(e.target.value)}
                        className="mt-1 bg-slate-800/50 border-slate-700 text-slate-100"
                      />
                    </div>
                    <div>
                      <Label className="text-xs text-slate-400">Weightage (%)</Label>
                      <Input
                        type="number"
                        value={editWeightage}
                        onChange={(e) => setEditWeightage(e.target.value)}
                        className="mt-1 bg-slate-800/50 border-slate-700 text-slate-100"
                        min={0}
                        max={100}
                      />
                    </div>
                    <div className="flex gap-2">
                      <Button
                        size="sm"
                        className="bg-indigo-600 hover:bg-indigo-500"
                        onClick={() => saveEditing(goal.id)}
                        disabled={updateGoalMutation.isPending}
                      >
                        <Save className="mr-2 h-3 w-3" />
                        Save
                      </Button>
                      <Button
                        size="sm"
                        variant="outline"
                        className="border-slate-700"
                        onClick={() => setEditingGoalId(null)}
                      >
                        Cancel
                      </Button>
                    </div>
                  </div>
                ) : (
                  <>
                    <div className="flex flex-wrap gap-1.5">
                      {goal.uom_type && (
                        <Badge
                          variant="outline"
                          className={`text-[10px] px-1.5 py-0 border ${UOM_COLORS[goal.uom_type]}`}
                        >
                          {UOM_LABELS[goal.uom_type] || goal.uom_type}
                        </Badge>
                      )}
                      {goal.thrust_area_name && (
                        <Badge
                          variant="outline"
                          className="text-[10px] px-1.5 py-0 border bg-purple-500/10 text-purple-400 border-purple-500/30"
                        >
                          {goal.thrust_area_name}
                        </Badge>
                      )}
                      {goal.is_shared && (
                        <Badge
                          variant="outline"
                          className="text-[10px] px-1.5 py-0 border bg-cyan-500/10 text-cyan-400 border-cyan-500/30"
                        >
                          Shared
                        </Badge>
                      )}
                    </div>
                    <div className="grid grid-cols-2 gap-2 text-sm">
                      <div>
                        <span className="text-slate-500">Target</span>
                        <p className="font-semibold text-slate-200">{goal.target_value}</p>
                      </div>
                      <div>
                        <span className="text-slate-500">Weightage</span>
                        <p className="font-semibold text-indigo-400">{goal.weightage}%</p>
                      </div>
                    </div>
                    {goal.description && (
                      <p className="text-xs text-slate-500">{goal.description}</p>
                    )}
                  </>
                )}
              </CardContent>
            </Card>
          ))}
        </div>

        {/* Return reason dialog */}
        <Dialog open={returnDialogOpen} onOpenChange={setReturnDialogOpen}>
          <DialogContent className="bg-slate-900 border border-slate-800 text-slate-100">
            <DialogHeader>
              <DialogTitle className="text-slate-100">Return Goal Sheet</DialogTitle>
              <DialogDescription className="text-slate-400">
                Provide a reason for returning this sheet to the employee.
              </DialogDescription>
            </DialogHeader>
            <div className="space-y-3">
              <Textarea
                value={returnReason}
                onChange={(e) => setReturnReason(e.target.value)}
                placeholder="e.g., Adjust weightage percentages or target values..."
                className="bg-slate-800/50 border-slate-700 text-slate-100 placeholder:text-slate-600"
                rows={3}
              />
            </div>
            <DialogFooter>
              <Button
                variant="outline"
                onClick={() => setReturnDialogOpen(false)}
                className="border-slate-700 text-slate-300"
              >
                Cancel
              </Button>
              <Button
                className="bg-amber-600 hover:bg-amber-500"
                onClick={() => {
                  if (!returnReason.trim()) {
                    setError("Please provide a reason for returning");
                    return;
                  }
                  returnMutation.mutate();
                }}
                disabled={returnMutation.isPending}
              >
                {returnMutation.isPending ? (
                  <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
                ) : null}
                Return Sheet
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>
    </DashboardLayout>
  );
}
