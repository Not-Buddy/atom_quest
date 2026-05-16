import { Toaster } from "@/components/ui/toaster";
import { Toaster as Sonner } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { AuthProvider } from "@/contexts/AuthContext";
import { ThemeProvider } from "@/contexts/ThemeProvider";
import LandingPage from "./pages/LandingPage";
import LeaderboardPage from "./pages/LeaderboardPage";
import FacultyLoginPage from "./pages/FacultyLoginPage";
import FacultyDashboard from "./pages/FacultyDashboard";
import StudentLoginPage from "./pages/StudentLoginPage";
import StudentDashboard from "./pages/StudentDashboard";
import ForgotPasswordPage from "./pages/ForgotPasswordPage";
import ResetPasswordPage from "./pages/ResetPasswordPage";
import NotFound from "./pages/NotFound";

const queryClient = new QueryClient();

// Redirect component for backend email links
const ResetPasswordRedirect = () => {
  const searchParams = new URLSearchParams(window.location.search);
  const token = searchParams.get("token");
  return <Navigate to={`/reset-password?token=${token || ""}`} replace />;
};

const App = () => (
  <ThemeProvider attribute="class" defaultTheme="light" enableSystem>
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <TooltipProvider>
          <Toaster />
          <Sonner />
          <BrowserRouter>
            <Routes>
              <Route path="/" element={<LandingPage />} />
              <Route path="/leaderboard" element={<LeaderboardPage />} />
              <Route path="/faculty/login" element={<FacultyLoginPage />} />
              <Route path="/faculty/dashboard" element={<FacultyDashboard />} />
              <Route path="/student/login" element={<StudentLoginPage />} />
              <Route path="/student/dashboard" element={<StudentDashboard />} />
              <Route path="/forgot-password" element={<ForgotPasswordPage />} />
              <Route path="/reset-password" element={<ResetPasswordPage />} />
              <Route path="/auth/reset-password-form" element={<ResetPasswordRedirect />} />
              <Route path="*" element={<NotFound />} />
            </Routes>
          </BrowserRouter>
        </TooltipProvider>
      </AuthProvider>
    </QueryClientProvider>
  </ThemeProvider>
);

export default App;
