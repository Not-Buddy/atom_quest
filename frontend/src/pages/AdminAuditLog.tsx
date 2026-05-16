import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { useAuth } from "@/contexts/AuthContext";
import { fetchAuditLogs } from "@/lib/api";
import { AuditLogEntry } from "@/lib/types";
import { DashboardLayout } from "@/components/DashboardLayout";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Loader2, AlertCircle, Shield, Search, X } from "lucide-react";

export default function AdminAuditLog() {
  const { token } = useAuth();

  const [tableFilter, setTableFilter] = useState("");
  const [recordFilter, setRecordFilter] = useState("");
  const [appliedTable, setAppliedTable] = useState("");
  const [appliedRecord, setAppliedRecord] = useState("");
  const [page, setPage] = useState(1);
  const limit = 50;

  const { data, isLoading, isError } = useQuery({
    queryKey: ["audit-logs", appliedTable, appliedRecord, page],
    queryFn: () =>
      fetchAuditLogs(token!, {
        table_name: appliedTable || undefined,
        record_id: appliedRecord || undefined,
        page,
        limit,
      }),
    enabled: !!token,
  });

  const entries: AuditLogEntry[] = data?.data || [];
  const total = data?.total || 0;
  const totalPages = Math.ceil(total / limit);

  const applyFilters = () => {
    setAppliedTable(tableFilter.trim());
    setAppliedRecord(recordFilter.trim());
    setPage(1);
  };

  const clearFilters = () => {
    setTableFilter("");
    setRecordFilter("");
    setAppliedTable("");
    setAppliedRecord("");
    setPage(1);
  };

  const hasFilters = appliedTable || appliedRecord;

  return (
    <DashboardLayout>
      <div className="max-w-7xl mx-auto space-y-6">
        <div>
          <h1 className="text-2xl font-bold text-slate-100">Audit Log</h1>
          <p className="text-sm text-slate-400 mt-1">Track all system changes</p>
        </div>

        {/* Filters */}
        <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
          <CardContent className="py-4">
            <div className="flex items-end gap-3 flex-wrap">
              <div className="space-y-1.5 flex-1 min-w-[180px]">
                <Label className="text-xs text-slate-400">Table Name</Label>
                <Input
                  value={tableFilter}
                  onChange={(e) => setTableFilter(e.target.value)}
                  placeholder="e.g., goal_sheets"
                  className="bg-slate-800/50 border-slate-700 text-slate-100 text-sm placeholder:text-slate-600"
                  onKeyDown={(e) => e.key === "Enter" && applyFilters()}
                />
              </div>
              <div className="space-y-1.5 flex-1 min-w-[180px]">
                <Label className="text-xs text-slate-400">Record ID</Label>
                <Input
                  value={recordFilter}
                  onChange={(e) => setRecordFilter(e.target.value)}
                  placeholder="e.g., sheet_abc123"
                  className="bg-slate-800/50 border-slate-700 text-slate-100 text-sm placeholder:text-slate-600"
                  onKeyDown={(e) => e.key === "Enter" && applyFilters()}
                />
              </div>
              <div className="flex gap-2">
                <Button
                  onClick={applyFilters}
                  size="sm"
                  className="bg-indigo-600 hover:bg-indigo-500 text-white"
                >
                  <Search className="mr-2 h-3.5 w-3.5" />
                  Search
                </Button>
                {hasFilters && (
                  <Button
                    onClick={clearFilters}
                    variant="outline"
                    size="sm"
                    className="border-slate-700 text-slate-300"
                  >
                    <X className="mr-2 h-3.5 w-3.5" />
                    Clear
                  </Button>
                )}
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Table */}
        {isLoading ? (
          <div className="flex justify-center py-12">
            <Loader2 className="h-6 w-6 animate-spin text-indigo-500" />
          </div>
        ) : isError ? (
          <Alert variant="destructive" className="bg-red-950/50 border-red-800 text-red-400">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>Failed to load audit logs</AlertDescription>
          </Alert>
        ) : entries.length === 0 ? (
          <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
            <CardContent className="py-12 text-center">
              <Shield className="h-8 w-8 text-slate-600 mx-auto mb-3" />
              <p className="text-slate-400">No audit log entries found</p>
              {hasFilters && (
                <p className="text-xs text-slate-600 mt-1">Try clearing filters</p>
              )}
            </CardContent>
          </Card>
        ) : (
          <>
            <div className="rounded-xl border border-slate-800 overflow-auto">
              <Table>
                <TableHeader>
                  <TableRow className="border-slate-800 hover:bg-transparent">
                    <TableHead className="text-slate-400 whitespace-nowrap">Timestamp</TableHead>
                    <TableHead className="text-slate-400 whitespace-nowrap">Table</TableHead>
                    <TableHead className="text-slate-400 whitespace-nowrap">Record ID</TableHead>
                    <TableHead className="text-slate-400 whitespace-nowrap">Field</TableHead>
                    <TableHead className="text-slate-400 whitespace-nowrap">Old Value</TableHead>
                    <TableHead className="text-slate-400 whitespace-nowrap">New Value</TableHead>
                    <TableHead className="text-slate-400 whitespace-nowrap">Changed By</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {entries.map((entry) => (
                    <TableRow key={entry.id} className="border-slate-800">
                      <TableCell className="text-slate-400 text-xs whitespace-nowrap">
                        {new Date(entry.timestamp).toLocaleString()}
                      </TableCell>
                      <TableCell className="text-slate-300 text-xs font-mono">
                        {entry.table_name}
                      </TableCell>
                      <TableCell className="text-slate-400 text-xs font-mono max-w-[120px] truncate">
                        {entry.record_id}
                      </TableCell>
                      <TableCell className="text-slate-300 text-xs">
                        {entry.field}
                      </TableCell>
                      <TableCell className="text-slate-500 text-xs max-w-[150px] truncate">
                        {entry.old_value ?? "—"}
                      </TableCell>
                      <TableCell className="text-slate-300 text-xs max-w-[150px] truncate">
                        {entry.new_value ?? "—"}
                      </TableCell>
                      <TableCell className="text-slate-400 text-xs">
                        {entry.changed_by}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>

            {/* Pagination */}
            {totalPages > 1 && (
              <div className="flex items-center justify-between pt-2">
                <p className="text-xs text-slate-500">
                  Showing {(page - 1) * limit + 1}–{Math.min(page * limit, total)} of {total}
                </p>
                <div className="flex gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={page <= 1}
                    onClick={() => setPage((p) => p - 1)}
                    className="border-slate-700 text-slate-300"
                  >
                    Previous
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={page >= totalPages}
                    onClick={() => setPage((p) => p + 1)}
                    className="border-slate-700 text-slate-300"
                  >
                    Next
                  </Button>
                </div>
              </div>
            )}
          </>
        )}
      </div>
    </DashboardLayout>
  );
}
