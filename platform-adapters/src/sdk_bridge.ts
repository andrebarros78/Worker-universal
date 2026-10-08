import type {
  Adapter,AdapterResult,Invocation,AdapterManifest,
} from "../../adapter-sdk/typescript/sdk.ts";
import { PlatformAdapter } from "./adapters.ts";
import { executeSimulated } from "./runner.ts";
import type { AdapterPolicy,AdapterTask,AdapterId } from "./models.ts";

type InvocationPayload = {policy:AdapterPolicy;task:AdapterTask};

function invocationPayload(value:unknown):InvocationPayload {
  if(!value || typeof value!=="object") throw new Error("F09_SDK_INVALID_PAYLOAD");
  const candidate=value as Partial<InvocationPayload>;
  if(!candidate.policy || !candidate.task) throw new Error("F09_SDK_INVALID_PAYLOAD");
  return {policy:candidate.policy,task:candidate.task};
}

export class PlatformSdkAdapter implements Adapter {
  private readonly implementation:PlatformAdapter;
  constructor(id:AdapterId) { this.implementation=new PlatformAdapter(id); }

  manifest():AdapterManifest {return this.implementation.manifest();}

  async health():Promise<{lifecycle:string;readiness:string}> {
    const health=this.implementation.health();
    return {lifecycle:health.lifecycle,readiness:health.readiness};
  }

  async invoke(invocation:Invocation,signal?:AbortSignal):Promise<AdapterResult> {
    if(signal?.aborted) return this.error("cancelled","aborted_before_execution");
    if(!invocation.simulation) return this.error("needs_review","F09_LIVE_NOT_AUTHORIZED");
    if(!this.manifest().capabilities.includes(invocation.capabilityId)) {
      return this.error("failed","F09_UNKNOWN_CAPABILITY");
    }
    if(invocation.contractVersion.split(".")[0]!=="1") {
      return this.error("failed","F09_CONTRACT_MISMATCH");
    }
    try {
      const {policy,task}=invocationPayload(invocation.payload);
      if(task.missionId!==invocation.missionId) {
        throw new Error("F09_SDK_MISSION_ID_MISMATCH");
      }
      const report=await executeSimulated(this.implementation,policy,task);
      // A successful simulation is NEVER a durable productive Succeeded event.
      const status:AdapterResult["status"] =
        report.status==="simulated" ? "needs_review":
        report.status==="deadline_exceeded" ? "deadline_exceeded":"failed";
      return {
        status,
        payload:report,
        errors:status==="needs_review"?[]:[{reason:report.reason}],
        evidence:report.evidence.map(item=>({kind:item.kind,sha256:item.sha256,bytes:item.bytes})),
        artifactRefs:[],
        costMicrounits:0,
      };
    }catch(error){
      return this.error("failed",error instanceof Error?error.message:String(error));
    }
  }

  private error(status:AdapterResult["status"],reason:string):AdapterResult {
    return {status,errors:[{reason}],evidence:[],artifactRefs:[],costMicrounits:0};
  }
}

export function sdkAdapters():PlatformSdkAdapter[] {
  return (["generic_web","generic_form","generic_invoice","homeocta","99freelas"] as AdapterId[])
    .map(id=>new PlatformSdkAdapter(id));
}
