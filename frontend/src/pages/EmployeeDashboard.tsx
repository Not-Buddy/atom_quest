import { useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useAuth } from "@/contexts/AuthContext";
import { fetchMySheets, fetchActiveCycle, createSheet } from "@/lib/api";
import { GoalSheet } from "@/lib/types";
import { DashboardLayout } from "@/components/DashboardLayout";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { StatusBadge } from "@/components/StatusBadge";
import { Plus, Target, FileText, AlertCircle, Loader2, Calendar } from "lucide-react";

export default function EmployeeDashboard() {
  const { user, token } = useAuth();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const { data: sheetsData, isLoading: sheetsLoading, error: sheetsError } = useQuery({
    queryKey: ["my-sheets"],
    queryFn: () => fetchMySheets(token!),
    enabled: !!token,
  });

  const { data: cycleData, isLoading: cycleLoading } = useQuery({
    queryKey: ["active-cycle"],
    queryFn: () => fetchActiveCycle(token!),
    enabled: !!token,
  });

  const createSheetMutation = useMutation({
    mutationFn: async () => {
      if (!cycleData?.cycle?.id) throw new Error("No active cycle");
      return createSheet(token!, cycleData.cycle.id);
    },
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ["my-sheets"] });
      navigate(`/employee/goals/${data.sheet.id}`);
    },
  });

  const sheets: GoalSheet[] = sheetsData?.sheets || [];
  const activeCycle = cycleData?.cycle;

  return (
    <DashboardLayout>
      <div className="max-w-6xl mx-auto space-y-6">
        {/* Welcome */}
        <div>
          <h1 className="text-2xl font-bold text-slate-100">
            Welcome, {user?.name || "Employee"}
          </h1>
          <p className="text-slate-400 text-sm mt-1">
            Manage your goal sheets and track achievements
          </p>
        </div>

        {/* Active Cycle Card */}
        {cycleLoading ? (
          <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
            <CardContent className="py-8 flex items-center justify-center">
              <Loader2 className="h-5 w-5 animate-spin text-slate-500" />
            </CardContent>
          </Card>
        ) : activeCycle ? (
          <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800 border-l-indigo-500/50">
            <CardContent className="py-5">
              <div className="flex items-start justify-between gap-4 flex-wrap">
                <div className="flex items-center gap-3">
                  <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-indigo-600/20">
                    <Calendar className="h-5 w-5 text-indigo-400" />
                  </div>
                  <div>
                    <p className="text-sm text-slate-400">Active Cycle</p>
                    <p className="text-lg font-semibold text-slate-100">{activeCycle.name}</p>
                    {activeCycle.end_date && (
                      <p className="text-xs text-slate-500">
                        Ends: {new Date(activeCycle.end_date).toLocaleDateString()}
                      </p>
                    )}
                  </div>
                </div>
                <div className="flex gap-2">
                  <Button
                    size="sm"
                    variant="outline"
                    className="border-slate-700 text-slate-300 hover:bg-slate-800"
                    onClick={() => navigate("/employee/goals")}
                  >
                    <FileText className="mr-2 h-3.5 w-3.5" />
                    View My Goals
                  </Button>
                  <Button
                    size="sm"
                    className="bg-indigo-600 hover:bg-indigo-500 text-white"
                    onClick={() => createSheetMutation.mutate()}
                    disabled={createSheetMutation.isPending}
                  >
                    {createSheetMutation.isPending ? (
                      <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
                    ) : (
                      <Plus className="mr-2 h-3.5 w-3.5" />
                    )}
                    Create Goal Sheet
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>
        ) : (
          <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
            <CardContent className="py-8 text-center">
              <div className="flex justify-center mb-3">
                <AlertCircle className="h-8 w-8 text-slate-600" />
              </div>
              <p className="text-slate-400">No active goal cycle at this time</p>
              <p className="text-slate-600 text-sm mt-1">
                Check back when a new cycle begins
              </p>
            </CardContent>
          </Card>
        )}

        {/* Goal Sheets List */}
        <div>
          <h2 className="text-lg font-semibold text-slate-200 mb-4">My Goal Sheets</h2>

          {sheetsLoading ? (
            <div className="grid gap-3 sm:grid-cols-2">
              {[1, 2, 3].map((i) => (
                <Card key={i} className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
                  <CardContent className="py-8">
                    <div className="animate-pulse space-y-3">
                      <div className="h-4 w-3/4 rounded bg-slate-800" />
                      <div className="h-3 w-1/2 rounded bg-slate-800" />
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          ) : sheetsError ? (
            <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
              <CardContent className="py-6 text-center">
                <p className="text-red-400 text-sm">
                  Failed to load goal sheets. Please try again.
                </p>
              </CardContent>
            </Card>
          ) : sheets.length === 0 ? (
            <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
              <CardContent className="py-8 text-center">
                <Target className="h-8 w-8 text-slate-600 mx-auto mb-3" />
                <p className="text-slate-400">No goal sheets yet</p>
                <p className="text-slate-600 text-sm mt-1">
                  Create your first goal sheet to get started
                </p>
              </CardContent>
            </Card>
          ) : (
            <div className="grid gap-3 sm:grid-cols-2">
              {sheets.map((sheet) => (
                <Card
                  key={sheet.id}
                  className="bg-slate-900/80 backdrop-blur-sm border border-slate-800 hover:border-slate-700 cursor-pointer transition-colors"
                  onClick={() => navigate(`/employee/goals/${sheet.id}`)}
                >
                  <CardContent className="py-4">
                    <div className="flex items-start justify-between gap-3">
                      <div className="min-w-0">
                        <p className="font-medium text-slate-200 truncate">
                          {sheet.cycle_name || "Goal Sheet"}
                        </p>
                        <div className="flex items-center gap-2 mt-1 text-xs text-slate-500">
                          <span>{sheet.goals?.length || 0} goals</span>
                        </div>
                      </div>
                      <StatusBadge status={sheet.status} />
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </div>
      </div>
    </DashboardLayout>
  );
}
