import { useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { useAuth } from "@/contexts/AuthContext";
import { fetchManagerStats, fetchTeamMembers } from "@/lib/api";
import { GoalSheet } from "@/lib/types";
import { DashboardLayout } from "@/components/DashboardLayout";
import { StatusBadge } from "@/components/StatusBadge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Users, FileText, Clock, Loader2, ArrowRight, AlertCircle } from "lucide-react";

export default function ManagerDashboard() {
  const { user, token } = useAuth();
  const navigate = useNavigate();

  const { data: statsData, isLoading: statsLoading } = useQuery({
    queryKey: ["manager-stats"],
    queryFn: () => fetchManagerStats(token!),
    enabled: !!token,
  });

  const { data: teamData, isLoading: teamLoading } = useQuery({
    queryKey: ["team-members"],
    queryFn: () => fetchTeamMembers(token!),
    enabled: !!token,
  });

  const stats = statsData?.stats;
  const members: GoalSheet[] = teamData?.members || [];

  return (
    <DashboardLayout>
      <div className="max-w-6xl mx-auto space-y-6">
        <div>
          <h1 className="text-2xl font-bold text-slate-100">
            Manager Dashboard
          </h1>
          <p className="text-sm text-slate-400 mt-1">
            Welcome, {user?.name || "Manager"}
          </p>
        </div>

        {/* Stats Cards */}
        {statsLoading ? (
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
        ) : stats ? (
          <div className="grid gap-4 sm:grid-cols-3">
            <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
              <CardContent className="py-4">
                <div className="flex items-center gap-3">
                  <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-indigo-600/20">
                    <Users className="h-5 w-5 text-indigo-400" />
                  </div>
                  <div>
                    <p className="text-2xl font-bold text-slate-100">{stats.total_team_members}</p>
                    <p className="text-sm text-slate-400">Team Members</p>
                  </div>
                </div>
              </CardContent>
            </Card>
            <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
              <CardContent className="py-4">
                <div className="flex items-center gap-3">
                  <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-amber-500/20">
                    <Clock className="h-5 w-5 text-amber-400" />
                  </div>
                  <div>
                    <p className="text-2xl font-bold text-slate-100">{stats.pending_approvals}</p>
                    <p className="text-sm text-slate-400">Pending Approvals</p>
                  </div>
                </div>
              </CardContent>
            </Card>
            <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
              <CardContent className="py-4">
                <div className="flex items-center gap-3">
                  <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-emerald-500/20">
                    <FileText className="h-5 w-5 text-emerald-400" />
                  </div>
                  <div>
                    <p className="text-2xl font-bold text-slate-100">{stats.submitted_sheets}</p>
                    <p className="text-sm text-slate-400">Submitted</p>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        ) : null}

        {/* Team Members */}
        <div>
          <h2 className="text-lg font-semibold text-slate-200 mb-4">Team Goal Sheets</h2>

          {teamLoading ? (
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
          ) : members.length === 0 ? (
            <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
              <CardContent className="py-8 text-center">
                <Users className="h-8 w-8 text-slate-600 mx-auto mb-3" />
                <p className="text-slate-400">No team members with goal sheets</p>
              </CardContent>
            </Card>
          ) : (
            <div className="grid gap-3 sm:grid-cols-2">
              {members.map((m) => (
                <Card
                  key={m.id}
                  className="bg-slate-900/80 backdrop-blur-sm border border-slate-800 hover:border-slate-700 cursor-pointer transition-colors"
                  onClick={() => navigate(`/manager/review/${m.id}`)}
                >
                  <CardContent className="py-4">
                    <div className="flex items-start justify-between gap-3">
                      <div className="min-w-0">
                        <p className="font-medium text-slate-200 truncate">
                          {m.user_name || "Team Member"}
                        </p>
                        <p className="text-xs text-slate-500 mt-0.5">{m.user_department}</p>
                        <p className="text-xs text-slate-500 mt-0.5">
                          {m.cycle_name} &middot; {m.goals?.length || 0} goals
                        </p>
                      </div>
                      <div className="flex items-center gap-2 shrink-0">
                        <StatusBadge status={m.status} />
                        <ArrowRight className="h-4 w-4 text-slate-600" />
                      </div>
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
