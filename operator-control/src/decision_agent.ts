import { createHash } from "node:crypto";
import { capabilityForAction } from "./capabilities.ts";
import type { Action, DecisionProposal, TechnicalCapability } from "./contracts.ts";

const ID=/^[a-zA-Z0-9_.-]{1,120}$/;
const ACTIONS=new Set<Action>([
  "read_public_data","read_account_data","prepare_proposal","send_proposal",
  "deliver_work","issue_invoice","request_payment"
]);
export type ProposalInput={
  missionId:string;
  adapterId:string;
  action:Action;
  targetOrigin:string;
  requestedScopes:string[];
  observations:string[];
  unresolvedQuestions:string[];
  expectedIncomeCents:number|null;
  estimatedCostCents:number|null;
};
const hash=(v:string)=>createHash("sha256").update(v).digest("hex");

export function proposeAction(
  input:ProposalInput,
  catalogue?:TechnicalCapability[]
):DecisionProposal {
  if(!ID.test(input.missionId) || !ID.test(input.adapterId) || !ACTIONS.has(input.action))
    throw new Error("F13_PROPOSAL_INVALID_IDENTIFIERS");
  let url:URL;
  try{url=new URL(input.targetOrigin);}catch{throw new Error("F13_PROPOSAL_INVALID_ORIGIN");}
  if(url.protocol!=="https:" || url.origin!==input.targetOrigin || url.username || url.password)
    throw new Error("F13_PROPOSAL_INVALID_ORIGIN");
  if(!Array.isArray(input.requestedScopes) || !input.requestedScopes.every(s=>ID.test(s)) ||
     new Set(input.requestedScopes).size!==input.requestedScopes.length)
    throw new Error("F13_PROPOSAL_INVALID_SCOPE");
  const amounts=[input.expectedIncomeCents,input.estimatedCostCents];
  if(amounts.some(n=>n!==null && (!Number.isSafeInteger(n) || n<0)))
    throw new Error("F13_PROPOSAL_INVALID_ECONOMICS");
  const inspected=capabilityForAction(input.adapterId,input.action,catalogue);
  const source={
    schemaVersion:1 as const,missionId:input.missionId,adapterId:input.adapterId,
    action:input.action,targetOrigin:url.origin,
    requestedScopes:[...input.requestedScopes].sort(),
    technicalStatus:inspected.technicalStatus,
    observations:[...input.observations],
    unresolvedQuestions:[...input.unresolvedQuestions],
    expectedIncomeCents:input.expectedIncomeCents,
    estimatedCostCents:input.estimatedCostCents,
    generatedBy:"decision_agent" as const,
    decision:"pending_operator" as const,
  };
  const fingerprint=hash(JSON.stringify(source));
  return {...source,proposalId:fingerprint.slice(0,24),fingerprint};
}
