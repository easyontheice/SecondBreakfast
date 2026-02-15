import { useEffect, useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { HashRouter, Route, Routes } from "react-router-dom";
import { toast } from "sonner";
import { CleanupView } from "@/components/cleanup/CleanupView";
import { DryRunDialog } from "@/components/common/DryRunDialog";
import { Onboarding } from "@/components/common/Onboarding";
import { DashboardView } from "@/components/dashboard/DashboardView";
import { RulesView } from "@/components/rules/RulesView";
import { SettingsView } from "@/components/settings/SettingsView";
import { AppShell } from "@/components/shell/AppShell";
import {
  dryRun,
  getRules,
  onRunLog,
  onRunProgress,
  onWatcherStatus,
  runNow,
  setRules as saveRules,
  setSortRoot,
  startWatcher,
  stopWatcher,
  undoLastRun,
  watcherStatus
} from "@/lib/api";
import type { ActivityItem, PlanPreview, Rules, RunResult } from "@/types";

function createActivity(
  partial: Omit<ActivityItem, "id" | "at">,
  at = new Date().toISOString()
): ActivityItem {
  return {
    id: `${Date.now()}-${Math.random().toString(16).slice(2)}`,
    at,
    ...partial
  };
}

function App() {
  const [rules, setRules] = useState<Rules | null>(null);
  const [draftRules, setDraftRules] = useState<Rules | null>(null);
  const [loading, setLoading] = useState(true);
  const [running, setRunning] = useState(false);
  const [watcherRunning, setWatcherRunning] = useState(false);
  const [lastRun, setLastRun] = useState<RunResult | null>(null);
  const [progress, setProgress] = useState({ moved: 0, skipped: 0, errors: 0 });
  const [activity, setActivity] = useState<ActivityItem[]>([]);
  const [dryPlan, setDryPlan] = useState<PlanPreview | null>(null);
  const [dryOpen, setDryOpen] = useState(false);
  const [onboarding, setOnboarding] = useState(false);

  useEffect(() => {
    let active = true;

    async function boot() {
      try {
        const [nextRules, status] = await Promise.all([getRules(), watcherStatus()]);
        if (!active) {
          return;
        }

        setRules(nextRules);
        setDraftRules(nextRules);
        setWatcherRunning(status.running);

        const done = localStorage.getItem("secondbreakfast.onboarded") === "true";
        setOnboarding(!done);
      } catch (error) {
        toast.error(`Failed to load app state: ${String(error)}`);
      } finally {
        if (active) {
          setLoading(false);
        }
      }
    }

    void boot();

    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    let disposed = false;

    async function bind() {
      const unlistenProgress = await onRunProgress((payload) => {
        setProgress({ moved: payload.moved, skipped: payload.skipped, errors: payload.errors });
      });

      const unlistenLog = await onRunLog((payload) => {
        if (payload.level === "info") {
          return;
        }
        setActivity((prev) => [
          createActivity({
            level: payload.level,
            message: payload.message
          }),
          ...prev
        ]);
      });

      const unlistenWatcher = await onWatcherStatus((payload) => {
        setWatcherRunning(payload.running);
      });

      if (disposed) {
        unlistenProgress();
        unlistenLog();
        unlistenWatcher();
      }

      return () => {
        unlistenProgress();
        unlistenLog();
        unlistenWatcher();
      };
    }

    const cleanupPromise = bind();

    return () => {
      disposed = true;
      void cleanupPromise.then((cleanup) => cleanup?.());
    };
  }, []);

  const sortRoot = useMemo(() => rules?.global.sortRoot ?? "", [rules]);

  if (loading || !rules || !draftRules) {
    return (
      <div className="grid min-h-screen place-items-center text-sm text-muted-foreground">
        Loading SecondBreakfast...
      </div>
    );
  }

  const liveSetRules = async (nextRules: Rules) => {
    setDraftRules(nextRules);
    await saveRules(nextRules);
    setRules(nextRules);
  };

  const handleRunNow = async () => {
    try {
      setRunning(true);
      toast.info("Run started");
      const result = await runNow();
      setLastRun(result);
      setProgress({ moved: result.moved, skipped: result.skipped, errors: result.errors });
      const totalProcessed = result.moved + result.skipped;
      setActivity((prev) => [
        createActivity({
          level: result.errors > 0 ? "warn" : "info",
          message: `${result.moved}/${totalProcessed} files moved`,
          sourcePath: `session ${result.sessionId}`
        }),
        ...prev
      ]);
      toast.success(`Run complete: ${result.moved} moved`);
    } catch (error) {
      toast.error(`Run failed: ${String(error)}`);
    } finally {
      setRunning(false);
    }
  };

  const handleDryRun = async () => {
    try {
      const plan = await dryRun();
      setDryPlan(plan);
      setDryOpen(true);
    } catch (error) {
      toast.error(`Dry run failed: ${String(error)}`);
    }
  };

  const handleUndo = async () => {
    try {
      const result = await undoLastRun();
      toast.success(`Undo complete: ${result.restored} restored`);
      setActivity((prev) => [
        createActivity({
          level: result.errors > 0 ? "warn" : "info",
          message: `Undo: ${result.restored} restored, ${result.skipped} skipped`
        }),
        ...prev
      ]);
    } catch (error) {
      toast.error(`Undo failed: ${String(error)}`);
    }
  };

  const handleWatcherToggle = async () => {
    try {
      if (watcherRunning) {
        await stopWatcher();
      } else {
        await startWatcher();
      }
      const status = await watcherStatus();
      setWatcherRunning(status.running);
    } catch (error) {
      toast.error(`Watcher update failed: ${String(error)}`);
    }
  };

  const handleSortRootChange = async (path?: string) => {
    try {
      const nextPath = path ?? (await open({ directory: true, multiple: false }));
      if (typeof nextPath !== "string") {
        return;
      }
      await setSortRoot(nextPath);
      const updated = await getRules();
      setRules(updated);
      setDraftRules(updated);
      toast.success("Sort folder updated");
    } catch (error) {
      toast.error(`Failed to change sort folder: ${String(error)}`);
    }
  };

  if (onboarding) {
    return (
      <Onboarding
        sortRoot={sortRoot}
        onPick={(path) => void handleSortRootChange(path)}
        onStart={async () => {
          await startWatcher();
          localStorage.setItem("secondbreakfast.onboarded", "true");
          setOnboarding(false);
          toast.success("Watcher started");
        }}
      />
    );
  }

  return (
    <>
      <HashRouter>
        <AppShell
          watcherRunning={watcherRunning}
          sortRoot={sortRoot}
          running={running}
          onRunNow={() => void handleRunNow()}
          onDryRun={() => void handleDryRun()}
          onToggleWatcher={() => void handleWatcherToggle()}
        >
          <Routes>
            <Route
              path="/"
              element={
                <DashboardView
                  sortRoot={sortRoot}
                  activity={activity}
                  lastRun={lastRun}
                  progress={progress}
                  onChangeSortRoot={() => void handleSortRootChange()}
                  onUndo={() => void handleUndo()}
                  onClearActivity={() => setActivity([])}
                  running={running}
                />
              }
            />
            <Route
              path="/rules"
              element={
                <RulesView
                  rules={draftRules}
                  onChange={setDraftRules}
                  onSave={async () => {
                    try {
                      await saveRules(draftRules);
                      setRules(draftRules);
                      toast.success("Rules saved");
                    } catch (error) {
                      toast.error(`Failed to save rules: ${String(error)}`);
                    }
                  }}
                  onRevert={() => setDraftRules(rules)}
                  onExport={() => {
                    const blob = new Blob([JSON.stringify(draftRules, null, 2)], {
                      type: "application/json"
                    });
                    const url = URL.createObjectURL(blob);
                    const a = document.createElement("a");
                    a.href = url;
                    a.download = "rules.json";
                    a.click();
                    URL.revokeObjectURL(url);
                  }}
                  onImport={(payload) => {
                    try {
                      const parsed = JSON.parse(payload) as Rules;
                      setDraftRules(parsed);
                      toast.success("Rules imported into draft");
                    } catch {
                      toast.error("Invalid rules.json");
                    }
                  }}
                />
              }
            />
            <Route
              path="/cleanup"
              element={<CleanupView rules={draftRules} onChange={(next) => void liveSetRules(next)} />}
            />
            <Route
              path="/settings"
              element={
                <SettingsView
                  rules={draftRules}
                  onChange={(next) => void liveSetRules(next)}
                  onChangeSortRoot={(path) => void handleSortRootChange(path)}
                />
              }
            />
          </Routes>
        </AppShell>
      </HashRouter>

      <DryRunDialog plan={dryPlan} open={dryOpen} onOpenChange={setDryOpen} />
    </>
  );
}

export default App;

