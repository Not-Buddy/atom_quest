import { createContext, useContext, useState, useEffect, ReactNode } from "react";
import { departments, Department } from "@/lib/mockData";
import { facultyLogin, studentLogin } from "@/lib/api";

interface BaseUser {
  user_id: string;
  name: string;
  role: string;
  email?: string;
}

interface FacultyUser extends BaseUser {
  type: "faculty";
  specialization?: string;
  department?: Department;
  username?: string;
}

interface StudentUser extends BaseUser {
  type: "student";
  ra_number?: string;
}

type User = FacultyUser | StudentUser;

interface AuthContextType {
  user: User | null;
  token: string | null;
  login: (username: string, password: string, specialization: string) => Promise<boolean>;
  loginStudent: (email: string, password: string) => Promise<boolean>;
  logout: () => void;
  isAuthenticated: boolean;
  isLoading: boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  // Load token and user from localStorage on mount
  useEffect(() => {
    const storedToken = localStorage.getItem("auth_token");
    const storedUser = localStorage.getItem("auth_user");
    
    if (storedToken && storedUser) {
      setToken(storedToken);
      setUser(JSON.parse(storedUser));
    }
    setIsLoading(false);
  }, []);

  const persistSession = (sessionToken: string, sessionUser: User) => {
    setUser(sessionUser);
    setToken(sessionToken);
    localStorage.setItem("auth_token", sessionToken);
    localStorage.setItem("auth_user", JSON.stringify(sessionUser));
  };

  const login = async (username: string, password: string, specialization: string): Promise<boolean> => {
    try {
      const data = await facultyLogin({ specialization, username, password });
      
      // Check if data is a Response object (which shouldn't happen, but handle it)
      // Response objects have specific properties like 'ok', 'status', 'type', 'url', etc.
      if (data && typeof data === "object" && (
        "bodyUsed" in data || 
        ("ok" in data && "status" in data && "type" in data && "url" in data)
      )) {
        throw new Error("Login failed: Server response could not be parsed. The API may be returning an invalid response format. Please check server logs.");
      }
      
      const tokenFromApi = (data as { token?: string }).token;
      const facultyPayload = (data as { faculty?: Record<string, unknown>; user?: Record<string, unknown> }).faculty
        ?? (data as { faculty?: Record<string, unknown>; user?: Record<string, unknown> }).user
        ?? {};

      if (!tokenFromApi) {
        throw new Error("Login failed: token missing in response");
      }

      const matchedDept = departments.find((dept) => dept.name === specialization || dept.id === specialization || dept.shortName === specialization);
      const facultyUser: FacultyUser = {
        user_id: (facultyPayload.id as string) || (facultyPayload.user_id as string) || username,
        name: (facultyPayload.name as string) || (facultyPayload.username as string) || username,
        email: (facultyPayload.email as string | undefined) || undefined,
        role: "faculty",
        type: "faculty",
        specialization,
        department: matchedDept,
        username,
      };

      persistSession(tokenFromApi, facultyUser);
      return true;
    } catch (error) {
      if (error instanceof TypeError && error.message === "Failed to fetch") {
        throw new Error("Unable to connect to server. Please check your internet connection or try again later.");
      }
      throw error;
    }
  };

  const loginStudent = async (email: string, password: string): Promise<boolean> => {
    try {
      const data = await studentLogin({ email, password });
      
      // Extract token from response - handle multiple formats
      let tokenFromApi: string | undefined;
      let studentPayload: Record<string, unknown> = {};

      // Check if data is a Response object (which shouldn't happen, but handle it)
      // Response objects have specific properties like 'ok', 'status', 'type', 'url', etc.
      if (data && typeof data === "object" && (
        "bodyUsed" in data || 
        ("ok" in data && "status" in data && "type" in data && "url" in data)
      )) {
        throw new Error("Login failed: Server response could not be parsed. The API may be returning an invalid response format. Please check server logs.");
      }

      if (typeof data === "object" && data !== null) {
        const dataObj = data as Record<string, unknown>;
        tokenFromApi = (dataObj.token as string | undefined) 
          || (dataObj.data as any)?.token as string | undefined;
        studentPayload = (dataObj.student as Record<string, unknown>) 
          || (dataObj.user as Record<string, unknown>)
          || dataObj.data as Record<string, unknown>
          || {};
      }

      if (!tokenFromApi) {
        throw new Error("Login failed: token missing in response");
      }

      const studentUser: StudentUser = {
        user_id: (studentPayload.registration_number as string) || (studentPayload.ra_number as string) || (studentPayload.user_id as string) || email,
        ra_number: (studentPayload.registration_number as string) || (studentPayload.ra_number as string) || undefined,
        name: (studentPayload.full_name as string) || (studentPayload.student_name as string) || (studentPayload.name as string) || email,
        email: (studentPayload.email as string | undefined) || email,
        role: "student",
        type: "student",
      };

      persistSession(tokenFromApi, studentUser);
      return true;
    } catch (error) {
      if (error instanceof TypeError && error.message === "Failed to fetch") {
        throw new Error("Unable to connect to server. Please check your internet connection or try again later.");
      }
      throw error;
    }
  };

  const logout = () => {
    setUser(null);
    setToken(null);
    localStorage.removeItem("auth_token");
    localStorage.removeItem("auth_user");
  };

  return (
    <AuthContext.Provider value={{ user, token, login, loginStudent, logout, isAuthenticated: !!user, isLoading }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
}
