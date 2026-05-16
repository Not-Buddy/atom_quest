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
    
    // Handle 401 Unauthorized specifically
    if (response.status === 401) {
      message = "Invalid token";
    }
    
    // Try to parse JSON error and extract the error message
    try {
      const errorJson = JSON.parse(errorText);
      if (errorJson.error) {
        message = errorJson.error;
      }
    } catch {
      // If not JSON, use the text as is
    }
    
    throw new Error(message);
  }

  // Try parsing JSON; if it fails, provide better error message
  // NEVER return a Response object - always throw an error if parsing fails
  let text: string;
  try {
    text = await response.text();
  } catch (textError) {
    throw new Error(`Failed to read response body: ${textError instanceof Error ? textError.message : 'Unknown error'}`);
  }

  if (!text || text.trim() === '') {
    throw new Error("Empty response body received from server. This may indicate a CORS or network issue.");
  }

  try {
    const jsonData = JSON.parse(text) as T;
    return jsonData;
  } catch (parseError) {
    throw new Error(`Invalid JSON response from server: ${parseError instanceof Error ? parseError.message : 'Unknown error'}`);
  }
}

interface StudentLoginPayload {
  email: string;
  password: string;
}

interface ForgotPasswordPayload {
  email: string;
}

interface ResetPasswordPayload {
  new_password: string;
}

interface ChangePasswordPayload {
  current_password: string;
  new_password: string;
}

interface FacultyLoginPayload {
  specialization: string;
  username: string;
  password: string;
}

export interface ProfileLinksPayload {
  github_link?: string | null;
  linkedin_link?: string | null;
  leetcode_link?: string | null;
  codechef_link?: string | null;
  codeforces_link?: string | null;
}

export interface StudentStats {
  id: number;
  serial_number: number;
  registration_number: string;
  full_name: string;
  college: string;
  course: string;
  specialization: string;
  academic_year: string;
  email: string;
  github_username: string | null;
  linkedin_url: string | null;
  leetcode_username: string | null;
  leetcode_total_solved: number;
  leetcode_solved_last_30_days: number;
  has_leetcode_account: boolean;
  leetcode_prev_month_solved: number;
  leetcode_last_synced_at: string | null;
  codechef_username: string | null;
  codechef_total_solved: number;
  codechef_solved_last_30_days: number;
  has_codechef_account: boolean;
  codechef_prev_month_solved: number;
  codechef_last_synced_at: string | null;
  codeforces_username: string | null;
  codeforces_total_solved: number;
  codeforces_solved_last_30_days: number;
  has_codeforces_account: boolean;
  codeforces_prev_month_solved: number;
  codeforces_rating: number;
  codeforces_max_rating: number;
  codeforces_rank: string | null;
  codeforces_last_synced_at: string | null;
  total_platforms_solved: number;
  total_solved_last_30_days: number;
  updated_at: string;
  created_at: string;
}

export async function studentLogin(payload: StudentLoginPayload) {
  const { email, password } = payload;

  const response = await request<Record<string, unknown>>("/auth/login", {
    method: "POST",
    body: {
      email,
      password,
    },
  });
  return response;
}

export async function forgotPassword(payload: ForgotPasswordPayload) {
  const response = await request<{ message: string }>("/auth/forgot-password", {
    method: "POST",
    body: payload,
  });
  return response;
}

export async function resetPassword(token: string, payload: ResetPasswordPayload) {
  const response = await request<{ message: string }>(`/auth/reset-password-form?token=${encodeURIComponent(token)}`, {
    method: "POST",
    body: payload,
  });
  return response;
}

export async function changePassword(token: string, payload: ChangePasswordPayload) {
  const response = await request<{ message: string }>("/auth/change-password", {
    method: "POST",
    token,
    body: payload,
  });
  return response;
}

export async function facultyLogin(payload: FacultyLoginPayload) {
  const { specialization, username, password } = payload;

  // Script spec: POST /faculty/login
  const response = await request<Record<string, unknown>>("/faculty/login", {
    method: "POST",
    body: {
      specialization,
      username,
      password,
    },
  });
  return response;
}

