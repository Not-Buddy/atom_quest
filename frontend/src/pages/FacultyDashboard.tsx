import { useState, useEffect } from "react";
import { useNavigate, Link } from "react-router-dom";
import {
  Users,
  AlertTriangle,
  LogOut,
  Building2,
  UserX,
  Download,
  CheckCircle2,
  FileSpreadsheet,
  Trophy,
  ArrowLeft,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useAuth } from "@/contexts/AuthContext";
import { StatCard } from "@/components/StatCard";
import { toast } from "sonner";
import { ThemeToggle } from "@/components/ThemeToggle";
import { fetchFacultyProfile, downloadSubmissionsReport, fetchFacultyStats, downloadDefaultersReport, FacultyStats } from "@/lib/api";
import { departments } from "@/lib/mockData";

export default function FacultyDashboard() {
  const navigate = useNavigate();
  const { user, logout, token, isLoading: authLoading } = useAuth();
  const [isLoading, setIsLoading] = useState(true);
  const [reportLoading, setReportLoading] = useState(false);
  const [defaultersReportLoading, setDefaultersReportLoading] = useState(false);
  const [facultyProfile, setFacultyProfile] = useState<Record<string, unknown> | null>(null);
  const [facultyStats, setFacultyStats] = useState<FacultyStats | FacultyStats[] | null>(null);
  const [selectedSpecialization, setSelectedSpecialization] = useState<string>("");

  const isAdmin = Array.isArray(facultyStats);

  useEffect(() => {
    if (authLoading) return;
    
    if (!user || user.type !== "faculty") {
      navigate("/faculty/login");
    }
  }, [user, navigate, authLoading]);

  useEffect(() => {
    if (!token || !user || user.type !== "faculty") return;

    const loadFacultyData = async () => {
      setIsLoading(true);
      try {
        const [profile, stats] = await Promise.all([
          fetchFacultyProfile(token),
          fetchFacultyStats(token),
        ]);
        setFacultyProfile(profile);
        setFacultyStats(stats);
      } catch (error) {
        console.error("Failed to fetch faculty data:", error);
        toast.error("Failed to load faculty data");
      } finally {
        setIsLoading(false);
      }
    };

    loadFacultyData();
  }, [token, user]);

  const monthlyTarget = 15;

  // Calculate stats based on whether user is admin or regular faculty
  const currentStats: FacultyStats | null = isAdmin 
    ? (facultyStats as FacultyStats[]).find(s => s.specialization === selectedSpecialization) || null
    : (facultyStats as FacultyStats);

  const totalStudents = currentStats?.total_students || 0;
  const studentsWithProfile = currentStats?.with_leetcode_profiles || 0;
  const studentsWithoutProfile = currentStats?.without_leetcode_profiles || 0;
  const defaulters = currentStats?.defaulters || 0;

  const departmentName = currentStats?.specialization || 
    (user?.type === "faculty" ? user.specialization : undefined) || 
    "Department";
  const academicYear = currentStats?.academic_year || "";

  const getShortDeptInfo = (fullName: string, year: string) => {
    const dept = departments.find(d => d.name === fullName);
    const shortName = dept?.shortName || fullName;
    
    const yearDisplay = year === "I" ? "1st Year" :
                       year === "II" ? "2nd Year" :
                       year === "III" ? "3rd Year" :
                       year === "IV" ? "4th Year" :
                       year === "VI" ? "All Years" :
                       "All Years";
    
    return { short: shortName, year: yearDisplay };
  };

  const deptInfo = getShortDeptInfo(departmentName, academicYear);

  const handleLogout = () => {
    logout();
    toast.success("Logged out successfully");
    navigate("/");
  };

  const handleDownloadReport = async () => {
    if (!token) {
      toast.error("Authentication token not found. Please login again.");
      return;
    }

    if (isAdmin && !selectedSpecialization) {
      toast.error("Please select a specialization first");
      return;
    }

    setReportLoading(true);
    try {
      await downloadSubmissionsReport(token, isAdmin ? selectedSpecialization : undefined);
      toast.success("Submissions report downloaded successfully");
    } catch (error) {
      console.error("Failed to download report:", error);
      toast.error(error instanceof Error ? error.message : "Failed to download report");
    } finally {
      setReportLoading(false);
    }
  };

  const handleDownloadDefaultersReport = async () => {
    if (!token) {
      toast.error("Authentication token not found. Please login again.");
      return;
    }

    if (isAdmin && !selectedSpecialization) {
      toast.error("Please select a specialization first");
      return;
    }

    setDefaultersReportLoading(true);
    try {
      await downloadDefaultersReport(token, isAdmin ? selectedSpecialization : undefined);
      toast.success("Defaulters report downloaded successfully");
    } catch (error) {
      console.error("Failed to download defaulters report:", error);
      toast.error(error instanceof Error ? error.message : "Failed to download defaulters report");
    } finally {
      setDefaultersReportLoading(false);
    }
  };

  // Get all specializations for admin
  const allSpecializations = isAdmin 
    ? (facultyStats as FacultyStats[]).map(s => s.specialization)
    : [];

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="container flex items-center justify-between px-4 py-3">
          <div className="flex items-center gap-3">
            <Link to="/">
              <Button variant="ghost" size="icon" className="rounded-full">
                <ArrowLeft className="h-5 w-5" />
              </Button>
            </Link>
            <img src="/logo.svg" alt="SRM Logo" className="h-16 w-16 object-contain -my-2" />
            <div>
              <h1 className="text-lg font-semibold">{deptInfo.short}</h1>
              <p className="text-xs text-muted-foreground">{deptInfo.year}</p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <ThemeToggle />
            <Link to="/leaderboard">
              <Button variant="outline" size="sm">
                <Trophy className="h-4 w-4 mr-2" />
                Leaderboard
              </Button>
            </Link>
            <Button variant="ghost" size="sm" onClick={handleLogout}>
              <LogOut className="h-4 w-4 mr-2" />
              Logout
            </Button>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="container mx-auto px-4 py-8">
        <div className="space-y-6">
          {/* Admin Specialization Selector */}
          {isAdmin && (
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Building2 className="h-5 w-5" />
                  Select Specialization
                </CardTitle>
                <CardDescription>
                  Choose a specialization to view stats and download reports
                </CardDescription>
              </CardHeader>
              <CardContent>
                <Select value={selectedSpecialization} onValueChange={setSelectedSpecialization}>
                  <SelectTrigger className="w-full">
                    <SelectValue placeholder="Select a specialization" />
                  </SelectTrigger>
                  <SelectContent>
                    {allSpecializations.map((spec) => {
                      const dept = departments.find(d => d.name === spec);
                      return (
                        <SelectItem key={spec} value={spec}>
                          {dept?.shortName || spec}
                        </SelectItem>
                      );
                    })}
                  </SelectContent>
                </Select>
              </CardContent>
            </Card>
          )}

          {/* Stats Cards */}
          {(!isAdmin || selectedSpecialization) && (
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
              <StatCard
                title="Total Students"
                value={totalStudents}
                icon={Users}
                description="In department"
              />
              <StatCard
                title="With LeetCode Profile"
                value={studentsWithProfile}
                icon={CheckCircle2}
                description={`${totalStudents > 0 ? Math.round((studentsWithProfile / totalStudents) * 100) : 0}% coverage`}
              />
              <StatCard
                title="Without Profile"
                value={studentsWithoutProfile}
                icon={UserX}
                description="Need to register"
              />
              <StatCard
                title="Defaulters"
                value={defaulters}
                icon={AlertTriangle}
                description={`Below ${monthlyTarget} problems`}
              />
            </div>
          )}

          {/* Download Reports Card */}
          {(!isAdmin || selectedSpecialization) && (
            <div className="grid gap-4 sm:grid-cols-2">
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <FileSpreadsheet className="h-5 w-5" />
                    Student Submissions Report
                  </CardTitle>
                  <CardDescription>
                    Download an Excel report containing all student profile submissions
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <Button 
                    onClick={handleDownloadReport} 
                    disabled={reportLoading}
                    size="lg"
                    className="w-full"
                  >
                    {reportLoading ? (
                      <>
                        <div className="h-4 w-4 border-2 border-white border-t-transparent rounded-full animate-spin mr-2" />
                        Downloading...
                      </>
                    ) : (
                      <>
                        <Download className="h-4 w-4 mr-2" />
                        Download Submissions
                      </>
                    )}
                  </Button>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <AlertTriangle className="h-5 w-5" />
                    Defaulters Report
                  </CardTitle>
                  <CardDescription>
                    Download Excel report of students with &lt;15 questions in last 30 days
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <Button 
                    onClick={handleDownloadDefaultersReport} 
                    disabled={defaultersReportLoading}
                    size="lg"
                    className="w-full"
                  >
                    {defaultersReportLoading ? (
                      <>
                        <div className="h-4 w-4 border-2 border-white border-t-transparent rounded-full animate-spin mr-2" />
                        Downloading...
                      </>
                    ) : (
                      <>
                        <Download className="h-4 w-4 mr-2" />
                        Download Defaulters
                      </>
                    )}
                  </Button>
                </CardContent>
              </Card>
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
