// TMA F14: exactly one allowed web origin and one 127.0.0.1 port.
// No native debug, no user-tab focus, no cookie APIs, no remote telemetry.
importScripts("identity.js");
const ENDPOINT="http://127.0.0.1:"+String(self.TMA_BRIDGE_PORT??18764);
const APPROVED_ORIGIN="https://www.homeocta.com";
let polling=false;
async function handleJob(job){
  if(job?.schemaVersion!==1 || job.type!=="course.read"||
     job.origin!==APPROVED_ORIGIN||typeof job.jobId!=="string"||
     !Number.isFinite(job.expiresAt)||Date.now()>=job.expiresAt) return null;
  const tabs=await chrome.tabs.query({url:["https://www.homeocta.com/*"]});
  let answer={jobId:job.jobId,status:"blocked",reason:"COURSE_NOT_FOUND"};
  for(const tab of tabs){
    if(!Number.isInteger(tab.id)||!tab.url)continue;
    try{
      if(new URL(tab.url).origin!==APPROVED_ORIGIN)continue;
      let response;
      try { response=await chrome.tabs.sendMessage(tab.id,{type:"TMA_COURSE_READ",
        jobId:job.jobId,accountSelector:job.accountSelector,lessonSelector:job.lessonSelector}); }
      catch {
        // Extensions installed after the tab was opened require isolated-script injection.
        await chrome.scripting.executeScript({target:{tabId:tab.id},files:["content.js"]});
        response=await chrome.tabs.sendMessage(tab.id,{type:"TMA_COURSE_READ",
          jobId:job.jobId,accountSelector:job.accountSelector,lessonSelector:job.lessonSelector});
      }
      if(response?.jobId!==job.jobId)continue;
      if(response.status==="read"){
        answer=response;break;
      }
      if(response.status==="blocked" && response.reason==="AUTHENTICATED_CONTENT_NOT_REACHED")
        answer=response;
    }catch{ /* Other tab is never touched or exposed. */ }
  }
  return answer;
}
async function poll(){
  if(polling)return;
  polling=true;
  try{
    const response=await fetch(ENDPOINT+"/v1/job",{cache:"no-store",
      headers:{"X-TMA-Bridge-Token":self.TMA_BRIDGE_TOKEN}});
    if(!response.ok)return;
    const {job}=await response.json();
    if(!job)return;
    const result=await handleJob(job);
    if(!result)return;
    await fetch(ENDPOINT+"/v1/result",{method:"POST",
      headers:{"Content-Type":"application/json","X-TMA-Bridge-Token":self.TMA_BRIDGE_TOKEN},body:JSON.stringify(result)});
  }catch{ /* A Worker mission may not be running. Never navigate or interrupt tabs. */ }
  finally{polling=false;}
}
chrome.runtime.onInstalled.addListener(()=>{
  chrome.alarms.create("tma-homeocta-bridge",{periodInMinutes:0.5});
  void poll();
});
chrome.runtime.onStartup.addListener(()=>{
  chrome.alarms.create("tma-homeocta-bridge",{periodInMinutes:0.5});
  void poll();
});
chrome.alarms.onAlarm.addListener(a=>{if(a.name==="tma-homeocta-bridge")void poll();});
chrome.alarms.create("tma-homeocta-bridge",{periodInMinutes:0.5});
void poll();
