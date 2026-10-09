export type Role = "capability" | "engineering" | "decision_agent" | "operator";
export type Action =
  | "read_public_data"
  | "read_account_data"
  | "prepare_proposal"
  | "send_proposal"
  | "deliver_work"
  | "issue_invoice"
  | "request_payment";

export type TechnicalCapability = {
  adapterId:string;
  capabilityId:string;
  supportedActions:Action[];
  mode:"available"|"not_implemented"|"requires_qualification";
  evidenceRefs:string[];
};

export type DecisionProposal = {
  schemaVersion:1;
  proposalId:string;
  missionId:string;
  adapterId:string;
  action:Action;
  targetOrigin:string;
  requestedScopes:string[];
  technicalStatus:"implemented"|"engineering_gap";
  observations:string[];
  unresolvedQuestions:string[];
  expectedIncomeCents:number|null;
  estimatedCostCents:number|null;
  generatedBy:"decision_agent";
  decision:"pending_operator";
  fingerprint:string;
};

export type OperatorDecision = {
  schemaVersion:1;
  proposalFingerprint:string;
  operatorId:string;
  action:"approve"|"reject"|"defer";
  authorizedScopes:string[];
  authorizedOrigin:string;
  issuedAtMs:number;
  expiresAtMs:number;
  nonce:string;
  signatureHex:string;
};
export type AuthorizationReceipt = {
  schemaVersion:1;
  proposalFingerprint:string;
  operatorId:string;
  action:DecisionProposal["action"];
  missionId:string;
  adapterId:string;
  approvedScopes:string[];
  authorizedOrigin:string;
  decisionNonce:string;
  auditStatus:"operator_authorized";
  technicalStatus:DecisionProposal["technicalStatus"];
};
