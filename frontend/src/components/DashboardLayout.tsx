import { ReactNode } from "react";
import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";
import { AppSidebar } from "@/components/AppSidebar";
import { useAuth } from "@/contexts/AuthContext";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Menu } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useIsMobile } from "@/hooks/use-mobile";

interface DashboardLayoutProps {
  children: ReactNode;
}

export function DashboardLayout({ children }: DashboardLayoutProps) {
  const { user } = useAuth();
  const isMobile = useIsMobile();

  const initials = user?.name
    ? user.name
        .split(" ")
        .map((n) => n[0])
        .join("")
        .toUpperCase()
        .slice(0, 2)
    : "?";

  return (
    <SidebarProvider defaultOpen={!isMobile}>
      <div className="flex min-h-screen w-full bg-slate-950">
        <AppSidebar />
        <div className="flex flex-1 flex-col min-h-screen">
          <header className="sticky top-0 z-30 flex h-14 items-center justify-between border-b border-slate-800 bg-slate-950/80 backdrop-blur-sm px-4">
            <div className="flex items-center gap-2">
              <SidebarTrigger>
                <Menu className="h-4 w-4" />
              </SidebarTrigger>
            </div>
            <div className="flex items-center gap-3">
              {user && (
                <div className="flex items-center gap-2">
                  <span className="text-sm text-slate-400 hidden sm:inline">
                    {user.name}
                  </span>
                  <Avatar className="h-8 w-8">
                    <AvatarFallback className="bg-indigo-600 text-white text-xs">
                      {initials}
                    </AvatarFallback>
                  </Avatar>
                </div>
              )}
            </div>
          </header>
          <main className="flex-1 overflow-auto p-4 md:p-6 lg:p-8">
            {children}
          </main>
        </div>
      </div>
    </SidebarProvider>
  );
}
