import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import type { Rules } from "@/types";

interface CleanupViewProps {
  rules: Rules;
  onChange: (next: Rules) => void;
}

export function CleanupView({ rules, onChange }: CleanupViewProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="font-heading text-3xl">Cleanup</CardTitle>
        <CardDescription>
          Empty folder remnants are removed immediately after each completed run. Category folders and your Sort Folder are always protected.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-5">
        <div className="flex items-center justify-between rounded-xl border border-border/70 bg-background/50 px-4 py-3">
          <span>Cleanup enabled</span>
          <Switch
            checked={rules.global.cleanupEmptyFolders.enabled}
            onCheckedChange={(checked) =>
              onChange({
                ...rules,
                global: {
                  ...rules.global,
                  cleanupEmptyFolders: {
                    ...rules.global.cleanupEmptyFolders,
                    enabled: checked
                  }
                }
              })
            }
          />
        </div>

        <p className="text-sm text-muted-foreground">Deletion mode: Trash / Recycle Bin (permanent delete is disabled).</p>
      </CardContent>
    </Card>
  );
}
