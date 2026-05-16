import { useState, useEffect } from "react";
import { useNavigate, Link } from "react-router-dom";
import { Code2, LogOut, ArrowLeft, Github, Linkedin, ExternalLink, Save, Edit2, Check, User, Mail, Hash, Award, Trophy, MessageSquare, Lock, Eye, EyeOff, KeyRound, Code, MessageCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useAuth } from "@/contexts/AuthContext";
import { toast } from "sonner";
import { ThemeToggle } from "@/components/ThemeToggle";
import { Avatar, AvatarImage, AvatarFallback } from "@/components/ui/avatar";
import { fetchProfileLinks, updateProfileLinks, forgotPassword } from "@/lib/api";

interface ProfileUrls {
  leetcode: string;
  github: string;
  linkedin: string;
  codechef: string;
  codeforces: string;
}

// dont mind this, leave this alone
const SPECIAL_THEME = {
  ra: "RA2411030020146",
  colors: {
    base: "#800000",
    light: "#b32424",
    dark: "#4d0f0f",
    ring: "#a31f1f",
    text: "#ffffff",
  },
  badgeText: ":)",
};

export default function StudentDashboard() {
  const navigate = useNavigate();
  const { user, logout, token, isLoading: authLoading } = useAuth();
  const [urls, setUrls] = useState<ProfileUrls>({
    leetcode: "",
    github: "",
    linkedin: "",
    codechef: "",
    codeforces: "",
  });
  const [isEditing, setIsEditing] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  
  // Change password state
  const [isPasswordResetSending, setIsPasswordResetSending] = useState(false);

  // Fetch profile URLs from API
  const fetchProfileUrls = async () => {
    if (!token) {
      setIsLoading(false);
      return;
    }
    
    try {
      const data = await fetchProfileLinks(token);
      const newUrls = {
        leetcode: data.leetcode_link || "",
        github: data.github_link || "",
        linkedin: data.linkedin_link || "",
        codechef: data.codechef_link || "",
        codeforces: data.codeforces_link || "",
      };

      setUrls(newUrls);
    } catch (error) {
      console.error("Failed to fetch profile URLs:", error);
      const errorMessage = error instanceof Error ? error.message : "Failed to fetch profile URLs";
      
      // Handle authentication errors - logout and redirect
      if (errorMessage.includes("Invalid token") || errorMessage.includes("Unauthorized")) {
        toast.error("Your session has expired. Please login again.");
        logout();
        navigate("/student/login");
        return;
      }
      
      toast.error(errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  // Check if user is a student
  useEffect(() => {
    // Wait for auth to finish loading
    if (authLoading) return;
    
    if (!user || user.type !== "student") {
      navigate("/student/login");
    } else {
      fetchProfileUrls();
    }
  }, [user, navigate, authLoading, token]);

  const getInitials = (name: string) => {
    return name
      .split(" ")
      .map((n) => n[0])
      .join("")
      .toUpperCase()
      .slice(0, 2);
  };

  const isDeveloper = (userId: string) => {
    const developerIds = ["RA2311051020009", "RA2311003020064"];
    return developerIds.includes(userId);
  };

  const handleLogout = () => {
    logout();
    navigate("/student/login");
    toast.success("Logged out successfully");
  };

  const handleInputChange = (field: keyof ProfileUrls, value: string) => {
    setUrls((prev) => ({ ...prev, [field]: value }));
  };

  const validateUrl = (url: string, platform: string): boolean => {
    if (!url) return true; // Empty URLs are allowed
    try {
      const urlObj = new URL(url);
      const hostname = urlObj.hostname.toLowerCase();
      
      switch (platform) {
        case "leetcode":
          return hostname.includes("leetcode.com");
        case "github":
          return hostname.includes("github.com");
        case "linkedin":
          return hostname.includes("linkedin.com");
        case "codechef":
          return hostname.includes("codechef.com");
        case "codeforces":
          return hostname.includes("codeforces.com");
        default:
          return true;
      }
    } catch {
      return false;
    }
  };

  const handleSave = async () => {
    // Validate required fields
    if (!urls.github) {
      toast.error("GitHub profile is required");
      return;
    }
    if (!urls.linkedin) {
      toast.error("LinkedIn profile is required");
      return;
    }
    
    // At least one coding platform is required
    const hasCodingProfile = urls.leetcode || urls.codechef || urls.codeforces;
    if (!hasCodingProfile) {
      toast.error("At least one coding platform (LeetCode, CodeChef, or Codeforces) is required");
      return;
    }

    // Validate URLs
    if (urls.leetcode && !validateUrl(urls.leetcode, "leetcode")) {
      toast.error("Please enter a valid LeetCode URL");
      return;
    }
    if (urls.github && !validateUrl(urls.github, "github")) {
      toast.error("Please enter a valid GitHub URL");
      return;
    }
    if (urls.linkedin && !validateUrl(urls.linkedin, "linkedin")) {
      toast.error("Please enter a valid LinkedIn URL");
      return;
    }
    if (urls.codechef && !validateUrl(urls.codechef, "codechef")) {
      toast.error("Please enter a valid CodeChef URL");
      return;
    }
    if (urls.codeforces && !validateUrl(urls.codeforces, "codeforces")) {
      toast.error("Please enter a valid Codeforces URL");
      return;
    }

    if (!token) {
      toast.error("Authentication token not found. Please login again.");
      logout();
      navigate("/student/login");
      return;
    }

    setIsSaving(true);
    
    // Add 2 second delay
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    try {
      await updateProfileLinks(token, {
        leetcode_link: urls.leetcode || null,
        github_link: urls.github || null,
        linkedin_link: urls.linkedin || null,
        codechef_link: urls.codechef || null,
        codeforces_link: urls.codeforces || null,
      });

      setIsSaving(false);
      setIsEditing(false);
      toast.success("Profile URLs saved successfully!");
    } catch (error) {
      setIsSaving(false);
      console.error("Failed to save profile URLs:", error);
      const errorMessage = error instanceof Error ? error.message : "Failed to save profile URLs. Please try again.";
      
      // Handle authentication errors - logout and redirect
      if (errorMessage.includes("Invalid token") || errorMessage.includes("Unauthorized")) {
        toast.error("Your session has expired. Please login again.");
        logout();
        navigate("/student/login");
        return;
      }
      
      toast.error(errorMessage);
    }
  };

  const handleCancel = () => {
    // Re-fetch the URLs to restore original values
    fetchProfileUrls();
    setIsEditing(false);
  };

  const handleSendPasswordReset = async () => {
    if (!user?.email) {
      toast.error("Email not found. Please login again.");
      return;
    }

    setIsPasswordResetSending(true);

    try {
      await forgotPassword({ email: user.email });
      toast.success("Password reset link sent to your email!");
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to send password reset email. Please try again.";
      toast.error(errorMessage);
    } finally {
      setIsPasswordResetSending(false);
    }
  };

  if (!user || user.type !== "student") {
    return null;
  }

  if (authLoading || isLoading) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <div className="flex flex-col items-center gap-4">
          <div className="h-12 w-12 border-4 border-primary border-t-transparent rounded-full animate-spin" />
          <p className="text-muted-foreground">Loading your profile...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background flex flex-col">
      {/* Header */}
      <header className="border-b bg-card/50 backdrop-blur-sm sticky top-0 z-50">
        <div className="container mx-auto px-3 sm:px-4 py-3 flex items-center justify-between">
          <div className="flex items-center gap-2 sm:gap-4">
            <Link to="/">
              <Button variant="ghost" size="icon" className="rounded-full h-9 w-9">
                <ArrowLeft className="h-4 w-4 sm:h-5 sm:w-5" />
              </Button>
            </Link>
            <div className="flex items-center gap-2 sm:gap-3">
              <img src="/logo.svg" alt="SRM Logo" className="h-16 w-16 object-contain -my-2" />
              <div>
                <h1 className="font-bold text-sm sm:text-base md:text-lg text-foreground">Dashboard</h1>
                <p className="text-xs text-muted-foreground hidden sm:block">
                  {user.user_id === "RA2411030020146" ? (
                    <span className="inline-flex items-center gap-1">
                      <span className="opacity-50">✨</span>
                      {user.name}
                      <span className="opacity-50">✨</span>
                    </span>
                  ) : (
                    user.name
                  )}
                </p>
              </div>
            </div>
          </div>
          <div className="flex items-center gap-2 sm:gap-2 md:gap-3">
            <ThemeToggle />
            <a href="https://whatsapp.com/channel/0029VbBX2gIDp2QHHGlzM31J" target="_blank" rel="noopener noreferrer">
              <Button variant="default" size="sm" className="min-h-[40px] px-3 bg-green-600 hover:bg-green-700">
                <MessageCircle className="h-4 w-4 mr-1" />
                <span className="hidden sm:inline">Placements</span>
              </Button>
            </a>
            <a href="https://forms.gle/EYvRCHFSTx3845Fe7" target="_blank" rel="noopener noreferrer">
              <Button variant="ghost" size="sm" className="min-h-[40px] px-3">
                <MessageSquare className="h-4 w-4 mr-1" />
                <span className="hidden sm:inline">Help & Feedback</span>
              </Button>
            </a>
            <Link to="/leaderboard">
              <Button variant="outline" size="sm" className="text-xs sm:text-sm min-h-[40px] px-3">
                <Trophy className="h-4 w-4 sm:mr-2" />
                <span className="hidden sm:inline">View Leaderboard</span>
              </Button>
            </Link>
            <Button variant="ghost" size="sm" onClick={handleLogout} className="min-h-[40px] px-3">
              <LogOut className="h-4 w-4 sm:mr-2" />
              <span className="hidden sm:inline">Logout</span>
            </Button>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <div className="flex-1 container mx-auto px-4 sm:px-4 py-5 sm:py-6 md:py-8 max-w-4xl space-y-5 sm:space-y-6">
        {/* Profile Information Card */}
        <Card
          className={user.user_id === SPECIAL_THEME.ra ? "border rounded-xl shadow-none" : "border-primary/20 shadow-lg rounded-xl"}
          style={user.user_id === SPECIAL_THEME.ra ? { borderColor: SPECIAL_THEME.colors.base, boxShadow: `0 0 0 1px ${SPECIAL_THEME.colors.base}33` } : undefined}
        >
          <CardHeader
            className={user.user_id === SPECIAL_THEME.ra ? "border-b rounded-t-xl" : "border-b bg-gradient-to-r from-primary/5 to-primary/10"}
            style={user.user_id === SPECIAL_THEME.ra ? { backgroundColor: SPECIAL_THEME.colors.light } : undefined}
          >
            <CardTitle className="text-xl sm:text-2xl font-bold" style={user.user_id === SPECIAL_THEME.ra ? { color: SPECIAL_THEME.colors.text } : undefined}>
              Student Profile
            </CardTitle>
            <CardDescription className="text-sm sm:text-base" style={user.user_id === SPECIAL_THEME.ra ? { color: `${SPECIAL_THEME.colors.text}cc` } : undefined}>Your account information</CardDescription>
          </CardHeader>
          <CardContent className="pt-4 sm:pt-6">
            <div className="flex flex-col md:flex-row items-start gap-6 sm:gap-8">
              <div className="flex flex-col items-center gap-3 w-full md:w-auto">
                <Avatar
                  className="h-20 w-20 sm:h-24 sm:w-24 border-4 border-primary/20 shadow-md"
                  style={user.user_id === SPECIAL_THEME.ra ? { boxShadow: `0 0 0 2px ${SPECIAL_THEME.colors.ring}` } : undefined}
                >
                  {user.user_id === "RA2411030020146" && (
                    <AvatarImage
                      src="https://i.postimg.cc/tC4yx16N/image-(5).jpg"
                      alt={user.name}
                    />
                  )}
                  <AvatarFallback className="text-2xl sm:text-3xl font-bold gradient-primary text-primary-foreground">
                    {getInitials(user.name)}
                  </AvatarFallback> 
                </Avatar>
                <div className="text-center">
                  <p className="text-xs sm:text-sm font-medium text-muted-foreground">Account Status</p>
                  <div className="flex items-center gap-2 mt-1 justify-center">
                    <div className="h-2 w-2 rounded-full bg-green-500 animate-pulse"></div>
                    <span className="text-xs sm:text-sm font-semibold text-green-600 dark:text-green-400">Active</span>
                  </div>
                </div>
              </div>
              <div className="flex-1 grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6 w-full">
                <div className="space-y-2 p-3 sm:p-4 rounded-lg border bg-card/50 transition-shadow">
                  <div className="flex items-center gap-2 text-xs sm:text-sm font-medium text-muted-foreground">
                    <User className="h-3.5 w-3.5 sm:h-4 sm:w-4" />
                    <span>Full Name</span>
                  </div>
                  <p className="text-base sm:text-xl font-bold text-foreground break-words">{user.name}</p>
                </div>
                <div className="space-y-2 p-3 sm:p-4 rounded-lg border bg-card/50 transition-shadow">
                  <div className="flex items-center gap-2 text-xs sm:text-sm font-medium text-muted-foreground">
                    <Hash className="h-3.5 w-3.5 sm:h-4 sm:w-4" />
                    <span>Register Number</span>
                  </div>
                  <p className="text-base sm:text-xl font-bold font-mono text-foreground tracking-tight break-all">{user.user_id}</p>
                </div>
                <div className="space-y-2 p-3 sm:p-4 rounded-lg border bg-card/50 transition-shadow">
                  <div className="flex items-center gap-2 text-xs sm:text-sm font-medium text-muted-foreground">
                    <Mail className="h-3.5 w-3.5 sm:h-4 sm:w-4" />
                    <span>Email Address</span>
                  </div>
                  <p className="text-sm sm:text-base font-medium text-foreground break-all">{user.email}</p>
                </div>
                <div className="space-y-2 p-3 sm:p-4 rounded-lg border bg-card/50 transition-shadow">
                  <div className="flex items-center gap-2 text-xs sm:text-sm font-medium text-muted-foreground">
                    <Award className="h-3.5 w-3.5 sm:h-4 sm:w-4" />
                    <span>Role</span>
                  </div>
                  <div className="flex items-center gap-2 flex-wrap">
                    <span className="inline-flex items-center px-2.5 py-1 rounded-full text-xs sm:text-sm font-semibold bg-primary/10 text-primary border border-primary/20">
                      {user.role.charAt(0).toUpperCase() + user.role.slice(1)}
                    </span>
                    {isDeveloper(user.user_id) && (
                      <span className="inline-flex items-center px-2.5 py-1 rounded-full text-xs sm:text-sm font-semibold bg-red-100 dark:bg-red-950 text-red-700 dark:text-red-400 border border-red-300 dark:border-red-800">
                        Developer
                      </span>
                    )}
                    {user.user_id === SPECIAL_THEME.ra && (
                      <span
                        className="inline-flex items-center px-2.5 py-1 rounded-full text-xs sm:text-sm font-semibold"
                        style={{ backgroundColor: SPECIAL_THEME.colors.base, color: SPECIAL_THEME.colors.text, border: `1px solid ${SPECIAL_THEME.colors.base}` }}
                      >
                        {SPECIAL_THEME.badgeText}
                      </span>
                    )}
                  </div>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Profile URLs Card */}
        <Card>
          <CardHeader>
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
              <div>
                <CardTitle className="text-lg sm:text-xl md:text-2xl flex items-center gap-2">
                  Profile URLs
                </CardTitle>
                <CardDescription className="mt-1.5 sm:mt-2 text-xs sm:text-sm">
                  Submit or update your LeetCode, GitHub, LinkedIn, CodeChef, and Codeforces profile URLs.
                  You can edit these anytime after login.
                </CardDescription>
              </div>
              {!isEditing && (
                <Button onClick={() => setIsEditing(true)} variant="outline" className="min-h-[44px] px-4 self-start sm:self-auto">
                  <Edit2 className="h-4 w-4 mr-2" />
                  <span>Edit</span>
                </Button>
              )}
            </div>
          </CardHeader>
          <CardContent className="space-y-5 sm:space-y-6">
            {/* LinkedIn URL */}
            <div className="space-y-2">
              <Label htmlFor="linkedin" className="flex items-center gap-2 text-sm">
                <Linkedin className="h-3.5 w-3.5 sm:h-4 sm:w-4 text-muted-foreground" />
                LinkedIn Profile URL
                <span className="text-xs text-red-500">*</span>
              </Label>
              <div className="flex gap-2">
                <Input
                  id="linkedin"
                  type="url"
                  placeholder="https://linkedin.com/in/yourusername"
                  value={urls.linkedin}
                  onChange={(e) => handleInputChange("linkedin", e.target.value)}
                  disabled={!isEditing}
                  className="flex-1 text-sm min-h-[44px]"
                />
                {urls.linkedin && !isEditing && (
                  <Button
                    variant="outline"
                    size="icon"
                    asChild
                    className="min-h-[44px] min-w-[44px] flex-shrink-0"
                  >
                    <a href={urls.linkedin} target="_blank" rel="noopener noreferrer">
                      <ExternalLink className="h-4 w-4" />
                    </a>
                  </Button>
                )}
              </div>
            </div>

            {/* GitHub URL */}
            <div className="space-y-2">
              <Label htmlFor="github" className="flex items-center gap-2 text-sm">
                <Github className="h-3.5 w-3.5 sm:h-4 sm:w-4 text-muted-foreground" />
                GitHub Profile URL
                <span className="text-xs text-red-500">*</span>
              </Label>
              <div className="flex gap-2">
                <Input
                  id="github"
                  type="url"
                  placeholder="https://github.com/yourusername"
                  value={urls.github}
                  onChange={(e) => handleInputChange("github", e.target.value)}
                  disabled={!isEditing}
                  className="flex-1 text-sm min-h-[44px]"
                />
                {urls.github && !isEditing && (
                  <Button
                    variant="outline"
                    size="icon"
                    asChild
                    className="min-h-[44px] min-w-[44px] flex-shrink-0"
                  >
                    <a href={urls.github} target="_blank" rel="noopener noreferrer">
                      <ExternalLink className="h-4 w-4" />
                    </a>
                  </Button>
                )}
              </div>
            </div>

            {/* LeetCode URL */}
            <div className="space-y-2">
              <Label htmlFor="leetcode" className="flex items-center gap-2 text-sm">
                <Code2 className="h-3.5 w-3.5 sm:h-4 sm:w-4 text-muted-foreground" />
                LeetCode Profile URL
                <span className="text-xs text-muted-foreground">(Required: At least one coding platform)</span>
              </Label>
              <div className="flex gap-2">
                <Input
                  id="leetcode"
                  type="url"
                  placeholder="https://leetcode.com/yourusername"
                  value={urls.leetcode}
                  onChange={(e) => handleInputChange("leetcode", e.target.value)}
                  disabled={!isEditing}
                  className="flex-1 text-sm min-h-[44px]"
                />
                {urls.leetcode && !isEditing && (
                  <Button
                    variant="outline"
                    size="icon"
                    asChild
                    className="min-h-[44px] min-w-[44px] flex-shrink-0"
                  >
                    <a href={urls.leetcode} target="_blank" rel="noopener noreferrer">
                      <ExternalLink className="h-4 w-4" />
                    </a>
                  </Button>
                )}
              </div>
            </div>

            {/* CodeChef URL */}
            <div className="space-y-2">
              <Label htmlFor="codechef" className="flex items-center gap-2 text-sm">
                <Trophy className="h-3.5 w-3.5 sm:h-4 sm:w-4 text-muted-foreground" />
                CodeChef Profile URL
                <span className="text-xs text-muted-foreground">(Optional)</span>
              </Label>
              <div className="flex gap-2">
                <Input
                  id="codechef"
                  type="url"
                  placeholder="https://codechef.com/users/yourusername"
                  value={urls.codechef}
                  onChange={(e) => handleInputChange("codechef", e.target.value)}
                  disabled={!isEditing}
                  className="flex-1 text-sm min-h-[44px]"
                />
                {urls.codechef && !isEditing && (
                  <Button
                    variant="outline"
                    size="icon"
                    asChild
                    className="min-h-[44px] min-w-[44px] flex-shrink-0"
                  >
                    <a href={urls.codechef} target="_blank" rel="noopener noreferrer">
                      <ExternalLink className="h-4 w-4" />
                    </a>
                  </Button>
                )}
              </div>
            </div>

            {/* Codeforces URL */}
            <div className="space-y-2">
              <Label htmlFor="codeforces" className="flex items-center gap-2 text-sm">
                <Code className="h-3.5 w-3.5 sm:h-4 sm:w-4 text-muted-foreground" />
                Codeforces Profile URL
                <span className="text-xs text-muted-foreground">(Optional)</span>
              </Label>
              <div className="flex gap-2">
                <Input
                  id="codeforces"
                  type="url"
                  placeholder="https://codeforces.com/profile/yourusername"
                  value={urls.codeforces}
                  onChange={(e) => handleInputChange("codeforces", e.target.value)}
                  disabled={!isEditing}
                  className="flex-1 text-sm min-h-[44px]"
                />
                {urls.codeforces && !isEditing && (
                  <Button
                    variant="outline"
                    size="icon"
                    asChild
                    className="min-h-[44px] min-w-[44px] flex-shrink-0"
                  >
                    <a href={urls.codeforces} target="_blank" rel="noopener noreferrer">
                      <ExternalLink className="h-4 w-4" />
                    </a>
                  </Button>
                )}
              </div>
            </div>

            {/* Action Buttons */}
            {isEditing && (
              <div className="flex flex-col sm:flex-row gap-3 pt-4 border-t">
                <Button
                  onClick={handleSave}
                  disabled={isSaving}
                  className="w-full sm:flex-1 min-h-[48px] text-base font-semibold"
                >
                  {isSaving ? (
                    <span className="flex items-center justify-center gap-2">
                      <span className="h-4 w-4 border-2 border-primary-foreground border-t-transparent rounded-full animate-spin" />
                      Saving...
                    </span>
                  ) : (
                    <span className="flex items-center justify-center gap-2">
                      <Save className="h-5 w-5" />
                      Save Changes
                    </span>
                  )}
                </Button>
                <Button
                  onClick={handleCancel}
                  variant="outline"
                  disabled={isSaving}
                  className="w-full sm:w-auto min-h-[48px] text-base font-semibold"
                >
                  Cancel
                </Button>
              </div>
            )}

            {!isEditing && (
              <div className="pt-4 border-t">
                <div className="flex items-center gap-2 text-sm text-muted-foreground">
                  <Check className="h-4 w-4 text-success" />
                  <span>Your profile URLs are saved. Click Edit to make changes.</span>
                </div>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Change Password Card */}
        <Card>
          <CardHeader>
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
              <div>
                <CardTitle className="text-lg sm:text-xl md:text-2xl flex items-center gap-2">
                  <KeyRound className="h-5 w-5" />
                  Change Password
                </CardTitle>
                <CardDescription className="mt-1.5 sm:mt-2 text-xs sm:text-sm">
                  Request a password reset link to update your account password
                </CardDescription>
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-start gap-3 text-sm text-muted-foreground bg-muted/30 p-4 rounded-lg border">
              <Mail className="h-4 w-4 mt-0.5 flex-shrink-0" />
              <div className="flex-1">
                <p className="font-medium text-foreground mb-1">Secure Password Reset</p>
                <p className="text-xs mb-3">
                  We'll send a secure password reset link to your registered email address: <span className="font-semibold text-foreground">{user.email}</span>
                </p>
                <p className="text-xs">
                  Click the link in the email to create a new password for your account.
                </p>
              </div>
            </div>
            
            <Button
              onClick={handleSendPasswordReset}
              disabled={isPasswordResetSending}
              className="w-full sm:w-auto min-h-[48px] text-base font-semibold"
            >
              {isPasswordResetSending ? (
                <span className="flex items-center justify-center gap-2">
                  <span className="h-4 w-4 border-2 border-primary-foreground border-t-transparent rounded-full animate-spin" />
                  Sending Reset Link...
                </span>
              ) : (
                <span className="flex items-center justify-center gap-2">
                  <Mail className="h-5 w-5" />
                  Send Password Reset Link
                </span>
              )}
            </Button>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
