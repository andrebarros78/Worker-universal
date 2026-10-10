import assert from "node:assert/strict";
import test from "node:test";
import {createHash} from "node:crypto";
import {mkdtemp,mkdir,rm,readdir} from "node:fs/promises";
import {join} from "node:path";
import {tmpdir} from "node:os";
import {makeLessonRecord,persistLessonRecord,loadLessonRecord,recallLesson} from "../src/course_knowledge.ts";
import type {CourseReadResult} from "../src/edge_course_reader.ts";

// Memory fixture is synthetic by design, NOT claimed as an authenticated course.
const sha=(s:string)=>createHash("sha256").update(s).digest("hex");
function sample(url:string,body:string):CourseReadResult {
  return {status:"read",url,title:"Synthetic memory verification only",text:body,
    sha256:sha(body),observedAt:"2026-10-09T12:00:00.000Z",
    source:"existing_edge_cdp_background_tab",externalEffect:false};
}
test("one objective preserves two independently sourced lesson records across reload",async()=>{
  const temp=join(tmpdir(),"tma-memory-record-tests");
  await mkdir(temp,{recursive:true});
  const dir=await mkdtemp(join(temp,"case-"));
  const a="In the fictional training task, a reservation code must be confirmed before check-in begins. If it is unavailable, the operator must stop and escalate to the responsible team for review.";
  const b="In the fictional training task, a receipt is checked against the corresponding document amount. A discrepancy means the operation must be held for independent verification before proceeding.";
  try{
    const inputs=[
      {url:"https://www.homeocta.com/test/lesson-one",text:a},
      {url:"https://www.homeocta.com/test/lesson-two",text:b}
    ];
    for(const v of inputs){
      const id="OCTA-"+sha(v.url).slice(0,16);
      const result=makeLessonRecord(id,sample(v.url,v.text));
      await persistLessonRecord(dir,result);
    }
    const entries=await readdir(dir);
    assert.equal(entries.length,2);
    for(const v of inputs){
      const id="OCTA-"+sha(v.url).slice(0,16);
      const memory=await loadLessonRecord(dir,id);
      assert.equal(memory.sourceSha256,sha(v.text));
      assert.equal(memory.learningStatus,"captured_unassessed");
      assert.equal(memory.sourceUrl,v.url);
      assert.equal(recallLesson(memory,"extraterrestrial astronomy").status,"no_grounding");
    }
  }finally{await rm(dir,{recursive:true,force:true})}
});
