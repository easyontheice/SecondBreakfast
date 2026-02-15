import { open } from "@tauri-apps/plugin-dialog";
import { Folder } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import type { Rules } from "@/types";

interface SettingsViewProps {
  rules: Rules;
  onChange: (rules: Rules) => void;
  onChangeSortRoot: (path: string) => void;
}

export function SettingsView({ rules, onChange, onChangeSortRoot }: SettingsViewProps) {
  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Sort Folder</CardTitle>
          <CardDescription>Choose where drops are monitored and sorted.</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex gap-2">
            <Input value={rules.global.sortRoot} readOnly />
            <Button
              variant="secondary"
              onClick={async () => {
                const selected = await open({ directory: true, multiple: false });
                if (typeof selected === "string") {
                  onChangeSortRoot(selected);
                }
              }}
            >
              <Folder className="mr-2 h-4 w-4" />
              Change sortRoot
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Sorting Behavior</CardTitle>
        </CardHeader>
        <CardContent className="space-y-5">
          <div className="flex items-center justify-between">
            <span>Unknown extension goes to Misc</span>
            <Switch
              checked={rules.global.unknownGoesToMisc}
              onCheckedChange={(checked) =>
                onChange({ ...rules, global: { ...rules.global, unknownGoesToMisc: checked } })
              }
            />
          </div>

          <div className="flex items-center justify-between">
            <span>No extension goes to Misc</span>
            <Switch
              checked={rules.global.noExtensionGoesToMisc}
              onCheckedChange={(checked) =>
                onChange({ ...rules, global: { ...rules.global, noExtensionGoesToMisc: checked } })
              }
            />
          </div>

          <div>
            <p className="mb-2 text-sm">Collision policy</p>
            <Select
              value={rules.global.collisionPolicy}
              onValueChange={(value: "rename") =>
                onChange({ ...rules, global: { ...rules.global, collisionPolicy: value } })
              }
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="rename">Rename (file (1).ext)</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span>Min file age</span>
              <span>{rules.global.minFileAgeSeconds}s</span>
            </div>
            <Slider
              value={[rules.global.minFileAgeSeconds]}
              min={1}
              max={120}
              step={1}
              onValueChange={([value]) => onChange({ ...rules, global: { ...rules.global, minFileAgeSeconds: value } })}
            />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Protected Folders</CardTitle>
        </CardHeader>
        <CardContent className="text-sm text-muted-foreground">
          {rules.categories.map((category) => category.targetSubfolder).join(", ")}, {rules.misc.targetSubfolder}
        </CardContent>
      </Card>
    </div>
  );
}
