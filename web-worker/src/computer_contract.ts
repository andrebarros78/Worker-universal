export type ComputerWorkerMode = "isolated_windows_session" | "vm";

export type ComputerWorkerContract = {
  schemaVersion: 1;
  capabilityId: string;
  mode: ComputerWorkerMode;
  dedicatedSession: boolean;
  nativeInputIsolation: "no_operator_mouse_keyboard";
  sessionRef?: string;
  maxRuntimeMs: number;
  evidenceRequired: boolean;
};

export function validateComputerWorkerContract(
  contract: ComputerWorkerContract,
): void {
  if (!contract.capabilityId.trim()) {
    throw new Error("capabilityId required");
  }
  if (!contract.dedicatedSession) {
    throw new Error("shared_operator_desktop_forbidden");
  }
  if (contract.nativeInputIsolation !== "no_operator_mouse_keyboard") {
    throw new Error("native_input_isolation_required");
  }
  if (contract.maxRuntimeMs <= 0) {
    throw new Error("maxRuntimeMs must be positive");
  }
}
