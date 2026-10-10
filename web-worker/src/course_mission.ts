import {createHash,randomUUID} from "node:crypto";
import {mkdir,writeFile} from "node:fs/promises";
import {join,resolve,sep} from "node:path";
import {homedir} from "node:os";
import {verifyObjectiveReadGrant} from "../../operator-control/src/objective_grant.ts";
import type {SignedObjectiveReadGrant} from "../../operator-control/src/objective_grant.ts";
import {readCourseFromExistingEdge} from "./edge_course_reader.ts";
import {readCourseFromEdgeExtension} from "./edge_extension_reader.ts";
import {readPortableBridgeRuntime} from "./portable_browser_bridge.ts";
import type {CourseReadFailure} from "./edge_course_reader.ts";
import {makeLessonRecord,persistLessonRecord,loadLessonRecord} from "./course_knowledge.ts";

const ID=/^[A-Za-z0-9_.:-]{1,140}$/;
const SHA256=(s:string)=>createHash("sha256").update(s).digest("hex");
export type CourseLessonTarget={url:string;accountSelector:string;lessonSelector:string};
export type CourseMissionRequest={
  schemaVersion:1;missionId:string;grant:SignedObjectiveReadGrant;
  cdpPort:number;lessons:CourseLessonTarget[];
};
export type CourseMissionReport={
  schemaVersion:1;missionId:string;executor:"tma-web-worker";
  trainingOnly:true;productionProven:false;missionProven:false;
  status:"blocked"|"captured_unassessed";
  reason:string;captureCount:number;sourceEvidence:{url:string;sourceSha256:string;recordFile:string}[];
  observedAt:string;
};
type RunOptions={signingKey:Buffer;authorizedOperators:ReadonlySet<string>;evidenceRoot:string};
function outputRoot(path:string):string {
  const resolved=resolve(path);
  // Installed product writes to per-user state. Tests can supply a trusted base.
  const state=process.env.LOCALAPPDATA||process.env.XDG_STATE_HOME||
    join(homedir(),".local","state");
  const base=resolve(process.env.TMA_COURSE_EVIDENCE_BASE??join(state,"TMA","course-evidence"));
  if(!resolved.toLowerCase().startsWith((base+sep).toLowerCase()))
    throw Error("COURSE_EVIDENCE_ROOT_NOT_ISOLATED");
  return resolved;
}
export async function runCourseMission(req:CourseMissionRequest,opt:RunOptions):Promise<CourseMissionReport> {
  if(req?.schemaVersion!==1||!ID.test(req.missionId)||
     !Array.isArray(req.lessons)||req.lessons.length<1||req.lessons.length>20)
    throw Error("COURSE_MISSION_INVALID");
  // Trust boundary: verify the signed grant BEFORE browser access or file mutation.
  const receipt=verifyObjectiveReadGrant(req.grant,opt.signingKey,Date.now(),opt.authorizedOperators);
  if(receipt.missionId!==req.missionId||receipt.adapterId!=="homeocta"||
     receipt.authorizedOrigin!=="https://www.homeocta.com")
    throw Error("COURSE_MISSION_GRANT_MISMATCH");
  if(!Number.isSafeInteger(req.cdpPort)||req.cdpPort<1024||req.cdpPort>65535)
    throw Error("COURSE_CDP_PORT_INVALID");
  for(const lesson of req.lessons) {
    if(typeof lesson?.url!=="string"||typeof lesson.accountSelector!=="string"||
       typeof lesson.lessonSelector!=="string"||
       !lesson.accountSelector.trim()||!lesson.lessonSelector.trim()||
       new URL(lesson.url).origin!==receipt.authorizedOrigin)
      throw Error("COURSE_LESSON_SCOPE_DENIED");
  }
  const target=outputRoot(opt.evidenceRoot);
  await mkdir(target,{recursive:true});
  const sourceEvidence:CourseMissionReport["sourceEvidence"]=[];
  let reason="NONE";
  for(const lesson of req.lessons) {
    let response=await readCourseFromExistingEdge({
      grant:req.grant,signingKey:opt.signingKey,authorizedOperatorIds:opt.authorizedOperators,
      url:lesson.url,port:req.cdpPort,accountSelector:lesson.accountSelector,
      lessonSelector:lesson.lessonSelector,timeoutMs:10000
    });
    // The extension reads the authorized inactive tab without CDP or desktop focus.
    if(response.status==="blocked"&&response.reason==="EDGE_PROFILE_NOT_ACCESSIBLE"){
      // Runtime binding follows the logged-on USER, not the USB mount point.
      // A trusted test ingress may override it, but installation is never faked.
      let identity:{extensionId:string;bridgeToken:string;port:number}|null=null;
      const fromEnv=process.env.TMA_EDGE_EXTENSION_ID&&process.env.TMA_EDGE_BRIDGE_TOKEN;
      if(fromEnv){
        identity={extensionId:process.env.TMA_EDGE_EXTENSION_ID!,
          bridgeToken:process.env.TMA_EDGE_BRIDGE_TOKEN!,
          port:Number(process.env.TMA_EDGE_BRIDGE_PORT??18764)};
      }else{
        const saved=await readPortableBridgeRuntime().catch(()=>null);
        if(saved?.extensionId)identity={extensionId:saved.extensionId,
          bridgeToken:saved.bridgeToken,port:saved.port};
      }
      if(identity){
        response=await readCourseFromEdgeExtension({
          grant:req.grant,signingKey:opt.signingKey,authorizedOperatorIds:opt.authorizedOperators,
          url:lesson.url,port:req.cdpPort,accountSelector:lesson.accountSelector,
          lessonSelector:lesson.lessonSelector,extensionId:identity.extensionId,
          bridgeToken:identity.bridgeToken,listenPort:identity.port,waitMs:38000
        });
      }
    }
    if(response.status==="blocked"){reason=response.reason as CourseReadFailure;break;}
    // Each distinct source has its own immutable durable record; never clobber another lesson.
    const recordId=req.grant.receipt.objectiveId.slice(0,90)+"-"+SHA256(req.grant.receipt.objectiveId+"|"+response.url).slice(0,24);
    const record=makeLessonRecord(recordId,response);
    const lessonDir=join(target,"lessons");
    let recordFile:string;
    try {recordFile=await persistLessonRecord(lessonDir,record);}
    catch(error){
      if((error as NodeJS.ErrnoException).code!=="EEXIST") throw error;
      const persisted=await loadLessonRecord(lessonDir,recordId);
      if(persisted.sourceSha256!==record.sourceSha256||persisted.sourceUrl!==record.sourceUrl)
        throw Error("SOURCE_CHANGED_REQUIRES_REVIEW");
      recordFile=join(lessonDir,recordId+".json");
    }
    sourceEvidence.push({url:response.url,sourceSha256:record.sourceSha256,recordFile});
  }
  const report:CourseMissionReport={
    schemaVersion:1,missionId:req.missionId,executor:"tma-web-worker",
    trainingOnly:true,productionProven:false,missionProven:false,
    status:reason==="NONE"?"captured_unassessed":"blocked",
    reason,captureCount:sourceEvidence.length,sourceEvidence,
    observedAt:new Date().toISOString()
  };
  await writeFile(join(target,req.missionId+"-execution-"+randomUUID()+".json"),
    JSON.stringify(report,null,2)+"\n",{flag:"wx",encoding:"utf8"});
  return report;
}
