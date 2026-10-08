import assert from "node:assert/strict";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { SyntheticFixtureServer } from "../../web-worker/test/fixture_server.ts";
import { registeredAdapters } from "../src/adapters.ts";
import { executeSimulated } from "../src/runner.ts";
import type { AdapterPolicy,AdapterTask,AdapterId } from "../src/models.ts";

function policy(adapterId:AdapterId,origin:string):AdapterPolicy {
  return {
    schemaVersion:1,adapterId,mode:"simulation",
    grantedScopes:["platform.read","practice.fill"],
    origin,eligibilityVerified:false,authorizationVerified:false,
    productionQualified:false,humanReviewed:false,externalEffectsApproved:false,
  };
}
function task(root:string,url:string,id:AdapterId):AdapterTask {
  return {
    missionId:"f09-"+id,
    taskId:"task-"+id,
    url,deadlineMs:8000,
    evidenceDir:join(root,id,"evidence"),
    sessionStatePath:join(root,id,"session.json"),
    ...(id==="generic_form"?{formFields:[{label:"Full name",value:"Synthetic User"}]}:{}),
  };
}
test("F02 adapter manifests and F08 quarantine for five adapters",()=>{
  const ids=registeredAdapters();
  assert.deepEqual(ids.map(x=>x.id),["generic_web","generic_form","generic_invoice","homeocta","99freelas"]);
  for(const adapter of ids) {
    assert.equal(adapter.manifest().sdkContractVersion,"1.0.0");
    assert.equal(adapter.manifest().simulationSupported,true);
    assert.equal(adapter.health().promotion,"experimental");
    assert.equal(adapter.health().readiness,"configured");
  }
});

test("all five adapters run on isolated F04 Chromium with F07-compatible evidence",async()=>{
  const fixture=new SyntheticFixtureServer();
  const root=await mkdtemp(join(tmpdir(),"tma-f09-"));
  await fixture.start();
  try {
    for(const adapter of registeredAdapters()) {
      const config=policy(adapter.id,new URL(fixture.url("/form")).origin);
      const work=task(root,fixture.url("/form"),adapter.id);
      const plan=adapter.plan(config,work);
      assert.ok(plan.mission.steps.every(step=>!["click","download","upload"].includes(step.kind)));
      assert.deepEqual(plan.requiredEvidence,["dom","screenshot"]);
      if(["homeocta","generic_invoice"].includes(adapter.id)) {
        assert.equal(plan.delegatedCapability,"vision.ocr.tesseract");
      }
      const outcome=await executeSimulated(adapter,config,work);
      assert.equal(outcome.status,"simulated",adapter.id+":"+outcome.reason);
      assert.equal(outcome.validatedEvidence,true);
      assert.ok(outcome.evidence.some(x=>x.kind==="dom"));
      assert.ok(outcome.evidence.some(x=>x.kind==="screenshot"));
      assert.equal(outcome.externalEffect,false);
    }
  } finally {
    await fixture.stop();
    await rm(root,{recursive:true,force:true});
  }
});

test("all five recover real Chromium crash with bounded F04 replay",async()=>{
  const fixture=new SyntheticFixtureServer();
  const root=await mkdtemp(join(tmpdir(),"tma-f09-recovery-"));
  await fixture.start();
  try {
    for(const adapter of registeredAdapters()) {
      const config=policy(adapter.id,new URL(fixture.url("/form")).origin);
      const work=task(root,fixture.url("/form"),adapter.id);
      const outcome=await executeSimulated(adapter,config,work,true);
      assert.equal(outcome.status,"simulated",adapter.id+":"+outcome.reason);
      assert.equal(outcome.recoveryCount,1);
    }
  } finally {
    await fixture.stop();
    await rm(root,{recursive:true,force:true});
  }
});

test("all adapters fail closed for unauthorized external origin and live actions",()=>{
  for(const adapter of registeredAdapters()) {
    const config=policy(adapter.id,"http://127.0.0.1:5050");
    const work=task("C:/test","http://127.0.0.1:5050/form",adapter.id);
    assert.throws(()=>adapter.plan(config,{...work,url:"https://example.org/x"}),/ORIGIN_DENIED/);
    assert.throws(()=>adapter.plan({...config,mode:"read_only"},work),/LIVE_NOT_AUTHORIZED/);
    assert.throws(()=>adapter.plan({...config,grantedScopes:[]},work),/SCOPE_DENIED/);
    assert.throws(()=>adapter.plan({...config,externalEffectsApproved:true},work),/EXTERNAL_EFFECT_DENIED/);
    assert.throws(()=>adapter.plan({...config,origin:"https://homeocta.com"}, {...work,url:"https://homeocta.com/"}),/SIMULATION_LOOPBACK_ONLY/);
  }
});

test("HomeOcta qualification and 99Freelas proposals cannot be filled",()=>{
  for(const id of ["homeocta","99freelas"] as const) {
    const adapter=registeredAdapters().find(x=>x.id===id)!;
    const config=policy(id,"http://127.0.0.1:5050");
    const work=task("C:/test","http://127.0.0.1:5050/form",id);
    assert.throws(()=>adapter.plan(config,{...work,formFields:[{label:"Full name",value:"Someone"}]}));
  }
});

test("deadline exhaustion does not produce simulated completion",async()=>{
  const adapter=registeredAdapters()[0];
  const fixture=new SyntheticFixtureServer();
  const root=await mkdtemp(join(tmpdir(),"tma-f09-timeout-"));
  await fixture.start();
  try{
    const config=policy(adapter.id,new URL(fixture.url("/form")).origin);
    const work={...task(root,fixture.url("/form"),adapter.id),deadlineMs:1};
    const result=await executeSimulated(adapter,config,work);
    assert.notEqual(result.status,"simulated");
    assert.equal(result.externalEffect,false);
  }finally{
    await fixture.stop();
    await rm(root,{recursive:true,force:true});
  }
});
