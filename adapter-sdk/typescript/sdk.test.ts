import test from "node:test";
import assert from "node:assert/strict";
import { validateManifest } from "./sdk.ts";

test("valid manifest passes", () => {
  assert.doesNotThrow(() =>
    validateManifest({
      adapterId: "local.echo",
      adapterVersion: "1.0.0",
      sdkContractVersion: "1.0.0",
      capabilities: ["native.local.echo"],
      simulationSupported: true,
    }),
  );
});

test("empty capability list fails", () => {
  assert.throws(() =>
    validateManifest({
      adapterId: "bad",
      adapterVersion: "1.0.0",
      sdkContractVersion: "1.0.0",
      capabilities: [],
      simulationSupported: false,
    }),
  );
});
