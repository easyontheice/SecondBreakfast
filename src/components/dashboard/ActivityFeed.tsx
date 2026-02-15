import { AlertTriangle, CheckCircle2, Info } from "lucide-react";
import type { ActivityItem } from "@/types";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

interface ActivityFeedProps {
  items: ActivityItem[];
}

function groupedByDay(items: ActivityItem[]) {
  const map = new Map<string, ActivityItem[]>();
  for (const item of items) {
    const day = new Date(item.at).toLocaleDateString();
    const bucket = map.get(day) ?? [];
    bucket.push(item);
    map.set(day, bucket);
  }
  return Array.from(map.entries());
}

export function ActivityFeed({ items }: ActivityFeedProps) {
  const grouped = groupedByDay(items.slice(0, 120));

  return (
    <Card className="h-full">
      <CardHeader>
        <CardTitle className="text-base">Live Activity Feed</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {items.length === 0 ? <p className="text-sm text-muted-foreground">No activity yet.</p> : null}
        {grouped.map(([day, entries]) => (
          <div key={day} className="space-y-3">
            <p className="text-xs uppercase tracking-wide text-muted-foreground">{day}</p>
            {entries.map((item) => {
              const icon = item.level === "error" ? AlertTriangle : item.level === "warn" ? Info : CheckCircle2;
              const Icon = icon;
              return (
                <div key={item.id} className="rounded-xl border border-border/70 bg-background/60 p-3">
                  <div className="flex items-start justify-between gap-3">
                    <div className="flex gap-2">
                      <Icon className="mt-0.5 h-4 w-4" />
                      <div>
                        <p className="text-sm">{item.message}</p>
                        {item.level === "error" ? (
                          <details className="mt-1 text-xs text-muted-foreground">
                            <summary className="cursor-pointer">Details</summary>
                            {item.sourcePath ? <p>from: {item.sourcePath}</p> : null}
                            {item.destinationPath ? <p>to: {item.destinationPath}</p> : null}
                          </details>
                        ) : (
                          <>
                            {item.sourcePath ? <p className="text-xs text-muted-foreground">from: {item.sourcePath}</p> : null}
                            {item.destinationPath ? <p className="text-xs text-muted-foreground">to: {item.destinationPath}</p> : null}
                          </>
                        )}
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <Badge>{item.level}</Badge>
                      <span className="text-xs text-muted-foreground">{new Date(item.at).toLocaleTimeString()}</span>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        ))}
      </CardContent>
    </Card>
  );
}
