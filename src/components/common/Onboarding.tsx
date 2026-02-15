import { open } from "@tauri-apps/plugin-dialog";
import { FolderOpen } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

interface OnboardingProps {
  sortRoot: string;
  onPick: (path: string) => void;
  onStart: () => void;
}

const folders = [
  "Documents",
  "Images",
  "Video",
  "Audio",
  "Archives",
  "Code",
  "Executables",
  "Data",
  "Misc"
];

export function Onboarding({ sortRoot, onPick, onStart }: OnboardingProps) {
  return (
    <div className="mx-auto flex min-h-screen w-full max-w-3xl items-center p-6">
      <Card className="w-full">
        <CardHeader>
          <CardTitle className="text-2xl">Choose your Sort Folder</CardTitle>
          <CardDescription>
            SortRoot will watch this folder and auto-sort anything dropped into it.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="rounded-xl border border-border/80 bg-background/70 p-3 text-sm">{sortRoot}</div>
          <Button
            variant="secondary"
            onClick={async () => {
              const selected = await open({ directory: true, multiple: false });
              if (typeof selected === "string") {
                onPick(selected);
              }
            }}
          >
            <FolderOpen className="mr-2 h-4 w-4" />
            Pick folder
          </Button>

          <div className="rounded-xl border border-border/80 bg-background/60 p-4">
            <p className="mb-2 text-sm text-muted-foreground">These subfolders will be created:</p>
            <div className="flex flex-wrap gap-2 text-xs">
              {folders.map((folder) => (
                <span key={folder} className="rounded-full border border-border px-2 py-1">
                  {folder}
                </span>
              ))}
            </div>
          </div>

          <Button className="w-full" onClick={onStart}>
            Start Watching
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
