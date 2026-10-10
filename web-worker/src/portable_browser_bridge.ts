import {createHash,randomBytes} from "node:crypto";
import {existsSync} from "node:fs";
import {mkdir,readFile,writeFile,rename,copyFile} from "node:fs/promises";
import {homedir} from "node:os";
import {join,resolve} from "node:path";
import {fileURLToPath} from "node:url";

/**
 * F14 product lifecycle, NOT a HomeOcta acceptance-test fixture.
 * Source follows this module when the product is moved to another disk/PC.
 * Per-machine credentials are created under the current USER's data directory,
 * never on the removable source media.
 *
 * Installing/loading an extension into a signed-in Edge profile is a browser
 * permission step. Staging does NOT pretend to install it.
 */
const SOURCE=resolve(fileURLToPath(new URL("../extension/",import.meta.url)));
const MANIFEST="manifest.json";
const ID=/^[a-p]{32}$/;
const TOKEN=/^[0-9a-f]{64}$/;
const FILES=[MANIFEST,"background.js","content.js"] as const;
export type PortableBridgeConfig={
 schemaVersion:1;
 site:"homeocta";
 port:number;
 tokenHex:string;
 sourceSha256:string;
 extensionId:string|null;
};
export type PortableBridgeStatus={
 schemaVersion:1;
 site:"homeocta";
 extensionDir:string;
 configFile:string;
 port:number;
 extensionId:string|null;
 browserIntegration:"requires_browser_installation"|"id_declared_not_verified";
 mediaPortable:true;
 packageFingerprint:string;
};
type Options={
 env?:NodeJS.ProcessEnv;
 stateRoot?:string;
 sourceDir?:string;
 port?:number;
};
function stateBase(options:Options):string{
 if(options.stateRoot)return resolve(options.stateRoot);
 const env=options.env??process.env;
 // Windows normally exposes LOCALAPPDATA; XDG is for developer/non-Windows tests.
 const profile=env.LOCALAPPDATA||env.XDG_STATE_HOME||
   (process.platform==="win32"?join(homedir(),"AppData","Local"):join(homedir(),".local","state"));
 return resolve(profile);
}
function paths(opts:Options){
 const root=join(stateBase(opts),"TMA","browser-bridge","homeocta");
 return {root,ext:join(root,"extension"),config:join(root,"bridge.local.json")};
}
function validPort(port:number):boolean{
 return Number.isSafeInteger(port)&&port>=1024&&port<=65535;
}
async function sourceSnapshot(sourceDir:string){
 const bytes=await Promise.all(FILES.map(file=>readFile(join(sourceDir,file))));
 const h=createHash("sha256");
 for(let i=0;i<FILES.length;i++){h.update(FILES[i]);h.update(bytes[i]);}
 const manifest=JSON.parse(bytes[0].toString("utf8")) as {
  manifest_version:number;permissions:string[];host_permissions:string[];
 };
 const expected=["http://127.0.0.1:18764/*","https://www.homeocta.com/*"];
 if(manifest.manifest_version!==3 ||
   JSON.stringify([...manifest.permissions].sort())!==JSON.stringify(["alarms","scripting"])||
   JSON.stringify([...manifest.host_permissions].sort())!==JSON.stringify(expected))
    throw Error("PORTABLE_BRIDGE_SOURCE_SCOPE_INVALID");
 return {hash:h.digest("hex"),manifest};
}
function validateConfig(raw:unknown):PortableBridgeConfig {
 const v=raw as PortableBridgeConfig;
 if(!v||v.schemaVersion!==1||v.site!=="homeocta"||
    !validPort(v.port)||!TOKEN.test(v.tokenHex)||!TOKEN.test(v.sourceSha256)||
    (v.extensionId!==null&&!ID.test(v.extensionId)))
   throw Error("PORTABLE_BRIDGE_CONFIG_INVALID");
 return v;
}
async function atomicSave(path:string,text:string){
 const temp=path+".tmp-"+randomBytes(6).toString("hex");
 await writeFile(temp,text,{encoding:"utf8",flag:"wx",mode:0o600});
 await rename(temp,path);
}
async function writeIfDifferent(path:string,content:string):Promise<void>{
 const old=await readFile(path,"utf8").catch(()=>null);
 if(old!==content)await atomicSave(path,content);
}
export async function preparePortableBrowserBridge(opts:Options={}):Promise<PortableBridgeStatus>{
 const source=resolve(opts.sourceDir??SOURCE);
 const snapshot=await sourceSnapshot(source);
 const p=paths(opts);
 await mkdir(p.ext,{recursive:true,mode:0o700});
 const old=await readFile(p.config,"utf8").catch(()=>null);
 const config:PortableBridgeConfig=old?validateConfig(JSON.parse(old)):{
   schemaVersion:1,site:"homeocta",port:opts.port??18764,
   tokenHex:randomBytes(32).toString("hex"),
   sourceSha256:snapshot.hash,extensionId:null
 };
 if(opts.port!==undefined&&opts.port!==config.port)
   throw Error("PORTABLE_BRIDGE_PORT_CHANGE_REQUIRES_MIGRATION");
 if(!validPort(config.port))throw Error("PORTABLE_BRIDGE_PORT_INVALID");
 // One stable per-user path gives Edge a stable unpacked extension identity.
 const manifest={...snapshot.manifest,
  host_permissions:[
   "https://www.homeocta.com/*",
   "http://127.0.0.1:"+config.port+"/*"
  ]
 };
 await writeIfDifferent(join(p.ext,MANIFEST),JSON.stringify(manifest,null,2)+"\n");
 for(const file of FILES.slice(1))
   await writeIfDifferent(join(p.ext,file),await readFile(join(source,file),"utf8"));
 // Per-machine secret is NEVER included in the source package on USB.
 await writeIfDifferent(join(p.ext,"identity.js"),
   "self.TMA_BRIDGE_TOKEN="+JSON.stringify(config.tokenHex)+";\n"+
   "self.TMA_BRIDGE_PORT="+String(config.port)+";\n");
 if(config.sourceSha256!==snapshot.hash||!existsSync(p.config)){
   config.sourceSha256=snapshot.hash;
   await atomicSave(p.config,JSON.stringify(config,null,2)+"\n");
 }
 return {schemaVersion:1,site:"homeocta",extensionDir:p.ext,configFile:p.config,
  port:config.port,extensionId:config.extensionId,mediaPortable:true,
  browserIntegration:config.extensionId?"id_declared_not_verified":"requires_browser_installation",
  packageFingerprint:snapshot.hash};
}
export async function readPortableBridgeRuntime(opts:Options={}):Promise<{
 extensionId:string|null;bridgeToken:string;port:number;extensionDir:string;
}> {
 const p=paths(opts);
 const data=validateConfig(JSON.parse(await readFile(p.config,"utf8")));
 if(!existsSync(join(p.ext,MANIFEST)))throw Error("PORTABLE_BRIDGE_NOT_STAGED");
 return {extensionId:data.extensionId,bridgeToken:data.tokenHex,port:data.port,extensionDir:p.ext};
}
/**
 * Only store the ID after browser-side discovery. This does not claim the
 * browser actually installed/activated it; productive acceptance checks that.
 */
export async function declareBrowserExtensionId(id:string,opts:Options={}):Promise<void>{
 if(!ID.test(id))throw Error("PORTABLE_BRIDGE_EXTENSION_ID_INVALID");
 const p=paths(opts);
 const data=validateConfig(JSON.parse(await readFile(p.config,"utf8")));
 data.extensionId=id;
 await atomicSave(p.config,JSON.stringify(data,null,2)+"\n");
}
