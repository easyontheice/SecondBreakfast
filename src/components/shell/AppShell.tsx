import { useEffect, useMemo, useState } from "react";
import { NavLink, useLocation } from "react-router-dom";
import { LayoutDashboard, Pause, Play, Settings, Trash2, Wrench, X } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface AppShellProps {
  appVersion: string;
  watcherRunning: boolean;
  sortRoot: string;
  running: boolean;
  onRunNow: () => void;
  onDryRun: () => void;
  onToggleWatcher: () => void;
  children: React.ReactNode;
}

const links = [
  { to: "/", label: "Dashboard", icon: LayoutDashboard },
  { to: "/rules", label: "Rules", icon: Wrench },
  { to: "/cleanup", label: "Cleanup", icon: Trash2 },
  { to: "/settings", label: "Settings", icon: Settings }
];

const TAGLINE = "Because files deserve proper placement before elevenses.";

function useVersionedTagline(appVersion: string) {
  const [visible, setVisible] = useState(false);
  const storageKey = useMemo(() => `secondbreakfast_tagline_dismissed_v${appVersion}`, [appVersion]);

  useEffect(() => {
    if (!appVersion) {
      return;
    }

    if (localStorage.getItem(storageKey) !== "true") {
      setVisible(true);
    }
  }, [appVersion, storageKey]);

  useEffect(() => {
    if (!visible) {
      return;
    }

    const dismiss = () => {
      localStorage.setItem(storageKey, "true");
      setVisible(false);
    };

    const timer = window.setTimeout(dismiss, 8000);
    const interaction = () => dismiss();

    window.addEventListener("pointerdown", interaction, { once: true, capture: true });
    window.addEventListener("keydown", interaction, { once: true });

    return () => {
      window.clearTimeout(timer);
      window.removeEventListener("pointerdown", interaction, true);
      window.removeEventListener("keydown", interaction);
    };
  }, [visible, storageKey]);

  return { visible, dismiss: () => setVisible(false), storageKey };
}

export function AppShell({
  appVersion,
  watcherRunning,
  sortRoot,
  running,
  onRunNow,
  onDryRun,
  onToggleWatcher,
  children
}: AppShellProps) {
  const location = useLocation();
  const { visible, dismiss, storageKey } = useVersionedTagline(appVersion);
  const showBanner = visible;

  const closeBanner = () => {
    localStorage.setItem(storageKey, "true");
    dismiss();
  };

  return (
    <div className="min-h-screen bg-[radial-gradient(circle_at_15%_0%,hsl(var(--primary)/0.12),transparent_45%),radial-gradient(circle_at_85%_100%,hsl(var(--accent)/0.16),transparent_45%)]">
      <div className="mx-auto grid max-w-[1440px] gap-6 px-6 py-7 md:grid-cols-[270px_1fr]">
        <aside className="rounded-2xl border border-border bg-gradient-to-b from-[#121810] to-[#0f150f] p-4 shadow-soft">
          <div className="mb-4 border-b border-border/70 pb-4">
            <h1 className="font-heading text-3xl font-semibold leading-none tracking-tight text-foreground">
              SecondBreakfast
            </h1>
            <p className="mt-1 text-xs tracking-[0.22em] text-muted-foreground">drop-zone sorter</p>
          </div>

          <nav className="space-y-1.5">
            {links.map((link) => {
              const Icon = link.icon;
              return (
                <NavLink
                  key={link.to}
                  to={link.to}
                  className={({ isActive }) =>
                    cn(
                      "flex items-center gap-3 rounded-xl px-3 py-2.5 text-sm text-muted-foreground transition-all hover:bg-[hsl(var(--accent)/0.2)] hover:text-foreground",
                      isActive &&
                        "bg-[linear-gradient(90deg,hsl(var(--accent)/0.35),hsl(var(--primary)/0.28))] text-foreground shadow-[inset_0_1px_0_hsl(var(--foreground)/0.16)]"
                    )
                  }
                >
                  <Icon className="h-4 w-4" />
                  {link.label}
                </NavLink>
              );
            })}
          </nav>

          <div className="mt-6 rounded-xl border border-[hsl(var(--primary)/0.4)] bg-[linear-gradient(180deg,hsl(var(--foreground)/0.1),hsl(var(--foreground)/0.04))] p-3 text-xs text-muted-foreground">
            <p className="font-semibold uppercase tracking-[0.16em] text-foreground/90">Watching</p>
            <p className="mt-1 truncate font-medium text-foreground/90">{sortRoot}</p>
          </div>
        </aside>

        <div className="space-y-5">
          <header className="rounded-2xl border border-[hsl(var(--primary)/0.45)] bg-[linear-gradient(160deg,hsl(var(--card)),hsl(var(--background)))] p-4 shadow-soft">
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div>
                <p className="text-[11px] uppercase tracking-[0.2em] text-muted-foreground">SecondBreakfast Desk</p>
                <h2 className="font-heading text-3xl font-semibold leading-none tracking-tight text-foreground">
                  Shire Sorting Board
                </h2>
              </div>
              <div className="flex flex-wrap items-center gap-2">
                <Badge className={watcherRunning ? "border-[hsl(var(--primary)/0.5)] bg-[hsl(var(--primary)/0.2)] text-foreground" : ""}>
                  {watcherRunning ? "Watcher Running" : "Watcher Paused"}
                </Badge>
                <Button variant="secondary" disabled={running} onClick={onDryRun}>
                  Dry Run
                </Button>
                <Button disabled={running} onClick={onRunNow}>
                  Run Now
                </Button>
                <Button variant="outline" disabled={running} onClick={onToggleWatcher}>
                  {watcherRunning ? <Pause className="mr-2 h-4 w-4" /> : <Play className="mr-2 h-4 w-4" />}
                  {watcherRunning ? "Pause" : "Resume"}
                </Button>
              </div>
            </div>
          </header>

          <div
            className={cn(
              "overflow-hidden transition-all duration-200",
              showBanner ? "max-h-48 opacity-100" : "max-h-0 opacity-0"
            )}
          >
            <div
              className={cn(
                "relative rounded-2xl border border-[hsl(var(--primary)/0.55)] bg-[linear-gradient(180deg,#2b2317,#1f1912)] p-4 shadow-soft",
                location.pathname === "/" || location.pathname === "/rules" ? "" : "opacity-95"
              )}
            >
              <p className="pr-10 text-sm leading-relaxed text-foreground">{TAGLINE}</p>
              <button
                type="button"
                onClick={closeBanner}
                className="absolute right-3 top-3 rounded-md border border-[hsl(var(--primary)/0.35)] bg-background/40 p-1 text-muted-foreground transition hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                aria-label="Dismiss welcome banner"
              >
                <X className="h-4 w-4" />
              </button>
            </div>
          </div>

          <main>{children}</main>
        </div>
      </div>
    </div>
  );
}
