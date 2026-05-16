import { Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { useAuth } from "@/contexts/AuthContext";
import { fetchAdminStats } from "@/lib/api";
import { DashboardLayout } from "@/components/DashboardLayout";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Users,
  Activity,
  Building,
  Shield,
  FileText,
  BarChart3,
  RefreshCw,
  Settings,
  Loader2,
} from "lucide-react";

const quickLinks = [
  {
    title: "Manage Users",
    description: "Add, edit, or remove users and assign roles",
    href: "/admin/users",
    icon: Users,
    color: "bg-indigo-600/20 text-indigo-400",
  },
  {
    title: "Manage Cycles",
    description: "Create and manage goal setting cycles",
    href: "/admin/cycles",
    icon: RefreshCw,
    color: "bg-emerald-600/20 text-emerald-400",
  },
  {
    title: "Departments",
    description: "Manage organizational departments",
    href: "/admin/departments",
    icon: Building,
    color: "bg-purple-600/20 text-purple-400",
  },
  {
    title: "Audit Log",
    description: "View all system changes and activity",
    href: "/admin/audit",
    icon: Shield,
    color: "bg-amber-600/20 text-amber-400",
  },
  {
    title: "Reports",
    description: "View achievement reports and analytics",
    href: "/reports",
    icon: BarChart3,
    color: "bg-cyan-600/20 text-cyan-400",
  },
  {
    title: "Settings",
    description: "Configure system-wide parameters",
    href: "/admin/settings",
    icon: Settings,
    color: "bg-slate-600/20 text-slate-400",
  },
];

export default function AdminDashboard() {
  const { user } = useAuth();
  const { data, isLoading } = useQuery({
    queryKey: ["admin-stats"],
    queryFn: () => fetchAdminStats(localStorage.getItem("auth_token") || ""),
  });

  const stats = data?.stats;

  return (
    <DashboardLayout>
      <div className="max-w-6xl mx-auto space-y-6">
        <div>
          <h1 className="text-2xl font-bold text-slate-100">Admin Dashboard</h1>
          <p className="text-sm text-slate-400 mt-1">Welcome, {user?.name || "Admin"}</p>
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
        ) : (
          <div className="grid gap-4 sm:grid-cols-3">
            <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
              <CardContent className="py-4">
                <div className="flex items-center gap-3">
                  <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-indigo-600/20">
                    <Users className="h-5 w-5 text-indigo-400" />
                  </div>
                  <div>
                    <p className="text-2xl font-bold text-slate-100">{stats?.total_users || 0}</p>
                    <p className="text-sm text-slate-400">Total Users</p>
                  </div>
                </div>
              </CardContent>
            </Card>
            <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
              <CardContent className="py-4">
                <div className="flex items-center gap-3">
                  <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-emerald-600/20">
                    <Activity className="h-5 w-5 text-emerald-400" />
                  </div>
                  <div>
                    <p className="text-2xl font-bold text-slate-100">{stats?.active_cycles || 0}</p>
                    <p className="text-sm text-slate-400">Active Cycles</p>
                  </div>
                </div>
              </CardContent>
            </Card>
            <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
              <CardContent className="py-4">
                <div className="flex items-center gap-3">
                  <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-purple-600/20">
                    <Building className="h-5 w-5 text-purple-400" />
                  </div>
                  <div>
                    <p className="text-2xl font-bold text-slate-100">{stats?.departments || 0}</p>
                    <p className="text-sm text-slate-400">Departments</p>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        )}

        {/* Quick Links */}
        <div>
          <h2 className="text-lg font-semibold text-slate-200 mb-4">Quick Links</h2>
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {quickLinks.map((link) => (
              <Link key={link.href} to={link.href}>
                <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800 hover:border-slate-700 transition-colors h-full">
                  <CardContent className="py-4">
                    <div className="flex items-start gap-3">
                      <div className={`flex h-10 w-10 items-center justify-center rounded-lg ${link.color}`}>
                        <link.icon className="h-5 w-5" />
                      </div>
                      <div className="min-w-0">
                        <p className="font-medium text-slate-200">{link.title}</p>
                        <p className="text-xs text-slate-500 mt-0.5">{link.description}</p>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </Link>
            ))}
          </div>
        </div>
      </div>
    </DashboardLayout>
  );
}
