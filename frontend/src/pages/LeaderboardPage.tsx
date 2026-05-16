import { useState, useEffect } from "react";
import { Link, useNavigate } from "react-router-dom";
import { Code2, Trophy, ArrowLeft, Clock, RefreshCw, Medal, Award, Crown, ExternalLink, Code, MessageCircle, MessageSquare } from "lucide-react";
import { useAuth } from "@/contexts/AuthContext";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { ThemeToggle } from "@/components/ThemeToggle";
import { fetchLeaderboard, LeaderboardResponse } from "@/lib/api";
import { toast } from "sonner";

const yearLabels: Record<string, string> = {
  "I": "1st Year",
  "II": "2nd Year",
  "III": "3rd Year",
  "IV": "4th Year",
};

function RankBadge({ rank }: { rank: number }) {
  if (rank === 1) {
    return (
      <div className="flex items-center gap-1.5">
        <Crown className="h-5 w-5 text-warning" />
        <span className="font-bold text-warning">1</span>
      </div>
    );
  }
  if (rank === 2) {
    return (
      <div className="flex items-center gap-1.5">
        <Medal className="h-5 w-5 text-muted-foreground" />
        <span className="font-bold text-muted-foreground">2</span>
      </div>
    );
  }
  if (rank === 3) {
    return (
      <div className="flex items-center gap-1.5">
        <Award className="h-5 w-5 text-accent" />
        <span className="font-bold text-accent">3</span>
      </div>
    );
  }
  return <span className="font-medium text-muted-foreground">{rank}</span>;
}

