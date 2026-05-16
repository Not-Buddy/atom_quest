import { useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { useAuth } from "@/contexts/AuthContext";
import { listTeamSheets } from "@/lib/api";
import type { GoalSheetSummary } from "@/lib/types";
import { DashboardLayout } from "@/components/DashboardLayout";
import { StatusBadge } from "@/components/StatusBadge";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Users, FileText, Clock, Loader2, AlertCircle, ArrowRight, Eye, MessageSquare } from "lucide-react";

export default function ManagerDashboard() {
  const { user, token } = useAuth();
  const navigate = useNavigate();

  const { data: sheets = [], isLoading, error } = useQuery<GoalSheetSummary[]>({
    queryKey: ["team-sheets"],
    queryFn: () => listTeamSheets(token!),
    enabled: !!token,
  });

  const uniqueUserIds = [...new Set(sheets.map((s) => s.user_id))];
  const totalTeamMembers = uniqueUserIds.length;
  const pendingCount = sheets.filter((s) => s.status === "submitted").length;

  const stats = [
    {
      label: "Total Members",
      value: totalTeamMembers,
      icon: Users,
      color: "bg-indigo-600/20 text-indigo-400",
    },
    {
      label: "Pending Approvals",
      value: pendingCount,
      icon: Clock,
      color: "bg-amber-500/20 text-amber-400",
    },
    {
      label: "Submitted",
      value: pendingCount,
      icon: FileText,
      color: "bg-emerald-500/20 text-emerald-400",
    },
  ];

  return (
    <DashboardLayout>
      <div className="max-w-6xl mx-auto space-y-6">
        <div>
          <h1 className="text-2xl font-bold text-slate-100">Manager Dashboard</h1>
          <p className="text-sm text-slate-400 mt-1">
            Welcome, {user?.full_name || "Manager"}
          </p>
        </div>

        {/* Stats Cards */}
        {isLoading ? (
          <div className="grid gap-4 sm:grid-cols-3">
            {[1, 2, 3].map((i) => (
              <Card key={i} className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
                <CardContent className="py-6">
                  <div className="animate-pulse space-y-2">
                    <div className="h-4 w-20 rounded bg-slate-800" />
                    <div className="h-8 w-12 rounded bg-slate-800" />
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        ) : error ? (
          <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
            <CardContent className="py-6 text-center">
              <AlertCircle className="h-5 w-5 text-red-400 mx-auto mb-2" />
              <p className="text-red-400 text-sm">Failed to load team data</p>
            </CardContent>
          </Card>
        ) : (
          <div className="grid gap-4 sm:grid-cols-3">
            {stats.map((s) => (
              <Card
                key={s.label}
                className="bg-slate-900/80 backdrop-blur-sm border border-slate-800"
              >
                <CardContent className="py-4">
                  <div className="flex items-center gap-3">
                    <div className={`flex h-10 w-10 items-center justify-center rounded-lg ${s.color}`}>
                      <s.icon className="h-5 w-5" />
                    </div>
                    <div>
                      <p className="text-2xl font-bold text-slate-100">{s.value}</p>
                      <p className="text-sm text-slate-400">{s.label}</p>
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        )}

        {/* Team Goal Sheets */}
        <div>
          <h2 className="text-lg font-semibold text-slate-200 mb-4">Team Goal Sheets</h2>

          {isLoading ? (
            <div className="grid gap-3 sm:grid-cols-2">
              {[1, 2, 3].map((i) => (
                <Card key={i} className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
                  <CardContent className="py-6">
                    <div className="animate-pulse space-y-2">
                      <div className="h-4 w-2/3 rounded bg-slate-800" />
                      <div className="h-3 w-1/3 rounded bg-slate-800" />
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          ) : error ? null : sheets.length === 0 ? (
            <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
              <CardContent className="py-8 text-center">
                <Users className="h-8 w-8 text-slate-600 mx-auto mb-3" />
                <p className="text-slate-400">No team members with goal sheets</p>
              </CardContent>
            </Card>
          ) : (
            <div className="grid gap-3 sm:grid-cols-2">
              {sheets.map((sheet: GoalSheetSummary) => (
                <Card
                  key={sheet.id}
                  className="bg-slate-900/80 backdrop-blur-sm border border-slate-800 hover:border-slate-700 transition-colors"
                >
                  <CardContent className="py-4">
                    <div className="flex items-start justify-between gap-3">
                      <div className="min-w-0 flex-1">
                        <p className="font-medium text-slate-200 truncate">
                          {sheet.user_name || "Team Member"}
                        </p>
                        <p className="text-xs text-slate-500 mt-0.5">
                          {sheet.cycle_name || "Unknown Cycle"}
                        </p>
                        <div className="flex items-center gap-3 mt-2 text-xs text-slate-500">
                          <span>{sheet.goal_count} goals</span>
                          <span>Weightage: {sheet.total_weightage}%</span>
                        </div>
                      </div>
                      <div className="flex items-center gap-2 shrink-0">
                        <StatusBadge status={sheet.status} />
                      </div>
                    </div>
                    <div className="flex items-center gap-2 mt-3 pt-3 border-t border-slate-800/50">
                      <Button
                        size="sm"
                        variant="outline"
                        className="border-slate-700 text-slate-300 hover:bg-slate-800/50 text-xs h-8"
                        onClick={(e) => {
                          e.stopPropagation();
                          navigate(`/manager/review/${sheet.id}`);
                        }}
                      >
                        <Eye className="mr-1.5 h-3 w-3" />
                        Review
                      </Button>
                      <Button
                        size="sm"
                        variant="outline"
                        className="border-slate-700 text-slate-300 hover:bg-slate-800/50 text-xs h-8"
                        onClick={(e) => {
                          e.stopPropagation();
                          navigate(`/manager/checkin/${sheet.id}`);
                        }}
                      >
                        <MessageSquare className="mr-1.5 h-3 w-3" />
                        Check-in
                      </Button>
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
