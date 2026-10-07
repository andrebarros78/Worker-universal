export type Step = {
  id: string;
  action: string;
  timeoutMs: number;
};

export type MissionEnvelope = {
  schemaVersion: 1;
  missionId: string;
  taskId: string;
  deadlineMs: number;
  steps: Step[];
};

export type StepExecutor = (step: Step) => Promise<void>;

export type WorkerReport = {
  status: "succeeded" | "failed" | "deadline_exceeded";
  completedSteps: string[];
  elapsedMs: number;
  error?: string;
};

function nowMs(): number {
  return Number(process.hrtime.bigint()) / 1_000_000;
}

export class TimedBrowserWorker {
  private readonly executeStep: StepExecutor;

  constructor(executeStep: StepExecutor) {
    this.executeStep = executeStep;
  }

  async run(mission: MissionEnvelope): Promise<WorkerReport> {
    if (mission.deadlineMs <= 0) {
      throw new Error("deadlineMs must be positive");
    }
    const started = nowMs();
    const completed: string[] = [];

    for (const step of mission.steps) {
      const elapsedBefore = nowMs() - started;
      if (elapsedBefore >= mission.deadlineMs) {
        return {
          status: "deadline_exceeded",
          completedSteps: completed,
          elapsedMs: elapsedBefore,
        };
      }

      const remaining = mission.deadlineMs - elapsedBefore;
      const timeout = Math.max(1, Math.min(step.timeoutMs, remaining));
      try {
        await Promise.race([
          this.executeStep(step),
          new Promise<never>((_, reject) =>
            setTimeout(() => reject(new Error("step_timeout")), timeout),
          ),
        ]);
      } catch (error) {
        const elapsed = nowMs() - started;
        return {
          status: elapsed >= mission.deadlineMs ? "deadline_exceeded" : "failed",
          completedSteps: completed,
          elapsedMs: elapsed,
          error: error instanceof Error ? error.message : String(error),
        };
      }

      completed.push(step.id);
      const elapsedAfter = nowMs() - started;
      if (elapsedAfter > mission.deadlineMs) {
        return {
          status: "deadline_exceeded",
          completedSteps: completed,
          elapsedMs: elapsedAfter,
        };
      }
    }

    return {
      status: "succeeded",
      completedSteps: completed,
      elapsedMs: nowMs() - started,
    };
  }
}

async function selfTest(): Promise<number> {
  const worker = new TimedBrowserWorker(async () => {
    await new Promise(resolve => setTimeout(resolve, 5));
  });
  const report = await worker.run({
    schemaVersion: 1,
    missionId: "self-test",
    taskId: "web-001",
    deadlineMs: 200,
    steps: [
      { id: "observe", action: "observe", timeoutMs: 50 },
      { id: "act", action: "act", timeoutMs: 50 },
      { id: "verify", action: "verify", timeoutMs: 50 },
    ],
  });
  console.log(JSON.stringify({ component: "tma-web-worker", ...report }));
  return report.status === "succeeded" ? 0 : 1;
}

if (process.argv[1]?.endsWith("worker.ts") && process.argv[2] === "self-test") {
  process.exitCode = await selfTest();
}
