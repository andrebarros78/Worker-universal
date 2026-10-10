import assert from "node:assert/strict";
import test from "node:test";
import {exitCodeForCourseReport} from "../src/course_worker_cli.ts";
import type {CourseMissionReport} from "../src/course_mission.ts";
const base:CourseMissionReport={
 schemaVersion:1,missionId:"homologation-only",executor:"tma-web-worker",
 trainingOnly:true,productionProven:false,missionProven:false,
 status:"blocked",reason:"EDGE_PROFILE_NOT_ACCESSIBLE",
 captureCount:0,sourceEvidence:[],observedAt:"2026-10-09T19:00:00.000Z"
};
test("Worker exit code 3 when no authenticated course can be read",()=>{
 assert.equal(exitCodeForCourseReport(base),3);
});
test("capturing lesson without assessment MUST NOT return zero success",()=>{
 const captured={...base,status:"captured_unassessed" as const,reason:"NONE",captureCount:1};
 assert.equal(exitCodeForCourseReport(captured),2);
});
test("forged proof and contradictory state cannot return success",()=>{
 const forged={...base,status:"captured_unassessed" as const,missionProven:true} as unknown as CourseMissionReport;
 assert.throws(()=>exitCodeForCourseReport(forged),/COURSE_PROOF_STATE_INCONSISTENT/);
});
test("unknown states cannot return success",()=>{
 const bad={...base,status:"learned"} as unknown as CourseMissionReport;
 assert.throws(()=>exitCodeForCourseReport(bad),/COURSE_PROOF_STATE_INCONSISTENT/);
});
