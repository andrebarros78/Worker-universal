import assert from "node:assert/strict";
import test from "node:test";
import {loadModuleRegistry,moduleSummary,validateRegistry,verifyModulePaths} from "../src/module_catalog.ts";

test("M architecture: sixteen stable product modules, no construction-phase identifiers",async()=>{
 const reg=await loadModuleRegistry();
 assert.equal(reg.modules.length,16);
 assert.deepEqual(reg.modules.map(m=>m.id),Array.from({length:16},(_,i)=>"M"+(i+1)));
 assert.equal(reg.modules[0].name,"Cérebro e Decisão");
 assert.equal(reg.modules[1].name,"Core de Missões");
 assert.equal(reg.modules[2].name,"Capacidades e Ferramentas");
 assert.ok(reg.modules.every(m=>!Object.hasOwn(m,"phase")&&!Object.hasOwn(m,"phaseId")));
 assert.ok(reg.modules.every(m=>m.productiveAcceptance==="not_proven"));
 await verifyModulePaths(reg);
 assert.equal(moduleSummary(reg).length,16);
});
test("M architecture: rejects construction phases as module identity",async()=>{
 const reg=await loadModuleRegistry();
 const invalid=structuredClone(reg);
 invalid.modules[0].id="F14";
 assert.throws(()=>validateRegistry(invalid),/PRODUCT_MODULE_INVALID/);
 const leak=structuredClone(reg) as unknown as {modules:Record<string,unknown>[]};
 leak.modules[2].phaseId="F03";
 assert.throws(()=>validateRegistry(leak as never),/PHASE_LEAK/);
});
test("M architecture: rejects duplicated M ownership and dependency cycles",async()=>{
 const reg=await loadModuleRegistry();
 const duplicate=structuredClone(reg);
 duplicate.modules[1].paths=[duplicate.modules[0].paths[0]];
 assert.throws(()=>validateRegistry(duplicate),/OWNERSHIP_DUPLICATE/);
 const cyclic=structuredClone(reg);
 cyclic.modules[13].dependsOn=["M1"];
 assert.throws(()=>validateRegistry(cyclic),/DEPENDENCY_CYCLE/);
});
test("M architecture: rejects missing implementation and nonexistent dependencies",async()=>{
 const reg=await loadModuleRegistry();
 const missing=structuredClone(reg);
 missing.modules[0].paths=["never-present/module"];
 await assert.rejects(()=>verifyModulePaths(missing),/IMPLEMENTATION_MISSING/);
 const unresolved=structuredClone(reg);
 unresolved.modules[0].dependsOn=["M17"];
 assert.throws(()=>validateRegistry(unresolved),/DEPENDENCY_UNKNOWN/);
});
test("M architecture: explicit verification status cannot impersonate production acceptance",async()=>{
 const reg=await loadModuleRegistry();
 assert.ok(reg.modules.some(m=>m.engineeringStatus==="verified_local"));
 assert.ok(reg.modules.some(m=>m.engineeringStatus==="partial"));
 assert.ok(reg.modules.every(m=>m.productiveAcceptance==="not_proven"));
 assert.equal(reg.namingPolicy.F,"construction_history_only");
 assert.equal(reg.namingPolicy.M,"permanent_product_module");
});
