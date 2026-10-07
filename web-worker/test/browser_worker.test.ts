import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { basename, join } from "node:path";
import test from "node:test";

import { BrowserRuntime } from "../src/browser_runtime.ts";
import type { BrowserMission } from "../src/browser_runtime.ts";
import { SyntheticFixtureServer } from "./fixture_server.ts";

async function workspace(name: string): Promise<string> {
  return mkdtemp(join(tmpdir(), "tma-f04-" + name + "-"));
}

function mission(
  root: string,
  steps: BrowserMission["steps"],
  deadlineMs = 10_000,
): BrowserMission {
  return {
    schemaVersion: 1,
    missionId: "mission-f04",
    taskId: "browser-task",
    deadlineMs,
    evidenceDir: join(root, "evidence"),
    sessionStatePath: join(root, "session.json"),
    steps,
  };
}

test("non-headless browser mode is rejected", () => {
  assert.throws(() => new BrowserRuntime({ headless: false }), /headless_required/);
});

test("real headless browser fills form, uploads, downloads and captures evidence", async () => {
  const root = await workspace("form");
  const fixture = new SyntheticFixtureServer();
  await fixture.start();
  try {
    const upload = join(root, "payload.txt");
    await writeFile(upload, "synthetic upload\n", "utf8");

    const runtime = new BrowserRuntime();
    assert.equal(runtime.isHeadless(), true);

    const report = await runtime.executeMission(
      mission(root, [
        {
          id: "goto-form",
          kind: "goto",
          url: fixture.url("/form"),
          timeoutMs: 2_000,
          replaySafe: true,
        },
        {
          id: "fill-name",
          kind: "fill",
          target: { label: "Full name" },
          value: "Andre Test",
          timeoutMs: 1_000,
          replaySafe: true,
        },
        {
          id: "upload-file",
          kind: "upload",
          target: { label: "Attachment" },
          filePath: upload,
          timeoutMs: 1_000,
          replaySafe: true,
        },
        {
          id: "assert-upload",
          kind: "assertText",
          target: { testId: "upload-status" },
          expected: basename(upload),
          timeoutMs: 1_000,
          replaySafe: true,
        },
        {
          id: "submit",
          kind: "click",
          target: { role: "button", name: "Submit mission" },
          timeoutMs: 1_000,
        },
        {
          id: "assert-submit",
          kind: "assertText",
          target: { testId: "submit-status" },
          expected: "Submitted: Andre Test",
          timeoutMs: 1_000,
          replaySafe: true,
        },
        {
          id: "download",
          kind: "download",
          target: { role: "link", name: "Download receipt" },
          fileName: "receipt.txt",
          timeoutMs: 2_000,
        },
        {
          id: "observe-final",
          kind: "observe",
          timeoutMs: 2_000,
          replaySafe: true,
        },
      ]),
    );

    assert.equal(report.status, "succeeded");
    assert.equal(report.completedSteps.length, 8);
    assert.ok(report.browserPids.length >= 1);
    assert.ok(report.evidence.some((item) => item.kind === "download"));
    assert.ok(report.evidence.some((item) => item.kind === "dom"));
    assert.ok(report.evidence.some((item) => item.kind === "screenshot"));
    assert.ok(report.evidence.every((item) => item.sha256.length === 64));
    assert.equal(
      await readFile(join(root, "evidence", "downloads", "receipt.txt"), "utf8"),
      "receipt:synthetic\n",
    );
    assert.ok(report.evidenceManifest);
  } finally {
    await fixture.stop();
    await rm(root, { recursive: true, force: true });
  }
});

test("download filename cannot escape evidence directory", async () => {
  const root = await workspace("download-sanitize");
  const fixture = new SyntheticFixtureServer();
  await fixture.start();
  try {
    const report = await new BrowserRuntime().executeMission(
      mission(root, [
        {
          id: "goto-form",
          kind: "goto",
          url: fixture.url("/form"),
          timeoutMs: 2_000,
          replaySafe: true,
        },
        {
          id: "download",
          kind: "download",
          target: { role: "link", name: "Download receipt" },
          fileName: "../escape.txt",
          timeoutMs: 2_000,
        },
      ]),
    );
    assert.equal(report.status, "succeeded");
    assert.equal(
      await readFile(join(root, "evidence", "downloads", "escape.txt"), "utf8"),
      "receipt:synthetic\n",
    );
    await assert.rejects(readFile(join(root, "evidence", "escape.txt"), "utf8"));
  } finally {
    await fixture.stop();
    await rm(root, { recursive: true, force: true });
  }
});

test("reused runtime resets mission telemetry", async () => {
  const rootA = await workspace("reuse-a");
  const rootB = await workspace("reuse-b");
  const runtime = new BrowserRuntime();
  try {
    const first = await runtime.executeMission(
      mission(rootA, [
        { id: "one", kind: "wait", waitMs: 1, timeoutMs: 500, replaySafe: true },
      ]),
    );
    const second = await runtime.executeMission(
      mission(rootB, [
        { id: "two", kind: "wait", waitMs: 1, timeoutMs: 500, replaySafe: true },
      ]),
    );
    assert.equal(first.status, "succeeded");
    assert.equal(second.status, "succeeded");
    assert.equal(first.recoveryCount, 0);
    assert.equal(second.recoveryCount, 0);
    assert.equal(first.browserPids.length, 1);
    assert.equal(second.browserPids.length, 1);
  } finally {
    await rm(rootA, { recursive: true, force: true });
    await rm(rootB, { recursive: true, force: true });
  }
});

