import type { AdapterId, AdapterPolicy, AdapterTask } from "./models.ts";

const allowedIds: AdapterId[] = [
  "generic_web", "generic_form", "generic_invoice", "homeocta", "99freelas",
];

function originOf(value: string): URL {
  const parsed = new URL(value);
  if (!["http:", "https:"].includes(parsed.protocol)) {
    throw new Error("F09_POLICY_DENIED_NON_HTTP");
  }
  if (parsed.username || parsed.password) {
    throw new Error("F09_POLICY_DENIED_URL_CREDENTIALS");
  }
  return parsed;
}

export function assertPolicy(config: AdapterPolicy, task: AdapterTask): void {
  if (config.schemaVersion !== 1 || !allowedIds.includes(config.adapterId)) {
    throw new Error("F09_POLICY_INVALID_CONFIG");
  }
  if (!task.missionId.trim() || !task.taskId.trim() || task.deadlineMs <= 0) {
    throw new Error("F09_POLICY_INVALID_TASK");
  }
  if (!config.grantedScopes.includes("platform.read")) {
    throw new Error("F09_POLICY_SCOPE_DENIED");
  }
  const target = originOf(task.url);
  const base = originOf(config.origin);
  if (target.origin !== base.origin) {
    throw new Error("F09_POLICY_ORIGIN_DENIED");
  }
  if (config.mode !== "simulation") {
    // An actual platform connection requires a separate, explicit production
    // authorization and qualification workflow not present in this phase.
    throw new Error("F09_POLICY_LIVE_NOT_AUTHORIZED");
  }
  if (!["127.0.0.1", "localhost"].includes(target.hostname)) {
    throw new Error("F09_POLICY_SIMULATION_LOOPBACK_ONLY");
  }
  if (config.externalEffectsApproved) {
    throw new Error("F09_POLICY_SIMULATION_EXTERNAL_EFFECT_DENIED");
  }
  if (task.formFields?.length && !config.grantedScopes.includes("practice.fill")) {
    throw new Error("F09_POLICY_FILL_SCOPE_DENIED");
  }
  if (config.adapterId === "99freelas" && task.formFields?.length) {
    throw new Error("F09_POLICY_99FREELAS_READ_ONLY");
  }
  if (config.adapterId === "homeocta" && task.formFields?.length) {
    throw new Error("F09_POLICY_HOMEOCTA_QUALIFICATION_DENIED");
  }
  if (task.formFields?.some((field) => !field.label.trim())) {
    throw new Error("F09_POLICY_INVALID_FIELD");
  }
}
