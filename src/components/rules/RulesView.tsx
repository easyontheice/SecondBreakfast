import { type ComponentType, useMemo, useState } from "react";
import { Archive, Code, Database, Download, FileText, Film, Folder, Image, Music, Upload } from "lucide-react";
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

const categoryIcons: Record<string, ComponentType<{ className?: string }>> = {
  Documents: FileText,
  Images: Image,
  Video: Film,
  Audio: Music,
  Archives: Archive,
  Code,
  Executables: Folder,
  Data: Database
};

export function RulesView({ rules, onChange, onSave, onRevert, onExport, onImport }: RulesViewProps) {
  const [newExtByCategory, setNewExtByCategory] = useState<Record<string, string>>({});

  const totalExtensions = useMemo(
    () => rules.categories.reduce((sum, category) => sum + category.extensions.length, 0),
    [rules.categories]
  );

  return (
    <div className="space-y-6 pb-24">
      <div className="flex items-center justify-between">
        <h3 className="font-heading text-3xl font-semibold">Rules</h3>
        <Badge>{totalExtensions} extensions</Badge>
      </div>

      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        {rules.categories.map((category) => {
          const Icon = categoryIcons[category.name] ?? FileText;
          return (
            <Card key={category.id} className="bg-[linear-gradient(180deg,hsl(var(--card)),hsl(var(--card)/0.9))]">
              <CardHeader>
                <CardTitle className="flex items-center justify-between text-lg">
                  <span className="flex items-center gap-2">
                    <Icon className="h-4 w-4 text-[hsl(var(--primary))]" />
                    {category.name}
                  </span>
                  <Badge className="border-[hsl(var(--primary)/0.55)]">{category.extensions.length}</Badge>
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="flex flex-wrap gap-2">
                  {category.extensions.map((ext) => (
                    <button
                      key={ext}
                      className="rounded-full border border-[hsl(var(--primary)/0.35)] bg-[hsl(var(--secondary)/0.6)] px-2.5 py-1 text-xs transition hover:border-[hsl(var(--primary)/0.6)] hover:bg-[hsl(var(--primary)/0.18)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
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
          );
        })}
      </section>

      <footer className="fixed bottom-4 left-1/2 z-20 flex w-[min(900px,92vw)] -translate-x-1/2 items-center justify-between rounded-2xl border border-[hsl(var(--border)/0.9)] bg-[hsl(var(--card)/0.96)] p-3 shadow-soft backdrop-blur">
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
