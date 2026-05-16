import { Toaster } from "@/components/ui/toaster";
import { Toaster as Sonner } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { AuthProvider } from "@/contexts/AuthContext";
import { ThemeProvider } from "@/contexts/ThemeProvider";

import LoginPage from "./pages/LoginPage";
import AzureCallbackPage from "./pages/AzureCallbackPage";
import EmployeeDashboard from "./pages/EmployeeDashboard";
import GoalSheetEditor from "./pages/GoalSheetEditor";
import AchievementEntry from "./pages/AchievementEntry";
import ManagerDashboard from "./pages/ManagerDashboard";
import TeamGoalReview from "./pages/TeamGoalReview";
import CheckinView from "./pages/CheckinView";
import AdminDashboard from "./pages/AdminDashboard";
import AdminUsers from "./pages/AdminUsers";
import AdminAuditLog from "./pages/AdminAuditLog";
import ReportsPage from "./pages/ReportsPage";
import ForgotPasswordPage from "./pages/ForgotPasswordPage";
import ResetPasswordPage from "./pages/ResetPasswordPage";
import NotFound from "./pages/NotFound";
import { RoleGuard } from "./components/RoleGuard";

const queryClient = new QueryClient();

const ResetPasswordRedirect = () => {
  const searchParams = new URLSearchParams(window.location.search);
  const token = searchParams.get("token");
  return <Navigate to={`/reset-password?token=${token || ""}`} replace />;
};

const App = () => (
  <ThemeProvider attribute="class" defaultTheme="dark" enableSystem>
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <TooltipProvider>
          <Toaster />
          <Sonner />
          <BrowserRouter>
            <Routes>
              {/* Public routes */}
              <Route path="/login" element={<LoginPage />} />
              <Route path="/auth/azure/callback" element={<AzureCallbackPage />} />
              <Route path="/forgot-password" element={<ForgotPasswordPage />} />
              <Route path="/reset-password" element={<ResetPasswordPage />} />
              <Route path="/auth/reset-password-form" element={<ResetPasswordRedirect />} />

              {/* Employee routes */}
              <Route path="/employee" element={<RoleGuard allowedRoles={["employee", "manager", "admin"]}><EmployeeDashboard /></RoleGuard>} />
              <Route path="/employee/goals/:sheetId" element={<RoleGuard allowedRoles={["employee", "manager", "admin"]}><GoalSheetEditor /></RoleGuard>} />
              <Route path="/employee/achievements/:sheetId" element={<RoleGuard allowedRoles={["employee", "manager", "admin"]}><AchievementEntry /></RoleGuard>} />

              {/* Manager routes */}
              <Route path="/manager" element={<RoleGuard allowedRoles={["manager", "admin"]}><ManagerDashboard /></RoleGuard>} />
              <Route path="/manager/review/:sheetId" element={<RoleGuard allowedRoles={["manager", "admin"]}><TeamGoalReview /></RoleGuard>} />
              <Route path="/manager/checkin/:sheetId" element={<RoleGuard allowedRoles={["manager", "admin"]}><CheckinView /></RoleGuard>} />

              {/* Admin routes */}
              <Route path="/admin" element={<RoleGuard allowedRoles={["admin"]}><AdminDashboard /></RoleGuard>} />
              <Route path="/admin/users" element={<RoleGuard allowedRoles={["admin"]}><AdminUsers /></RoleGuard>} />
              <Route path="/admin/audit" element={<RoleGuard allowedRoles={["admin"]}><AdminAuditLog /></RoleGuard>} />

              {/* Reports (manager + admin) */}
              <Route path="/reports" element={<RoleGuard allowedRoles={["manager", "admin"]}><ReportsPage /></RoleGuard>} />

              {/* Default redirect */}
              <Route path="/" element={<Navigate to="/login" replace />} />
              <Route path="*" element={<NotFound />} />
            </Routes>
          </BrowserRouter>
        </TooltipProvider>
      </AuthProvider>
    </QueryClientProvider>
  </ThemeProvider>
);

export default App;
