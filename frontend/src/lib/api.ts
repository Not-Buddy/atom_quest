import type {
  LoginPayload, AuthResponse, User,
  GoalSheetResponse, GoalSheetSummary, GoalResponse,
  AchievementResponse, AchievementUpdatePayload,
  CreateGoalPayload, UpdateGoalPayload,
  ManagerEditGoalPayload, ReturnSheetPayload, SharedGoalPushPayload,
  CreateCheckinPayload, CreateCyclePayload,
  Department, ThrustArea, CreateUserPayload, CreateDepartmentPayload, CreateThrustAreaPayload,
  AuditLogEntry, AchievementReportEntry, CompletionDashboardEntry,
  GoalCycle, CheckinCommentResponse,
} from "@/lib/types";

const API_BASE_URL = import.meta.env.VITE_API_URL ?? "/api";
type HttpMethod = "GET" | "POST" | "PUT" | "DELETE";

async function request<T>(path: string, options: { method?: HttpMethod; token?: string; body?: unknown; headers?: Record<string, string> } = {}): Promise<T> {
  const { method = "GET", token, body, headers = {} } = options;
  const response = await fetch(`${API_BASE_URL}${path}`, {
    method,
    headers: { "Content-Type": "application/json", Accept: "application/json", ...(token ? { Authorization: `Bearer ${token}` } : {}), ...headers },
    body: body ? JSON.stringify(body) : undefined,
  });
  if (!response.ok) {
    const errorText = await response.text().catch(() => "");
    let message = errorText || response.statusText || "Request failed";
    if (response.status === 401) message = "Invalid token";
    try { const errorJson = JSON.parse(errorText); if (errorJson.error) message = errorJson.error; } catch { /* not JSON */ }
    throw new Error(message);
  }
  let text: string;
  try { text = await response.text(); } catch (e) { throw new Error(`Failed to read response: ${e instanceof Error ? e.message : e}`); }
  if (!text || !text.trim()) throw new Error("Empty response");
  try { return JSON.parse(text) as T; } catch (e) { throw new Error(`Invalid JSON: ${e instanceof Error ? e.message : e}`); }
}

// ── Auth ───────────────────────────────────────────────────
export async function login(payload: LoginPayload): Promise<AuthResponse> {
  return await request<AuthResponse>("/auth/login", { method: "POST", body: payload });
}
export async function fetchMe(token: string): Promise<User> {
  return await request<User>("/auth/me", { token });
}
export async function forgotPassword(email: string): Promise<{ message: string }> {
  return await request<{ message: string }>("/auth/forgot-password", { method: "POST", body: { email } });
}
export async function resetPassword(token: string, newPassword: string): Promise<{ message: string }> {
  return await request<{ message: string }>(`/auth/reset-password-form?token=${encodeURIComponent(token)}`, { method: "POST", body: { new_password: newPassword } });
}

// ── Azure AD / Entra ID SSO ────────────────────────────────
export async function getAzureLoginUrl(): Promise<{ login_url: string }> {
  return await request<{ login_url: string }>("/auth/azure/login");
}
export async function azureCallback(code: string, state?: string): Promise<AuthResponse> {
  const params = new URLSearchParams({ code });
  if (state) params.set("state", state);
  return await request<AuthResponse>(`/auth/azure/callback?${params.toString()}`);
}
export async function syncAzureOrg(token: string): Promise<{ synced: number; total_azure_users: number; message?: string }> {
  return await request<{ synced: number; total_azure_users: number; message?: string }>("/auth/azure/sync-org", { method: "PUT", token });
}

// ── Employee Endpoints ─────────────────────────────────────
export async function createGoalSheet(token: string): Promise<GoalSheetResponse> {
  return await request<GoalSheetResponse>("/goals/sheets", { method: "POST", token });
}
export async function listGoalSheets(token: string): Promise<GoalSheetSummary[]> {
  return await request<GoalSheetSummary[]>("/goals/sheets", { token });
}
export async function getGoalSheet(token: string, sheetId: number): Promise<GoalSheetResponse> {
  return await request<GoalSheetResponse>(`/goals/sheets/${sheetId}`, { token });
}
export async function submitSheet(token: string, sheetId: number): Promise<string> {
  return await request<string>(`/goals/sheets/${sheetId}/submit`, { method: "PUT", token });
}
export async function addGoal(token: string, sheetId: number, payload: CreateGoalPayload): Promise<GoalResponse> {
  return await request<GoalResponse>(`/goals/sheets/${sheetId}/goals`, { method: "POST", token, body: payload });
}
export async function updateGoal(token: string, goalId: number, payload: UpdateGoalPayload): Promise<GoalResponse> {
  return await request<GoalResponse>(`/goals/${goalId}`, { method: "PUT", token, body: payload });
}
export async function deleteGoal(token: string, goalId: number): Promise<string> {
  return await request<string>(`/goals/${goalId}`, { method: "DELETE", token });
}

// ── Achievement Endpoints ──────────────────────────────────
export async function getAchievements(token: string, sheetId: number): Promise<AchievementResponse[]> {
  return await request<AchievementResponse[]>(`/achievements/sheet/${sheetId}`, { token });
}
export async function updateAchievement(token: string, goalId: number, quarter: string, payload: AchievementUpdatePayload): Promise<AchievementResponse> {
  return await request<AchievementResponse>(`/achievements/${goalId}/${quarter}`, { method: "PUT", token, body: payload });
}

