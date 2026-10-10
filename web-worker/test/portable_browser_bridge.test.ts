import assert from "node:assert/strict";
import test from "node:test";
import {mkdtemp,mkdir,readFile,copyFile,rename,readdir,rm} from "node:fs/promises";
import {join,resolve} from "node:path";
import {
 preparePortableBrowserBridge,readPortableBridgeRuntime,declareBrowserExtensionId
} from "../src/portable_browser_bridge.ts";
const TEST_BASE="C:\\ProgramData\\SentinelX\\workspace\\tma-f14-portability-tests";
const SRC=resolve("web-worker/extension");
async function copySource(dest:string){
 await mkdir(dest,{recursive:true});
 for(const f of ["manifest.json","background.js","content.js"])
   await copyFile(join(SRC,f),join(dest,f));
}
test("F14 packaged Worker can move source path and retain per-machine configuration",async()=>{
 await mkdir(TEST_BASE,{recursive:true});
 const folder=await mkdtemp(join(TEST_BASE,"portable-"));
 try{
  const sourceA=join(folder,"USB-A","TIMED-MISSION-AGENT","web-worker","extension");
  const sourceB=join(folder,"USB-B","TIMED-MISSION-AGENT","web-worker","extension");
  const userA=join(folder,"USER-A","AppData","Local");
  const userB=join(folder,"USER-B","AppData","Local");
  await copySource(sourceA);
  const a=await preparePortableBrowserBridge({sourceDir:sourceA,stateRoot:userA});
  assert.equal(a.browserIntegration,"requires_browser_installation");
  assert.ok(a.extensionDir.startsWith(resolve(userA)));
  assert.equal(a.mediaPortable,true);
  assert.ok(!a.extensionDir.includes("USB-A"));
  const first=await readPortableBridgeRuntime({stateRoot:userA});
  assert.match(first.bridgeToken,/^[a-f0-9]{64}$/);
  assert.equal(first.port,18764);
  const staged=await readdir(a.extensionDir);
  assert.deepEqual(staged.sort(),["background.js","content.js","identity.js","manifest.json"]);
  const generated=await readFile(join(a.extensionDir,"identity.js"),"utf8");
  assert.ok(generated.includes(first.bridgeToken));
  const copied=await readFile(join(sourceA,"background.js"),"utf8");
  assert.equal(copied.includes(first.bridgeToken),false);
  await mkdir(join(folder,"USB-B","TIMED-MISSION-AGENT","web-worker"),{recursive:true});
  await rename(sourceA,sourceB);
  const reconnected=await preparePortableBrowserBridge({sourceDir:sourceB,stateRoot:userA});
  assert.equal(reconnected.extensionDir,a.extensionDir);
  assert.equal((await readPortableBridgeRuntime({stateRoot:userA})).bridgeToken,first.bridgeToken);
  const b=await preparePortableBrowserBridge({sourceDir:sourceB,stateRoot:userB});
  assert.equal(b.mediaPortable,true);
  assert.notEqual(b.extensionDir,a.extensionDir);
  assert.notEqual((await readPortableBridgeRuntime({stateRoot:userB})).bridgeToken,first.bridgeToken);
  const manifest=JSON.parse(await readFile(join(b.extensionDir,"manifest.json"),"utf8"));
  assert.deepEqual(manifest.host_permissions.sort(),
   ["https://www.homeocta.com/*","http://127.0.0.1:18764/*"].sort());
  await declareBrowserExtensionId("abcdefghijklmnopabcdefghijklmnop",{stateRoot:userB});
  const declared=await preparePortableBrowserBridge({stateRoot:userB,sourceDir:sourceB});
  assert.equal(declared.browserIntegration,"id_declared_not_verified");
  assert.equal(declared.extensionId,"abcdefghijklmnopabcdefghijklmnop");
  assert.equal(declared.mediaPortable,true);
 }finally{await rm(folder,{recursive:true,force:true});}
});
test("F14 portable bootstrap validates scope and rejects invalid IDs or unsafe port migration",async()=>{
 await mkdir(TEST_BASE,{recursive:true});
 const folder=await mkdtemp(join(TEST_BASE,"guards-"));
 try{
  const root=join(folder,"USER");
  await assert.rejects(
   ()=>preparePortableBrowserBridge({sourceDir:SRC,stateRoot:root,port:23}),
   /PORTABLE_BRIDGE_PORT_INVALID/
  );
  const valid=await preparePortableBrowserBridge({sourceDir:SRC,stateRoot:root,port:18764});
  assert.equal(valid.mediaPortable,true);
  await assert.rejects(
   ()=>declareBrowserExtensionId("wrong-id",{stateRoot:root}),
   /PORTABLE_BRIDGE_EXTENSION_ID_INVALID/
  );
  await assert.rejects(
   ()=>preparePortableBrowserBridge({sourceDir:SRC,stateRoot:root,port:18765}),
   /PORTABLE_BRIDGE_PORT_CHANGE_REQUIRES_MIGRATION/
  );
 }finally{await rm(folder,{recursive:true,force:true});}
});
