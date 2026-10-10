import assert from "node:assert/strict";
import test from "node:test";
import {createServer} from "node:net";
import {createHash} from "node:crypto";
import {createObjectiveAuthorization} from "../../operator-control/src/objective.ts";
import {proposeAction} from "../../operator-control/src/decision_agent.ts";
import {issueObjectiveReadGrant} from "../../operator-control/src/objective_grant.ts";
import {readCourseFromEdgeExtension} from "../src/edge_extension_reader.ts";

const origin="chrome-extension://abcdefghijklmnopabcdefghijklmnop";
const extId="abcdefghijklmnopabcdefghijklmnop";
const key=Buffer.alloc(32,0x51);
const bridgeToken="c".repeat(64);
const operators=new Set(["operator-primary"]);
const now=Date.now();
const obj=createObjectiveAuthorization({
 objectiveId:"F14_TEST_OBJECTIVE",operatorId:"operator-primary",
 objectiveText:"Read only HomeOcta background lesson for knowledge",
 targetOrigin:"https://www.homeocta.com",authorizedActions:["read_account_data"],
 authorizedScopes:["course.read"],sessionMode:"operator_current_browser_session",
 training:true,productiveHomologation:false,issuedAtMs:now-1000,expiresAtMs:now+90000
});
const proposal=proposeAction({
 missionId:"f14-bridge-test",adapterId:"homeocta",action:"read_account_data",
 targetOrigin:"https://www.homeocta.com",requestedScopes:["course.read"],
 observations:[],unresolvedQuestions:[],expectedIncomeCents:null,estimatedCostCents:null
});
const grant=issueObjectiveReadGrant(proposal,obj,key,now,operators);
const opts={grant,signingKey:key,authorizedOperatorIds:operators,
 url:"https://www.homeocta.com/",port:9222,accountSelector:".account",
 lessonSelector:"main article",extensionId:extId,bridgeToken,waitMs:1500};
async function freePort(){
 const server=createServer();
 await new Promise<void>(res=>server.listen(0,"127.0.0.1",res));
 const a=server.address();assert.ok(a && typeof a==="object");
 const port=a.port;await new Promise<void>(res=>server.close(()=>res()));
 return port;
}
async function job(port:number){
 for(let retry=0;retry<60;retry++){
  try {
   const res=await fetch("http://127.0.0.1:"+port+"/v1/job",{headers:{Origin:origin,"X-TMA-Bridge-Token":bridgeToken}});
   if(res.ok)return (await res.json()).job;
  }catch{}
  await new Promise(res=>setTimeout(res,20));
 }
 throw Error("TEST_LOCAL_EXTENSION_BRIDGE_NOT_LISTENING");
}
test("F14 invalid or forged grant cannot expose background-tab jobs",async()=>{
 const result=await readCourseFromEdgeExtension({...opts,listenPort:await freePort(),
  grant:{...grant,signatureHex:"0".repeat(64)}});
 assert.equal(result.status,"blocked");
 if(result.status==="blocked")assert.equal(result.reason,"OBJECTIVE_SCOPE_DENIED");
});
test("F14 a valid signed objective can read a simulated extension reply without CDP",async()=>{
 const port=await freePort();
 const promise=readCourseFromEdgeExtension({...opts,listenPort:port});
 const j=await job(port);
 assert.equal(j.origin,"https://www.homeocta.com");
 assert.equal(j.type,"course.read");
 const missingToken=await fetch("http://127.0.0.1:"+port+"/v1/job",{headers:{Origin:origin}});
 assert.equal(missingToken.status,403);
 const incorrectToken=await fetch("http://127.0.0.1:"+port+"/v1/job",
   {headers:{Origin:origin,"X-TMA-Bridge-Token":"a".repeat(64)}});
 assert.equal(incorrectToken.status,403);
 const extensionWithoutOrigin=await fetch("http://127.0.0.1:"+port+"/v1/job",
   {headers:{"X-TMA-Bridge-Token":bridgeToken}});
 assert.equal(extensionWithoutOrigin.status,200);
 const foreign=await fetch("http://127.0.0.1:"+port+"/v1/job",
   {headers:{Origin:"https://attacker.example","X-TMA-Bridge-Token":bridgeToken}});
 assert.equal(foreign.status,403);
 const body="Esta aula de homologação é estritamente sintética e testa o transporte de leitura autenticada. O objetivo é provar que somente o conteúdo da origem permitida pode chegar ao Worker, sem acessar cookies, senhas ou outras abas.";
 const res=await fetch("http://127.0.0.1:"+port+"/v1/result",{
  method:"POST",headers:{Origin:origin,"Content-Type":"application/json","X-TMA-Bridge-Token":bridgeToken},
  body:JSON.stringify({jobId:j.jobId,status:"read",url:"https://www.homeocta.com/curso/ficticio",
   title:"Synthetic Test",text:body})
 });
 assert.equal(res.status,200);
 const proof=await promise;
 assert.equal(proof.status,"read");
 if(proof.status==="read"){
  assert.equal(proof.source,"existing_edge_extension_background_tab");
  assert.equal(proof.sha256,createHash("sha256").update(body).digest("hex"));
 }
});
test("F14 extension cannot inject evidence from unapproved site",async()=>{
 const port=await freePort();
 const promise=readCourseFromEdgeExtension({...opts,listenPort:port});
 const j=await job(port);
 const res=await fetch("http://127.0.0.1:"+port+"/v1/result",{
  method:"POST",headers:{Origin:origin,"Content-Type":"application/json","X-TMA-Bridge-Token":bridgeToken},
  body:JSON.stringify({jobId:j.jobId,status:"read",url:"https://unapproved.example/test",
   title:"Other site",text:"a".repeat(120)})
 });
 assert.equal(res.status,200);
 const proof=await promise;
 assert.equal(proof.status,"blocked");
 if(proof.status==="blocked")assert.equal(proof.reason,"COURSE_READ_FAILED");
});
test("F14 closed/uninstalled extension returns real timeout failure, never evidence",async()=>{
 const result=await readCourseFromEdgeExtension({...opts,listenPort:await freePort(),waitMs:250});
 assert.equal(result.status,"blocked");
 if(result.status==="blocked")assert.equal(result.reason,"EXTENSION_NOT_CONNECTED");
});
