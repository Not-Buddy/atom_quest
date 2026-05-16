import { useState } from "react";
import { useNavigate, Link } from "react-router-dom";
import { Code2, LogIn, ArrowLeft, Lock, User, Building2, Trophy } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useAuth } from "@/contexts/AuthContext";
import { departments } from "@/lib/mockData";
import { toast } from "sonner";
import { ThemeToggle } from "@/components/ThemeToggle";

export default function FacultyLoginPage() {
  const navigate = useNavigate();
  const { login } = useAuth();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [departmentId, setDepartmentId] = useState("");
  const [isLoading, setIsLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!username || !password || !departmentId) {
      toast.error("Please fill in all fields");
      return;
    }

    const selectedDept = departments.find((dept) => dept.id === departmentId);
    const specialization = selectedDept?.name || departmentId;

    setIsLoading(true);
    
    // Add 2 second delay
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    try {
      const success = await login(username, password, specialization);
      
      if (success) {
        toast.success("Login successful!");
        navigate("/faculty/dashboard");
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
              <h1 className="font-bold text-sm sm:text-base md:text-lg text-foreground">Faculty Portal</h1>
              <p className="text-xs text-muted-foreground hidden sm:block">Login to dashboard</p>
            </div>
          </div>
          <div className="flex items-center gap-1 sm:gap-2 md:gap-3">
            <ThemeToggle />
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
            <CardTitle className="text-xl sm:text-2xl">Faculty Login</CardTitle>
            <CardDescription className="text-sm">
              Access your department dashboard to monitor student progress
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-4 sm:space-y-5">
              <div className="space-y-2">
                <Label htmlFor="department" className="flex items-center gap-2 text-sm">
                  <Building2 className="h-3.5 w-3.5 sm:h-4 sm:w-4 text-muted-foreground" />
                  Department
                </Label>
                <Select value={departmentId} onValueChange={setDepartmentId}>
                  <SelectTrigger id="department" className="h-10 sm:h-11">
                    <SelectValue placeholder="Select your department" />
                  </SelectTrigger>
                  <SelectContent>
                    {departments.map((dept) => (
                      <SelectItem key={dept.id} value={dept.id}>
                        {dept.shortName}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-2">
                <Label htmlFor="username" className="flex items-center gap-2 text-sm">
                  <User className="h-3.5 w-3.5 sm:h-4 sm:w-4 text-muted-foreground" />
                  Username
                </Label>
                <Input
                  id="username"
                  type="text"
                  placeholder="Enter your username"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  className="h-10 sm:h-11"
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="password" className="flex items-center gap-2 text-sm">
                  <Lock className="h-3.5 w-3.5 sm:h-4 sm:w-4 text-muted-foreground" />
                  Password
                </Label>
                <Input
                  id="password"
                  type="password"
                  placeholder="Enter your password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="h-10 sm:h-11"
                />
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
