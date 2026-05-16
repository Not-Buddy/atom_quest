import { Link, useNavigate } from "react-router-dom";
import { Code2, Trophy, BarChart3, GraduationCap, ChevronRight, Users, Target, TrendingUp, MessageSquare, MessageCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { ThemeToggle } from "@/components/ThemeToggle";
import { useAuth } from "@/contexts/AuthContext";

export default function LandingPage() {
  const navigate = useNavigate();
  const { user } = useAuth();

  const handleDashboardClick = () => {
    if (user?.type === "student") {
      navigate("/student/dashboard");
    } else if (user?.type === "faculty") {
      navigate("/faculty/dashboard");
    }
  };

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b bg-card/50 backdrop-blur-sm sticky top-0 z-50">
        <div className="container mx-auto px-3 sm:px-4 py-3 flex items-center justify-between">
          <div className="flex items-center gap-2 sm:gap-3">
            <img src="/logo.svg" alt="SRM Logo" className="h-16 w-16 object-contain -my-2" />
            <div className="hidden sm:block">
              <h1 className="font-bold text-base sm:text-lg text-foreground">LeetCode Tracker</h1>
              <p className="text-xs text-muted-foreground hidden md:block">Made for SRM Ramapuram</p>
            </div>
            <h1 className="font-bold text-sm sm:hidden text-foreground">LeetCode</h1>
          </div>
          <nav className="flex items-center gap-1 sm:gap-2 md:gap-4">
            <a href="https://forms.gle/EYvRCHFSTx3845Fe7" target="_blank" rel="noopener noreferrer">
              <Button variant="ghost" size="sm" className="text-xs sm:text-sm px-2 sm:px-3">
                <MessageSquare className="h-4 w-4 mr-1" />
                <span className="hidden sm:inline">Help</span>
              </Button>
            </a>
            <a href="https://whatsapp.com/channel/0029VbBX2gIDp2QHHGlzM31J" target="_blank" rel="noopener noreferrer">
              <Button variant="default" size="sm" className="text-xs sm:text-sm px-2 sm:px-3 bg-green-600 hover:bg-green-700">
                <MessageCircle className="h-4 w-4 mr-1" />
                <span className="hidden sm:inline">Placements</span>
              </Button>
            </a>
            <Link to="/leaderboard">
              <Button variant="ghost" size="sm" className="hidden md:flex">Leaderboard</Button>
              <Button variant="ghost" size="icon" className="md:hidden h-9 w-9">
                <Trophy className="h-4 w-4" />
              </Button>
            </Link>
            {user ? (
              <Button variant="ghost" size="sm" className="text-xs sm:text-sm px-2 sm:px-3" onClick={handleDashboardClick}>
                Go to Dashboard
              </Button>
            ) : (
              <Link to="/student/login">
                <Button variant="ghost" size="sm" className="text-xs sm:text-sm px-2 sm:px-3">Student</Button>
              </Link>
            )}
            <ThemeToggle />
          </nav>
        </div>
      </header>

      {/* Hero Section */}
      <section className="gradient-hero text-primary-foreground py-12 sm:py-16 md:py-24 relative overflow-hidden">
        <div className="absolute inset-0 bg-[url('data:image/svg+xml,%3Csvg%20width%3D%2260%22%20height%3D%2260%22%20viewBox%3D%220%200%2060%2060%22%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%3E%3Cg%20fill%3D%22none%22%20fill-rule%3D%22evenodd%22%3E%3Cg%20fill%3D%22%23ffffff%22%20fill-opacity%3D%220.05%22%3E%3Cpath%20d%3D%22M36%2034v-4h-2v4h-4v2h4v4h2v-4h4v-2h-4zm0-30V0h-2v4h-4v2h4v4h2V6h4V4h-4zM6%2034v-4H4v4H0v2h4v4h2v-4h4v-2H6zM6%204V0H4v4H0v2h4v4h2V6h4V4H6z%22%2F%3E%3C%2Fg%3E%3C%2Fg%3E%3C%2Fsvg%3E')] opacity-50" />
        
        <div className="container mx-auto px-4 relative">
          <div className="max-w-3xl mx-auto text-center">
            <div className="inline-flex items-center gap-2 bg-primary-foreground/10 backdrop-blur-sm px-3 py-1.5 sm:px-4 sm:py-2 rounded-full mb-4 sm:mb-6 animate-fade-in">
              <GraduationCap className="h-3.5 w-3.5 sm:h-4 sm:w-4" />
              <span className="text-xs sm:text-sm font-medium">SRM LeetCode Monitoring</span>
            </div>
            
            <h1 className="text-3xl sm:text-4xl md:text-5xl lg:text-6xl font-extrabold mb-4 sm:mb-6 animate-slide-up leading-tight">
              Track Your Problem
              <span className="block mt-1 sm:mt-2">Solving Journey :)</span>
            </h1>
            
            <p className="text-sm sm:text-base md:text-lg lg:text-xl opacity-90 mb-6 sm:mb-8 animate-slide-up px-2" style={{ animationDelay: "0.1s" }}>
              Monitor your LeetCode progress, compete with peers, and achieve monthly targets legally.
            </p>
            
            <div className="flex flex-col sm:flex-row gap-3 sm:gap-4 justify-center animate-slide-up px-4 sm:px-0" style={{ animationDelay: "0.2s" }}>
              <Link to="/leaderboard" className="w-full sm:w-auto">
                <Button size="lg" className="w-full sm:w-auto bg-white text-primary hover:bg-white/90 shadow-lg h-11 sm:h-12">
                  <Trophy className="h-4 w-4 sm:h-5 sm:w-5" />
                  View Leaderboard
                  <ChevronRight className="h-4 w-4 sm:h-5 sm:w-5" />
                </Button>
              </Link>
              {user ? (
                <Button variant="outline" size="lg" className="w-full sm:w-auto border-white/30 text-white hover:bg-white/10 bg-transparent h-11 sm:h-12" onClick={handleDashboardClick}>
                  Go to Dashboard
                </Button>
              ) : (
                <Link to="/student/login" className="w-full sm:w-auto">
                  <Button variant="outline" size="lg" className="w-full sm:w-auto border-white/30 text-white hover:bg-white/10 bg-transparent h-11 sm:h-12">
                    Student Login
                  </Button>
                </Link>
              )}
            </div>
          </div>
        </div>
      </section>

      {/* Why LeetCode Section */}
      <section className="py-12 sm:py-16 md:py-20 bg-muted/50">
        <div className="container mx-auto px-4">
          <div className="text-center mb-8 sm:mb-12">
            <h2 className="text-2xl sm:text-3xl md:text-4xl font-bold text-foreground mb-3 sm:mb-4">
              Why LeetCode Progress Matters
            </h2>
            <p className="text-sm sm:text-base text-muted-foreground max-w-2xl mx-auto px-4">
              Regular practice on LeetCode builds the foundation for technical interviews and strengthens problem solving skills.
            </p>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6 max-w-5xl mx-auto">
            {[
              {
                icon: Code2,
                title: "Interview Readiness",
                description: "Top tech companies use LeetCode-style problems in their interviews. Regular practice prepares you for success.",
              },
              {
                icon: BarChart3,
                title: "Structured Progress",
                description: "Monthly targets of 15 problems ensure consistent growth. Track your cumulative achievements over time.",
              },
              {
                icon: Trophy,
                title: "Healthy Competition",
                description: "Compete with peers in your year and department. Leaderboards motivate continuous improvement.",
              },
            ].map((item, i) => (
              <Card key={i} className="group hover:scale-[1.02] transition-transform duration-300">
                <CardContent className="pt-5 sm:pt-6">
                  <div className="w-11 h-11 sm:w-12 sm:h-12 rounded-lg bg-primary/10 flex items-center justify-center mb-3 sm:mb-4 group-hover:bg-primary/20 transition-colors">
                    <item.icon className="h-5 w-5 sm:h-6 sm:w-6 text-primary" />
                  </div>
                  <h3 className="text-base sm:text-lg font-semibold text-foreground mb-2">{item.title}</h3>
                  <p className="text-muted-foreground text-xs sm:text-sm">{item.description}</p>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      </section>

      {/* How It Works */}
      <section className="py-12 sm:py-16 md:py-20">
        <div className="container mx-auto px-4">
          <div className="text-center mb-8 sm:mb-12">
            <h2 className="text-2xl sm:text-3xl md:text-4xl font-bold text-foreground mb-3 sm:mb-4">
              How It Works
            </h2>
            <p className="text-sm sm:text-base text-muted-foreground max-w-2xl mx-auto px-4">
              A simple system designed to track and encourage consistent problem-solving practice.
            </p>
          </div>

          <div className="max-w-4xl mx-auto">
            <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-6 sm:gap-8">
              {[
                { step: "01", title: "Create Profile", desc: "Login and submit your details" },
                { step: "02", title: "Solve Problems", desc: "Practice daily and solve 15+ problems monthly" },
                { step: "03", title: "Track Progress", desc: "Your progress is synced automatically daily" },
                { step: "04", title: "Climb Ranks", desc: "Compete with peers on the leaderboard" },
              ].map((item, i) => (
                <div key={i} className="text-center">
                  <div className="text-4xl sm:text-5xl font-extrabold text-primary/20 mb-2 sm:mb-3">{item.step}</div>
                  <h3 className="font-semibold text-base sm:text-lg text-foreground mb-1.5 sm:mb-2">{item.title}</h3>
                  <p className="text-xs sm:text-sm text-muted-foreground">{item.desc}</p>
                </div>
              ))}
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-12 sm:py-16 md:py-20 gradient-primary text-primary-foreground">
        <div className="container mx-auto px-4 text-center">
          <h2 className="text-2xl sm:text-3xl md:text-4xl font-bold mb-3 sm:mb-4">
            Ready to See the Rankings?
          </h2>
          <p className="text-sm sm:text-base md:text-lg opacity-90 mb-6 sm:mb-8 max-w-xl mx-auto px-4">
            Check out the leaderboard to see how students are progressing across all years and departments.
          </p>
          <Link to="/leaderboard">
            <Button size="lg" className="bg-white text-primary hover:bg-white/90 shadow-lg h-11 sm:h-12">
              <Trophy className="h-4 w-4 sm:h-5 sm:w-5" />
              View Public Leaderboard
              <ChevronRight className="h-4 w-4 sm:h-5 sm:w-5" />
            </Button>
          </Link>
        </div>
      </section>

      {/* Footer */}
      <footer className="py-6 sm:py-8 border-t bg-card">
        <div className="container mx-auto px-4 text-center">
          <div className="flex items-center justify-center gap-2 mb-3 sm:mb-4">
            <div className="gradient-primary p-1.5 rounded-lg">
              <Code2 className="h-4 w-4 text-primary-foreground" />
            </div>
            <span className="font-semibold text-sm sm:text-base text-foreground">LeetCode Progress Tracker</span>
          </div>
          <p className="text-xs sm:text-sm text-muted-foreground">
            Made by humans on Earth using rust
          </p>
        </div>
      </footer>
    </div>
  );
}
