import type {
  LoginPayload,
  AuthResponse,
  User,
  GoalSheet,
  GoalSheetSummary,
  Goal,
  Achievement,
  CreateGoalPayload,
  UpdateGoalPayload,
  AchievementUpdatePayload,
  CreateCyclePayload,
  GoalCycle,
  Department,
  ThrustArea,
  CreateUserPayload,
  CreateDepartmentPayload,
  CreateThrustAreaPayload,
  ManagerEditGoalPayload,
  ReturnSheetPayload,
  SharedGoalPushPayload,
  CreateCheckinPayload,
  AuditLogEntry,
  AchievementReportEntry,
  CompletionDashboardEntry,
} from "@/lib/types";

const API_BASE_URL = import.meta.env.VITE_API_URL ?? "/api";

type HttpMethod = "GET" | "POST" | "PUT" | "DELETE";

async function request<T>(path: string, options: { method?: HttpMethod; token?: string; body?: unknown; headers?: Record<string, string> } = {}): Promise<T> {
  const { method = "GET", token, body, headers = {} } = options;

  const response = await fetch(`${API_BASE_URL}${path}`, {
    method,
    headers: {
      "Content-Type": body ? "application/json" : "application/json",
      Accept: "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...headers,
    },
    body: body ? JSON.stringify(body) : undefined,
  });

  if (!response.ok) {
    const errorText = await response.text().catch(() => "");
    let message = errorText || response.statusText || "Request failed";
    if (response.status === 401) message = "Invalid token";
    try {
      const errorJson = JSON.parse(errorText);
      if (errorJson.error) message = errorJson.error;
    } catch { /* not JSON */ }
    throw new Error(message);
  }

  let text: string;
  try { text = await response.text(); } catch (textError) {
    throw new Error(`Failed to read response body: ${textError instanceof Error ? textError.message : 'Unknown error'}`);
  }
  if (!text || text.trim() === '') {
    throw new Error("Empty response body received from server.");
  }
  try { return JSON.parse(text) as T; } catch (parseError) {
    throw new Error(`Invalid JSON response from server: ${parseError instanceof Error ? parseError.message : 'Unknown error'}`);
  }
}

// ============================================================
// Auth endpoints
// ============================================================
export async function login(payload: LoginPayload): Promise<AuthResponse> {
  return await request<AuthResponse>("/auth/login", { method: "POST", body: payload });
}

export async function fetchMe(token: string): Promise<User> {
  return await request<User>("/auth/me", { method: "GET", token });
}

export async function forgotPassword(email: string): Promise<{ message: string }> {
  return await request<{ message: string }>("/auth/forgot-password", { method: "POST", body: { email } });
}

export async function resetPassword(token: string, newPassword: string): Promise<{ message: string }> {
  return await request<{ message: string }>(`/auth/reset-password-form?token=${encodeURIComponent(token)}`, {
    method: "POST", body: { new_password: newPassword },
  });
}

// Goal Portal unified login
export async function loginUser(email: string, password: string) {
  return await request<{ token: string; user: Record<string, unknown> }>("/auth/login", {
    method: "POST", body: { email, password },
  });
}

// ============================================================
// Student / Faculty API (existing pages)
// ============================================================
export interface StudentLoginPayload { email: string; password: string; }
export interface ForgotPasswordPayload { email: string; }
export interface ResetPasswordPayload { new_password: string; }
export interface ChangePasswordPayload { current_password: string; new_password: string; }
export interface FacultyLoginPayload { specialization: string; username: string; password: string; }
export interface ProfileLinksPayload {
  github_link?: string | null; linkedin_link?: string | null;
  leetcode_link?: string | null; codechef_link?: string | null; codeforces_link?: string | null;
}

export interface StudentStats {
  id: number; serial_number: number; registration_number: string; full_name: string;
  college: string; course: string; specialization: string; academic_year: string;
  email: string; github_username: string | null; linkedin_url: string | null;
  leetcode_username: string | null; leetcode_total_solved: number;
  leetcode_solved_last_30_days: number; has_leetcode_account: boolean;
  leetcode_prev_month_solved: number; leetcode_last_synced_at: string | null;
  codechef_username: string | null; codechef_total_solved: number;
  codechef_solved_last_30_days: number; has_codechef_account: boolean;
  codechef_prev_month_solved: number; codechef_last_synced_at: string | null;
  codeforces_username: string | null; codeforces_total_solved: number;
  codeforces_solved_last_30_days: number; has_codeforces_account: boolean;
  codeforces_prev_month_solved: number; codeforces_rating: number;
  codeforces_max_rating: number; codeforces_rank: string | null;
  codeforces_last_synced_at: string | null; total_platforms_solved: number;
  total_solved_last_30_days: number; updated_at: string; created_at: string;
}

