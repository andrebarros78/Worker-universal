import test from "node:test";
import assert from "node:assert/strict";
import { TimedBrowserWorker } from "../src/worker.ts";

test("completes steps inside SLA", async () => {
  const worker = new TimedBrowserWorker(async () => {});
  const result = await worker.run({
    schemaVersion: 1,
    missionId: "m1",
    taskId: "t1",
    deadlineMs: 100,
    steps: [
      { id: "a", action: "observe", timeoutMs: 20 },
      { id: "b", action: "verify", timeoutMs: 20 },
    ],
  });
  assert.equal(result.status, "succeeded");
  assert.deepEqual(result.completedSteps, ["a", "b"]);
});

test("step timeout is contained", async () => {
  const worker = new TimedBrowserWorker(async () => {
    await new Promise(resolve => setTimeout(resolve, 40));
  });
  const result = await worker.run({
    schemaVersion: 1,
    missionId: "m2",
    taskId: "t2",
    deadlineMs: 200,
    steps: [{ id: "slow", action: "wait", timeoutMs: 5 }],
  });
  assert.equal(result.status, "failed");
  assert.equal(result.error, "step_timeout");
});

test("mission deadline is enforced", async () => {
  const worker = new TimedBrowserWorker(async () => {
    await new Promise(resolve => setTimeout(resolve, 25));
  });
  const result = await worker.run({
    schemaVersion: 1,
    missionId: "m3",
    taskId: "t3",
    deadlineMs: 10,
    steps: [{ id: "late", action: "wait", timeoutMs: 100 }],
  });
  assert.equal(result.status, "deadline_exceeded");
});
