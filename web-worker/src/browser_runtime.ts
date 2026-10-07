import { existsSync } from "node:fs";
import { mkdir } from "node:fs/promises";
import { basename, dirname, join } from "node:path";
import { chromium } from "playwright";
import type { Browser, BrowserContext, BrowserServer, Page } from "playwright";

import { EvidenceStore } from "./evidence.ts";
import type { EvidenceRecord } from "./evidence.ts";
import { resolveLocator } from "./semantic_locator.ts";
import type { SemanticTarget } from "./semantic_locator.ts";

export type BrowserStep =
  | { id: string; kind: "goto"; url: string; timeoutMs: number; replaySafe?: boolean; captureAfter?: boolean }
  | { id: string; kind: "observe"; timeoutMs: number; replaySafe?: boolean }
  | { id: string; kind: "fill"; target: SemanticTarget; value: string; timeoutMs: number; replaySafe?: boolean; captureAfter?: boolean }
  | { id: string; kind: "click"; target: SemanticTarget; timeoutMs: number; replaySafe?: boolean; captureAfter?: boolean }
  | { id: string; kind: "upload"; target: SemanticTarget; filePath: string; timeoutMs: number; replaySafe?: boolean; captureAfter?: boolean }
  | { id: string; kind: "download"; target: SemanticTarget; timeoutMs: number; replaySafe?: boolean; fileName?: string }
  | { id: string; kind: "assertText"; target: SemanticTarget; expected: string; timeoutMs: number; replaySafe?: boolean; captureAfter?: boolean }
  | { id: string; kind: "saveSession"; timeoutMs: number; replaySafe?: boolean }
  | { id: string; kind: "wait"; waitMs: number; timeoutMs: number; replaySafe?: boolean };

export type BrowserMission = {
  schemaVersion: 1;
  missionId: string;
  taskId: string;
  deadlineMs: number;
  evidenceDir: string;
  sessionStatePath: string;
  steps: BrowserStep[];
};

export type BrowserWorkerStatus = "succeeded" | "failed" | "deadline_exceeded";

export type BrowserWorkerReport = {
  status: BrowserWorkerStatus;
  completedSteps: string[];
  elapsedMs: number;
  recoveryCount: number;
  sessionRestored: boolean;
  browserPids: number[];
  evidence: EvidenceRecord[];
  evidenceManifest?: string;
  error?: string;
};

export type BrowserRuntimeOptions = {
  headless?: boolean;
  recoveryBudget?: number;
  viewport?: { width: number; height: number };
  beforeStep?: (
    step: BrowserStep,
    attempt: number,
    runtime: BrowserRuntime,
  ) => Promise<void> | void;
};

