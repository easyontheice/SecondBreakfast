import { useEffect, useState } from "react";
import { NavLink } from "react-router-dom";
import { LayoutDashboard, Pause, Play, Settings, Trash2, Wrench } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { cn } from "@/lib/utils";

interface AppShellProps {
  watcherRunning: boolean;
  sortRoot: string;
  running: boolean;
  onRunNow: () => void;
  onDryRun: () => void;
  onToggleWatcher: () => void;
  children: React.ReactNode;
}

type ThemeName = "shire" | "frost" | "midnight" | "parchment";

const THEME_STORAGE_KEY = "theme";
const DEFAULT_THEME: ThemeName = "shire";

const themeOptions: Array<{ value: ThemeName; label: string }> = [
  { value: "shire", label: "Shire" },
  { value: "frost", label: "Frost" },
  { value: "midnight", label: "Midnight" },
  { value: "parchment", label: "Parchment" }
];

function isThemeName(value: string | null | undefined): value is ThemeName {
  return value === "shire" || value === "frost" || value === "midnight" || value === "parchment";
}

function resolveInitialTheme(): ThemeName {
  if (typeof window === "undefined") {
    return DEFAULT_THEME;
  }

  const htmlTheme = document.documentElement.dataset.theme;
  if (isThemeName(htmlTheme)) {
    return htmlTheme;
  }

  try {
    const stored = window.localStorage.getItem(THEME_STORAGE_KEY);
    if (isThemeName(stored)) {
      return stored;
    }
  } catch {
    // Ignore storage read failures and fall back to default theme.
  }

  return DEFAULT_THEME;
}

const links = [
  { to: "/", label: "Dashboard", icon: LayoutDashboard },
  { to: "/rules", label: "Rules", icon: Wrench },
  { to: "/cleanup", label: "Cleanup", icon: Trash2 },
  { to: "/settings", label: "Settings", icon: Settings }
];

export function AppShell({
  watcherRunning,
  sortRoot,
  running,
  onRunNow,
  onDryRun,
  onToggleWatcher,
  children
}: AppShellProps) {
  const [theme, setTheme] = useState<ThemeName>(() => resolveInitialTheme());

  useEffect(() => {
    document.documentElement.dataset.theme = theme;

    try {
      window.localStorage.setItem(THEME_STORAGE_KEY, theme);
    } catch {
      // Ignore storage write failures.
    }
  }, [theme]);

  return (
    <div className="min-h-screen bg-[radial-gradient(circle_at_15%_0%,hsl(var(--primary)/0.12),transparent_45%),radial-gradient(circle_at_85%_100%,hsl(var(--accent)/0.16),transparent_45%)]">
      <div className="mx-auto grid max-w-[1440px] gap-6 px-6 py-7 md:grid-cols-[270px_1fr]">
        <aside className="rounded-2xl border border-border bg-gradient-to-b from-[hsl(var(--surface))] to-[hsl(var(--background))] p-4 shadow-soft">
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
              <div className="flex flex-wrap items-center">
                <h2 className="font-heading text-3xl font-semibold leading-none tracking-tight text-foreground">Sorting Board</h2>
                <div className="ml-0 flex items-center gap-2 sm:ml-[120px]">
                  <span className="text-xs font-semibold uppercase tracking-[0.16em] text-muted-foreground">Theme</span>
                  <Select value={theme} onValueChange={(value) => setTheme(isThemeName(value) ? value : DEFAULT_THEME)}>
                    <SelectTrigger className="h-9 w-[170px] bg-[hsl(var(--surface)/0.8)]">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {themeOptions.map((option) => (
                        <SelectItem key={option.value} value={option.value}>
                          {option.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
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

          <main>{children}</main>
        </div>
      </div>
    </div>
  );
}
