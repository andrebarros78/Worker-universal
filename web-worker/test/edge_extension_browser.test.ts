import assert from "node:assert/strict";
import test from "node:test";
import {createHash,randomUUID} from "node:crypto";
import {mkdtemp,mkdir,rm,readFile} from "node:fs/promises";
import {resolve,join,dirname} from "node:path";
import {tmpdir} from "node:os";
import {fileURLToPath} from "node:url";
import {chromium} from "playwright";
import {createObjectiveAuthorization} from "../../operator-control/src/objective.ts";
import {proposeAction} from "../../operator-control/src/decision_agent.ts";
import {issueObjectiveReadGrant} from "../../operator-control/src/objective_grant.ts";
import {readCourseFromEdgeExtension} from "../src/edge_extension_reader.ts";
import {preparePortableBrowserBridge,readPortableBridgeRuntime} from "../src/portable_browser_bridge.ts";
import {runCourseMission} from "../src/course_mission.ts";
import {loadLessonRecord} from "../src/course_knowledge.ts";

process.env.TMA_COURSE_EVIDENCE_BASE=tmpdir();
test("F14 real isolated Chromium background tab -> extension -> local Worker receiver",{
  timeout:85000
},async()=>{
 const base=await mkdtemp(join(tmpdir(),"tma-f14-browser-e2e-"));
 const profile=await mkdtemp(join(base,"profile-"));
 const stateRoot=join(profile,"LocalAppData");
 const prepared=await preparePortableBrowserBridge({stateRoot});
 const extension=prepared.extensionDir;
 const runtime=await readPortableBridgeRuntime({stateRoot});
 const bridgeToken=runtime.bridgeToken;
 assert.match(bridgeToken,/^[a-f0-9]{64}$/);
 let context:Awaited<ReturnType<typeof chromium.launchPersistentContext>>|undefined;
 try{
  context=await chromium.launchPersistentContext(profile,{
    executablePath:resolve(dirname(fileURLToPath(import.meta.url)),"..","runtime","chromium-win-x64","chrome.exe"),headless:true,
    args:["--disable-extensions-except="+extension,"--load-extension="+extension]
  });
  let worker=context.serviceWorkers().find(w=>w.url().startsWith("chrome-extension://"));
  if(!worker){
    try{worker=await context.waitForEvent("serviceworker",{timeout:12000});}catch{}
  }
  assert.ok(worker,"F14_ISOLATED_EXTENSION_DID_NOT_START");
  const id=new URL(worker.url()).hostname;
  assert.match(id,/^[a-p]{32}$/);
  console.log("F14_STAGED_EXTENSION_ID="+id);
  const account='<div class="account">Autorizado em teste sintético, sem autenticação real</div>';
  const lesson="Esta aula sintética avalia a leitura do texto do documento na aba inativa do navegador. "+
    "O Worker deve receber somente o conteúdo vinculado à origem autorizada, preservando a navegação e "+
    "as outras abas sem capturar credenciais nem transferir dados para serviços externos.";
  await context.route("https://www.homeocta.com/**",route=>route.fulfill({
    status:200,contentType:"text/html",
    body:"<html><head><title>F14 Test Only</title></head><body>"+account+
      "<main><article>"+lesson+"</article></main></body></html>"
  }));
  const page=await context.newPage();
  await page.goto("https://www.homeocta.com/curso/f14-simulado");
  await page.locator("main article").waitFor();
  const other=await context.newPage();
  await other.goto("about:blank");
  const permittedTabs=await worker.evaluate(async()=>chrome.tabs.query({url:["https://www.homeocta.com/*"]}));
  assert.equal(permittedTabs.length,1);
  const courseTab=permittedTabs[0];
  await worker.evaluate(async(win)=>chrome.tabs.create({windowId:win,url:"about:blank",active:true}),courseTab.windowId);
  const inactive=await worker.evaluate(async(id)=>(await chrome.tabs.get(id)).active,courseTab.id);
  assert.equal(inactive,false,"COURSE_TAB_MUST_BE_IN_BACKGROUND");
  const key=Buffer.alloc(32,0x5d);
  const now=Date.now();
  const obj=createObjectiveAuthorization({
    objectiveId:"F14_BROWSER_ISOLATED_E2E",operatorId:"operator-primary",
    objectiveText:"Test isolated Chromium course reading with no operator profile",
    targetOrigin:"https://www.homeocta.com",authorizedActions:["read_account_data"],
    authorizedScopes:["course.read"],sessionMode:"operator_current_browser_session",
    training:true,productiveHomologation:false,issuedAtMs:now-100,
    expiresAtMs:now+70000
  });
  const proposal=proposeAction({
    missionId:"f14-isolated-browser",adapterId:"homeocta",action:"read_account_data",
    targetOrigin:"https://www.homeocta.com",requestedScopes:["course.read"],
    observations:[],unresolvedQuestions:[],expectedIncomeCents:null,estimatedCostCents:null
  });
  const grant=issueObjectiveReadGrant(proposal,obj,key,now,new Set(["operator-primary"]));
  const pending=readCourseFromEdgeExtension({
    grant,signingKey:key,authorizedOperatorIds:new Set(["operator-primary"]),
    url:"https://www.homeocta.com/curso/f14-simulado",port:9222,
    accountSelector:".account",lessonSelector:"main article",
    extensionId:id,bridgeToken,waitMs:9000
  });
  // The real MV3 service worker polls the Worker automatically; no mock call.
  const output=await pending;
  assert.equal(output.status,"read",JSON.stringify(output));
  if(output.status==="read"){
    assert.equal(output.source,"existing_edge_extension_background_tab");
    const visibleLessonText=(await page.locator("main article").innerText()).trim();
    assert.equal(output.sha256,createHash("sha256").update(visibleLessonText).digest("hex"));
    assert.equal(output.url,"https://www.homeocta.com/curso/f14-simulado");
  }
  assert.equal(await other.url(),"about:blank");
  const remainsInactive=await worker.evaluate(async(id)=>(await chrome.tabs.get(id)).active,courseTab.id);
  assert.equal(remainsInactive,false);
  // The actual mission orchestrator, not only the extension reader, must
  // fall back from closed CDP to the isolated browser extension and save memory.
  const oldId=process.env.TMA_EDGE_EXTENSION_ID;
  const oldToken=process.env.TMA_EDGE_BRIDGE_TOKEN;
  const evidenceDir=join(base,"mission-"+randomUUID());
  process.env.TMA_EDGE_EXTENSION_ID=id;
  process.env.TMA_EDGE_BRIDGE_TOKEN=bridgeToken;
  try{
    const mission=runCourseMission({
      schemaVersion:1,missionId:proposal.missionId,grant,cdpPort:9223,
      lessons:[{url:"https://www.homeocta.com/curso/f14-simulado",
        accountSelector:".account",lessonSelector:"main article"}]
    },{signingKey:key,authorizedOperators:new Set(["operator-primary"]),
       evidenceRoot:evidenceDir});
    await new Promise(res=>setTimeout(res,3800));
    await worker.evaluate(()=>poll());
    const report=await mission;
    assert.equal(report.status,"captured_unassessed",JSON.stringify(report));
    assert.equal(report.captureCount,1);
    assert.equal(report.missionProven,false);
    const recordId=obj.objectiveId+"-"+createHash("sha256")
      .update(obj.objectiveId+"|https://www.homeocta.com/curso/f14-simulado")
      .digest("hex").slice(0,24);
    const saved=await loadLessonRecord(join(evidenceDir,"lessons"),recordId);
    assert.equal(saved.learningStatus,"captured_unassessed");
    assert.equal(saved.sourceSha256,output.sha256);
  }finally {
    if(oldId===undefined)delete process.env.TMA_EDGE_EXTENSION_ID;
    else process.env.TMA_EDGE_EXTENSION_ID=oldId;
    if(oldToken===undefined)delete process.env.TMA_EDGE_BRIDGE_TOKEN;
    else process.env.TMA_EDGE_BRIDGE_TOKEN=oldToken;
    await rm(evidenceDir,{recursive:true,force:true}).catch(()=>{});
  }
 }finally{
   if(context)await context.close().catch(()=>{});
   await rm(profile,{recursive:true,force:true}).catch(()=>{});
 }
});
