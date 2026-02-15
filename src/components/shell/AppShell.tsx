import { NavLink } from "react-router-dom";
import { LayoutDashboard, Pause, Play, Settings, Trash2, Wrench } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
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
  return (
    <div className="min-h-screen">
      <div className="mx-auto grid max-w-[1400px] gap-6 p-6 md:grid-cols-[240px_1fr]">
        <aside className="rounded-2xl border border-border/80 bg-card/80 p-4 shadow-soft">
          <div className="mb-4">
            <h1 className="text-xl font-bold tracking-tight">Folder Goblin</h1>
            <p className="text-xs text-muted-foreground">by easyontheice</p>
          </div>
          <nav className="space-y-1">
            {links.map((link) => {
              const Icon = link.icon;
              return (
                <NavLink
                  key={link.to}
                  to={link.to}
                  className={({ isActive }) =>
                    cn(
                      "flex items-center gap-3 rounded-xl px-3 py-2 text-sm text-muted-foreground transition-colors hover:bg-muted/70 hover:text-foreground",
                      isActive && "bg-primary text-primary-foreground hover:bg-primary"
                    )
                  }
                >
                  <Icon className="h-4 w-4" />
                  {link.label}
                </NavLink>
              );
            })}
          </nav>
          <div className="mt-6 rounded-xl border border-border/80 bg-background/60 p-3 text-xs text-muted-foreground">
            <p className="font-medium text-foreground">Watching</p>
            <p className="truncate">{sortRoot}</p>
          </div>
        </aside>

        <div className="space-y-6">
          <header className="flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-border/80 bg-card/80 p-4 shadow-soft">
            <div>
              <p className="text-xs uppercase tracking-[0.2em] text-muted-foreground">SortRoot Control</p>
              <h2 className="text-2xl font-semibold tracking-tight">Drop-zone sorter</h2>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <Badge className={watcherRunning ? "bg-primary/20 text-primary" : ""}>
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
          </header>
          <main>{children}</main>
        </div>
      </div>
    </div>
  );
}
