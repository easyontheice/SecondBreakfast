import { Folder, Undo } from "lucide-react";
import { ActivityFeed } from "@/components/dashboard/ActivityFeed";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { ActivityItem, RunResult } from "@/types";

interface DashboardViewProps {
  sortRoot: string;
  activity: ActivityItem[];
  lastRun: RunResult | null;
  progress: { moved: number; skipped: number; errors: number };
  onChangeSortRoot: () => void;
  onUndo: () => void;
  running: boolean;
}

export function DashboardView({
  sortRoot,
  activity,
  lastRun,
  progress,
  onChangeSortRoot,
  onUndo,
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

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle className="text-xl">Watching: {sortRoot}</CardTitle>
            <p className="text-sm text-muted-foreground">Drop files or folders in your sortRoot and the app will auto-sort.</p>
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
      </Card>

      <section className="grid gap-4 md:grid-cols-4">
        {stats.map((item) => (
          <Card key={item.label}>
            <CardContent className="pt-6">
              <p className="text-xs uppercase tracking-wide text-muted-foreground">{item.label}</p>
              <p className="mt-1 text-2xl font-semibold">{item.value}</p>
            </CardContent>
          </Card>
        ))}
      </section>

      <ActivityFeed items={activity} />
    </div>
  );
}
