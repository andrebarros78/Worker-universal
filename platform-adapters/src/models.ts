import type { BrowserMission } from "../../web-worker/src/browser_runtime.ts";

export type AdapterId =
  | "generic_web" | "generic_form" | "generic_invoice" | "homeocta" | "99freelas";
export type AdapterMode = "simulation" | "read_only";
export type AdapterPolicy = {
  schemaVersion: 1;
  adapterId: AdapterId;
  mode: AdapterMode;
  grantedScopes: string[];
  origin: string;
  eligibilityVerified: boolean;
  authorizationVerified: boolean;
  productionQualified: boolean;
  humanReviewed: boolean;
  externalEffectsApproved: boolean;
};
export type AdapterTask = {
  missionId: string;
  taskId: string;
  url: string;
  deadlineMs: number;
  evidenceDir: string;
  sessionStatePath: string;
  formFields?: { label: string; value: string }[];
};
export type AdapterPlan = {
  adapterId: AdapterId;
  capabilities: string[];
  mission: BrowserMission;
  requiredEvidence: string[];
  delegatedCapability?: string;
  status: "simulation_ready" | "read_only_ready";
};
export type AdapterOutcome = {
  adapterId: AdapterId;
  status: "simulated" | "rejected" | "failed" | "deadline_exceeded";
  reason: string;
  declaredSuccess: boolean;
  validatedEvidence: boolean;
  requiredEvidence: string[];
  evidence: Array<{ kind: string; sha256: string; bytes: number }>;
  recoveryCount: number;
  executionMode: AdapterMode;
  missionId: string;
  externalEffect: false;
};
