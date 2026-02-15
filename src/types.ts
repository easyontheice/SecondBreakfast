export type CollisionPolicy = "rename";

export interface Rules {
  global: {
    sortRoot: string;
    caseInsensitiveExt: boolean;
    collisionPolicy: CollisionPolicy;
    unknownGoesToMisc: boolean;
    noExtensionGoesToMisc: boolean;
    minFileAgeSeconds: number;
    cleanupEmptyFolders: {
      enabled: boolean;
      minAgeSeconds: number;
      mode: "trash";
    };
  };
  categories: CategoryRule[];
  misc: {
    name: string;
    targetSubfolder: string;
  };
}

export interface CategoryRule {
  id: string;
  name: string;
  targetSubfolder: string;
  extensions: string[];
}

export interface ValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
}

export interface PlanEntry {
  sourcePath: string;
  destinationPath: string;
  category: string;
  collisionRenamed: boolean;
}

export interface PlanSkip {
  path: string;
  reason: string;
}

export interface PlanGroup {
  category: string;
  count: number;
  entries: PlanEntry[];
}

export interface PlanPreview {
  sessionId: string;
  generatedAt: string;
  totalCandidates: number;
  moveCount: number;
  skipCount: number;
  errorCount: number;
  potentialConflicts: number;
  moves: PlanEntry[];
  skips: PlanSkip[];
  grouped: PlanGroup[];
}

export interface MovedFile {
  sourcePath: string;
  destinationPath: string;
  category: string;
  collisionRenamed: boolean;
}

export interface RunResult {
  sessionId: string;
  startedAt: string;
  finishedAt: string;
  moved: number;
  skipped: number;
  errors: number;
  movedFiles: MovedFile[];
  skips: PlanSkip[];
  errorDetails: PlanSkip[];
  cleanupTrashed: number;
  cleanupErrors: number;
}

export interface UndoDetail {
  sourcePath: string;
  destinationPath: string;
  status: string;
  message: string;
}

export interface UndoResult {
  sessionId: string | null;
  restored: number;
  skipped: number;
  conflicts: number;
  missing: number;
  errors: number;
  details: UndoDetail[];
}

export interface WatcherStatus {
  running: boolean;
  sortRoot: string;
}

export interface RunProgressEvent {
  moved: number;
  skipped: number;
  errors: number;
  currentPath?: string;
  destPath?: string;
}

export interface RunLogEvent {
  level: "info" | "warn" | "error";
  message: string;
}

export interface ActivityItem {
  id: string;
  level: "info" | "warn" | "error";
  message: string;
  sourcePath?: string;
  destinationPath?: string;
  at: string;
}
