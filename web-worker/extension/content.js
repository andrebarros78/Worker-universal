// TMA F14: runs ONLY on the approved HomeOcta origin, in isolated world.
// It cannot access cookies, passwords, storage, other tabs, or external sites.
(()=>{
  if(globalThis.__tmaHomeOctaCourseContentV1)return;
  globalThis.__tmaHomeOctaCourseContentV1=true;
  const ORIGIN="https://www.homeocta.com";
  chrome.runtime.onMessage.addListener((message,_sender,sendResponse)=>{
    if(message?.type!=="TMA_COURSE_READ" || location.origin!==ORIGIN)return false;
    const safe=()=>{
      if(typeof message.jobId!=="string"||typeof message.accountSelector!=="string"||
         typeof message.lessonSelector!=="string"||message.lessonSelector.length>180||
         message.accountSelector.length>180)return {status:"blocked",reason:"COURSE_NOT_FOUND"};
      const currentUrl=new URL(location.href);
      if(currentUrl.origin!==ORIGIN||document.querySelector('input[type="password"]')||
         ["login","signin","sign-in"].some(part=>currentUrl.pathname.toLowerCase().startsWith("/"+part)))
        return {status:"blocked",reason:"AUTHENTICATED_CONTENT_NOT_REACHED"};
      if(!document.querySelector(message.accountSelector))
        return {status:"blocked",reason:"AUTHENTICATED_CONTENT_NOT_REACHED"};
      const lesson=document.querySelector(message.lessonSelector);
      if(!lesson)return {status:"blocked",reason:"COURSE_NOT_FOUND"};
      const text=(lesson.innerText||lesson.textContent||"").trim().slice(0,90000);
      if(text.length<80)return {status:"blocked",reason:"COURSE_NOT_FOUND"};
      return {status:"read",url:location.href,title:document.title.slice(0,300),text};
    };
    try {sendResponse({jobId:message.jobId,...safe()});}
    catch {sendResponse({jobId:message.jobId,status:"blocked",reason:"COURSE_NOT_FOUND"});}
    return false;
  });
})();
