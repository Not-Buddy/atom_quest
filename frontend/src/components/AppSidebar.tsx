import { NavLink, useLocation } from "react-router-dom";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";
import { Button } from "@/components/ui/button";
import { useAuth } from "@/contexts/AuthContext";
import {
  LayoutDashboard,
  Target,
  ClipboardCheck,
  Users,
  FileText,
  BarChart3,
  Settings,
  LogOut,
  Shield,
  Activity,
  Building,
  TrendingUp,
} from "lucide-react";

interface NavItem {
  title: string;
  href: string;
  icon: React.ElementType;
}

const employeeNav: NavItem[] = [
  { title: "Dashboard", href: "/employee", icon: LayoutDashboard },
  { title: "My Goals", href: "/employee/goals", icon: Target },
  { title: "Achievements", href: "/employee/achievements", icon: ClipboardCheck },
  { title: "Reports", href: "/reports", icon: BarChart3 },
];

const managerNav: NavItem[] = [
  { title: "Dashboard", href: "/manager", icon: LayoutDashboard },
  { title: "Team Review", href: "/manager", icon: Users },
  { title: "Check-ins", href: "/manager", icon: ClipboardCheck },
  { title: "Reports", href: "/reports", icon: BarChart3 },
];

const adminNav: NavItem[] = [
  { title: "Dashboard", href: "/admin", icon: LayoutDashboard },
  { title: "Users", href: "/admin/users", icon: Users },
  { title: "Cycles", href: "/admin/cycles", icon: Activity },
  { title: "Departments", href: "/admin/departments", icon: Building },
  { title: "Thrust Areas", href: "/admin/thrust", icon: TrendingUp },
  { title: "Audit Log", href: "/admin/audit", icon: Shield },
  { title: "Reports", href: "/reports", icon: BarChart3 },
];

export function AppSidebar() {
  const { user, logout } = useAuth();
  const location = useLocation();

  const navItems =
    user?.role === "employee"
      ? employeeNav
      : user?.role === "manager"
        ? managerNav
        : user?.role === "admin"
          ? adminNav
          : [];

  return (
    <Sidebar className="bg-slate-900/80 backdrop-blur-sm border-r border-slate-800">
      <SidebarHeader className="px-4 py-5 border-b border-slate-800">
        <div className="flex items-center gap-2.5">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-indigo-600">
            <Target className="h-4 w-4 text-white" />
          </div>
          <div>
            <p className="text-sm font-bold text-slate-100">AtomQuest</p>
            <p className="text-[10px] text-slate-500">Goal Tracking Portal</p>
          </div>
        </div>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel className="text-slate-500">Navigation</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {navItems.map((item) => {
                const isActive =
                  location.pathname === item.href ||
                  (item.href !== "/" && location.pathname.startsWith(item.href + "/"));
                return (
                  <SidebarMenuItem key={item.href + item.title}>
                    <SidebarMenuButton
                      asChild
                      isActive={isActive}
                      tooltip={item.title}
                    >
                      <NavLink to={item.href} className="flex items-center gap-3">
                        <item.icon className="h-4 w-4" />
                        <span>{item.title}</span>
                      </NavLink>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                );
              })}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter className="border-t border-slate-800 p-4">
        {user && (
          <div className="mb-3 px-2">
            <p className="text-xs text-slate-500">{user.name}</p>
            <p className="text-[10px] text-slate-600 capitalize">{user.role}</p>
          </div>
        )}
        <Button
          variant="ghost"
          className="w-full justify-start gap-3 text-slate-400 hover:text-red-400 hover:bg-slate-800/50"
          onClick={logout}
        >
          <LogOut className="h-4 w-4" />
          <span>Logout</span>
        </Button>
      </SidebarFooter>
    </Sidebar>
  );
}
