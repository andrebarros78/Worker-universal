import {readFile} from "node:fs/promises";
import {existsSync} from "node:fs";
import {spawnSync} from "node:child_process";
import {fileURLToPath} from "node:url";
import {dirname,resolve} from "node:path";
import {runCourseMission} from "./course_mission.ts";
import type {CourseMissionRequest,CourseMissionReport} from "./course_mission.ts";

// A captured lesson is not an assessed lesson. Exit 0 is forbidden until independent proof exists.
export function exitCodeForCourseReport(report:CourseMissionReport):number {
  if(report.missionProven!==false || report.productionProven!==false ||
     !["blocked","captured_unassessed"].includes(report.status))
    throw Error("COURSE_PROOF_STATE_INCONSISTENT");
  return report.status==="blocked"?3:2;
}

// Standalone entrypoint executed by the Worker, not by a chat imitation of it.
// Signed grant originates in the trusted operator ingress and is supplied via a file.
// Secrets are read from a private runtime environment, never CLI args or evidence files.
export async function main(argv:string[],env:NodeJS.ProcessEnv):Promise<number>{
  if(argv.length===1&&argv[0]==="probe-edge-session") {
    if(process.platform!=="win32")return 64;
    const helper=resolve(dirname(fileURLToPath(import.meta.url)),"..","adapters","run_interactive_bridge.ps1");
    if(!existsSync(helper)){
      console.error("WORKER_INTERACTIVE_DIAGNOSTIC_NOT_PACKAGED");
      return 65;
    }
    const child=spawnSync("powershell.exe",["-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-File",helper],{
      encoding:"utf8",timeout:40000,windowsHide:true
    });
    if(child.stdout)process.stdout.write(child.stdout);
    if(child.stderr)process.stderr.write(child.stderr.slice(0,500));
    return child.status??5;
  }
  if(argv.length!==2||argv[0]!=="run-course") {
    console.error("WORKER_COURSE_COMMAND_DENIED");
    return 64;
  }
  try {
    const keyText=env.TMA_COURSE_GRANT_KEY_B64;
    const evidenceRoot=env.TMA_COURSE_EVIDENCE_ROOT;
    const operatorIds=env.TMA_COURSE_ALLOWED_OPERATORS;
    if(!keyText||!evidenceRoot||!operatorIds)
      throw Error("TRUSTED_INGRESS_NOT_CONFIGURED");
    const key=Buffer.from(keyText,"base64");
    if(key.length<32)throw Error("INVALID_INGRESS_KEY");
    const request=JSON.parse(await readFile(argv[1],"utf8")) as CourseMissionRequest;
    const report=await runCourseMission(request,{
      signingKey:key,
      authorizedOperators:new Set(operatorIds.split(",").map(s=>s.trim()).filter(Boolean)),
      evidenceRoot
    });
    console.log("WORKER_COURSE_MISSION="+JSON.stringify(report));
    return exitCodeForCourseReport(report);
  }catch(err){
    // No PII, secrets, or browser state in error logs.
    const message=err instanceof Error?err.message:"UNKNOWN_ERROR";
    console.error("WORKER_COURSE_MISSION_BLOCKED="+message);
    return 4;
  }
}
if(process.argv[1] && /course_worker_cli[.]ts$/.test(process.argv[1])){
  process.exitCode=await main(process.argv.slice(2),process.env);
}
