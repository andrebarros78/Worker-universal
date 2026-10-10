import { createHash } from "node:crypto";
import { chromium } from "playwright";
import {detachExistingEdgeTransport} from "./cdp_detach.ts";
import type { Browser, Page } from "playwright";
import {verifyObjectiveReadGrant} from "../../operator-control/src/objective_grant.ts";
import type {SignedObjectiveReadGrant} from "../../operator-control/src/objective_grant.ts";

// F13 remediation, not a modification of the closed F04/F09 simulation contracts.
// Uses only a pre-existing localhost CDP connection to the operator's Edge.
// Never copies cookies, profiles, passwords, storageState, or opens a new Edge process.
export type CourseReadFailure =
  | "OBJECTIVE_SCOPE_DENIED"
  | "EDGE_PROFILE_NOT_ACCESSIBLE"
  | "EDGE_NOT_VERIFIED"
  | "AUTHENTICATED_CONTENT_NOT_REACHED"
  | "COURSE_NOT_FOUND"
  | "SESSION_UNAVAILABLE"
  | "COURSE_READ_FAILED"
  | "EXTENSION_NOT_CONNECTED"
  | "EXTENSION_BRIDGE_UNAVAILABLE";

export type CourseReadResult =
  | {status:"read";url:string;title:string;text:string;sha256:string;observedAt:string;
     source:"existing_edge_cdp_background_tab"|"existing_edge_extension_background_tab";externalEffect:false}
  | {status:"blocked";reason:CourseReadFailure;detail:string;externalEffect:false};

export type CourseReadOptions = {
  grant:SignedObjectiveReadGrant;
  signingKey:Buffer;
  authorizedOperatorIds:ReadonlySet<string>;
  url:string;
  port:number;
  // Concrete selectors must be validated against the actual authenticated course.
  lessonSelector:string;
  accountSelector:string;
  timeoutMs?:number;
};

function blocked(reason:CourseReadFailure, detail:string):CourseReadResult {
  return {status:"blocked",reason,detail,externalEffect:false};
}
function safeOrigin(value:string):string {
  const url=new URL(value);
  if(url.protocol!=="https:" || url.username || url.password) throw Error("unsafe_origin");
  return url.origin;
}
function validate(options:CourseReadOptions):CourseReadFailure|null {
  let r;
  try {r=verifyObjectiveReadGrant(options.grant,options.signingKey,Date.now(),options.authorizedOperatorIds);}
  catch {return "OBJECTIVE_SCOPE_DENIED";}
  if(r.auditStatus!=="operator_objective_authorized" || r.productiveHomologation!==false ||
     r.action!=="read_account_data" || r.sessionMode!=="operator_current_browser_session" ||
     !r.approvedScopes.includes("course.read")) return "OBJECTIVE_SCOPE_DENIED";
  try {
    if(safeOrigin(options.url)!==r.authorizedOrigin) return "OBJECTIVE_SCOPE_DENIED";
  } catch { return "OBJECTIVE_SCOPE_DENIED"; }
  if(!options.lessonSelector?.trim() || !options.accountSelector?.trim())
    return "COURSE_NOT_FOUND";
  if(!Number.isSafeInteger(options.port) || options.port<1024 || options.port>65535)
    return "SESSION_UNAVAILABLE";
  return null;
}
async function findBackgroundPage(browser:Browser,targetId:string,timeoutMs:number):Promise<Page|undefined> {
  const deadline=Date.now()+timeoutMs;
  for(;Date.now()<deadline;) {
    for(const context of browser.contexts()) {
      for(const page of context.pages()) {
        const session=await context.newCDPSession(page);
        try {
          const info=await session.send("Target.getTargetInfo") as {targetInfo:{targetId:string}};
          if(info.targetInfo.targetId===targetId) return page;
        } finally { await session.detach(); }
      }
    }
    await new Promise(resolve=>setTimeout(resolve,75));
  }
  return undefined;
}

