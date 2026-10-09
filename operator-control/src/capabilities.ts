import type { Action, TechnicalCapability } from "./contracts.ts";
import { registeredAdapters } from "../../platform-adapters/src/adapters.ts";

// Pure inventory: names the technical functions actually implemented.
// It never decides whether a provider should be used commercially.
const registeredTechnicalActions:Record<string,ReadonlyArray<Action>>={
  "browser.read":["read_public_data"],
  "form.practice":[],
  "document.extract":[],
  "research.opportunity":["read_public_data"],
  "document.training":[],
};
export function inventory():TechnicalCapability[] {
  return registeredAdapters().flatMap(adapter=>{
    const manifest=adapter.manifest();
    return manifest.capabilities.map(capabilityId=>({
      adapterId:manifest.adapterId,
      capabilityId,
      supportedActions:[...(registeredTechnicalActions[capabilityId]??[])],
      mode:"requires_qualification" as const,
      evidenceRefs:["F09 adapter manifest","F08 qualification evidence required"],
    }));
  });
}
export function capabilityForAction(
  adapterId:string, action:Action, list:TechnicalCapability[]=inventory()
): {technicalStatus:"implemented"|"engineering_gap"; evidenceRefs:string[]} {
  const matches=list.filter(c=>c.adapterId===adapterId && c.supportedActions.includes(action));
  return matches.length
    ? {technicalStatus:"implemented",evidenceRefs:matches.flatMap(c=>c.evidenceRefs)}
    : {technicalStatus:"engineering_gap",evidenceRefs:["technical implementation/qualification not yet proven"]};
}
