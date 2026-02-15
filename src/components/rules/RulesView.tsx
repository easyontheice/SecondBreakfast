import { useMemo, useState } from "react";
import { Download, Upload } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import type { Rules } from "@/types";

interface RulesViewProps {
  rules: Rules;
  onChange: (rules: Rules) => void;
  onSave: () => void;
  onRevert: () => void;
  onExport: () => void;
  onImport: (payload: string) => void;
}

export function RulesView({ rules, onChange, onSave, onRevert, onExport, onImport }: RulesViewProps) {
  const [newExtByCategory, setNewExtByCategory] = useState<Record<string, string>>({});

  const totalExtensions = useMemo(
    () => rules.categories.reduce((sum, category) => sum + category.extensions.length, 0),
    [rules.categories]
  );

  return (
    <div className="space-y-6 pb-24">
      <div className="flex items-center justify-between">
        <h3 className="text-xl font-semibold">Rules</h3>
        <Badge>{totalExtensions} extensions</Badge>
      </div>

      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        {rules.categories.map((category) => (
          <Card key={category.id}>
            <CardHeader>
              <CardTitle className="flex items-center justify-between text-base">
                {category.name}
                <Badge>{category.extensions.length}</Badge>
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <div className="flex flex-wrap gap-2">
                {category.extensions.map((ext) => (
                  <button
                    key={ext}
                    className="rounded-full border border-border bg-secondary px-2.5 py-1 text-xs"
                    onClick={() => {
                      onChange({
                        ...rules,
                        categories: rules.categories.map((item) =>
                          item.id === category.id
                            ? {
                                ...item,
                                extensions: item.extensions.filter((entry) => entry !== ext)
                              }
                            : item
                        )
                      });
                    }}
                  >
                    .{ext}
                  </button>
                ))}
              </div>

              <div className="flex gap-2">
                <Input
                  placeholder="Add extension"
                  value={newExtByCategory[category.id] ?? ""}
                  onChange={(event) =>
                    setNewExtByCategory((prev) => ({
                      ...prev,
                      [category.id]: event.target.value
                    }))
                  }
                />
                <Button
                  variant="secondary"
                  onClick={() => {
                    const value = (newExtByCategory[category.id] ?? "").trim().replace(/^\./, "").toLowerCase();
                    if (!value) {
                      return;
                    }

                    onChange({
                      ...rules,
                      categories: rules.categories.map((item) =>
                        item.id === category.id && !item.extensions.includes(value)
                          ? { ...item, extensions: [...item.extensions, value] }
                          : item
                      )
                    });
                    setNewExtByCategory((prev) => ({ ...prev, [category.id]: "" }));
                  }}
                >
                  Add
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
      </section>

      <footer className="fixed bottom-4 left-1/2 z-20 flex w-[min(860px,92vw)] -translate-x-1/2 items-center justify-between rounded-2xl border border-border bg-card/95 p-3 shadow-soft backdrop-blur">
        <div className="text-sm text-muted-foreground">Save rules to apply extension changes.</div>
        <div className="flex gap-2">
          <input
            type="file"
            accept="application/json"
            className="hidden"
            id="rules-import"
            onChange={async (event) => {
              const file = event.target.files?.[0];
              if (!file) {
                return;
              }
              onImport(await file.text());
              event.target.value = "";
            }}
          />
          <Button variant="outline" onClick={onExport}>
            <Download className="mr-2 h-4 w-4" />
            Export
          </Button>
          <Button variant="outline" onClick={() => document.getElementById("rules-import")?.click()}>
            <Upload className="mr-2 h-4 w-4" />
            Import
          </Button>
          <Button variant="secondary" onClick={onRevert}>
            Revert
          </Button>
          <Button onClick={onSave}>Save</Button>
        </div>
      </footer>
    </div>
  );
}
