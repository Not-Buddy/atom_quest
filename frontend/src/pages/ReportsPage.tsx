import { useQuery } from "@tanstack/react-query";
import { useAuth } from "@/contexts/AuthContext";
import { fetchReports, downloadAchievementReport as downloadReport } from "@/lib/api";
import { CompletionDashboardItem, QoQTrendItem } from "@/lib/types";
import { DashboardLayout } from "@/components/DashboardLayout";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
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
  Download,
  Loader2,
  AlertCircle,
  TrendingUp,
  BarChart3,
  Award,
  Building,
} from "lucide-react";

export default function ReportsPage() {
  const { token } = useAuth();

  const { data, isLoading, isError } = useQuery({
    queryKey: ["reports"],
    queryFn: () => fetchReports(token!),
    enabled: !!token,
  });

  const achievementReport = data?.achievement_report || [];
  const completionDashboard: CompletionDashboardItem[] = data?.completion_dashboard || [];
  const qoqTrends: QoQTrendItem[] = data?.qoq_trends || [];

  const handleDownload = async () => {
    try {
      await downloadReport(token!);
    } catch (err) {
      console.error("Download failed:", err);
    }
  };

  return (
    <DashboardLayout>
      <div className="max-w-7xl mx-auto space-y-6">
        <div className="flex items-center justify-between flex-wrap gap-4">
          <div>
            <h1 className="text-2xl font-bold text-slate-100">Reports</h1>
            <p className="text-sm text-slate-400 mt-1">
              Achievement reports, completion metrics, and trends
            </p>
          </div>
        </div>

        {isLoading ? (
          <div className="flex justify-center py-20">
            <Loader2 className="h-8 w-8 animate-spin text-indigo-500" />
          </div>
        ) : isError ? (
          <Alert variant="destructive" className="bg-red-950/50 border-red-800 text-red-400">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>Failed to load reports</AlertDescription>
          </Alert>
        ) : (
          <>
            {/* Achievement Report Section */}
            <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
              <CardHeader className="pb-3">
                <div className="flex items-center justify-between gap-4 flex-wrap">
                  <div className="flex items-center gap-2">
                    <Award className="h-5 w-5 text-indigo-400" />
                    <CardTitle className="text-lg text-slate-100">Achievement Report</CardTitle>
                  </div>
                  <Button
                    variant="outline"
                    size="sm"
                    className="border-slate-700 text-slate-300 hover:bg-slate-800"
                    onClick={handleDownload}
                  >
                    <Download className="mr-2 h-3.5 w-3.5" />
                    Download Excel
                  </Button>
                </div>
              </CardHeader>
              <CardContent>
                {achievementReport.length === 0 ? (
                  <p className="text-sm text-slate-500 text-center py-6">
                    No achievement data available
                  </p>
                ) : (
                  <div className="rounded-lg border border-slate-800 overflow-auto">
                    <Table>
                      <TableHeader>
                        <TableRow className="border-slate-800 hover:bg-transparent">
                          <TableHead className="text-slate-400 text-xs">Employee</TableHead>
                          <TableHead className="text-slate-400 text-xs">Department</TableHead>
                          <TableHead className="text-slate-400 text-xs">Cycle</TableHead>
                          <TableHead className="text-slate-400 text-xs">Goal</TableHead>
                          <TableHead className="text-slate-400 text-xs">UOM</TableHead>
                          <TableHead className="text-slate-400 text-xs text-right">Target</TableHead>
                          <TableHead className="text-slate-400 text-xs text-right">Q1</TableHead>
                          <TableHead className="text-slate-400 text-xs text-right">Q2</TableHead>
                          <TableHead className="text-slate-400 text-xs text-right">Q3</TableHead>
                          <TableHead className="text-slate-400 text-xs text-right">Q4</TableHead>
                          <TableHead className="text-slate-400 text-xs text-right">Score</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {achievementReport.slice(0, 20).map((row, i) => (
                          <TableRow key={i} className="border-slate-800">
                            <TableCell className="text-slate-300 text-xs">{row.employee_name}</TableCell>
                            <TableCell className="text-slate-400 text-xs">{row.department}</TableCell>
                            <TableCell className="text-slate-400 text-xs">{row.cycle}</TableCell>
                            <TableCell className="text-slate-300 text-xs max-w-[150px] truncate">
                              {row.goal_title}
                            </TableCell>
                            <TableCell className="text-slate-400 text-xs">{row.uom}</TableCell>
                            <TableCell className="text-slate-300 text-xs text-right">{row.target}</TableCell>
                            <TableCell className="text-slate-400 text-xs text-right">{row.q1}</TableCell>
                            <TableCell className="text-slate-400 text-xs text-right">{row.q2}</TableCell>
                            <TableCell className="text-slate-400 text-xs text-right">{row.q3}</TableCell>
                            <TableCell className="text-slate-400 text-xs text-right">{row.q4}</TableCell>
                            <TableCell className="text-xs text-right font-semibold">
                              <span
                                className={
                                  row.total_score >= 80
                                    ? "text-emerald-400"
                                    : row.total_score >= 50
                                      ? "text-amber-400"
                                      : "text-red-400"
                                }
                              >
                                {Math.round(row.total_score)}%
                              </span>
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  </div>
                )}
              </CardContent>
            </Card>

            {/* Completion Dashboard */}
            <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
              <CardHeader className="pb-3">
                <div className="flex items-center gap-2">
                  <Building className="h-5 w-5 text-emerald-400" />
                  <CardTitle className="text-lg text-slate-100">Completion Dashboard</CardTitle>
                </div>
              </CardHeader>
              <CardContent>
                {completionDashboard.length === 0 ? (
                  <p className="text-sm text-slate-500 text-center py-6">No data available</p>
                ) : (
                  <div className="space-y-4">
                    {completionDashboard.map((item) => (
                      <div key={item.department} className="space-y-1.5">
                        <div className="flex items-center justify-between text-sm">
                          <span className="text-slate-300">{item.department}</span>
                          <span className="text-slate-400 text-xs">
                            {item.submitted}/{item.total_employees} submitted &middot;{" "}
                            {Math.round(item.completion_rate)}%
                          </span>
                        </div>
                        <Progress
                          value={item.completion_rate}
                          className="h-2 bg-slate-800 [&>div]:bg-emerald-500"
                        />
                      </div>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>

            {/* QoQ Trends */}
            <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
              <CardHeader className="pb-3">
                <div className="flex items-center gap-2">
                  <TrendingUp className="h-5 w-5 text-purple-400" />
                  <CardTitle className="text-lg text-slate-100">Quarter-over-Quarter Trends</CardTitle>
                </div>
              </CardHeader>
              <CardContent>
                {qoqTrends.length === 0 ? (
                  <p className="text-sm text-slate-500 text-center py-6">No trend data available</p>
                ) : (
                  <div className="rounded-lg border border-slate-800 overflow-auto">
                    <Table>
                      <TableHeader>
                        <TableRow className="border-slate-800 hover:bg-transparent">
                          <TableHead className="text-slate-400 text-xs">Quarter</TableHead>
                          <TableHead className="text-slate-400 text-xs">Department</TableHead>
                          <TableHead className="text-slate-400 text-xs text-right">Avg Score</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {qoqTrends.map((item, i) => (
                          <TableRow key={i} className="border-slate-800">
                            <TableCell className="text-slate-300 text-xs font-medium">
                              {item.quarter}
                            </TableCell>
                            <TableCell className="text-slate-400 text-xs">
                              {item.department}
                            </TableCell>
                            <TableCell className="text-xs text-right font-semibold">
                              <span
                                className={
                                  item.average_score >= 80
                                    ? "text-emerald-400"
                                    : item.average_score >= 50
                                      ? "text-amber-400"
                                      : "text-red-400"
                                }
                              >
                                {Math.round(item.average_score)}%
                              </span>
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  </div>
                )}
              </CardContent>
            </Card>
          </>
        )}
      </div>
    </DashboardLayout>
  );
}