test("semantic role locator survives layout drift and changed element id", async () => {
  const root = await workspace("drift");
  const fixture = new SyntheticFixtureServer();
  await fixture.start();
  try {
    const report = await new BrowserRuntime().executeMission(
      mission(root, [
        {
          id: "goto-drift-v2",
          kind: "goto",
          url: fixture.url("/drift?v=2"),
          timeoutMs: 2_000,
          replaySafe: true,
        },
        {
          id: "continue",
          kind: "click",
          target: { role: "button", name: "Continue", css: "#continue-original" },
          timeoutMs: 1_000,
        },
        {
          id: "assert-continued",
          kind: "assertText",
          target: { css: "#drift-status" },
          expected: "continued",
          timeoutMs: 1_000,
          replaySafe: true,
        },
      ]),
    );
    assert.equal(report.status, "succeeded");
  } finally {
    await fixture.stop();
    await rm(root, { recursive: true, force: true });
  }
});

test("real browser crash is recovered and saved session is restored", async () => {
  const root = await workspace("recovery");
  const fixture = new SyntheticFixtureServer();
  await fixture.start();
  let crashed = false;
  try {
    const runtime = new BrowserRuntime({
      recoveryBudget: 1,
      beforeStep: async (step, attempt, activeRuntime) => {
        if (step.id === "post-crash-session" && attempt === 0 && !crashed) {
          crashed = true;
          await activeRuntime.terminateBrowserProcessForTest();
        }
      },
    });

    const report = await runtime.executeMission(
      mission(root, [
        {
          id: "login",
          kind: "goto",
          url: fixture.url("/login"),
          timeoutMs: 2_000,
          replaySafe: true,
        },
        {
          id: "save-session",
          kind: "saveSession",
          timeoutMs: 1_000,
          replaySafe: true,
        },
        {
          id: "post-crash-session",
          kind: "goto",
          url: fixture.url("/session"),
          timeoutMs: 3_000,
          replaySafe: true,
        },
        {
          id: "assert-restored",
          kind: "assertText",
          target: { testId: "session-status" },
          expected: "Authenticated",
          timeoutMs: 1_000,
          replaySafe: true,
        },
      ]),
    );

    assert.equal(report.status, "succeeded");
    assert.equal(report.recoveryCount, 1);
    assert.equal(report.sessionRestored, true);
    assert.equal(report.browserPids.length, 2);
  } finally {
    await fixture.stop();
    await rm(root, { recursive: true, force: true });
  }
});

test("browser crash does not replay an unsafe step", async () => {
  const root = await workspace("unsafe");
  const fixture = new SyntheticFixtureServer();
  await fixture.start();
  let crashed = false;
  try {
    const runtime = new BrowserRuntime({
      recoveryBudget: 1,
      beforeStep: async (step, attempt, activeRuntime) => {
        if (step.id === "unsafe-submit" && attempt === 0 && !crashed) {
          crashed = true;
          await activeRuntime.terminateBrowserProcessForTest();
        }
      },
    });

    const report = await runtime.executeMission(
      mission(root, [
        {
          id: "goto-form",
          kind: "goto",
          url: fixture.url("/form"),
          timeoutMs: 2_000,
          replaySafe: true,
        },
        {
          id: "unsafe-submit",
          kind: "click",
          target: { role: "button", name: "Submit mission" },
          timeoutMs: 2_000,
          replaySafe: false,
        },
      ]),
    );

    assert.equal(report.status, "failed");
    assert.equal(report.recoveryCount, 0);
    assert.ok(report.evidence.some((item) => item.kind === "error"));
  } finally {
    await fixture.stop();
    await rm(root, { recursive: true, force: true });
  }
});

test("mission deadline terminates slow browser work", async () => {
  const root = await workspace("deadline");
  try {
    const report = await new BrowserRuntime().executeMission(
      mission(
        root,
        [
          {
            id: "slow-step",
            kind: "wait",
            waitMs: 250,
            timeoutMs: 1_000,
            replaySafe: true,
          },
        ],
        50,
      ),
    );
    assert.equal(report.status, "deadline_exceeded");
    assert.ok(report.elapsedMs < 1_000);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("failed assertion produces error evidence", async () => {
  const root = await workspace("failure-evidence");
  const fixture = new SyntheticFixtureServer();
  await fixture.start();
  try {
    const report = await new BrowserRuntime().executeMission(
      mission(root, [
        {
          id: "goto-session",
          kind: "goto",
          url: fixture.url("/session"),
          timeoutMs: 2_000,
          replaySafe: true,
        },
        {
          id: "wrong-assertion",
          kind: "assertText",
          target: { testId: "session-status" },
          expected: "Authenticated",
          timeoutMs: 1_000,
          replaySafe: true,
        },
      ]),
    );
    assert.equal(report.status, "failed");
    assert.ok(report.evidence.some((item) => item.kind === "error"));
    assert.ok(report.evidence.some((item) => item.kind === "screenshot"));
  } finally {
    await fixture.stop();
    await rm(root, { recursive: true, force: true });
  }
});
