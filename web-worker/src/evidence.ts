import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import type { Page } from "playwright";

export type EvidenceKind = "dom" | "screenshot" | "download" | "error" | "session";

export type EvidenceRecord = {
  sequence: number;
  stepId: string;
  kind: EvidenceKind;
  path: string;
  sha256: string;
  bytes: number;
};

async function digestFile(path: string): Promise<{ sha256: string; bytes: number }> {
  const data = await readFile(path);
  return {
    sha256: createHash("sha256").update(data).digest("hex"),
    bytes: data.length,
  };
}

function safeName(value: string): string {
  return value.replace(/[^a-zA-Z0-9._-]+/g, "_");
}

export class EvidenceStore {
  private readonly root: string;
  private readonly records: EvidenceRecord[] = [];
  private sequence = 0;

  constructor(root: string) {
    this.root = root;
  }

  async initialize(): Promise<void> {
    await mkdir(this.root, { recursive: true });
  }

  async capturePage(page: Page, stepId: string): Promise<EvidenceRecord[]> {
    const results: EvidenceRecord[] = [];
    const base = String(++this.sequence).padStart(4, "0") + "-" + safeName(stepId);

    const domPath = join(this.root, base + ".html");
    await writeFile(domPath, await page.content(), "utf8");
    results.push(await this.record(stepId, "dom", domPath));

    const screenshotPath = join(this.root, base + ".png");
    await page.screenshot({ path: screenshotPath, fullPage: true });
    results.push(await this.record(stepId, "screenshot", screenshotPath));

    return results;
  }

  async recordExisting(
    stepId: string,
    kind: EvidenceKind,
    path: string,
  ): Promise<EvidenceRecord> {
    return this.record(stepId, kind, path);
  }

  async writeError(stepId: string, payload: object): Promise<EvidenceRecord> {
    const path = join(
      this.root,
      String(++this.sequence).padStart(4, "0") + "-" + safeName(stepId) + "-error.json",
    );
    await writeFile(path, JSON.stringify(payload, null, 2) + "\n", "utf8");
    return this.record(stepId, "error", path);
  }

  all(): EvidenceRecord[] {
    return [...this.records];
  }

  async writeManifest(): Promise<string> {
    const path = join(this.root, "evidence-manifest.json");
    await writeFile(
      path,
      JSON.stringify({ schemaVersion: 1, records: this.records }, null, 2) + "\n",
      "utf8",
    );
    return path;
  }

  private async record(
    stepId: string,
    kind: EvidenceKind,
    path: string,
  ): Promise<EvidenceRecord> {
    const info = await digestFile(path);
    const record: EvidenceRecord = {
      sequence: this.records.length + 1,
      stepId,
      kind,
      path,
      sha256: info.sha256,
      bytes: info.bytes,
    };
    this.records.push(record);
    return record;
  }
}
