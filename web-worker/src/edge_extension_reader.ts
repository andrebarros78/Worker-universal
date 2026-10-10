import {createHash,randomUUID,timingSafeEqual} from "node:crypto";
import {createServer} from "node:http";
import type {IncomingMessage,ServerResponse} from "node:http";
import {verifyObjectiveReadGrant} from "../../operator-control/src/objective_grant.ts";
import type {CourseReadOptions,CourseReadResult} from "./edge_course_reader.ts";

/** F14: bounded, origin-scoped alternative when the operator's Edge has no CDP. */
export type ExtensionReadOptions=CourseReadOptions&{
  extensionId:string;
  bridgeToken:string;
  listenPort?:number;
  waitMs?:number;
};
const ID=/^[a-p]{32}$/;
const TOKEN=/^[a-f0-9]{64}$/;
const ORIGIN="https://www.homeocta.com";
const MAX_BODY=180_000;
const MAX_TEXT=90_000;
const blocked=(reason:"OBJECTIVE_SCOPE_DENIED"|"EXTENSION_NOT_CONNECTED"|"EXTENSION_BRIDGE_UNAVAILABLE"|"COURSE_NOT_FOUND"|"AUTHENTICATED_CONTENT_NOT_REACHED"|"COURSE_READ_FAILED",detail:string):CourseReadResult=>
  ({status:"blocked",reason,detail,externalEffect:false});

