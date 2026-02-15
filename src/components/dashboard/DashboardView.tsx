import { Folder, Undo } from "lucide-react";
import { ActivityFeed } from "@/components/dashboard/ActivityFeed";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import type { ActivityItem, RunResult } from "@/types";

interface DashboardViewProps {
  sortRoot: string;
  activity: ActivityItem[];
  lastRun: RunResult | null;
  progress: { moved: number; skipped: number; errors: number };
  onChangeSortRoot: () => void;
  onUndo: () => void;
  onClearActivity: () => void;
  running: boolean;
}

export function DashboardView({
  sortRoot,
  activity,
  lastRun,
  progress,
  onChangeSortRoot,
  onUndo,
  onClearActivity,
  running
}: DashboardViewProps) {
  const stats = [
    { label: "Moved", value: lastRun?.moved ?? progress.moved },
    { label: "Skipped", value: lastRun?.skipped ?? progress.skipped },
    { label: "Errors", value: lastRun?.errors ?? progress.errors },
    {
      label: "Last run",
      value: lastRun?.finishedAt ? new Date(lastRun.finishedAt).toLocaleTimeString() : "-"
    }
  ];

  const totalProcessed = (lastRun?.moved ?? 0) + (lastRun?.skipped ?? 0);

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle className="font-heading text-3xl">Watching: {sortRoot}</CardTitle>
            <p className="text-sm text-muted-foreground">Drop files or folders into your Sort Folder and the app will auto-sort.</p>
          </div>
          <div className="flex gap-2">
            <Button variant="outline" onClick={onChangeSortRoot}>
              <Folder className="mr-2 h-4 w-4" />
              Change
            </Button>
            <Button variant="secondary" disabled={running} onClick={onUndo}>
              <Undo className="mr-2 h-4 w-4" />
              Undo Last Run
            </Button>
          </div>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="rounded-xl border border-[hsl(var(--primary)/0.45)] bg-[hsl(var(--primary)/0.1)] px-3 py-2 text-sm">
            {lastRun ? `${lastRun.moved}/${totalProcessed} files moved in the last run` : "No completed runs yet."}
          </div>
          <div className="flex items-center justify-between text-xs text-muted-foreground">
            <span>{running ? "Sorting in progress" : "Idle"}</span>
            <span>
              {progress.moved} moved / {progress.skipped} skipped / {progress.errors} errors
            </span>
          </div>
          <Progress value={running ? undefined : 100} />
        </CardContent>
      </Card>

      <section className="grid gap-4 md:grid-cols-4">
        {stats.map((item) => (
          <Card key={item.label}>
            <CardContent className="pt-6">
              <p className="text-xs uppercase tracking-[0.16em] text-muted-foreground">{item.label}</p>
              <p className="mt-1 font-heading text-3xl font-semibold leading-none">{item.value}</p>
            </CardContent>
          </Card>
        ))}
      </section>

      <ActivityFeed items={activity} onClear={onClearActivity} />

      <section className="rounded-2xl border border-[hsl(var(--primary)/0.5)] bg-[linear-gradient(180deg,hsl(var(--foreground)/0.09),hsl(var(--foreground)/0.04))] px-5 py-4 text-center shadow-soft">
        <p className="font-heading text-2xl leading-tight text-foreground/95">
          Because files deserve proper placement before elevenses.
        </p>
      </section>
    </div>
  );
}