function nowMs(): number {
  return Number(process.hrtime.bigint()) / 1_000_000;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function isCrashError(error: unknown): boolean {
  const message = errorMessage(error).toLowerCase();
  return (
    message.includes("browser has been closed") ||
    message.includes("target page, context or browser has been closed") ||
    message.includes("connection closed") ||
    message.includes("browser closed") ||
    message.includes("browser disconnected")
  );
}

async function withTimeout<T>(operation: Promise<T>, timeoutMs: number): Promise<T> {
  let timer: NodeJS.Timeout | undefined;
  try {
    return await Promise.race([
      operation,
      new Promise<never>((_, reject) => {
        timer = setTimeout(() => reject(new Error("step_timeout")), timeoutMs);
      }),
    ]);
  } finally {
    if (timer) clearTimeout(timer);
  }
}

export class BrowserRuntime {
  private server?: BrowserServer;
  private browser?: Browser;
  private context?: BrowserContext;
  private currentPage?: Page;
  private readonly options: Required<Pick<BrowserRuntimeOptions, "headless" | "recoveryBudget">> &
    Omit<BrowserRuntimeOptions, "headless" | "recoveryBudget">;
  private sessionStatePath = "";
  private evidenceDir = "";
  private restored = false;
  private recoveries = 0;
  private readonly pids: number[] = [];

  constructor(options: BrowserRuntimeOptions = {}) {
    if (options.headless === false) {
      throw new Error("headless_required");
    }
    this.options = {
      headless: true,
      recoveryBudget: options.recoveryBudget ?? 1,
      viewport: options.viewport ?? { width: 1280, height: 800 },
      beforeStep: options.beforeStep,
    };
  }

  isHeadless(): boolean {
    return this.options.headless;
  }

  isConnected(): boolean {
    return Boolean(this.browser?.isConnected());
  }

  browserPids(): number[] {
    return [...this.pids];
  }

  recoveryCount(): number {
    return this.recoveries;
  }

  sessionWasRestored(): boolean {
    return this.restored;
  }

  page(): Page {
    if (!this.currentPage) throw new Error("browser_runtime_not_started");
    return this.currentPage;
  }

  async start(sessionStatePath: string, evidenceDir: string): Promise<void> {
    this.sessionStatePath = sessionStatePath;
    this.evidenceDir = evidenceDir;
    await mkdir(evidenceDir, { recursive: true });
    await mkdir(join(evidenceDir, "downloads"), { recursive: true });
    await mkdir(dirname(sessionStatePath), { recursive: true });
    await this.launch();
  }

  async stop(): Promise<void> {
    await this.closeHandles();
  }

  async terminateBrowserProcessForTest(): Promise<void> {
    const browserProcess = this.server?.process();
    if (!browserProcess) throw new Error("browser_process_unavailable");
    browserProcess.kill();
    await new Promise<void>((resolve) => {
      if (browserProcess.exitCode !== null) {
        resolve();
        return;
      }
      browserProcess.once("exit", () => resolve());
    });
  }

  async executeMission(mission: BrowserMission): Promise<BrowserWorkerReport> {
    if (mission.deadlineMs <= 0) throw new Error("deadlineMs must be positive");
    this.restored = false;
    this.recoveries = 0;
    this.pids.length = 0;
    const evidence = new EvidenceStore(mission.evidenceDir);
    await evidence.initialize();
    const started = nowMs();
    const completed: string[] = [];
    let status: BrowserWorkerStatus = "succeeded";
    let finalError: string | undefined;

    await this.start(mission.sessionStatePath, mission.evidenceDir);
    try {
      for (const step of mission.steps) {
        const elapsedBefore = nowMs() - started;
        if (elapsedBefore >= mission.deadlineMs) {
          status = "deadline_exceeded";
          finalError = "mission_deadline_before_step";
          break;
        }

        const remaining = mission.deadlineMs - elapsedBefore;
        const stepTimeout = Math.max(1, Math.min(step.timeoutMs, remaining));
        let attempt = 0;

        for (;;) {
          try {
            if (this.options.beforeStep) {
              await this.options.beforeStep(step, attempt, this);
            }
            await withTimeout(this.executeStep(step, evidence), stepTimeout);
            completed.push(step.id);
            break;
          } catch (error) {
            const elapsed = nowMs() - started;
            const crash = !this.isConnected() || isCrashError(error);
            const mayRecover =
              crash &&
              Boolean(step.replaySafe) &&
              this.recoveries < this.options.recoveryBudget;

            if (mayRecover) {
              this.recoveries += 1;
              attempt += 1;
              await this.relaunch();
              continue;
            }

            const timedOut = errorMessage(error) === "step_timeout";
            if (timedOut) {
              await this.closeHandles();
            }
            status = elapsed >= mission.deadlineMs ? "deadline_exceeded" : "failed";
            finalError = errorMessage(error);
            await evidence.writeError(step.id, {
              error: finalError,
              crash,
              replaySafe: Boolean(step.replaySafe),
              recoveryCount: this.recoveries,
            });
            if (this.isConnected()) {
              try {
                await evidence.capturePage(this.page(), step.id + "-failure");
              } catch {
              }
            }
            break;
          }
        }

        if (status !== "succeeded") break;
        if (nowMs() - started > mission.deadlineMs) {
          status = "deadline_exceeded";
          finalError = "mission_deadline_after_step";
          break;
        }
      }
    } finally {
      const manifest = await evidence.writeManifest();
      await this.stop();
      return {
        status,
        completedSteps: completed,
        elapsedMs: nowMs() - started,
        recoveryCount: this.recoveries,
        sessionRestored: this.restored,
        browserPids: this.browserPids(),
        evidence: evidence.all(),
        evidenceManifest: manifest,
        error: finalError,
      };
    }
  }

  private async launch(): Promise<void> {
    this.server = await chromium.launchServer({ headless: this.options.headless });
    const pid = this.server.process()?.pid;
    if (pid) this.pids.push(pid);
    this.browser = await chromium.connect(this.server.wsEndpoint());
    const hasState = existsSync(this.sessionStatePath);
    this.context = await this.browser.newContext({
      acceptDownloads: true,
      storageState: hasState ? this.sessionStatePath : undefined,
      viewport: this.options.viewport,
    });
    this.restored = this.restored || hasState;
    this.currentPage = await this.context.newPage();
  }

  private async relaunch(): Promise<void> {
    await this.closeHandles();
    await this.launch();
  }

  private async closeHandles(): Promise<void> {
    try { await this.context?.close(); } catch {}
    try { await this.browser?.close(); } catch {}
    try { await this.server?.close(); } catch {}
    this.currentPage = undefined;
    this.context = undefined;
    this.browser = undefined;
    this.server = undefined;
  }

  private async executeStep(step: BrowserStep, evidence: EvidenceStore): Promise<void> {
    const page = this.page();

    if (step.kind === "goto") {
      await page.goto(step.url, { waitUntil: "domcontentloaded" });
    } else if (step.kind === "observe") {
      await evidence.capturePage(page, step.id);
      return;
    } else if (step.kind === "fill") {
      const resolved = await resolveLocator(page, step.target);
      await resolved.locator.fill(step.value);
    } else if (step.kind === "click") {
      const resolved = await resolveLocator(page, step.target);
      await resolved.locator.click();
    } else if (step.kind === "upload") {
      const resolved = await resolveLocator(page, step.target);
      await resolved.locator.setInputFiles(step.filePath);
    } else if (step.kind === "download") {
      const resolved = await resolveLocator(page, step.target);
      const downloadPromise = page.waitForEvent("download");
      await resolved.locator.click();
      const download = await downloadPromise;
      const fileName = basename(step.fileName ?? download.suggestedFilename());
      const path = join(this.evidenceDir, "downloads", fileName);
      await download.saveAs(path);
      await evidence.recordExisting(step.id, "download", path);
    } else if (step.kind === "assertText") {
      const resolved = await resolveLocator(page, step.target);
      const actual = (await resolved.locator.textContent()) ?? "";
      if (!actual.includes(step.expected)) {
        throw new Error("assert_text_failed:expected=" + step.expected + ":actual=" + actual);
      }
    } else if (step.kind === "saveSession") {
      if (!this.context) throw new Error("browser_context_unavailable");
      await this.context.storageState({ path: this.sessionStatePath });
      await evidence.recordExisting(step.id, "session", this.sessionStatePath);
    } else if (step.kind === "wait") {
      await page.waitForTimeout(step.waitMs);
    } else {
      const unreachable: never = step;
      throw new Error("unsupported_browser_step:" + String(unreachable));
    }

    if ("captureAfter" in step && step.captureAfter) {
      await evidence.capturePage(page, step.id);
    }
  }
}
