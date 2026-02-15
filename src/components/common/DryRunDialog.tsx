import { AlertTriangle } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger
} from "@/components/ui/dialog";
import type { PlanPreview } from "@/types";

interface DryRunDialogProps {
  plan: PlanPreview | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function DryRunDialog({ plan, open, onOpenChange }: DryRunDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogTrigger asChild>
        <span />
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Dry Run Preview</DialogTitle>
          <DialogDescription>
            Planned moves by destination bucket. Conflicts will be renamed using the collision policy.
          </DialogDescription>
        </DialogHeader>

        {!plan ? (
          <p className="text-sm text-muted-foreground">No preview loaded.</p>
        ) : (
          <div className="max-h-[60vh] space-y-4 overflow-y-auto pr-2">
            <div className="grid gap-2 rounded-xl border border-border/80 bg-background/70 p-3 text-sm sm:grid-cols-4">
              <div>
                <p className="text-muted-foreground">Candidates</p>
                <p className="font-semibold">{plan.totalCandidates}</p>
              </div>
              <div>
                <p className="text-muted-foreground">Moves</p>
                <p className="font-semibold">{plan.moveCount}</p>
              </div>
              <div>
                <p className="text-muted-foreground">Skips</p>
                <p className="font-semibold">{plan.skipCount}</p>
              </div>
              <div>
                <p className="text-muted-foreground">Conflicts</p>
                <p className="font-semibold">{plan.potentialConflicts}</p>
              </div>
            </div>

            {plan.potentialConflicts > 0 ? (
              <div className="rounded-xl border border-destructive/50 bg-destructive/10 p-3 text-sm">
                <p className="flex items-center gap-2 font-medium text-destructive">
                  <AlertTriangle className="h-4 w-4" />
                  Rename collisions detected
                </p>
              </div>
            ) : null}

            {plan.grouped.map((group) => (
              <div key={group.category} className="rounded-xl border border-border/70 bg-background/60 p-3">
                <p className="mb-2 text-sm font-semibold">
                  {group.category} ({group.count})
                </p>
                <div className="space-y-2">
                  {group.entries.slice(0, 30).map((entry) => (
                    <div key={`${entry.sourcePath}-${entry.destinationPath}`} className="rounded-lg border border-border/60 p-2 text-xs">
                      <p className="truncate text-muted-foreground">{entry.sourcePath}</p>
                      <p className="truncate">{entry.destinationPath}</p>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}

        <div className="mt-4 flex justify-end">
          <Button variant="secondary" onClick={() => onOpenChange(false)}>
            Close
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
