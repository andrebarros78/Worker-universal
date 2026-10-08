import type { BrowserStep } from "../../web-worker/src/browser_runtime.ts";
import { validateManifest } from "../../adapter-sdk/typescript/sdk.ts";
import type { AdapterManifest } from "../../adapter-sdk/typescript/sdk.ts";
import type { AdapterId, AdapterPlan, AdapterPolicy, AdapterTask } from "./models.ts";
import { assertPolicy } from "./policy.ts";

const ids: AdapterId[] = [
  "generic_web", "generic_form", "generic_invoice", "homeocta", "99freelas",
];
const capabilities: Record<AdapterId,string[]> = {
  generic_web:["browser.read"],
  generic_form:["browser.read","form.practice"],
  generic_invoice:["browser.read","document.extract"],
  homeocta:["browser.read","document.training"],
  "99freelas":["browser.read","research.opportunity"],
};
const delegated: Partial<Record<AdapterId,string>> = {
  generic_invoice:"vision.ocr.tesseract",
  homeocta:"vision.ocr.tesseract",
  "99freelas":"research.local",
};

export class PlatformAdapter {
  readonly id: AdapterId;
  constructor(id: AdapterId) {
    if (!ids.includes(id)) throw new Error("F09_UNKNOWN_ADAPTER");
    this.id = id;
    validateManifest(this.manifest());
  }

  manifest(): AdapterManifest {
    return {
      adapterId:this.id,
      adapterVersion:"0.6.1",
      sdkContractVersion:"1.0.0",
      capabilities:capabilities[this.id],
      simulationSupported:true,
      configSchemaRef:"contracts/platform-adapter-config.schema.json",
      secretRefs:[],
    };
  }

  health(): { lifecycle:string; readiness:string; promotion:string } {
    return { lifecycle:"healthy", readiness:"configured", promotion:"experimental" };
  }

  plan(config:AdapterPolicy,task:AdapterTask):AdapterPlan {
    if (config.adapterId !== this.id) throw new Error("F09_POLICY_ADAPTER_MISMATCH");
    assertPolicy(config,task);

    const steps:BrowserStep[] = [
      {id:"navigate",kind:"goto",url:task.url,timeoutMs:2000,replaySafe:true},
    ];
    if (this.id === "generic_form") {
      for (const [index,field] of (task.formFields??[]).entries()) {
        steps.push({
          id:"practice-field-"+index,
          kind:"fill",
          target:{label:field.label},
          value:field.value,
          timeoutMs:1000,
          replaySafe:true,
        });
      }
    } else if (task.formFields?.length) {
      throw new Error("F09_POLICY_FIELDS_UNSUPPORTED");
    }

    // Capture both independent F07 evidence types. No click/submit in F09.
    steps.push({id:"observe",kind:"observe",timeoutMs:2000,replaySafe:true});
    const mission = {
      schemaVersion:1 as const,
      missionId:task.missionId,
      taskId:task.taskId,
      deadlineMs:task.deadlineMs,
      evidenceDir:task.evidenceDir,
      sessionStatePath:task.sessionStatePath,
      steps,
    };
    return {
      adapterId:this.id,
      capabilities:[...capabilities[this.id]],
      mission,
      requiredEvidence:["dom","screenshot"],
      delegatedCapability:delegated[this.id],
      status:"simulation_ready",
    };
  }
}

export function registeredAdapters():PlatformAdapter[] {
  return ids.map((id)=>new PlatformAdapter(id));
}
