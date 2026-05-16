import { useState } from "react";
import { useNavigate, Link } from "react-router-dom";
import { Code2, LogIn, ArrowLeft, Lock, Hash, Trophy, Mail, Eye, EyeOff, MessageCircle, MessageSquare } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useAuth } from "@/contexts/AuthContext";
import { toast } from "sonner";
import { ThemeToggle } from "@/components/ThemeToggle";

export default function StudentLoginPage() {
  const navigate = useNavigate();
  const { loginStudent } = useAuth();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!email || !password) {
      toast.error("Please fill in all required fields");
      return;
    }

    setIsLoading(true);
    
    try {
      const success = await loginStudent(email, password);
      
      if (success) {
        toast.success("Login successful!");
        navigate("/student/dashboard");
      } else {
        toast.error("Invalid credentials. Please try again.");
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : "An error occurred during login. Please try again.";
      toast.error(errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-background flex flex-col">
      {/* Header */}
      <header className="border-b bg-card/50 backdrop-blur-sm">
        <div className="container mx-auto px-3 sm:px-4 py-3 sm:py-4 flex items-center justify-between">
          <div className="flex items-center gap-2 sm:gap-4">
            <Link to="/">
              <Button variant="ghost" size="icon" className="rounded-full h-9 w-9">
                <ArrowLeft className="h-4 w-4 sm:h-5 sm:w-5" />
              </Button>
            </Link>
            <img src="/logo.svg" alt="SRM Logo" className="h-16 w-16 object-contain -my-2" />
            <div>
              <h1 className="font-bold text-sm sm:text-base md:text-lg text-foreground">Student Portal</h1>
              <p className="text-xs text-muted-foreground hidden sm:block">Login to your dashboard</p>
            </div>
          </div>
          <div className="flex items-center gap-1 sm:gap-2 md:gap-3">
            <ThemeToggle />
            <a href="https://forms.gle/EYvRCHFSTx3845Fe7" target="_blank" rel="noopener noreferrer">
              <Button variant="ghost" size="sm" className="text-xs sm:text-sm h-9 px-2 sm:px-3">
                <MessageSquare className="h-3.5 w-3.5 sm:mr-1" />
                <span className="hidden sm:inline">Help</span>
              </Button>
            </a>
            <a href="https://whatsapp.com/channel/0029VbBX2gIDp2QHHGlzM31J" target="_blank" rel="noopener noreferrer">
              <Button variant="default" size="sm" className="text-xs sm:text-sm h-9 px-2 sm:px-3 bg-green-600 hover:bg-green-700">
                <MessageCircle className="h-3.5 w-3.5 sm:mr-1" />
                <span className="hidden sm:inline">Placements</span>
              </Button>
            </a>
            <Link to="/leaderboard">
              <Button variant="outline" size="sm" className="text-xs sm:text-sm h-9 px-2 sm:px-3">
                <Trophy className="h-3.5 w-3.5 sm:mr-2" />
                <span className="hidden sm:inline">Leaderboard</span>
              </Button>
            </Link>
          </div>
        </div>
      </header>

      {/* Login Form */}
      <div className="flex-1 flex items-center justify-center p-3 sm:p-4">
        <Card className="w-full max-w-md">
          <CardHeader className="text-center space-y-3 sm:space-y-4">
            <div className="gradient-primary w-14 h-14 sm:w-16 sm:h-16 rounded-xl sm:rounded-2xl flex items-center justify-center mx-auto">
              <Lock className="h-7 w-7 sm:h-8 sm:w-8 text-primary-foreground" />
            </div>
            <CardTitle className="text-xl sm:text-2xl">Student Login</CardTitle>
            <CardDescription className="text-sm">
              Access your dashboard to manage your profile URLs
            </CardDescription>
            <div className="bg-muted/50 border border-muted-foreground/20 rounded-lg p-3 text-left">
              <p className="text-xs text-muted-foreground">
                <span className="font-semibold text-foreground">First time logging in?</span><br />
                Use your Register Number (RA Number) as your password.
              </p>
            </div>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-4 sm:space-y-5">
              <div className="space-y-2">
                <Label htmlFor="email" className="flex items-center gap-2 text-sm">
                  <Mail className="h-3.5 w-3.5 sm:h-4 sm:w-4 text-muted-foreground" />
                  SRM Email
                </Label>
                <Input
                  id="email"
                  type="email"
                  placeholder="netid@srmist.edu.in"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  className="h-10 sm:h-11"
                  required
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="password" className="flex items-center gap-2 text-sm">
                  <Lock className="h-3.5 w-3.5 sm:h-4 sm:w-4 text-muted-foreground" />
                  Password
                </Label>
                <div className="relative">
                  <Input
                    id="password"
                    type={showPassword ? "text" : "password"}
                    placeholder="Enter your password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    className="h-10 sm:h-11 pr-10"
                    required
                  />
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    className="absolute right-0 top-0 h-10 sm:h-11 w-10 hover:bg-transparent"
                    onClick={() => setShowPassword(!showPassword)}
                  >
                    {showPassword ? (
                      <EyeOff className="h-4 w-4 text-muted-foreground" />
                    ) : (
                      <Eye className="h-4 w-4 text-muted-foreground" />
                    )}
                  </Button>
                </div>
                <div className="flex justify-end">
                  <Link
                    to="/forgot-password"
                    className="text-xs text-primary hover:underline"
                  >
                    Forgot password?
                  </Link>
                </div>
              </div>

              <Button type="submit" className="w-full h-10 sm:h-11" size="lg" disabled={isLoading}>
                {isLoading ? (
                  <span className="flex items-center gap-2">
                    <span className="h-4 w-4 border-2 border-primary-foreground border-t-transparent rounded-full animate-spin" />
                    Signing in...
                  </span>
                ) : (
                  <span className="flex items-center gap-2">
                    <LogIn className="h-4 w-4" />
                    Sign In
                  </span>
                )}
              </Button>
            </form>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