export async function fetchProfileLinks(token: string) {
  // Script spec: GET /profile/links
  const response = await request<{ github_link?: string; linkedin_link?: string; leetcode_link?: string; codechef_link?: string; codeforces_link?: string }>("/profile/links", {
    method: "GET",
    token,
  });
  return response;
}

export async function fetchStudentStats(token: string): Promise<StudentStats> {
  const response = await request<StudentStats>("/student/me", {
    method: "GET",
    token,
  });
  return response;
}

export async function updateProfileLinks(token: string, links: ProfileLinksPayload) {
  // Script spec: POST /profile/links (not PUT)
  const payload = {
    github_link: links.github_link ?? null,
    linkedin_link: links.linkedin_link ?? null,
    leetcode_link: links.leetcode_link ?? null,
    codechef_link: links.codechef_link ?? null,
    codeforces_link: links.codeforces_link ?? null,
  };

  const response = await request("/profile/links", {
    method: "POST",
    token,
    body: payload,
  });
  return response;
}

export async function fetchFacultyProfile(token: string) {
  return await request<Record<string, unknown>>("/faculty/me", {
    method: "GET",
    token,
  });
}

export async function downloadSubmissionsReport(token: string, specialization?: string) {
  const url = specialization 
    ? `${API_BASE_URL}/faculty/reports/submissions?specialization=${encodeURIComponent(specialization)}`
    : `${API_BASE_URL}/faculty/reports/submissions`;
    
  const response = await fetch(url, {
    method: "GET",
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });

  if (!response.ok) {
    const text = await response.text().catch(() => response.statusText);
    throw new Error(text || "Failed to download report");
  }

  const blob = await response.blob();
  const filename = response.headers.get("content-disposition")?.split("filename=")?.[1]?.replace(/"/g, "") ??
    `student_submissions_${new Date().toISOString().replace(/[:.]/g, "-")}.xlsx`;

  const link = document.createElement("a");
  link.href = window.URL.createObjectURL(blob);
  link.download = filename;
  link.style.display = "none";
  document.body.appendChild(link);
  link.click();
  link.remove();
}

export async function downloadDefaultersReport(token: string, specialization?: string) {
  const url = specialization
    ? `${API_BASE_URL}/faculty/reports/defaulters?specialization=${encodeURIComponent(specialization)}`
    : `${API_BASE_URL}/faculty/reports/defaulters`;
    
  const response = await fetch(url, {
    method: "GET",
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });

  if (!response.ok) {
    const text = await response.text().catch(() => response.statusText);
    throw new Error(text || "Failed to download defaulters report");
  }

  const blob = await response.blob();
  const filename = response.headers.get("content-disposition")?.split("filename=")?.[1]?.replace(/"/g, "") ??
    `defaulters_report_${new Date().toISOString().replace(/[:.]/g, "-")}.xlsx`;

  const link = document.createElement("a");
  link.href = window.URL.createObjectURL(blob);
  link.download = filename;
  link.style.display = "none";
  document.body.appendChild(link);
  link.click();
  link.remove();
}

export interface FacultyStats {
  specialization: string;
  academic_year: string;
  total_students: number;
  with_leetcode_profiles: number;
  without_leetcode_profiles: number;
  defaulters: number;
}

export async function fetchFacultyStats(token: string): Promise<FacultyStats | FacultyStats[]> {
  return await request<FacultyStats | FacultyStats[]>("/faculty/stats", {
    method: "GET",
    token,
  });
}

export interface LeaderboardEntry {
  rank: number;
  full_name: string;
  registration_number: string;
  specialization: string | null;
  academic_year: string | null;
  github_username: string | null;
  leetcode_username: string | null;
  codechef_username: string | null;
  codeforces_username: string | null;
  linkedin_url: string | null;
  total_solved_last_30_days: number;
}

export interface LeaderboardResponse {
  academic_year: string;
  total_students: number;
  leaderboard: LeaderboardEntry[];
}

export async function fetchLeaderboard(academicYear: "I" | "II" | "III" | "IV"): Promise<LeaderboardResponse> {
  return await request<LeaderboardResponse>(`/leaderboard?academic_year=${academicYear}`, {
    method: "GET",
  });
}

export { API_BASE_URL };
