import { createHmac, timingSafeEqual } from "node:crypto";
import { open, mkdir } from "node:fs/promises";
import { join } from "node:path";
import type { OperatorDecision, DecisionProposal, AuthorizationReceipt } from "./contracts.ts";

// This module verifies a separately authenticated operator decision.
// It does not create proposals, assess platform business rules, or execute adapters.
const HEX64=/^[a-f0-9]{64}$/;
const IDENT=/^[A-Za-z0-9_.-]{1,120}$/;
const NONCE=/^[A-Za-z0-9_-]{16,120}$/;
function message(decision:Omit<OperatorDecision,"signatureHex">):string {
  return JSON.stringify({
    schemaVersion:decision.schemaVersion,
    proposalFingerprint:decision.proposalFingerprint,
    operatorId:decision.operatorId,
    action:decision.action,
    authorizedScopes:decision.authorizedScopes,
    authorizedOrigin:decision.authorizedOrigin,
    issuedAtMs:decision.issuedAtMs,
    expiresAtMs:decision.expiresAtMs,
    nonce:decision.nonce,
  });
}
export function operatorSignature(
  fields:Omit<OperatorDecision,"signatureHex">, secret:Buffer
):string {
  if(secret.length<32) throw new Error("F13_OPERATOR_KEY_TOO_SHORT");
  return createHmac("sha256",secret).update(message(fields)).digest("hex");
}
export function checkOperatorApproval(
  proposal:DecisionProposal,
  decision:OperatorDecision,
  operatorKey:Buffer,
  authorizedOperatorIds:ReadonlySet<string>,
  nowMs:number,
):AuthorizationReceipt {
  if(decision.schemaVersion!==1 || !IDENT.test(decision.operatorId) ||
    !authorizedOperatorIds.has(decision.operatorId))
    throw new Error("F13_OPERATOR_ID_NOT_AUTHORIZED");
  if(decision.action!=="approve")
    throw new Error("F13_OPERATOR_HAS_NOT_APPROVED");
  if(!HEX64.test(decision.signatureHex) || !HEX64.test(decision.proposalFingerprint) ||
     !NONCE.test(decision.nonce))
    throw new Error("F13_OPERATOR_DECISION_INVALID");
  if(decision.proposalFingerprint!==proposal.fingerprint ||
     decision.authorizedOrigin!==proposal.targetOrigin ||
     !Array.isArray(decision.authorizedScopes) ||
     decision.authorizedScopes.length!==proposal.requestedScopes.length ||
     decision.authorizedScopes.some((scope,i)=>scope!==proposal.requestedScopes[i]))
    throw new Error("F13_OPERATOR_DECISION_SCOPE_MISMATCH");
  if(!Number.isSafeInteger(nowMs) || !Number.isSafeInteger(decision.issuedAtMs) ||
     !Number.isSafeInteger(decision.expiresAtMs) ||
     nowMs<decision.issuedAtMs || nowMs>=decision.expiresAtMs ||
     decision.expiresAtMs-decision.issuedAtMs>15*60_000)
    throw new Error("F13_OPERATOR_DECISION_EXPIRED");
  const {signatureHex,...unsigned}=decision;
  const expected=Buffer.from(operatorSignature(unsigned,operatorKey),"hex");
  if(!timingSafeEqual(expected,Buffer.from(signatureHex,"hex")))
    throw new Error("F13_OPERATOR_SIGNATURE_INVALID");
  return {
    schemaVersion:1,proposalFingerprint:proposal.fingerprint,
    operatorId:decision.operatorId,action:proposal.action,missionId:proposal.missionId,
    adapterId:proposal.adapterId,approvedScopes:[...decision.authorizedScopes],
    authorizedOrigin:decision.authorizedOrigin,decisionNonce:decision.nonce,
    auditStatus:"operator_authorized",technicalStatus:proposal.technicalStatus,
  };
}

// Reserves once, without running a browser, API or payment action. An external
// F05 claim/fencing + technical/qualification/permission gate is still required.
export async function reserveAuthorization(
  receipt:AuthorizationReceipt,
  ledgerDir:string,
):Promise<void>{
  await mkdir(ledgerDir,{recursive:true});
  const path=join(ledgerDir,receipt.decisionNonce+".json");
  const file=await open(path,"wx",0o600);
  try{await file.writeFile(JSON.stringify(receipt)+"\n");await file.sync();}
  finally{await file.close();}
}
