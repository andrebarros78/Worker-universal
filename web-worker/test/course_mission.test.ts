import assert from "node:assert/strict";
import test from "node:test";
import {spawnSync} from "node:child_process";
import {createServer} from "node:net";
import {mkdtemp,mkdir,readFile,readdir,rm,writeFile} from "node:fs/promises";
import {join} from "node:path";
import {tmpdir} from "node:os";
import {createObjectiveAuthorization} from "../../operator-control/src/objective.ts";
import {proposeAction} from "../../operator-control/src/decision_agent.ts";
import {issueObjectiveReadGrant} from "../../operator-control/src/objective_grant.ts";
import {runCourseMission} from "../src/course_mission.ts";

process.env.TMA_COURSE_EVIDENCE_BASE=tmpdir();
const baseDir=join(tmpdir(),"tma-course-homologation-tests");
const key=Buffer.alloc(32,0x71);
const operators=new Set(["operator-primary"]);
function make(now=Date.now()){
  const objective=createObjectiveAuthorization({
    objectiveId:"CURSO_OCTA_KNOWLEDGE",operatorId:"operator-primary",
    objectiveText:"Attend authenticated Curso Octa in Edge for training and knowledge",
    targetOrigin:"https://www.homeocta.com",authorizedActions:["read_account_data"],
    authorizedScopes:["course.read"],sessionMode:"operator_current_browser_session",
    training:true,productiveHomologation:false,issuedAtMs:now-100,expiresAtMs:now+60000
  });
  const p=proposeAction({
    missionId:"course-octa-native-worker",adapterId:"homeocta",action:"read_account_data",
    targetOrigin:"https://www.homeocta.com",requestedScopes:["course.read"],
    observations:[],unresolvedQuestions:[],estimatedCostCents:null,expectedIncomeCents:null
  });
  const grant=issueObjectiveReadGrant(p,objective,key,now,operators);
  return {
    schemaVersion:1 as const,missionId:p.missionId,grant,
    cdpPort:9222,lessons:[{
      url:"https://www.homeocta.com/",accountSelector:"[data-account-marker]",
      lessonSelector:"[data-lesson-body]"
    }]
  };
}
async function withRoot(fn:(root:string)=>Promise<void>){
  await mkdir(baseDir,{recursive:true});
  const tmp=await mkdtemp(join(baseDir,"run-"));
  try {await fn(tmp);}finally{await rm(tmp,{recursive:true,force:true});}
}
test("native Worker refuses a forged objective before any browser or evidence write",async()=>{
  await withRoot(async root=>{
    const request=make();
    request.grant={...request.grant,receipt:{...request.grant.receipt,operatorId:"another-operator"}};
    await assert.rejects(()=>runCourseMission(request,{signingKey:key,authorizedOperators:operators,evidenceRoot:root}),
      /GRANT_SIGNATURE_INVALID/);
    assert.deepEqual(await readdir(root),[]);
  });
});
test("native Worker rejects unapproved origin and commercial action",async()=>{
  await withRoot(async root=>{
    const r=make();
    r.lessons[0].url="https://unrelated.example/";
    await assert.rejects(()=>runCourseMission(r,{signingKey:key,authorizedOperators:operators,evidenceRoot:root}),
       /COURSE_LESSON_SCOPE_DENIED/);
    assert.deepEqual(await readdir(root),[]);
  });
});
test("native Worker makes a blocked receipt rather than claiming learning from closed CDP",async()=>{
  await withRoot(async root=>{
    const server=createServer();
    await new Promise<void>(res=>server.listen(0,"127.0.0.1",res));
    const addr=server.address(); assert.ok(addr && typeof addr==="object");
    const port=addr.port;
    await new Promise<void>(res=>server.close(()=>res()));
    const request=make();request.cdpPort=port;
    const report=await runCourseMission(request,{signingKey:key,authorizedOperators:operators,evidenceRoot:root});
    assert.equal(report.status,"blocked");
    assert.equal(report.reason,"EDGE_PROFILE_NOT_ACCESSIBLE");
    assert.equal(report.captureCount,0);
    assert.equal(report.productionProven,false);
    assert.equal(report.missionProven,false);
    const entries=await readdir(root);
    assert.equal(entries.length,1);
    const receipt=JSON.parse(await readFile(join(root,entries[0]),"utf8"));
    assert.equal(receipt.executor,"tma-web-worker");
    assert.equal(receipt.status,"blocked");
  });
});
test("real native Worker CLI process emits verified blocked receipt instead of fabricated training",async()=>{
  await withRoot(async root=>{
    const req=make();
    const reqPath=join(root,"request.json");
    await writeFile(reqPath,JSON.stringify(req),"utf8");
    const out=join(root,"receipts");
    const child=spawnSync(process.execPath,["web-worker/src/course_worker_cli.ts","run-course",reqPath],{
      cwd:process.cwd(),encoding:"utf8",timeout:13000,
      env:{...process.env,TMA_COURSE_GRANT_KEY_B64:key.toString("base64"),
        TMA_COURSE_EVIDENCE_ROOT:out,TMA_COURSE_ALLOWED_OPERATORS:"operator-primary"}
    });
    assert.equal(child.status,3,"stdout="+child.stdout+" stderr="+child.stderr);
    assert.match(child.stdout,/WORKER_COURSE_MISSION=/);
    assert.match(child.stdout,/"executor":"tma-web-worker"/);
    const saved=await readdir(out);
    assert.equal(saved.length,1);
    const data=JSON.parse(await readFile(join(out,saved[0]),"utf8"));
    assert.equal(data.missionProven,false);
    assert.equal(data.captureCount,0);
  });
});
test("CLI without trusted ingress never contacts Edge or writes records",async()=>{
 await withRoot(async root=>{
  const requestPath=join(root,"request.json");await writeFile(requestPath,JSON.stringify(make()));
  const p=spawnSync(process.execPath,["web-worker/src/course_worker_cli.ts","run-course",requestPath],{
    cwd:process.cwd(),encoding:"utf8",timeout:7000,
    env:{...process.env,TMA_COURSE_GRANT_KEY_B64:"",TMA_COURSE_ALLOWED_OPERATORS:"",TMA_COURSE_EVIDENCE_ROOT:""}
  });
  assert.equal(p.status,4);assert.match(p.stderr,/TRUSTED_INGRESS_NOT_CONFIGURED/);
  assert.deepEqual(await readdir(root),["request.json"]);
 });
});


test("course evidence cannot be redirected to a lookalike SentinelX directory",async()=>{

 const request=make();

 const invalid="C:\\Users\\Public\\sentinelx\\workspace\\course-output";

 await assert.rejects(()=>runCourseMission(request,{

  signingKey:key,authorizedOperators:operators,evidenceRoot:invalid

 }),/COURSE_EVIDENCE_ROOT_NOT_ISOLATED/);

});