// ── Manager Endpoints ──────────────────────────────────────
export async function listTeamSheets(token: string): Promise<GoalSheetSummary[]> {
  return await request<GoalSheetSummary[]>("/manager/team/sheets", { token });
}
export async function approveSheet(token: string, sheetId: number): Promise<string> {
  return await request<string>(`/manager/sheets/${sheetId}/approve`, { method: "PUT", token });
}
export async function returnSheet(token: string, sheetId: number, payload: ReturnSheetPayload): Promise<string> {
  return await request<string>(`/manager/sheets/${sheetId}/return`, { method: "PUT", token, body: payload });
}
export async function managerEditGoal(token: string, sheetId: number, goalId: number, payload: ManagerEditGoalPayload): Promise<string> {
  return await request<string>(`/manager/sheets/${sheetId}/goals/${goalId}`, { method: "PUT", token, body: payload });
}
export async function pushSharedGoal(token: string, payload: SharedGoalPushPayload): Promise<string> {
  return await request<string>("/manager/shared-goals", { method: "POST", token, body: payload });
}
export async function listTeamCheckins(token: string): Promise<CheckinCommentResponse[]> {
  return await request<CheckinCommentResponse[]>("/manager/team/checkins", { token });
}
export async function addCheckinComment(token: string, sheetId: number, payload: CreateCheckinPayload): Promise<string> {
  return await request<string>(`/manager/checkins/${sheetId}`, { method: "POST", token, body: payload });
}

// ── Admin Endpoints ────────────────────────────────────────
export async function listCycles(token: string): Promise<GoalCycle[]> {
  return await request<GoalCycle[]>("/admin/cycles", { token });
}
export async function createCycle(token: string, payload: CreateCyclePayload): Promise<GoalCycle> {
  return await request<GoalCycle>("/admin/cycles", { method: "POST", token, body: payload });
}
export async function updateCycle(token: string, cycleId: number, payload: Partial<CreateCyclePayload>): Promise<GoalCycle> {
  return await request<GoalCycle>(`/admin/cycles/${cycleId}`, { method: "PUT", token, body: payload });
}
export async function listDepartments(token: string): Promise<Department[]> {
  return await request<Department[]>("/admin/departments", { token });
}
export async function createDepartment(token: string, payload: CreateDepartmentPayload): Promise<Department> {
  return await request<Department>("/admin/departments", { method: "POST", token, body: payload });
}
export async function listThrustAreas(token: string): Promise<ThrustArea[]> {
  return await request<ThrustArea[]>("/admin/thrust-areas", { token });
}
export async function createThrustArea(token: string, payload: CreateThrustAreaPayload): Promise<ThrustArea> {
  return await request<ThrustArea>("/admin/thrust-areas", { method: "POST", token, body: payload });
}
export async function listUsers(token: string): Promise<User[]> {
  return await request<User[]>("/admin/users", { token });
}
export async function createUser(token: string, payload: CreateUserPayload): Promise<User> {
  return await request<User>("/admin/users", { method: "POST", token, body: payload });
}
export async function updateUser(token: string, userId: number, payload: Partial<CreateUserPayload>): Promise<User> {
  return await request<User>(`/admin/users/${userId}`, { method: "PUT", token, body: payload });
}
export async function deleteUser(token: string, userId: number): Promise<string> {
  return await request<string>(`/admin/users/${userId}`, { method: "DELETE", token });
}
export async function unlockSheet(token: string, sheetId: number): Promise<string> {
  return await request<string>(`/admin/sheets/${sheetId}/unlock`, { method: "PUT", token });
}
export async function viewAuditLog(token: string, params?: { table_name?: string; record_id?: number; limit?: number; offset?: number }): Promise<AuditLogEntry[]> {
  const sp = new URLSearchParams();
  if (params?.table_name) sp.set("table_name", params.table_name);
  if (params?.record_id != null) sp.set("record_id", String(params.record_id));
  if (params?.limit != null) sp.set("limit", String(params.limit));
  if (params?.offset != null) sp.set("offset", String(params.offset));
  const q = sp.toString();
  return await request<AuditLogEntry[]>(`/admin/audit-log${q ? `?${q}` : ""}`, { token });
}

// ── Reports ────────────────────────────────────────────────
export async function achievementReport(token: string): Promise<AchievementReportEntry[]> {
  return await request<AchievementReportEntry[]>("/reports/achievement", { token });
}
export async function completionDashboard(token: string): Promise<CompletionDashboardEntry[]> {
  return await request<CompletionDashboardEntry[]>("/reports/completion-dashboard", { token });
}
export async function achievementReportExcel(token: string): Promise<Blob> {
  const r = await fetch(`${API_BASE_URL}/reports/achievement?format=excel`, { headers: { Authorization: `Bearer ${token}` } });
  if (!r.ok) throw new Error("Export failed");
  return r.blob();
}
async function downloadBlob(token: string, path: string, filename: string) {
  const r = await fetch(`${API_BASE_URL}${path}`, { headers: { Authorization: `Bearer ${token}` } });
  if (!r.ok) throw new Error("Download failed");
  const blob = await r.blob();
  const a = document.createElement("a"); a.href = URL.createObjectURL(blob);
  a.download = filename; a.style.display = "none"; document.body.appendChild(a); a.click(); a.remove();
}
export async function downloadAchievementReport(token: string) { await downloadBlob(token, "/reports/achievement?format=excel", "achievement_report.xlsx"); }
export async function downloadDashboardReport(token: string) { await downloadBlob(token, "/reports/completion-dashboard?format=excel", "completion_dashboard.xlsx"); }

export { API_BASE_URL };