export interface FacultyStats {
  specialization: string; academic_year: string; total_students: number;
  with_leetcode_profiles: number; without_leetcode_profiles: number; defaulters: number;
}
export interface LeaderboardEntry {
  rank: number; full_name: string; registration_number: string;
  specialization: string | null; academic_year: string | null;
  github_username: string | null; leetcode_username: string | null;
  codechef_username: string | null; codeforces_username: string | null;
  linkedin_url: string | null; total_solved_last_30_days: number;
}
export interface LeaderboardResponse { academic_year: string; total_students: number; leaderboard: LeaderboardEntry[]; }

export async function studentLogin(payload: StudentLoginPayload) {
  return await request<Record<string, unknown>>("/auth/login", { method: "POST", body: payload });
}
export async function forgotPasswordRequest(payload: ForgotPasswordPayload) {
  return await request<{ message: string }>("/auth/forgot-password", { method: "POST", body: payload });
}
export async function resetPasswordRequest(token: string, payload: ResetPasswordPayload) {
  return await request<{ message: string }>(`/auth/reset-password-form?token=${encodeURIComponent(token)}`, { method: "POST", body: payload });
}
export async function changePassword(token: string, payload: ChangePasswordPayload) {
  return await request<{ message: string }>("/auth/change-password", { method: "POST", token, body: payload });
}
export async function facultyLogin(payload: FacultyLoginPayload) {
  return await request<Record<string, unknown>>("/faculty/login", { method: "POST", body: payload });
}
export async function fetchProfileLinks(token: string) {
  return await request<{ github_link?: string; linkedin_link?: string; leetcode_link?: string; codechef_link?: string; codeforces_link?: string }>("/profile/links", { method: "GET", token });
}
export async function fetchStudentStats(token: string): Promise<StudentStats> {
  return await request<StudentStats>("/student/me", { method: "GET", token });
}
export async function updateProfileLinks(token: string, links: ProfileLinksPayload) {
  return await request("/profile/links", { method: "POST", token, body: links });
}
export async function fetchFacultyProfile(token: string) {
  return await request<Record<string, unknown>>("/faculty/me", { method: "GET", token });
}
export async function downloadSubmissionsReport(token: string, specialization?: string) {
  const url = specialization
    ? `${API_BASE_URL}/faculty/reports/submissions?specialization=${encodeURIComponent(specialization)}`
    : `${API_BASE_URL}/faculty/reports/submissions`;
  const response = await fetch(url, { method: "GET", headers: { Authorization: `Bearer ${token}` } });
  if (!response.ok) { const text = await response.text().catch(() => response.statusText); throw new Error(text || "Failed to download report"); }
  const blob = await response.blob();
  const filename = response.headers.get("content-disposition")?.split("filename=")?.[1]?.replace(/"/g, "") ?? `student_submissions_${new Date().toISOString().replace(/[:.]/g, "-")}.xlsx`;
  const link = document.createElement("a"); link.href = window.URL.createObjectURL(blob);
  link.download = filename; link.style.display = "none"; document.body.appendChild(link); link.click(); link.remove();
}
export async function downloadDefaultersReport(token: string, specialization?: string) {
  const url = specialization
    ? `${API_BASE_URL}/faculty/reports/defaulters?specialization=${encodeURIComponent(specialization)}`
    : `${API_BASE_URL}/faculty/reports/defaulters`;
  const response = await fetch(url, { method: "GET", headers: { Authorization: `Bearer ${token}` } });
  if (!response.ok) { const text = await response.text().catch(() => response.statusText); throw new Error(text || "Failed to download defaulters report"); }
  const blob = await response.blob();
  const filename = response.headers.get("content-disposition")?.split("filename=")?.[1]?.replace(/"/g, "") ?? `defaulters_report_${new Date().toISOString().replace(/[:.]/g, "-")}.xlsx`;
  const link = document.createElement("a"); link.href = window.URL.createObjectURL(blob);
  link.download = filename; link.style.display = "none"; document.body.appendChild(link); link.click(); link.remove();
}
export async function fetchFacultyStats(token: string): Promise<FacultyStats | FacultyStats[]> {
  return await request<FacultyStats | FacultyStats[]>("/faculty/stats", { method: "GET", token });
}
export async function fetchLeaderboard(academicYear: "I" | "II" | "III" | "IV"): Promise<LeaderboardResponse> {
  return await request<LeaderboardResponse>(`/leaderboard?academic_year=${academicYear}`, { method: "GET" });
}

// ============================================================
// Goal Portal - Employee
// ============================================================
export async function createGoalSheet(token: string): Promise<GoalSheet> {
  return await request<GoalSheet>("/employee/goal-sheets", { method: "POST", token });
}
export async function listGoalSheets(token: string): Promise<GoalSheetSummary[]> {
  return await request<GoalSheetSummary[]>("/employee/goal-sheets", { method: "GET", token });
}
export async function getGoalSheet(token: string, sheetId: number): Promise<GoalSheet> {
  return await request<GoalSheet>(`/employee/goal-sheets/${sheetId}`, { method: "GET", token });
}
export async function submitSheet(token: string, sheetId: number): Promise<{ message: string }> {
  return await request<{ message: string }>(`/employee/goal-sheets/${sheetId}/submit`, { method: "POST", token });
}
export async function addGoal(token: string, sheetId: number, payload: CreateGoalPayload): Promise<Goal> {
  return await request<Goal>(`/employee/goal-sheets/${sheetId}/goals`, { method: "POST", token, body: payload });
}
export async function updateGoal(token: string, goalId: number, payload: UpdateGoalPayload): Promise<Goal> {
  return await request<Goal>(`/employee/goals/${goalId}`, { method: "PUT", token, body: payload });
}
export async function deleteGoal(token: string, goalId: number): Promise<{ message: string }> {
  return await request<{ message: string }>(`/employee/goals/${goalId}`, { method: "DELETE", token });
}
export async function getAchievements(token: string, sheetId: number): Promise<Achievement[]> {
  return await request<Achievement[]>(`/employee/goal-sheets/${sheetId}/achievements`, { method: "GET", token });
}
export async function updateAchievement(token: string, goalId: number, quarter: string, payload: AchievementUpdatePayload): Promise<Achievement> {
  return await request<Achievement>(`/employee/achievements/${goalId}/${quarter}`, { method: "PUT", token, body: payload });
}
export async function fetchActiveCycle(token: string) {
  return await request<{ cycle: import("./types").GoalCycle | null }>("/goal/cycles/active", { token });
}

// ============================================================
// Goal Portal - Manager
// ============================================================
export async function listTeamSheets(token: string): Promise<GoalSheetSummary[]> {
  return await request<GoalSheetSummary[]>("/manager/team-sheets", { method: "GET", token });
}
export async function approveSheet(token: string, sheetId: number): Promise<{ message: string }> {
  return await request<{ message: string }>(`/manager/sheets/${sheetId}/approve`, { method: "POST", token });
}
export async function returnSheet(token: string, sheetId: number, payload: ReturnSheetPayload): Promise<{ message: string }> {
  return await request<{ message: string }>(`/manager/sheets/${sheetId}/return`, { method: "POST", token, body: payload });
}
export async function managerEditGoal(token: string, sheetId: number, goalId: number, payload: ManagerEditGoalPayload): Promise<{ message: string }> {
  return await request<{ message: string }>(`/manager/sheets/${sheetId}/goals/${goalId}`, { method: "PUT", token, body: payload });
}
export async function pushSharedGoal(token: string, payload: SharedGoalPushPayload): Promise<{ message: string }> {
  return await request<{ message: string }>("/manager/shared-goals/push", { method: "POST", token, body: payload });
}
export async function viewTeamCheckins(token: string) {
  return await request("/manager/team-checkins", { method: "GET", token });
}
export async function addCheckinComment(token: string, sheetId: number, payload: CreateCheckinPayload): Promise<{ message: string }> {
  return await request<{ message: string }>(`/manager/sheets/${sheetId}/checkins`, { method: "POST", token, body: payload });
}
export async function fetchManagerStats(token: string) {
  return await request<{ stats: import("./types").ManagerStats }>("/goal/manager/stats", { token });
}

// ============================================================
// Goal Portal - Admin
// ============================================================
export async function listCycles(token: string): Promise<GoalCycle[]> {
  return await request<GoalCycle[]>("/admin/cycles", { method: "GET", token });
}
export async function createCycle(token: string, payload: CreateCyclePayload): Promise<GoalCycle> {
  return await request<GoalCycle>("/admin/cycles", { method: "POST", token, body: payload });
}
export async function updateCycle(token: string, cycleId: number, payload: Partial<CreateCyclePayload>): Promise<GoalCycle> {
  return await request<GoalCycle>(`/admin/cycles/${cycleId}`, { method: "PUT", token, body: payload });
}
export async function listDepartments(token: string): Promise<Department[]> {
  return await request<Department[]>("/admin/departments", { method: "GET", token });
}
export async function createDepartment(token: string, payload: CreateDepartmentPayload): Promise<Department> {
  return await request<Department>("/admin/departments", { method: "POST", token, body: payload });
}
export async function listThrustAreas(token: string): Promise<ThrustArea[]> {
  return await request<ThrustArea[]>("/admin/thrust-areas", { method: "GET", token });
}
export async function createThrustArea(token: string, payload: CreateThrustAreaPayload): Promise<ThrustArea> {
  return await request<ThrustArea>("/admin/thrust-areas", { method: "POST", token, body: payload });
}
export async function listUsers(token: string): Promise<User[]> {
  return await request<User[]>("/admin/users", { method: "GET", token });
}
export async function createUser(token: string, payload: CreateUserPayload): Promise<User> {
  return await request<User>("/admin/users", { method: "POST", token, body: payload });
}
export async function updateUser(token: string, userId: number, payload: Partial<CreateUserPayload>): Promise<User> {
  return await request<User>(`/admin/users/${userId}`, { method: "PUT", token, body: payload });
}
export async function deleteUser(token: string, userId: number): Promise<{ message: string }> {
  return await request<{ message: string }>(`/admin/users/${userId}`, { method: "DELETE", token });
}
export async function unlockSheet(token: string, sheetId: number): Promise<{ message: string }> {
  return await request<{ message: string }>(`/admin/sheets/${sheetId}/unlock`, { method: "POST", token });
}
export async function viewAuditLog(token: string, params?: { table_name?: string; record_id?: number; limit?: number; offset?: number }): Promise<AuditLogEntry[]> {
  const sp = new URLSearchParams();
  if (params?.table_name) sp.set("table_name", params.table_name);
  if (params?.record_id != null) sp.set("record_id", String(params.record_id));
  if (params?.limit != null) sp.set("limit", String(params.limit));
  if (params?.offset != null) sp.set("offset", String(params.offset));
  const q = sp.toString();
  return await request<AuditLogEntry[]>(`/admin/audit-log${q ? `?${q}` : ""}`, { method: "GET", token });
}
export async function fetchAdminStats(token: string) {
  return await request<{ stats: import("./types").AdminStats }>("/goal/admin/stats", { token });
}

// ============================================================
// Goal Portal - Reports
// ============================================================
export async function achievementReport(token: string, format?: string): Promise<AchievementReportEntry[] | Blob> {
  const q = format ? `?format=${encodeURIComponent(format)}` : "";
  if (format === "csv" || format === "xlsx") {
    const response = await fetch(`${API_BASE_URL}/reports/achievement${q}`, { method: "GET", headers: { Authorization: `Bearer ${token}` } });
    if (!response.ok) { const t = await response.text().catch(() => ""); throw new Error(t || "Failed to download report"); }
    return await response.blob();
  }
  return await request<AchievementReportEntry[]>(`/reports/achievement${q}`, { method: "GET", token });
}
export async function completionDashboard(token: string, format?: string): Promise<CompletionDashboardEntry[] | Blob> {
  const q = format ? `?format=${encodeURIComponent(format)}` : "";
  if (format === "csv" || format === "xlsx") {
    const response = await fetch(`${API_BASE_URL}/reports/completion${q}`, { method: "GET", headers: { Authorization: `Bearer ${token}` } });
    if (!response.ok) { const t = await response.text().catch(() => ""); throw new Error(t || "Failed to download report"); }
    return await response.blob();
  }
  return await request<CompletionDashboardEntry[]>(`/reports/completion${q}`, { method: "GET", token });
}

// ============================================================
// Goal Portal - Bridge / Adapter functions for new pages
// ============================================================
export async function fetchMySheets(token: string) {
  return await request<{ sheets: import("./types").GoalSheet[] }>("/goal/sheets/my", { token });
}
export async function fetchSheet(token: string, sheetId: string) {
  return await request<{ sheet: import("./types").GoalSheet }>(`/goal/sheets/${sheetId}`, { token });
}
export async function createSheet(token: string, cycleId: string) {
  return await request<{ sheet: import("./types").GoalSheet }>("/goal/sheets", { method: "POST", token, body: { cycle_id: cycleId } });
}
export async function createGoal(token: string, sheetId: string, goal: Omit<import("./types").Goal, "id" | "sheet_id" | "created_at" | "updated_at" | "order_index">) {
  return await request<{ goal: import("./types").Goal }>(`/goal/sheets/${sheetId}/goals`, { method: "POST", token, body: goal });
}
export async function editGoal(token: string, goalId: string, data: Partial<import("./types").Goal>) {
  return await request<{ goal: import("./types").Goal }>(`/goal/goals/${goalId}`, { method: "PUT", token, body: data });
}
export async function removeGoal(token: string, goalId: string) {
  return await request<{ success: boolean }>(`/goal/goals/${goalId}`, { method: "DELETE", token });
}
export async function submitSheetForApproval(token: string, sheetId: string) {
  return await request<{ sheet: import("./types").GoalSheet }>(`/goal/sheets/${sheetId}/submit`, { method: "POST", token });
}
export async function approveGoalSheet(token: string, sheetId: string) {
  return await request<{ sheet: import("./types").GoalSheet }>(`/goal/sheets/${sheetId}/approve`, { method: "POST", token });
}
export async function returnGoalSheet(token: string, sheetId: string, reason: string) {
  return await request<{ sheet: import("./types").GoalSheet }>(`/goal/sheets/${sheetId}/return`, { method: "POST", token, body: { reason } });
}
export async function fetchAchievements(token: string, sheetId: string, quarter?: import("./types").Quarter) {
  const q = quarter ? `?quarter=${quarter}` : "";
  return await request<{ achievements: import("./types").Achievement[] }>(`/goal/sheets/${sheetId}/achievements${q}`, { token });
}
export async function saveAchievements(token: string, sheetId: string, quarter: import("./types").Quarter, data: { goal_id: string; actual_value: number; status: import("./types").AchievementStatus; comment?: string }[]) {
  return await request<{ achievements: import("./types").Achievement[] }>(`/goal/sheets/${sheetId}/achievements`, { method: "POST", token, body: { quarter, entries: data } });
}
export async function fetchCheckin(token: string, sheetId: string) {
  return await request<{ checkin: import("./types").CheckinEntry[] }>(`/goal/sheets/${sheetId}/checkin`, { token });
}
export async function saveCheckin(token: string, sheetId: string, entries: { goal_id: string; q1_actual: number; q2_actual: number; q3_actual: number; q4_actual: number; q1_status: import("./types").AchievementStatus; q2_status: import("./types").AchievementStatus; q3_status: import("./types").AchievementStatus; q4_status: import("./types").AchievementStatus; q1_score: number; q2_score: number; q3_score: number; q4_score: number; comment?: string }[]) {
  return await request<{ checkin: import("./types").CheckinEntry[] }>(`/goal/sheets/${sheetId}/checkin`, { method: "POST", token, body: { entries } });
}
export async function fetchTeamMembers(token: string) {
  return await request<{ members: import("./types").GoalSheet[] }>("/goal/manager/team", { token });
}
export async function fetchUsers(token: string) {
  return await request<{ users: import("./types").AppUser[] }>("/goal/admin/users", { token });
}
export async function createAppUser(token: string, user: { name: string; email: string; password: string; role: import("./types").UserRole; department: string; manager_id?: string | null }) {
  return await request<{ user: import("./types").AppUser }>("/goal/admin/users", { method: "POST", token, body: user });
}
export async function updateAppUser(token: string, userId: string, data: Partial<import("./types").AppUser & { password?: string }>) {
  return await request<{ user: import("./types").AppUser }>(`/goal/admin/users/${userId}`, { method: "PUT", token, body: data });
}
export async function deleteAppUser(token: string, userId: string) {
  return await request<{ success: boolean }>(`/goal/admin/users/${userId}`, { method: "DELETE", token });
}
export async function fetchAuditLogs(token: string, params?: { table_name?: string; record_id?: string; page?: number; limit?: number }) {
  const sp = new URLSearchParams();
  if (params?.table_name) sp.set("table_name", params.table_name);
  if (params?.record_id) sp.set("record_id", params.record_id);
  if (params?.page !== undefined) sp.set("page", String(params.page));
  if (params?.limit !== undefined) sp.set("limit", String(params.limit));
  const qs = sp.toString();
  return await request<import("./types").PaginatedResponse<import("./types").AuditLogEntry>>(`/goal/admin/audit-log${qs ? `?${qs}` : ""}`, { token });
}
export async function fetchReports(token: string) {
  return await request<import("./types").ReportData>("/goal/reports", { token });
}
export async function downloadAchievementReport(token: string) {
  const url = `${API_BASE_URL}/goal/reports/achievement/download`;
  const response = await fetch(url, { headers: { Authorization: `Bearer ${token}` } });
  if (!response.ok) throw new Error("Failed to download report");
  const blob = await response.blob();
  const filename = `achievement_report_${new Date().toISOString().replace(/[:.]/g, "-")}.xlsx`;
  const link = document.createElement("a"); link.href = window.URL.createObjectURL(blob);
  link.download = filename; link.style.display = "none"; document.body.appendChild(link); link.click(); link.remove();
}

export { API_BASE_URL };