export default function LeaderboardPage() {
  const [selectedYear, setSelectedYear] = useState<"I" | "II" | "III" | "IV">("I");
  const navigate = useNavigate();
  const { user, isLoading: authLoading } = useAuth();

  const [leaderboardData, setLeaderboardData] = useState<LeaderboardResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  // Fetch leaderboard data when year changes
  useEffect(() => {
    const loadLeaderboard = async () => {
      setIsLoading(true);
      try {
        const data = await fetchLeaderboard(selectedYear);
        setLeaderboardData(data);
      } catch (error) {
        console.error("Failed to fetch leaderboard:", error);
        toast.error(error instanceof Error ? error.message : "Failed to load leaderboard");
        setLeaderboardData(null);
      } finally {
        setIsLoading(false);
      }
    };

    loadLeaderboard();
  }, [selectedYear]);

  const handleBackClick = () => {
    // Don't navigate if auth is still loading
    if (authLoading) return;
    
    if (user) {
      // Navigate to appropriate dashboard based on user type
      if (user.type === "student") {
        navigate("/student/dashboard");
      } else if (user.type === "faculty") {
        navigate("/faculty/dashboard");
      } else {
        navigate("/");
      }
    } else {
      navigate("/");
    }
  };

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b bg-card/50 backdrop-blur-sm sticky top-0 z-50">
        <div className="container mx-auto px-3 sm:px-4 py-3 flex items-center justify-between">
          <div className="flex items-center gap-2 sm:gap-4">
            <Button variant="ghost" size="icon" className="rounded-full h-9 w-9" onClick={handleBackClick}>
              <ArrowLeft className="h-4 w-4 sm:h-5 sm:w-5" />
            </Button>
            <div className="flex items-center gap-2 sm:gap-3">
              <img src="/logo.svg" alt="SRM Logo" className="h-16 w-16 object-contain -my-2" />
              <div>
                <h1 className="font-bold text-sm sm:text-base md:text-lg text-foreground">Leaderboard</h1>
                <p className="text-xs text-muted-foreground hidden sm:block">View student rankings</p>
              </div>
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
            {user ? (
              <Button variant="outline" size="sm" className="text-xs sm:text-sm h-9 px-2 sm:px-3" onClick={handleBackClick}>
                Go to Dashboard
              </Button>
            ) : (
              <Link to="/student/login">
                <Button variant="outline" size="sm" className="text-xs sm:text-sm h-9 px-2 sm:px-3">
                  Student Login
                </Button>
              </Link>
            )}
          </div>
        </div>
      </header>

      {/* Hero Banner */}
      <section className="gradient-hero text-primary-foreground py-8 sm:py-10 md:py-12">
        <div className="container mx-auto px-4">
          <div className="flex items-center justify-center gap-2 sm:gap-3 mb-3 sm:mb-4">
            <Trophy className="h-8 w-8 sm:h-9 sm:w-9 md:h-10 md:w-10" />
            <h1 className="text-2xl sm:text-3xl md:text-4xl font-extrabold">Leaderboard</h1>
          </div>
          <p className="text-center text-sm sm:text-base text-primary-foreground/80 max-w-xl mx-auto px-4">
            Rankings based on problems solved across LeetCode, CodeChef, and Codeforces in the last 30 days.
          </p>
        </div>
      </section>

      {/* Leaderboard Content */}
      <section className="py-4 sm:py-6 md:py-8">
        <div className="container mx-auto px-3 sm:px-4">
          <Card className="mb-4 sm:mb-6">
            <CardContent className="py-3 sm:py-4">
              <Tabs value={selectedYear} onValueChange={(val) => setSelectedYear(val as "I" | "II" | "III" | "IV")}>
                <TabsList className="grid grid-cols-4 w-full max-w-md mx-auto h-auto">
                  {(["I", "II", "III", "IV"] as const).map((year) => (
                    <TabsTrigger key={year} value={year} className="font-semibold text-xs sm:text-sm py-2">
                      <span className="hidden sm:inline">{yearLabels[year]}</span>
                      <span className="sm:hidden">{year}</span>
                    </TabsTrigger>
                  ))}
                </TabsList>
              </Tabs>
            </CardContent>
          </Card>

          {/* Leaderboard Table */}
          <Card>
            <CardHeader className="border-b">
              <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 sm:gap-4">
                <CardTitle className="text-lg sm:text-xl flex items-center gap-2">
                  <Trophy className="h-4 w-4 sm:h-5 sm:w-5 text-primary" />
                  {yearLabels[selectedYear]} Rankings
                </CardTitle>
                {leaderboardData && (
                  <Badge variant="secondary" className="w-fit text-xs sm:text-sm">
                    {leaderboardData.total_students} Students
                  </Badge>
                )}
              </div>
            </CardHeader>
            <CardContent className="p-0">
              {isLoading ? (
                <div className="flex items-center justify-center py-12">
                  <div className="h-8 w-8 border-4 border-primary border-t-transparent rounded-full animate-spin" />
                </div>
              ) : (
                <div className="overflow-x-auto">
                  <Table>
                    <TableHeader>
                      <TableRow className="bg-muted/50">
                        <TableHead className="w-16 sm:w-20 text-center text-xs sm:text-sm">Rank</TableHead>
                        <TableHead className="text-xs sm:text-sm">Name</TableHead>
                        <TableHead className="text-xs sm:text-sm min-w-[120px] sm:min-w-0">RA Number</TableHead>
                        <TableHead className="text-xs sm:text-sm">Specialization</TableHead>
                        <TableHead className="text-xs sm:text-sm">Profiles</TableHead>
                        <TableHead className="text-right text-xs sm:text-sm min-w-[100px] sm:min-w-0">30-Day Solved</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {leaderboardData && leaderboardData.leaderboard.length > 0 ? (
                        leaderboardData.leaderboard.map((student) => (
                          <TableRow
                            key={student.registration_number}
                            className={student.rank <= 3 ? "bg-muted/30" : ""}
                          >
                            <TableCell className="text-center">
                              <RankBadge rank={student.rank} />
                            </TableCell>
                            <TableCell className="font-medium text-xs sm:text-sm">
                              {student.full_name}
                            </TableCell>
                            <TableCell className="font-mono font-medium text-xs sm:text-sm">
                              {student.registration_number}
                            </TableCell>
                            <TableCell>
                              <Badge variant="outline" className="text-xs">{student.specialization || "N/A"}</Badge>
                            </TableCell>
                            <TableCell>
                              <div className="flex items-center gap-1.5">
                                {student.leetcode_username && (
                                  <a
                                    href={`https://leetcode.com/${student.leetcode_username}`}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    className="inline-flex items-center text-xs hover:underline text-primary font-medium"
                                    title="LeetCode Profile"
                                  >
                                    <span>LC</span>
                                  </a>
                                )}
                                {student.codechef_username && (
                                  <a
                                    href={`https://www.codechef.com/users/${student.codechef_username}`}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    className="inline-flex items-center text-xs hover:underline text-primary font-medium"
                                    title="CodeChef Profile"
                                  >
                                    <span>CC</span>
                                  </a>
                                )}
                                {student.codeforces_username && (
                                  <a
                                    href={`https://codeforces.com/profile/${student.codeforces_username}`}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    className="inline-flex items-center text-xs hover:underline text-primary font-medium"
                                    title="Codeforces Profile"
                                  >
                                    <span>CF</span>
                                  </a>
                                )}
                                {!student.leetcode_username && !student.codechef_username && !student.codeforces_username && (
                                  <span className="text-xs text-muted-foreground">None</span>
                                )}
                              </div>
                            </TableCell>
                            <TableCell className="text-right">
                              <span className="font-bold text-primary text-base sm:text-lg">
                                {student.total_solved_last_30_days}
                              </span>
                            </TableCell>
                          </TableRow>
                        ))
                      ) : (
                        <TableRow>
                          <TableCell colSpan={6} className="text-center py-8 sm:py-12 text-muted-foreground text-sm">
                            No students found for this year
                          </TableCell>
                        </TableRow>
                      )}
                    </TableBody>
                  </Table>
                </div>
              )}
            </CardContent>
          </Card>

          {/* Info Footer */}
          <Card className="mt-4 sm:mt-6">
            <CardContent className="py-3 sm:py-4">
              <div className="flex items-center justify-center gap-2 text-xs sm:text-sm text-muted-foreground">
                <RefreshCw className="h-3.5 w-3.5 sm:h-4 sm:w-4" />
                <span>Leaderboard includes LeetCode, CodeChef, and Codeforces activity from the last 30 days</span>
              </div>
            </CardContent>
          </Card>
        </div>
      </section>
    </div>
  );
}
