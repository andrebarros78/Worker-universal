import assert from "node:assert/strict";
import { mkdtemp,rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { SyntheticFixtureServer } from "../../web-worker/test/fixture_server.ts";
import { sdkAdapters } from "../src/sdk_bridge.ts";
import type { Invocation } from "../../adapter-sdk/typescript/sdk.ts";

test("F02 Adapter SDK invokes five simulated adapters without productive success",async()=>{
  const fixture=new SyntheticFixtureServer();
  const root=await mkdtemp(join(tmpdir(),"tma-f09-sdk-"));
  await fixture.start();
  try{
    for(const adapter of sdkAdapters()){
      const id=adapter.manifest().adapterId;
      const missionId="sdk-"+id;
      const config={
        schemaVersion:1,adapterId:id,mode:"simulation",
        grantedScopes:["platform.read","practice.fill"],
        origin:new URL(fixture.url("/form")).origin,
        eligibilityVerified:false,authorizationVerified:false,
        productionQualified:false,humanReviewed:false,externalEffectsApproved:false,
      };
      const task={
        missionId,taskId:"task-"+id,url:fixture.url("/form"),deadlineMs:8000,
        evidenceDir:join(root,id,"evidence"),sessionStatePath:join(root,id,"session.json"),
        ...(id==="generic_form"?{formFields:[{label:"Full name",value:"Synthetic User"}]}:{}),
      };
      const invocation:Invocation={
        invocationId:"invoke-"+id,
        missionId,
        capabilityId:adapter.manifest().capabilities[0],
        contractVersion:"1.0.0",
        timeoutMs:8000,
        simulation:true,
        payload:{policy:config,task},
        secretRefs:[],artifactRefs:[],
      };
      const health=await adapter.health();
      assert.equal(health.readiness,"configured");
      const outcome=await adapter.invoke(invocation);
      assert.equal(outcome.status,"needs_review");
      assert.equal((outcome.payload as {status:string}).status,"simulated");
      assert.ok(outcome.evidence.some(x=>x.kind==="dom"));
      assert.ok(outcome.evidence.some(x=>x.kind==="screenshot"));
    }
  }finally{
    await fixture.stop();
    await rm(root,{recursive:true,force:true});
  }
});

test("F02 SDK refuses live invoke, malformed payload, incompatible contract and cancellation",async()=>{
  const adapter=sdkAdapters()[0];
  const base:Invocation={
    invocationId:"reject",missionId:"m",capabilityId:"browser.read",
    contractVersion:"1.0.0",timeoutMs:100,simulation:true,
    payload:{},secretRefs:[],artifactRefs:[],
  };
  assert.equal((await adapter.invoke({...base,simulation:false})).status,"needs_review");
  assert.equal((await adapter.invoke(base)).status,"failed");
  assert.equal((await adapter.invoke({...base,contractVersion:"2.0.0"})).status,"failed");
  assert.equal((await adapter.invoke({...base,capabilityId:"unknown"})).status,"failed");
  const controller=new AbortController();controller.abort();
  assert.equal((await adapter.invoke(base,controller.signal)).status,"cancelled");
});
