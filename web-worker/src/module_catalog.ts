import {readFile,stat} from "node:fs/promises";
import {resolve,dirname,sep} from "node:path";
import {fileURLToPath} from "node:url";

export type ProductModule={
 id:string;name:string;responsibility:string;runtime:string;
 paths:string[];dependsOn:string[];
 engineeringStatus:"verified_local"|"partial"|"source_only";
 productiveAcceptance:"not_proven"|"proven";
};
export type ProductModuleRegistry={
 schemaVersion:1;product:"TIMED-MISSION-AGENT";
 namingPolicy:{M:"permanent_product_module";F:"construction_history_only"};
 moduleCount:number;modules:ProductModule[];
};
const root=resolve(dirname(fileURLToPath(import.meta.url)),"..","..");
const REGISTRY_FILE=resolve(root,"architecture","PRODUCT_MODULES.json");
const ID=/^M(?:[1-9]|1[0-6])$/;
const PHASE=/^F[0-9]+$/i;
const statuses=new Set(["verified_local","partial","source_only"]);
const bounded=(v:unknown,max=450):v is string=>
 typeof v==="string"&&v.trim().length>0&&v.length<=max;

export function validateRegistry(reg:ProductModuleRegistry):void{
 if(!reg||reg.schemaVersion!==1||reg.product!=="TIMED-MISSION-AGENT"||
   reg.namingPolicy?.M!=="permanent_product_module"||
   reg.namingPolicy?.F!=="construction_history_only"||
   reg.moduleCount!==16||!Array.isArray(reg.modules)||reg.modules.length!==16)
   throw Error("PRODUCT_MODULE_REGISTRY_CONTRACT_INVALID");
 const keys=new Set<string>(),paths=new Set<string>();
 for(const item of reg.modules){
   if(!item||!ID.test(item.id)||keys.has(item.id)||
      !bounded(item.name,100)||!bounded(item.responsibility)||
      !bounded(item.runtime,100)||!statuses.has(item.engineeringStatus)||
      !["not_proven","proven"].includes(item.productiveAcceptance)||
      !Array.isArray(item.paths)||item.paths.length===0||
      !Array.isArray(item.dependsOn))
     throw Error("PRODUCT_MODULE_INVALID_OR_DUPLICATE");
   // A product module must never be a construction phase.
   if(PHASE.test(item.id)||Object.hasOwn(item,"phase")||
      Object.hasOwn(item,"phaseId")||Object.hasOwn(item,"constructionPhase"))
     throw Error("PRODUCT_MODULE_PHASE_LEAK");
   keys.add(item.id);
 }
 const expected=Array.from({length:16},(_,i)=>"M"+(i+1));
 if(expected.some(x=>!keys.has(x)))throw Error("PRODUCT_MODULE_SEQUENCE_BROKEN");
 for(const item of reg.modules){
   if(new Set(item.dependsOn).size!==item.dependsOn.length)
     throw Error("MODULE_DEPENDENCY_DUPLICATE_"+item.id);
   if(item.dependsOn.some(x=>!keys.has(x)||x===item.id))
     throw Error("MODULE_DEPENDENCY_UNKNOWN_"+item.id);
   for(const p of item.paths){
     if(!bounded(p,230)||p.includes("\\")||p.startsWith("/")||p.includes(":")||
        p.split("/").some(x=>x===".."||x===""||x==="."))throw Error("MODULE_PATH_INVALID_"+item.id);
     const rel=p.toLowerCase();
     if(paths.has(rel))throw Error("MODULE_IMPLEMENTATION_PATH_OWNERSHIP_DUPLICATE_"+p);
     paths.add(rel);
   }
 }
 const lookup=new Map(reg.modules.map(x=>[x.id,x]));
 const visited=new Set<string>(),visiting=new Set<string>();
 function walk(id:string):void{
   if(visiting.has(id))throw Error("MODULE_DEPENDENCY_CYCLE_"+id);
   if(visited.has(id))return;
   visiting.add(id);
   for(const dep of lookup.get(id)!.dependsOn)walk(dep);
   visiting.delete(id);visited.add(id);
 }
 for(const id of expected)walk(id);
}
export async function loadModuleRegistry(path=REGISTRY_FILE):Promise<ProductModuleRegistry>{
 const input=JSON.parse(await readFile(path,"utf8")) as ProductModuleRegistry;
 validateRegistry(input);
 return input;
}
export async function verifyModulePaths(reg:ProductModuleRegistry,base=root):Promise<void>{
 validateRegistry(reg);
 for(const mod of reg.modules)for(const p of mod.paths){
   const full=resolve(base,p);
   if(!full.startsWith(resolve(base)+sep))throw Error("MODULE_PATH_OUTSIDE_PRODUCT_"+mod.id);
   const file=await stat(full).catch(()=>null);
   if(!file||(file.isFile()===false&&file.isDirectory()===false))
     throw Error("MODULE_IMPLEMENTATION_MISSING_"+mod.id+"_"+p);
 }
}
export function moduleSummary(reg:ProductModuleRegistry){
 return reg.modules.map(m=>({
   id:m.id,name:m.name,responsibility:m.responsibility,
   runtime:m.runtime,dependsOn:m.dependsOn,
   engineeringStatus:m.engineeringStatus,
   productiveAcceptance:m.productiveAcceptance
 }));
}
async function main(args:string[]):Promise<number>{
 try{
  const command=args[0];
  if(!["list","inspect","verify"].includes(command)||args.length>(command==="inspect"?2:1))
    throw Error("MODULE_COMMAND_INVALID");
  const reg=await loadModuleRegistry();
  await verifyModulePaths(reg);
  if(command==="verify"){
    console.log("PRODUCT_MODULE_ARCHITECTURE=PASS modules="+reg.moduleCount+
      " naming=M-product,F-construction productiveAcceptance=independent");
    return 0;
  }
  if(command==="list"){
    for(const mod of moduleSummary(reg)){
      console.log(mod.id+" | "+mod.name+" | "+mod.responsibility+
        " | engenharia="+mod.engineeringStatus+
        " | producao="+mod.productiveAcceptance);
    }
    return 0;
  }
  const id=args[1]?.toUpperCase()??"";
  const mod=reg.modules.find(m=>m.id===id);
  if(!mod)throw Error("MODULE_ID_UNKNOWN");
  console.log(JSON.stringify(mod,null,2));
  return 0;
 }catch(err){
  console.error("PRODUCT_MODULE_REGISTRY_FAIL="+(err instanceof Error?err.message:String(err)));
  return 4;
 }
}
if(process.argv[1]&&/module_catalog[.]ts$/i.test(process.argv[1]))
 process.exitCode=await main(process.argv.slice(2));
