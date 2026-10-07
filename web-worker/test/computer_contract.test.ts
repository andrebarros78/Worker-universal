import assert from "node:assert/strict";
import test from "node:test";

import { validateComputerWorkerContract } from "../src/computer_contract.ts";

test("dedicated isolated Windows session contract is accepted", () => {
  assert.doesNotThrow(() =>
    validateComputerWorkerContract({
      schemaVersion: 1,
      capabilityId: "computer.windows.isolated",
      mode: "isolated_windows_session",
      dedicatedSession: true,
      nativeInputIsolation: "no_operator_mouse_keyboard",
      maxRuntimeMs: 60_000,
      evidenceRequired: true,
    }),
  );
});

test("shared operator desktop is rejected", () => {
  assert.throws(
    () =>
      validateComputerWorkerContract({
        schemaVersion: 1,
        capabilityId: "computer.windows.isolated",
        mode: "isolated_windows_session",
        dedicatedSession: false,
        nativeInputIsolation: "no_operator_mouse_keyboard",
        maxRuntimeMs: 60_000,
        evidenceRequired: true,
      }),
    /shared_operator_desktop_forbidden/,
  );
});
