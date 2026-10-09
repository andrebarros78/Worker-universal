import test from "node:test";
import assert from "node:assert/strict";
import { mkdtemp,readFile,rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { inventory, capabilityForAction } from "../src/capabilities.ts";
import { proposeAction } from "../src/decision_agent.ts";
import { checkOperatorApproval,operatorSignature,reserveAuthorization } from "../src/operator.ts";
import type { OperatorDecision, DecisionProposal } from "../src/contracts.ts";

const operatorId="operator-primary";
const key=Buffer.from("Test fixture only. Never a production operator key. ABCDEFGHIJK");
const now=100_000;
const scopes=["platform.read"];
const proposal=()=>proposeAction({
 missionId:"mission-operator-01",adapterId:"99freelas",
 action:"read_public_data",targetOrigin:"https://www.99freelas.com.br",
 requestedScopes:scopes,observations:["Public data can be examined as an opportunity signal"],
 unresolvedQuestions:["Operator chooses the platform and strategy"],
 expectedIncomeCents:null,estimatedCostCents:null
});
function sign(p:DecisionProposal, fields:Partial<Omit<OperatorDecision,"signatureHex">>={}):OperatorDecision {
 const unsigned:Omit<OperatorDecision,"signatureHex">={
  schemaVersion:1,proposalFingerprint:p.fingerprint,operatorId,action:"approve",
  authorizedScopes:[...p.requestedScopes],authorizedOrigin:p.targetOrigin,
  issuedAtMs:now-100,expiresAtMs:now+5000,nonce:"nonce_example_1234567890",
  ...fields
 };
 return {...unsigned,signatureHex:operatorSignature(unsigned,key)};
}
test("capabilities describe technical reality without commercial verdicts",()=>{
 const list=inventory();
 assert.ok(list.some(c=>c.adapterId==="99freelas"&&c.capabilityId==="browser.read"));
 assert.ok(list.some(c=>c.adapterId==="homeocta"));
 assert.equal(capabilityForAction("99freelas","read_public_data",list).technicalStatus,"implemented");
 assert.equal(capabilityForAction("99freelas","send_proposal",list).technicalStatus,"engineering_gap");
 assert.equal(capabilityForAction("99freelas","deliver_work",list).technicalStatus,"engineering_gap");
 assert.ok(list.every(c=>c.mode==="requires_qualification"));
});
test("decision agent proposes, never consents or executes",()=>{
 const proposal=proposeAction({
  missionId:"m-proposal",adapterId:"99freelas",action:"send_proposal",
  targetOrigin:"https://www.99freelas.com.br",requestedScopes:["platform.submit"],
  observations:["Operator requested analysis"],
  unresolvedQuestions:["Does account consent apply?"],expectedIncomeCents:5000,
  estimatedCostCents:400
 });
 assert.equal(proposal.technicalStatus,"engineering_gap");
 assert.equal(proposal.decision,"pending_operator");
 assert.equal(proposal.generatedBy,"decision_agent");
 assert.equal(proposal.expectedIncomeCents,5000);
 assert.ok(!("signatureHex" in proposal));
});
test("operator explicitly approves exact proposal and no broader scopes",()=>{
 const p=proposal();
 const decision=sign(p);
 const receipt=checkOperatorApproval(p,decision,key,new Set([operatorId]),now);
 assert.equal(receipt.auditStatus,"operator_authorized");
 assert.equal(receipt.authorizedOrigin,p.targetOrigin);
 assert.deepEqual(receipt.approvedScopes,scopes);
 assert.equal(receipt.technicalStatus,"implemented");
});
test("operator reject and defer cannot be execution authorization",()=>{
 const p=proposal();
 for (const action of ["reject","defer"] as const){
  const decision=sign(p,{action});
  assert.throws(()=>checkOperatorApproval(p,decision,key,new Set([operatorId]),now),/HAS_NOT_APPROVED/);
 }
});
test("operator signature is required; agent cannot forge it",()=>{
 const p=proposal();
 const approval=sign(p);
 const forged={...approval,signatureHex:"0".repeat(64)};
 assert.throws(()=>checkOperatorApproval(p,forged,key,new Set([operatorId]),now),/SIGNATURE_INVALID/);
 assert.throws(()=>checkOperatorApproval(p,approval,Buffer.alloc(32,55),new Set([operatorId]),now),/SIGNATURE_INVALID/);
 assert.throws(()=>checkOperatorApproval(p,approval,key,new Set(["other"]),now),/ID_NOT_AUTHORIZED/);
});
test("expiry, maximum approval duration, wrong origin or wrong scope are denied",()=>{
 const p=proposal();
 assert.throws(()=>checkOperatorApproval(p,sign(p),key,new Set([operatorId]),now+6000),/EXPIRED/);
 assert.throws(()=>checkOperatorApproval(p,sign(p,{expiresAtMs:now+16*60_000}),key,new Set([operatorId]),now),/EXPIRED/);
 assert.throws(()=>checkOperatorApproval(p,sign(p,{authorizedOrigin:"https://evil.example"}),key,new Set([operatorId]),now),/SCOPE_MISMATCH/);
 assert.throws(()=>checkOperatorApproval(p,sign(p,{authorizedScopes:["platform.submit"]}),key,new Set([operatorId]),now),/SCOPE_MISMATCH/);
});
test("decision for another mission cannot authorize current mission",()=>{
 const p=proposal();
 const other=proposeAction({
  missionId:"mission-02",adapterId:"99freelas",action:"read_public_data",
  targetOrigin:"https://www.99freelas.com.br",requestedScopes:scopes,
  observations:["public"],unresolvedQuestions:[],expectedIncomeCents:null,estimatedCostCents:null
 });
 assert.throws(()=>checkOperatorApproval(other,sign(p),key,new Set([operatorId]),now),/SCOPE_MISMATCH/);
});
test("operator authorization has a durable one-time nonce; replay denied",async()=>{
 const p=proposal();
 const root=await mkdtemp(join(tmpdir(),"tma-f13-operator-"));
 try{
  const receipt=checkOperatorApproval(p,sign(p),key,new Set([operatorId]),now);
  await reserveAuthorization(receipt,root);
  const data=JSON.parse(await readFile(join(root,receipt.decisionNonce+".json"),"utf8"));
  assert.deepEqual(data,receipt);
  await assert.rejects(()=>reserveAuthorization(receipt,root),/EEXIST/);
 }finally{await rm(root,{recursive:true,force:true});}
});
test("invalid proposal data does not silently become authorization",()=>{
 assert.throws(()=>proposeAction({
  missionId:"../other",adapterId:"99freelas",action:"send_proposal",
  targetOrigin:"https://www.99freelas.com.br",requestedScopes:[],
  observations:[],unresolvedQuestions:[],expectedIncomeCents:0,estimatedCostCents:0
 }),/INVALID_IDENTIFIERS/);
});
