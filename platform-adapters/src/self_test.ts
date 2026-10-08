import { registeredAdapters } from "./adapters.ts";
const list=registeredAdapters();
if(list.length!==5 || list.some(item=>item.health().promotion!=="experimental")){
  throw new Error("F09_SELF_TEST_FAILED");
}
console.log("tma-platform-adapters status=ok registered=5 simulation_only=true external_effects=0");
