// Goal Setting & Tracking Portal - TypeScript Types

export type UserRole = "employee" | "manager" | "admin";

export type SheetStatus = "draft" | "submitted" | "approved" | "locked" | "returned";

export type UomType = "min_numeric" | "max_numeric" | "timeline" | "zero" | "percent";

export type AchievementStatus = "not_started" | "on_track" | "completed";

export type Quarter = "Q1" | "Q2" | "Q3" | "Q4";

export type ThrustArea = "Innovation" | "Quality" | "Productivity" | "Customer Focus" | "Safety" | "Sustainability";

export interface AppUser {
  id: string;
  name: string;
  email: string;
  role: UserRole;
  department: string;
  manager_id: string | null;
  manager_name?: string;
  created_at: string;
  updated_at: string;
}

export interface GoalCycle {
  id: string;
  name: string;
  description?: string;
  start_date: string;
  end_date: string;
  status: "active" | "completed" | "upcoming";
}

export interface Goal {
  id: string;
  sheet_id: string;
  title: string;
  description?: string;
  uom_type: UomType;
  target_value: number;
  target_date?: string | null;
  weightage: number;
  thrust_area: ThrustArea;
  order_index?: number;
  created_at?: string;
  updated_at?: string;
}

export interface Achievement {
  id: string;
  goal_id: string;
  quarter: Quarter;
  actual_value: number;
  status: AchievementStatus;
  comment?: string;
  created_at?: string;
  updated_at?: string;
}

export interface GoalSheet {
  id: string;
  user_id: string;
  user_name?: string;
  user_email?: string;
  user_department?: string;
  cycle_id: string;
  cycle_name?: string;
  status: SheetStatus;
  goals: Goal[];
  achievements?: Achievement[];
  submitted_at?: string;
  approved_at?: string;
  locked_at?: string;
  returned_at?: string;
  return_reason?: string;
  created_at?: string;
  updated_at?: string;
}

export interface CheckinEntry {
  goal_id: string;
  goal_title?: string;
  q1_actual: number;
  q2_actual: number;
  q3_actual: number;
  q4_actual: number;
  q1_status: AchievementStatus;
  q2_status: AchievementStatus;
  q3_status: AchievementStatus;
  q4_status: AchievementStatus;
  q1_score: number;
  q2_score: number;
  q3_score: number;
  q4_score: number;
  comment?: string;
}

export interface AuditLogEntry {
  id: string;
  timestamp: string;
  table_name: string;
  record_id: string;
  field: string;
  old_value: string | null;
  new_value: string | null;
  changed_by: string;
}

export interface ManagerStats {
  total_team_members: number;
  pending_approvals: number;
  submitted_sheets: number;
}

export interface AdminStats {
  total_users: number;
  active_cycles: number;
  departments: number;
}

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  limit: number;
}

export interface ReportData {
  achievement_report: AchievementReportRow[];
  completion_dashboard: CompletionDashboardItem[];
  qoq_trends: QoQTrendItem[];
}

export interface AchievementReportRow {
  employee_name: string;
  department: string;
  cycle: string;
  goal_title: string;
  uom: string;
  target: number;
  q1: number;
  q2: number;
  q3: number;
  q4: number;
  total_score: number;
}

export interface CompletionDashboardItem {
  department: string;
  total_employees: number;
  submitted: number;
  approved: number;
  completion_rate: number;
}

export interface QoQTrendItem {
  quarter: string;
  average_score: number;
  department: string;
}

// UoM type display configuration
export const UOM_LABELS: Record<UomType, string> = {
  min_numeric: "Minimize",
  max_numeric: "Maximize",
  timeline: "Timeline",
  zero: "Zero",
  percent: "Percent",
};

export const UOM_COLORS: Record<UomType, string> = {
  min_numeric: "bg-blue-500/20 text-blue-400 border-blue-500/50",
  max_numeric: "bg-orange-500/20 text-orange-400 border-orange-500/50",
  timeline: "bg-purple-500/20 text-purple-400 border-purple-500/50",
  zero: "bg-red-500/20 text-red-400 border-red-500/50",
  percent: "bg-green-500/20 text-green-400 border-green-500/50",
};

export const STATUS_CONFIG: Record<SheetStatus, { label: string; variant: "default" | "secondary" | "success" | "warning" | "destructive" | "outline" | "accent" }> = {
  draft: { label: "Draft", variant: "secondary" },
  submitted: { label: "Submitted", variant: "default" },
  approved: { label: "Approved", variant: "success" },
  locked: { label: "Locked", variant: "accent" },
  returned: { label: "Returned", variant: "warning" },
};

export const THRUST_AREAS: ThrustArea[] = [
  "Innovation",
  "Quality",
  "Productivity",
  "Customer Focus",
  "Safety",
  "Sustainability",
];

export const QUARTERS: Quarter[] = ["Q1", "Q2", "Q3", "Q4"];

// ============================================================
// API types (used by the existing api.ts imports)
// ============================================================
export interface LoginPayload { email: string; password: string; }
export interface AuthResponse { token: string; user: User; }
export interface User { id: string | number; user_id?: string; name: string; email: string; role: string; department?: string; manager_id?: number | null; }
export interface GoalSheetSummary { id: string | number; user_id: string | number; user_name?: string; cycle_name?: string; status: SheetStatus; goal_count?: number; goals?: Goal[]; }
export interface CreateGoalPayload { title: string; description?: string; uom_type: string; target_value: number; target_date?: string; weightage: number; thrust_area: string; }
export interface UpdateGoalPayload { title?: string; description?: string; uom_type?: string; target_value?: number; target_date?: string; weightage?: number; thrust_area?: string; }
export interface AchievementUpdatePayload { actual_value: number; status: string; comment?: string; }
export interface CreateCyclePayload { name: string; description?: string; start_date: string; end_date: string; }
export interface Department { id: string | number; name: string; shortName?: string; }
export interface ThrustArea { id: string | number; name: string; }
export interface CreateUserPayload { name: string; email: string; password: string; role: string; department?: string; manager_id?: number | null; }
export interface CreateDepartmentPayload { name: string; shortName?: string; }
export interface CreateThrustAreaPayload { name: string; }
export interface ManagerEditGoalPayload { target_value?: number; weightage?: number; }
export interface ReturnSheetPayload { reason: string; }
export interface SharedGoalPushPayload { sheet_id: number; goal_id: number; target_sheets: number[]; }
export interface CreateCheckinPayload { goal_id: number; quarter: string; comment: string; }
export interface AchievementReportEntry { employee_name: string; department: string; cycle: string; goal_title: string; uom: string; target: number; q1: number; q2: number; q3: number; q4: number; total_score: number; }
export interface CompletionDashboardEntry { department: string; total_employees: number; submitted: number; approved: number; completion_rate: number; }
