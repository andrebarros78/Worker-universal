import {createHash} from "node:crypto";
import {mkdir,readFile,writeFile} from "node:fs/promises";
import {join} from "node:path";
import type {CourseReadResult} from "./edge_course_reader.ts";

// A source-backed learning record. Indexed recall is not semantic comprehension,
// and does not itself promote training or productive homologation to success.
export type LessonRecord={
  schemaVersion:1;objectiveId:string;sourceUrl:string;title:string;observedAt:string;
  sourceSha256:string;capturedText:string;captureMethod:"existing_edge_cdp_background_tab"|"existing_edge_extension_background_tab";
  learningStatus:"captured_unassessed";recordSha256:string;
};
const sha256=(s:string)=>createHash("sha256").update(s).digest("hex");
const TOKEN=/[\p{L}\p{N}]{3,}/gu;
const tokenize=(s:string):Set<string>=>new Set((s.toLocaleLowerCase("pt-BR").match(TOKEN)??[]));
function fileName(id:string) {
  if(!/^[A-Za-z0-9_.:-]{1,140}$/.test(id)) throw Error("INVALID_OBJECTIVE_ID");
  return id.replace(/:/g,"_")+".json";
}
function hashable(r:Omit<LessonRecord,"recordSha256">):string {
  return JSON.stringify(r);
}
export function makeLessonRecord(objectiveId:string,read:CourseReadResult):LessonRecord {
  if(read.status!=="read") throw Error("NO_AUTHENTICATED_LESSON_EVIDENCE");
  if(sha256(read.text)!==read.sha256) throw Error("SOURCE_EVIDENCE_HASH_MISMATCH");
  if(read.text.trim().length<80 || !read.title.trim()) throw Error("INSUFFICIENT_LESSON_EVIDENCE");
  fileName(objectiveId);
  const withoutHash:Omit<LessonRecord,"recordSha256">={
    schemaVersion:1,objectiveId,sourceUrl:read.url,title:read.title,observedAt:read.observedAt,
    sourceSha256:read.sha256,capturedText:read.text,
    captureMethod:read.source,learningStatus:"captured_unassessed"
  };
  return {...withoutHash,recordSha256:sha256(hashable(withoutHash))};
}
export function validateLessonRecord(record:LessonRecord):boolean {
  const {recordSha256,...rest}=record;
  return record.schemaVersion===1 && record.learningStatus==="captured_unassessed" &&
    sha256(record.capturedText)===record.sourceSha256 &&
    sha256(hashable(rest))===recordSha256;
}
export async function persistLessonRecord(directory:string,record:LessonRecord):Promise<string> {
  if(!validateLessonRecord(record)) throw Error("LEARNING_EVIDENCE_INVALID");
  await mkdir(directory,{recursive:true});
  const path=join(directory,fileName(record.objectiveId));
  await writeFile(path,JSON.stringify(record,null,2)+"\n",{encoding:"utf8",flag:"wx",mode:0o600});
  return path;
}
export async function loadLessonRecord(directory:string,objectiveId:string):Promise<LessonRecord> {
  const content=await readFile(join(directory,fileName(objectiveId)),"utf8");
  const record:LessonRecord=JSON.parse(content);
  if(!validateLessonRecord(record) || record.objectiveId!==objectiveId)
    throw Error("LEARNING_EVIDENCE_INVALID");
  return record;
}
export function recallLesson(record:LessonRecord,question:string):{
  status:"source_excerpt"|"no_grounding";excerpt?:string;sourceUrl:string;confidence:"unassessed";
} {
  if(!validateLessonRecord(record)) throw Error("LEARNING_EVIDENCE_INVALID");
  const query=tokenize(question);
  if(query.size===0) return {status:"no_grounding",sourceUrl:record.sourceUrl,confidence:"unassessed"};
  const sentences=record.capturedText.split(/(?<=[.!?])\s+|\n+/).map(s=>s.trim()).filter(s=>s.length>=25);
  const ranked=sentences.map(sentence=>{
    const words=tokenize(sentence);
    const overlap=[...query].filter(x=>words.has(x)).length;
    return {sentence,overlap};
  }).sort((a,b)=>b.overlap-a.overlap);
  if(!ranked[0] || ranked[0].overlap===0)
    return {status:"no_grounding",sourceUrl:record.sourceUrl,confidence:"unassessed"};
  return {status:"source_excerpt",excerpt:ranked[0].sentence.slice(0,1000),
    sourceUrl:record.sourceUrl,confidence:"unassessed"};
}