export async function readCourseFromEdgeExtension(opt:ExtensionReadOptions):Promise<CourseReadResult>{
  let receipt;
  try{receipt=verifyObjectiveReadGrant(opt.grant,opt.signingKey,Date.now(),opt.authorizedOperatorIds);}
  catch{return blocked("OBJECTIVE_SCOPE_DENIED","signed course objective could not be verified");}
  if(!ID.test(opt.extensionId)||!TOKEN.test(opt.bridgeToken)||receipt.action!=="read_account_data"||
     receipt.sessionMode!=="operator_current_browser_session"||
     !receipt.approvedScopes.includes("course.read")||receipt.authorizedOrigin!==ORIGIN||
     typeof opt.url!=="string"||new URL(opt.url).origin!==ORIGIN||
     !opt.accountSelector?.trim()||!opt.lessonSelector?.trim())
    return blocked("OBJECTIVE_SCOPE_DENIED","origin, extension ID, or selectors not approved");
  const port=opt.listenPort??18764;
  if(!Number.isInteger(port)||port<1024||port>65535)
    return blocked("EXTENSION_BRIDGE_UNAVAILABLE","invalid loopback port");
  const waitMs=Math.min(Math.max(opt.waitMs??35000,100),60000);
  const jobId=randomUUID();
  const expectedOrigin="chrome-extension://"+opt.extensionId;
  const expiresAt=Date.now()+waitMs;
  const job={schemaVersion:1,type:"course.read",jobId,origin:ORIGIN,url:opt.url,
    accountSelector:opt.accountSelector,lessonSelector:opt.lessonSelector,expiresAt};
  let completed=false;
  let finish:(r:CourseReadResult)=>void=()=>{};
  const result=new Promise<CourseReadResult>(res=>{finish=res;});
  const respond=(res:ServerResponse,status:number,body:object)=>{
    res.statusCode=status;
    res.setHeader("Content-Type","application/json; charset=utf-8");
    res.setHeader("Cache-Control","no-store");
    res.setHeader("Access-Control-Allow-Origin",expectedOrigin);
    res.setHeader("Vary","Origin");
    res.end(JSON.stringify(body));
  };
  const server=createServer(async (req:IncomingMessage,res:ServerResponse)=>{
    try {
      // MV3 extension fetch can omit Origin; a per-install 256-bit bearer
      // is mandatory. If Origin is provided it must still match exactly.
      if((req.headers.origin!==undefined&&req.headers.origin!==expectedOrigin)||Date.now()>expiresAt){
        res.writeHead(403,{"Cache-Control":"no-store"});res.end();return;
      }
      if(req.method==="OPTIONS"){
        res.setHeader("Access-Control-Allow-Origin",expectedOrigin);
        res.setHeader("Access-Control-Allow-Methods","GET, POST, OPTIONS");
        res.setHeader("Access-Control-Allow-Headers","Content-Type, X-TMA-Bridge-Token");
        res.setHeader("Access-Control-Max-Age","0");
        res.writeHead(204);res.end();return;
      }
      const presented=String(req.headers["x-tma-bridge-token"]??"");
      const expected=Buffer.from(opt.bridgeToken,"utf8");
      const given=Buffer.from(presented,"utf8");
      if(given.length!==expected.length||!timingSafeEqual(given,expected)){
        res.writeHead(403,{"Cache-Control":"no-store"});res.end();return;
      }
      if(req.method==="GET"&&req.url==="/v1/job"){
        respond(res,200,{job:completed?null:job});return;
      }
      if(req.method!=="POST"||req.url!=="/v1/result"||completed){
        respond(res,404,{error:"NOT_FOUND"});return;
      }
      const chunks:Buffer[]=[];
      let len=0;
      for await(const part of req){
        const chunk=Buffer.isBuffer(part)?part:Buffer.from(part);
        len+=chunk.length;
        if(len>MAX_BODY){respond(res,413,{error:"BODY_TOO_LARGE"});return;}
        chunks.push(chunk);
      }
      const payload=JSON.parse(Buffer.concat(chunks).toString("utf8")) as {
        jobId?:string;status?:string;reason?:string;url?:string;title?:string;text?:string;
      };
      if(payload.jobId!==jobId){respond(res,400,{error:"JOB_MISMATCH"});return;}
      let observation:CourseReadResult;
      if(payload.status==="blocked"){
        const reasons=new Set(["COURSE_NOT_FOUND","AUTHENTICATED_CONTENT_NOT_REACHED"]);
        observation=blocked(reasons.has(payload.reason??"")?
          payload.reason as "COURSE_NOT_FOUND"|"AUTHENTICATED_CONTENT_NOT_REACHED":
          "COURSE_READ_FAILED","extension reported absent or unauthenticated course content");
      }else if(payload.status==="read"&&typeof payload.url==="string"&&
               new URL(payload.url).origin===ORIGIN &&
               typeof payload.title==="string"&&payload.title.trim().length>0&&
               typeof payload.text==="string"&&payload.text.trim().length>=80&&
               payload.text.length<=MAX_TEXT){
        const text=payload.text.trim();
        observation={status:"read",url:payload.url,title:payload.title.slice(0,300),text,
          sha256:createHash("sha256").update(text).digest("hex"),
          observedAt:new Date().toISOString(),
          source:"existing_edge_extension_background_tab",externalEffect:false};
      }else{
        observation=blocked("COURSE_READ_FAILED","extension payload violates read-only contract");
      }
      completed=true;
      respond(res,200,{received:true,accepted:observation.status==="read"});
      finish(observation);
    }catch{
      if(!res.headersSent)respond(res,400,{error:"INVALID_MESSAGE"});
    }
  });
  try {
    await new Promise<void>((resolve,reject)=>{
      server.once("error",reject);
      server.listen(port,"127.0.0.1",()=>{server.removeListener("error",reject);resolve();});
    });
  }catch{return blocked("EXTENSION_BRIDGE_UNAVAILABLE","localhost receiver unavailable");}
  let timer:ReturnType<typeof setTimeout>|undefined;
  try {
    return await Promise.race([result,new Promise<CourseReadResult>(res=>{
      timer=setTimeout(()=>res(blocked("EXTENSION_NOT_CONNECTED",
        "no installed Edge extension delivered authenticated lesson evidence")),waitMs);
    })]);
  }finally{
    if(timer)clearTimeout(timer);
    server.close();
    server.closeAllConnections();
  }
}
