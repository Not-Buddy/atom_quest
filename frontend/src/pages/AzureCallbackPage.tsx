import { useEffect, useState } from "react";
import { useNavigate, Link } from "react-router-dom";
import { useAuth } from "@/contexts/AuthContext";
import { azureCallback } from "@/lib/api";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Loader2, AlertCircle, ShieldCheck } from "lucide-react";

export default function AzureCallbackPage() {
  const { loginWithToken } = useAuth();
  const navigate = useNavigate();
  const [error, setError] = useState("");
  const [status, setStatus] = useState<"loading" | "success" | "error">("loading");

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const code = params.get("code");

    if (!code) {
      setError("Missing authorization code. Please try signing in again.");
      setStatus("error");
      return;
    }

    const state = params.get("state") || undefined;

    (async () => {
      try {
        const data = await azureCallback(code, state);
        if (!data.token || !data.user) {
          throw new Error("Invalid response from SSO provider");
        }
        loginWithToken(data.token, data.user);
        setStatus("success");

        const routes: Record<string, string> = {
          employee: "/employee",
          manager: "/manager",
          admin: "/admin",
        };
        const target = routes[data.user.role] || "/employee";
        navigate(target, { replace: true });
      } catch (err) {
        setError(err instanceof Error ? err.message : "SSO login failed. Please try again.");
        setStatus("error");
      }
    })();
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <div className="min-h-screen bg-slate-950 flex flex-col items-center justify-center px-4">
      {/* Background blurs */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute -top-40 -right-40 h-[500px] w-[500px] rounded-full bg-indigo-600/10 blur-3xl" />
        <div className="absolute -bottom-40 -left-40 h-[500px] w-[500px] rounded-full bg-indigo-400/5 blur-3xl" />
      </div>

      <div className="w-full max-w-md relative z-10 text-center">
        {/* Microsoft icon */}
        <div className="inline-flex items-center justify-center h-16 w-16 rounded-2xl bg-slate-800 mb-6">
          <svg className="h-8 w-8" viewBox="0 0 21 21" xmlns="http://www.w3.org/2000/svg">
            <rect x="1" y="1" width="9" height="9" fill="#f25022" />
            <rect x="11" y="1" width="9" height="9" fill="#7fba00" />
            <rect x="1" y="11" width="9" height="9" fill="#00a4ef" />
            <rect x="11" y="11" width="9" height="9" fill="#ffb900" />
          </svg>
        </div>

        {status === "loading" && (
          <>
            <Loader2 className="h-8 w-8 animate-spin text-indigo-400 mx-auto mb-4" />
            <h2 className="text-xl font-semibold text-slate-200 mb-2">
              Signing you in…
            </h2>
            <p className="text-slate-400 text-sm">
              Completing Microsoft SSO authentication
            </p>
          </>
        )}

        {status === "success" && (
          <>
            <ShieldCheck className="h-8 w-8 text-green-400 mx-auto mb-4" />
            <h2 className="text-xl font-semibold text-green-400 mb-2">
              Signed in successfully
            </h2>
            <p className="text-slate-400 text-sm">
              Redirecting to your dashboard…
            </p>
          </>
        )}

        {status === "error" && (
          <>
            <Alert variant="destructive" className="mb-6 bg-red-950/50 border-red-800 text-left">
              <AlertCircle className="h-4 w-4 mt-0.5" />
              <AlertDescription className="ml-2">{error}</AlertDescription>
            </Alert>

            <Link to="/login">
              <Button variant="outline" className="border-slate-700 text-slate-300 hover:bg-slate-800">
                Back to Login
              </Button>
            </Link>
          </>
        )}
      </div>
    </div>
  );
}
