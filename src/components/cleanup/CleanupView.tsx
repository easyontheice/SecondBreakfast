import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Slider } from "@/components/ui/slider";
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
        <CardTitle>Cleanup</CardTitle>
        <CardDescription>
          Delete empty remnants only. Category folders and sortRoot are always protected. Mode is locked to Trash.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="flex items-center justify-between">
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

        <div className="space-y-2">
          <div className="flex items-center justify-between text-sm">
            <span>Folder min age</span>
            <span>{rules.global.cleanupEmptyFolders.minAgeSeconds}s</span>
          </div>
          <Slider
            value={[rules.global.cleanupEmptyFolders.minAgeSeconds]}
            min={10}
            max={600}
            step={5}
            onValueChange={([value]) =>
              onChange({
                ...rules,
                global: {
                  ...rules.global,
                  cleanupEmptyFolders: {
                    ...rules.global.cleanupEmptyFolders,
                    minAgeSeconds: value
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