export async function readCourseFromExistingEdge(options:CourseReadOptions):Promise<CourseReadResult> {
  const denied=validate(options);
  if(denied) return blocked(denied,"objective or CDP endpoint does not satisfy the read-only contract");
  const timeout=Math.min(Math.max(options.timeoutMs??12000,1000),30000);
  let browser:Browser|undefined;
  let page:Page|undefined;
  let browserSession:Awaited<ReturnType<Browser["newBrowserCDPSession"]>>|undefined;
  let targetId:string|undefined;
  try {
    // Localhost only. The broker does not start Edge, reuse profile files, or handle credentials.
    browser=await chromium.connectOverCDP("http://127.0.0.1:"+options.port,{timeout:Math.min(timeout,3000)});
    browserSession=await browser.newBrowserCDPSession();
    const version=await browserSession.send("Browser.getVersion") as {userAgent?:string;product?:string};
    if(!/Edg\//i.test(version.userAgent??"") && !/Edg/i.test(version.product??""))
      return blocked("EDGE_NOT_VERIFIED","CDP peer is not Microsoft Edge");
    if(browser.contexts().length===0)
      return blocked("EDGE_PROFILE_NOT_ACCESSIBLE","CDP connection exposes no existing browser context");
    const newTarget=await browserSession.send("Target.createTarget",{
      url:"about:blank",background:true
    }) as {targetId:string};
    targetId=newTarget.targetId;
    page=await findBackgroundPage(browser,targetId,Math.min(timeout,4000));
    if(!page) return blocked("SESSION_UNAVAILABLE","background target not attached");
    // Route is per-new-page only; do not change routing of any existing operator tab.
    await page.route("**/*",async route=>{
      const request=route.request();
      const method=request.method().toUpperCase();
      let origin="";
      try{origin=new URL(request.url()).origin;}catch{}
      if(!["GET","HEAD"].includes(method) ||
         (request.isNavigationRequest() && origin!==options.grant.receipt.authorizedOrigin))
        await route.abort("blockedbyclient");
      else await route.continue();
    });
    const response=await page.goto(options.url,{waitUntil:"domcontentloaded",timeout});
    const finalUrl=page.url();
    if(safeOrigin(finalUrl)!==options.grant.receipt.authorizedOrigin)
      return blocked("OBJECTIVE_SCOPE_DENIED","navigation left the approved origin");
    if(!response || response.status()>=400)
      return blocked("COURSE_READ_FAILED","course response missing or unsuccessful");
    if(await page.locator("input[type=password]").count() ||
       /\/(login|signin|sign-in)([/?#]|$)/i.test(new URL(finalUrl).pathname))
      return blocked("AUTHENTICATED_CONTENT_NOT_REACHED","page requested authentication");
    if(await page.locator(options.accountSelector).count()===0)
      return blocked("AUTHENTICATED_CONTENT_NOT_REACHED","account-specific marker not observed");
    const lesson=page.locator(options.lessonSelector).first();
    if(await lesson.count()===0)
      return blocked("COURSE_NOT_FOUND","lesson selector not found in authenticated page");
    const title=await page.title();
    const text=(await lesson.innerText({timeout:Math.min(timeout,5000)})).trim();
    if(text.length<80)
      return blocked("AUTHENTICATED_CONTENT_NOT_REACHED","no substantive lesson text observed");
    return {status:"read",url:finalUrl,title,text,sha256:createHash("sha256").update(text).digest("hex"),
      observedAt:new Date().toISOString(),source:"existing_edge_cdp_background_tab",externalEffect:false};
  } catch(error) {
    const detail=error instanceof Error?error.message:String(error);
    if(!browser) return blocked("EDGE_PROFILE_NOT_ACCESSIBLE","existing Edge has no reachable localhost CDP endpoint");
    return blocked("COURSE_READ_FAILED",detail.slice(0,180));
  } finally {
    // Only the target we created is closed. Existing tabs are never navigated or closed.
    if(page) await page.close().catch(()=>{});
    else if(targetId && browserSession)
      await browserSession.send("Target.closeTarget",{targetId}).catch(()=>{});
    if(browserSession) await browserSession.detach().catch(()=>{});
    if(browser) detachExistingEdgeTransport(browser);
  }
}
