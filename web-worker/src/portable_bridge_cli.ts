import {preparePortableBrowserBridge,readPortableBridgeRuntime,declareBrowserExtensionId} from "./portable_browser_bridge.ts";

// Product portable bootstrap; never touches signed-in Edge or claims installation.
async function main(args:string[]):Promise<number>{
 try{
  const action=args[0];
  if(action==="prepare"&&args.length===1){
    const result=await preparePortableBrowserBridge();
    console.log("TMA_PORTABLE_BROWSER="+JSON.stringify(result));
    return 0;
  }
  if(action==="status"&&args.length===1){
    const state=await readPortableBridgeRuntime();
    console.log("TMA_PORTABLE_BROWSER="+JSON.stringify({
      extensionDir:state.extensionDir,port:state.port,
      extensionId:state.extensionId,
      browserIntegration:state.extensionId?"id_declared_not_verified":"requires_browser_installation",
      mediaPortable:true
    }));
    return 0;
  }
  if(action==="declare-id"&&args.length===2){
    await declareBrowserExtensionId(args[1]);
    console.log("TMA_PORTABLE_BROWSER_EXTENSION_ID_DECLARED_NOT_VERIFIED");
    return 0;
  }
  console.error("TMA_PORTABLE_BROWSER_INVALID_COMMAND");
  return 64;
 }catch(err){
   console.error("TMA_PORTABLE_BROWSER_BLOCKED="+(err instanceof Error?err.message:"UNKNOWN"));
   return 4;
 }
}
if(process.argv[1]&&/portable_bridge_cli[.]ts$/.test(process.argv[1]))
 process.exitCode=await main(process.argv.slice(2));
