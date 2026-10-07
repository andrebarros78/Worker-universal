export type AdapterManifest = {
  adapterId: string;
  adapterVersion: string;
  sdkContractVersion: string;
  capabilities: string[];
  simulationSupported: boolean;
  configSchemaRef?: string;
  secretRefs?: string[];
};

export type Invocation = {
  invocationId: string;
  missionId: string;
  capabilityId: string;
  contractVersion: string;
  timeoutMs: number;
  payload: unknown;
  simulation: boolean;
  idempotencyKey?: string;
  sessionRef?: string;
  secretRefs: string[];
  artifactRefs: string[];
};

export type AdapterResult = {
  status: "succeeded" | "failed" | "cancelled" | "deadline_exceeded" | "needs_review";
  payload?: unknown;
  errors: Array<Record<string, unknown>>;
  evidence: Array<Record<string, unknown>>;
  artifactRefs: string[];
  sessionRef?: string;
  costMicrounits: number;
};

export interface Adapter {
  manifest(): AdapterManifest;
  health(): Promise<{ lifecycle: string; readiness: string }>;
  invoke(invocation: Invocation, signal?: AbortSignal): Promise<AdapterResult>;
}

export function validateManifest(manifest: AdapterManifest): void {
  if (!manifest.adapterId.trim()) throw new Error("adapterId required");
  if (manifest.capabilities.length === 0) throw new Error("at least one capability required");
  if (!manifest.sdkContractVersion.startsWith("1.")) {
    throw new Error("unsupported sdk contract major");
  }
}
