import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  PlanPreview,
  Rules,
  RunLogEvent,
  RunProgressEvent,
  RunResult,
  UndoResult,
  ValidationResult,
  WatcherStatus
} from "@/types";

export function getRules() {
  return invoke<Rules>("get_rules");
}

export function setRules(rules: Rules) {
  return invoke<void>("set_rules", { rules });
}

export function validateRules(rules: Rules) {
  return invoke<ValidationResult>("validate_rules", { rules });
}

export function setSortRoot(path: string) {
  return invoke<void>("set_sort_root", { path });
}

export function dryRun() {
  return invoke<PlanPreview>("dry_run");
}

export function runNow() {
  return invoke<RunResult>("run_now");
}

export function undoLastRun() {
  return invoke<UndoResult>("undo_last_run");
}

export function startWatcher() {
  return invoke<void>("start_watcher");
}

export function stopWatcher() {
  return invoke<void>("stop_watcher");
}

export function watcherStatus() {
  return invoke<WatcherStatus>("watcher_status");
}

export function onRunProgress(handler: (payload: RunProgressEvent) => void) {
  return listen<RunProgressEvent>("run_progress", (event) => handler(event.payload));
}

export function onRunLog(handler: (payload: RunLogEvent) => void) {
  return listen<RunLogEvent>("run_log", (event) => handler(event.payload));
}

export function onWatcherStatus(handler: (payload: WatcherStatus) => void) {
  return listen<WatcherStatus>("watcher_status", (event) => handler(event.payload));
}
