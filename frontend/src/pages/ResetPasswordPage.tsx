import { useState, useEffect } from "react";
import { useNavigate, useSearchParams, Link } from "react-router-dom";
import { ArrowLeft, Lock, CheckCircle, Eye, EyeOff, AlertCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { toast } from "sonner";
import { ThemeToggle } from "@/components/ThemeToggle";
import { resetPassword } from "@/lib/api";

export default function ResetPasswordPage() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const token = searchParams.get("token");

  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [showNewPassword, setShowNewPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [isSuccess, setIsSuccess] = useState(false);

  const passwordsMatch = newPassword === confirmPassword;
  const isPasswordValid = newPassword.length >= 8;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!token) {
      toast.error("Invalid or missing reset token");
      return;
    }

    if (!newPassword || !confirmPassword) {
      toast.error("Please fill in all fields");
      return;
    }

    if (!isPasswordValid) {
      toast.error("Password must be at least 8 characters long");
      return;
    }

    if (!passwordsMatch) {
      toast.error("Passwords do not match");
      return;
    }

    setIsLoading(true);

    try {
      await resetPassword(token, { new_password: newPassword });
      setIsSuccess(true);
      toast.success("Password reset successfully");
    } catch (error) {
      const errorMessage =
        error instanceof Error
          ? error.message
          : "Failed to reset password. Please try again.";
      toast.error(errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  if (!token) {
    return (
      <div className="min-h-screen bg-background flex flex-col">
        {/* Header */}
        <header className="border-b bg-card/50 backdrop-blur-sm">
          <div className="container mx-auto px-3 sm:px-4 py-3 sm:py-4 flex items-center justify-between">
            <div className="flex items-center gap-2 sm:gap-4">
              <Link to="/student/login">
                <Button variant="ghost" size="icon" className="rounded-full h-9 w-9">
                  <ArrowLeft className="h-4 w-4 sm:h-5 sm:w-5" />
                </Button>
              </Link>
              <img src="/logo.svg" alt="SRM Logo" className="h-16 w-16 object-contain -my-2" />
              <div>
                <h1 className="font-bold text-sm sm:text-base md:text-lg text-foreground">
                  Reset Password
                </h1>
              </div>
            </div>
            <ThemeToggle />
          </div>
        </header>

        {/* Invalid Token */}
        <div className="flex-1 flex items-center justify-center p-3 sm:p-4">
          <Card className="w-full max-w-md">
            <CardHeader className="text-center space-y-3 sm:space-y-4">
              <div className="w-14 h-14 sm:w-16 sm:h-16 rounded-xl sm:rounded-2xl flex items-center justify-center mx-auto bg-destructive">
                <AlertCircle className="h-7 w-7 sm:h-8 sm:w-8 text-destructive-foreground" />
              </div>
              <CardTitle className="text-xl sm:text-2xl">Invalid Link</CardTitle>
              <CardDescription className="text-sm">
                This password reset link is invalid or has expired. Please request a new one.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="flex flex-col gap-2">
                <Link to="/forgot-password" className="w-full">
                  <Button variant="default" className="w-full">
                    Request New Link
                  </Button>
                </Link>
                <Link to="/student/login" className="w-full">
                  <Button variant="outline" className="w-full">
                    Back to Login
                  </Button>
                </Link>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background flex flex-col">
      {/* Header */}
      <header className="border-b bg-card/50 backdrop-blur-sm">
        <div className="container mx-auto px-3 sm:px-4 py-3 sm:py-4 flex items-center justify-between">
          <div className="flex items-center gap-2 sm:gap-4">
            <Link to="/student/login">
              <Button variant="ghost" size="icon" className="rounded-full h-9 w-9">
                <ArrowLeft className="h-4 w-4 sm:h-5 sm:w-5" />
              </Button>
            </Link>
            <img src="/logo.svg" alt="SRM Logo" className="h-16 w-16 object-contain -my-2" />
            <div>
              <h1 className="font-bold text-sm sm:text-base md:text-lg text-foreground">
                Reset Password
              </h1>
              <p className="text-xs text-muted-foreground hidden sm:block">
                Create a new password
              </p>
            </div>
          </div>
          <ThemeToggle />
        </div>
      </header>

      {/* Form */}
      <div className="flex-1 flex items-center justify-center p-3 sm:p-4">
        <Card className="w-full max-w-md">
          <CardHeader className="text-center space-y-3 sm:space-y-4">
            <div
              className={`w-14 h-14 sm:w-16 sm:h-16 rounded-xl sm:rounded-2xl flex items-center justify-center mx-auto ${
                isSuccess ? "bg-green-500" : "gradient-primary"
              }`}
            >
              {isSuccess ? (
                <CheckCircle className="h-7 w-7 sm:h-8 sm:w-8 text-white" />
              ) : (
                <Lock className="h-7 w-7 sm:h-8 sm:w-8 text-primary-foreground" />
              )}
            </div>
            <CardTitle className="text-xl sm:text-2xl">
              {isSuccess ? "Password Reset!" : "Reset Password"}
            </CardTitle>
            <CardDescription className="text-sm">
              {isSuccess
                ? "Your password has been successfully reset. You can now log in with your new password."
                : "Enter your new password below."}
            </CardDescription>
          </CardHeader>
          <CardContent>
            {isSuccess ? (
              <Link to="/student/login" className="w-full">
                <Button variant="default" className="w-full">
                  Go to Login
                </Button>
              </Link>
            ) : (
              <form onSubmit={handleSubmit} className="space-y-4 sm:space-y-5">
                <div className="space-y-2">
                  <Label htmlFor="newPassword" className="flex items-center gap-2 text-sm">
                    <Lock className="h-3.5 w-3.5 sm:h-4 sm:w-4 text-muted-foreground" />
                    New Password
                  </Label>
                  <div className="relative">
                    <Input
                      id="newPassword"
                      type={showNewPassword ? "text" : "password"}
                      placeholder="Enter new password"
                      value={newPassword}
                      onChange={(e) => setNewPassword(e.target.value)}
                      className="h-10 sm:h-11 pr-10"
                      required
                      disabled={isLoading}
                      minLength={8}
                    />
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon"
                      className="absolute right-0 top-0 h-10 sm:h-11 w-10 hover:bg-transparent"
                      onClick={() => setShowNewPassword(!showNewPassword)}
                    >
                      {showNewPassword ? (
                        <EyeOff className="h-4 w-4 text-muted-foreground" />
                      ) : (
                        <Eye className="h-4 w-4 text-muted-foreground" />
                      )}
                    </Button>
                  </div>
                  {newPassword && !isPasswordValid && (
                    <p className="text-xs text-destructive">
                      Password must be at least 8 characters
                    </p>
                  )}
                </div>

                <div className="space-y-2">
                  <Label htmlFor="confirmPassword" className="flex items-center gap-2 text-sm">
                    <Lock className="h-3.5 w-3.5 sm:h-4 sm:w-4 text-muted-foreground" />
                    Confirm New Password
                  </Label>
                  <div className="relative">
                    <Input
                      id="confirmPassword"
                      type={showConfirmPassword ? "text" : "password"}
                      placeholder="Confirm new password"
                      value={confirmPassword}
                      onChange={(e) => setConfirmPassword(e.target.value)}
                      className="h-10 sm:h-11 pr-10"
                      required
                      disabled={isLoading}
                    />
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon"
                      className="absolute right-0 top-0 h-10 sm:h-11 w-10 hover:bg-transparent"
                      onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                    >
                      {showConfirmPassword ? (
                        <EyeOff className="h-4 w-4 text-muted-foreground" />
                      ) : (
                        <Eye className="h-4 w-4 text-muted-foreground" />
                      )}
                    </Button>
                  </div>
                  {confirmPassword && !passwordsMatch && (
                    <p className="text-xs text-destructive">Passwords do not match</p>
                  )}
                </div>

                <Button
                  type="submit"
                  className="w-full h-10 sm:h-11"
                  size="lg"
                  disabled={isLoading || !passwordsMatch || !isPasswordValid}
                >
                  {isLoading ? (
                    <span className="flex items-center gap-2">
                      <span className="h-4 w-4 border-2 border-primary-foreground border-t-transparent rounded-full animate-spin" />
                      Resetting...
                    </span>
                  ) : (
                    <span className="flex items-center gap-2">
                      <Lock className="h-4 w-4" />
                      Reset Password
                    </span>
                  )}
                </Button>
              </form>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
