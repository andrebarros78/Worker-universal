import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { BrowserRuntime } from "../../web-worker/src/browser_runtime.ts";
import type { BrowserStep } from "../../web-worker/src/browser_runtime.ts";
import type { AdapterOutcome, AdapterPlan, AdapterPolicy, AdapterTask } from "./models.ts";
import { PlatformAdapter } from "./adapters.ts";

function sha256(value:Buffer):string {
  return createHash("sha256").update(value).digest("hex");
}

export async function executeSimulated(
  adapter:PlatformAdapter,
  config:AdapterPolicy,
  task:AdapterTask,
  injectCrash=false,
):Promise<AdapterOutcome> {
  const plan:AdapterPlan=adapter.plan(config,task);
  const guardedPages=new WeakSet<object>();
  let crashed=false;
  const expectedOrigin=new URL(task.url).origin;
  const runtime=new BrowserRuntime({
    recoveryBudget:1,
    beforeStep:async (step:BrowserStep,attempt:number,worker:BrowserRuntime)=>{
      const page=worker.page();
      if (!guardedPages.has(page)) {
        guardedPages.add(page);
        await page.route("**/*",async(route)=>{
          let requestOrigin="";
          try { requestOrigin=new URL(route.request().url()).origin; } catch {}
          if(requestOrigin===expectedOrigin) await route.continue();
          else await route.abort();
        });
      }
      if(injectCrash && step.kind==="observe" && attempt===0 && !crashed) {
        crashed=true;
        await worker.terminateBrowserProcessForTest();
      }
    },
  });
  const report=await runtime.executeMission(plan.mission);
  const evidence:AdapterOutcome["evidence"]=[];
  for(const item of report.evidence) {
    const buffer=await readFile(item.path);
    if(buffer.length!==item.bytes || sha256(buffer)!==item.sha256) {
      return {
        adapterId:adapter.id,status:"failed",reason:"evidence_integrity_failed",
        declaredSuccess:false,validatedEvidence:false,requiredEvidence:plan.requiredEvidence,
        evidence:[],recoveryCount:report.recoveryCount,executionMode:config.mode,
        missionId:task.missionId,externalEffect:false,
      };
    }
    evidence.push({kind:item.kind,sha256:item.sha256,bytes:item.bytes});
  }
  const valid=plan.requiredEvidence.every((required)=>
    evidence.some((entry)=>entry.kind===required && entry.bytes>0 && /^[a-f0-9]{64}$/.test(entry.sha256))
  );
  const successful=report.status==="succeeded" && valid;
  return {
    adapterId:adapter.id,
    status:successful?"simulated":report.status==="deadline_exceeded"?"deadline_exceeded":"failed",
    reason:successful?"synthetic_evidence_verified":report.error??"evidence_incomplete",
    declaredSuccess:report.status==="succeeded",
    validatedEvidence:valid,
    requiredEvidence:plan.requiredEvidence,
    evidence,
    recoveryCount:report.recoveryCount,
    executionMode:config.mode,
    missionId:task.missionId,
    externalEffect:false,
  };
}
