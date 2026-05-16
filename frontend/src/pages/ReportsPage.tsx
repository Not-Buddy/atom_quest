import { useQuery } from "@tanstack/react-query";
import { useAuth } from "@/contexts/AuthContext";
import {
  achievementReport,
  completionDashboard,
  downloadAchievementReport,
  downloadDashboardReport,
} from "@/lib/api";
import type { AchievementReportEntry, CompletionDashboardEntry } from "@/lib/types";
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
import { Download, Loader2, AlertCircle, Award, Building, FileSpreadsheet } from "lucide-react";

export default function ReportsPage() {
  const { token } = useAuth();

  const {
    data: achievementData = [],
    isLoading: achievementLoading,
    isError: achievementError,
  } = useQuery({
    queryKey: ["achievement-report"],
    queryFn: () => achievementReport(token!),
    enabled: !!token,
  });

  const {
    data: completionData = [],
    isLoading: completionLoading,
    isError: completionError,
  } = useQuery({
    queryKey: ["completion-dashboard"],
    queryFn: () => completionDashboard(token!),
    enabled: !!token,
  });

  const handleDownloadAchievement = async () => {
    try {
      await downloadAchievementReport(token!);
    } catch (err) {
      console.error("Download failed:", err);
    }
  };

  const handleDownloadDashboard = async () => {
    try {
      await downloadDashboardReport(token!);
    } catch (err) {
      console.error("Download failed:", err);
    }
  };

  const isLoading = achievementLoading || completionLoading;

  return (
    <DashboardLayout>
      <div className="max-w-7xl mx-auto space-y-6">
        <div className="flex items-center justify-between flex-wrap gap-4">
          <div>
            <h1 className="text-2xl font-bold text-slate-100">Reports</h1>
            <p className="text-sm text-slate-400 mt-1">
              Achievement reports and completion dashboard
            </p>
          </div>
        </div>

        {isLoading ? (
          <div className="flex justify-center py-20">
            <Loader2 className="h-8 w-8 animate-spin text-indigo-500" />
          </div>
        ) : (
          <>
            <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
              <CardHeader className="pb-3">
                <div className="flex items-center justify-between gap-4 flex-wrap">
                  <div className="flex items-center gap-2">
                    <Award className="h-5 w-5 text-indigo-400" />
                    <CardTitle className="text-lg text-slate-100">Achievement Report</CardTitle>
                  </div>
                  <div className="flex gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      className="border-slate-700 text-slate-300 hover:bg-slate-800"
                      onClick={handleDownloadAchievement}
                    >
                      <Download className="mr-2 h-3.5 w-3.5" />
                      Download Excel
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      className="border-slate-700 text-slate-300 hover:bg-slate-800"
                      onClick={handleDownloadDashboard}
                    >
                      <FileSpreadsheet className="mr-2 h-3.5 w-3.5" />
                      Download Dashboard
                    </Button>
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                {achievementError ? (
                  <Alert
                    variant="destructive"
                    className="bg-red-950/50 border-red-800 text-red-400"
                  >
                    <AlertCircle className="h-4 w-4" />
                    <AlertDescription>Failed to load achievement report</AlertDescription>
                  </Alert>
                ) : achievementData.length === 0 ? (
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
                          <TableHead className="text-slate-400 text-xs text-right">Weight</TableHead>
                          <TableHead className="text-slate-400 text-xs text-right">Q1</TableHead>
                          <TableHead className="text-slate-400 text-xs text-right">Q2</TableHead>
                          <TableHead className="text-slate-400 text-xs text-right">Q3</TableHead>
                          <TableHead className="text-slate-400 text-xs text-right">Q4</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {achievementData.map((row, i) => (
                          <TableRow key={i} className="border-slate-800">
                            <TableCell className="text-slate-300 text-xs">
                              {row.user_name}
                            </TableCell>
                            <TableCell className="text-slate-400 text-xs">
                              {row.department ?? "—"}
                            </TableCell>
                            <TableCell className="text-slate-400 text-xs">
                              {row.cycle_name}
                            </TableCell>
                            <TableCell className="text-slate-300 text-xs max-w-[150px] truncate">
                              {row.goal_title}
                            </TableCell>
                            <TableCell className="text-slate-400 text-xs">{row.uom_type}</TableCell>
                            <TableCell className="text-slate-300 text-xs text-right">
                              {row.target_value}
                            </TableCell>
                            <TableCell className="text-slate-400 text-xs text-right">
                              {row.weightage}
                            </TableCell>
                            <TableCell className="text-xs text-right font-semibold">
                              <ScoreCell score={row.q1_score} />
                            </TableCell>
                            <TableCell className="text-xs text-right font-semibold">
                              <ScoreCell score={row.q2_score} />
                            </TableCell>
                            <TableCell className="text-xs text-right font-semibold">
                              <ScoreCell score={row.q3_score} />
                            </TableCell>
                            <TableCell className="text-xs text-right font-semibold">
                              <ScoreCell score={row.q4_score} />
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  </div>
                )}
              </CardContent>
            </Card>

            <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
              <CardHeader className="pb-3">
                <div className="flex items-center gap-2">
                  <Building className="h-5 w-5 text-emerald-400" />
                  <CardTitle className="text-lg text-slate-100">Completion Dashboard</CardTitle>
                </div>
              </CardHeader>
              <CardContent>
                {completionError ? (
                  <Alert
                    variant="destructive"
                    className="bg-red-950/50 border-red-800 text-red-400"
                  >
                    <AlertCircle className="h-4 w-4" />
                    <AlertDescription>Failed to load completion dashboard</AlertDescription>
                  </Alert>
                ) : completionData.length === 0 ? (
                  <p className="text-sm text-slate-500 text-center py-6">No data available</p>
                ) : (
                  <div className="space-y-5">
                    {completionData.map((item, i) => (
                      <CompletionRow key={i} item={item} />
                    ))}
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

function ScoreCell({ score }: { score: number | null }) {
  if (score == null) return <span className="text-slate-600">—</span>;
  return (
    <span
      className={
        score >= 80 ? "text-emerald-400" : score >= 50 ? "text-amber-400" : "text-red-400"
      }
    >
      {Math.round(score)}%
    </span>
  );
}

function CompletionRow({ item }: { item: CompletionDashboardEntry }) {
  const total = item.total_sheets || 1;
  const completed = item.approved_count + item.locked_count;
  const pct = Math.round((completed / total) * 100);

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between text-sm">
        <span className="font-medium text-slate-200">
          {item.department ?? "Unknown Department"}
        </span>
        <span className="text-xs text-slate-400">
          {completed}/{total} completed &middot; {pct}%
        </span>
      </div>
      <div className="flex gap-1 h-3">
        {item.draft_count > 0 && (
          <div
            className="bg-slate-600 rounded-sm h-full transition-all"
            style={{ width: `${(item.draft_count / total) * 100}%` }}
            title={`Draft: ${item.draft_count}`}
          />
        )}
        {item.submitted_count > 0 && (
          <div
            className="bg-amber-500 rounded-sm h-full transition-all"
            style={{ width: `${(item.submitted_count / total) * 100}%` }}
            title={`Submitted: ${item.submitted_count}`}
          />
        )}
        {item.approved_count > 0 && (
          <div
            className="bg-emerald-500 rounded-sm h-full transition-all"
            style={{ width: `${(item.approved_count / total) * 100}%` }}
            title={`Approved: ${item.approved_count}`}
          />
        )}
        {item.returned_count > 0 && (
          <div
            className="bg-red-500 rounded-sm h-full transition-all"
            style={{ width: `${(item.returned_count / total) * 100}%` }}
            title={`Returned: ${item.returned_count}`}
          />
        )}
        {item.locked_count > 0 && (
          <div
            className="bg-indigo-500 rounded-sm h-full transition-all"
            style={{ width: `${(item.locked_count / total) * 100}%` }}
            title={`Locked: ${item.locked_count}`}
          />
        )}
      </div>
    </div>
  );
}
