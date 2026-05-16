// AtomQuest — TypeScript types matching backend response shapes
// All IDs are numbers (backend returns i32)

export type UserRole = "employee" | "manager" | "admin";
export type SheetStatus = "draft" | "submitted" | "approved" | "locked" | "returned";
export type UomType = "min_numeric" | "max_numeric" | "min_percent" | "max_percent" | "timeline" | "zero";
export type AchievementStatus = "not_started" | "on_track" | "completed";
export type Quarter = "q1" | "q2" | "q3" | "q4";
export type QuarterLabel = "Q1" | "Q2" | "Q3" | "Q4";

// ── User ───────────────────────────────────────────────────
export interface User {
  id: number;
  email: string;
  full_name: string;
  department_id: number | null;
  role: UserRole;
  manager_id: number | null;
  created_at: string | null;
}

// ── Department ─────────────────────────────────────────────
export interface Department {
  id: number;
  name: string;
  short_name: string;
  created_at: string | null;
}

// ── ThrustArea ─────────────────────────────────────────────
export interface ThrustArea {
  id: number;
  name: string;
  department_id: number | null;
  created_by: number | null;
  created_at: string | null;
}

// ── Goal Cycle ─────────────────────────────────────────────
export interface GoalCycle {
  id: number;
  name: string;
  goal_setting_opens: string | null;
  q1_opens: string | null;
  q2_opens: string | null;
  q3_opens: string | null;
  q4_opens: string | null;
  is_active: boolean;
  created_by: number | null;
  created_at: string | null;
  updated_at: string | null;
}

// ── Achievement ────────────────────────────────────────────
export interface AchievementResponse {
  id: number;
  goal_id: number;
  quarter: Quarter;
  actual_value: number | null;
  actual_date: string | null;
  status: AchievementStatus;
  computed_score: number | null;
}

// ── Goal ───────────────────────────────────────────────────
export interface GoalResponse {
  id: number;
  sheet_id: number;
  thrust_area_id: number | null;
  thrust_area_name: string | null;
  title: string;
  description: string | null;
  uom_type: UomType;
  target_value: number;
  target_date: string | null;
  weightage: number;
  is_shared: boolean;
  shared_from_goal_id: number | null;
  sort_order: number;
  achievements: AchievementResponse[];
}

// ── Goal Sheet ─────────────────────────────────────────────
export interface GoalSheetSummary {
  id: number;
  user_id: number;
  user_name: string | null;
  cycle_id: number;
  cycle_name: string | null;
  status: SheetStatus;
  goal_count: number;
  total_weightage: number;
}

export interface CheckinCommentResponse {
  id: number;
  goal_sheet_id: number;
  quarter: string;
  manager_id: number;
  manager_name: string | null;
  comment: string;
  created_at: string | null;
}

export interface GoalSheetResponse {
  id: number;
  user_id: number;
  user_name: string | null;
  cycle_id: number;
  cycle_name: string | null;
  status: SheetStatus;
  submitted_at: string | null;
  approved_at: string | null;
  approved_by: number | null;
  returned_reason: string | null;
  goals: GoalResponse[];
  total_weightage: number;
  checkins: CheckinCommentResponse[];
}

// ── Audit Log ──────────────────────────────────────────────
export interface AuditLogEntry {
  id: number;
  table_name: string;
  record_id: number;
  field_name: string | null;
  old_value: string | null;
  new_value: string | null;
  changed_by: number | null;
  changed_at: string | null;
}

// ── Reports ────────────────────────────────────────────────
export interface AchievementReportEntry {
  user_name: string;
  department: string | null;
  cycle_name: string;
  sheet_status: string;
  goal_title: string;
  uom_type: string;
  target_value: number;
  weightage: number;
  q1_actual: number | null;
  q1_score: number | null;
  q2_actual: number | null;
  q2_score: number | null;
  q3_actual: number | null;
  q3_score: number | null;
  q4_actual: number | null;
  q4_score: number | null;
}

export interface CompletionDashboardEntry {
  department: string | null;
  total_sheets: number;
  draft_count: number;
  submitted_count: number;
  approved_count: number;
  returned_count: number;
  locked_count: number;
}

// ── API Payloads ───────────────────────────────────────────
export interface LoginPayload { email: string; password: string; }
export interface AuthResponse { token: string; user: User; }

export interface CreateGoalPayload {
  thrust_area_id: number | null;
  title: string;
  description?: string | null;
  uom_type: UomType;
  target_value: number;
  target_date?: string | null;
  weightage: number;
  is_shared?: boolean;
  sort_order?: number;
}

export interface UpdateGoalPayload {
  thrust_area_id?: number | null;
  title?: string;
  description?: string | null;
  uom_type?: UomType;
  target_value?: number;
  target_date?: string | null;
  weightage?: number;
  is_shared?: boolean;
  sort_order?: number;
}

export interface AchievementUpdatePayload {
  actual_value?: number | null;
  actual_date?: string | null;
  status?: AchievementStatus;
}

export interface ManagerEditGoalPayload {
  target_value?: number;
  weightage?: number;
}

export interface ReturnSheetPayload { reason: string; }

export interface SharedGoalPushPayload {
  sheet_ids: number[];
  thrust_area_id?: number | null;
  title: string;
  description?: string | null;
  uom_type: UomType;
  target_value: number;
  target_date?: string | null;
  weightage: number;
}

export interface CreateCheckinPayload { quarter: QuarterLabel; comment: string; }

export interface CreateCyclePayload {
  name: string;
  goal_setting_opens?: string | null;
  q1_opens?: string | null;
  q2_opens?: string | null;
  q3_opens?: string | null;
  q4_opens?: string | null;
  is_active?: boolean;
}

export interface CreateUserPayload {
  email: string;
  full_name: string;
  password: string;
  department_id: number | null;
  role: UserRole;
  manager_id: number | null;
}

export interface CreateDepartmentPayload { name: string; short_name: string; }
export interface CreateThrustAreaPayload { name: string; department_id?: number | null; }

// ── Display helpers ────────────────────────────────────────
export const UOM_LABELS: Record<UomType, string> = {
  min_numeric: "Minimize",
  max_numeric: "Maximize",
  min_percent: "Min %",
  max_percent: "Max %",
  timeline: "Timeline",
  zero: "Zero",
};

export const UOM_COLORS: Record<UomType, string> = {
  min_numeric: "bg-blue-500/20 text-blue-400 border-blue-500/50",
  max_numeric: "bg-orange-500/20 text-orange-400 border-orange-500/50",
  min_percent: "bg-green-500/20 text-green-400 border-green-500/50",
  max_percent: "bg-yellow-500/20 text-yellow-400 border-yellow-500/50",
  timeline: "bg-purple-500/20 text-purple-400 border-purple-500/50",
  zero: "bg-red-500/20 text-red-400 border-red-500/50",
};

export const STATUS_CONFIG: Record<SheetStatus, { label: string; variant: string }> = {
  draft: { label: "Draft", variant: "secondary" },
  submitted: { label: "Submitted", variant: "default" },
  approved: { label: "Approved", variant: "success" },
  locked: { label: "Locked", variant: "accent" },
  returned: { label: "Returned", variant: "warning" },
};

export const QUARTER_LABELS: QuarterLabel[] = ["Q1", "Q2", "Q3", "Q4"];
export const QUARTERS: Quarter[] = ["q1", "q2", "q3", "q4"];
