import { createContext, useContext, useState, useEffect, type ReactNode } from "react";
import { login as apiLogin, fetchMe } from "@/lib/api";
import type { User } from "@/lib/types";

interface AuthContextType {
  user: User | null;
  token: string | null;
  login: (email: string, password: string) => Promise<void>;
  logout: () => void;
  isAuthenticated: boolean;
  isLoading: boolean;
  isEmployee: boolean;
  isManager: boolean;
  isAdmin: boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const storedToken = localStorage.getItem("auth_token");
    const storedUser = localStorage.getItem("auth_user");

    if (storedToken && storedUser) {
      try {
        const parsedUser = JSON.parse(storedUser) as User;
        // Validate stored user has a valid role from the new system
        const validRoles = ["employee", "manager", "admin"];
        if (parsedUser && validRoles.includes(parsedUser.role)) {
          setToken(storedToken);
          setUser(parsedUser);
        } else {
          // Stale data from old app version or invalid format — clear it
          localStorage.removeItem("auth_token");
          localStorage.removeItem("auth_user");
        }
      } catch {
        localStorage.removeItem("auth_token");
        localStorage.removeItem("auth_user");
      }
    } else {
      // No stored data — clear any partial/invalid keys
      localStorage.removeItem("auth_token");
      localStorage.removeItem("auth_user");
    }
    setIsLoading(false);
  }, []);

  const login = async (email: string, password: string) => {
    const data = await apiLogin({ email, password });

    if (!data.token) {
      throw new Error("Login failed: token missing in response");
    }

    setToken(data.token);
    setUser(data.user);
    localStorage.setItem("auth_token", data.token);
    localStorage.setItem("auth_user", JSON.stringify(data.user));
  };

  const logout = () => {
    setUser(null);
    setToken(null);
    localStorage.removeItem("auth_token");
    localStorage.removeItem("auth_user");
  };

  const isEmployee = user?.role === "employee";
  const isManager = user?.role === "manager";
  const isAdmin = user?.role === "admin";

  return (
    <AuthContext.Provider
      value={{
        user,
        token,
        login,
        logout,
        isAuthenticated: !!user,
        isLoading,
        isEmployee,
        isManager,
        isAdmin,
      }}
    >
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
