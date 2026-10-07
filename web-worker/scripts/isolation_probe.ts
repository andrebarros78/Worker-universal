import { execFileSync } from "node:child_process";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { BrowserRuntime } from "../src/browser_runtime.ts";

const root = await mkdtemp(join(tmpdir(), "tma-f04-isolation-"));
const runtime = new BrowserRuntime();

try {
  await runtime.start(join(root, "session.json"), join(root, "evidence"));
  const pid = runtime.browserPids().at(-1);
  if (!pid) throw new Error("browser_pid_missing");

  const command =
    "$p=Get-CimInstance Win32_Process -Filter \"ProcessId = " +
    pid +
    "\"; " +
    "$gp=Get-Process -Id " +
    pid +
    "; " +
    "$e=Get-Process explorer -ErrorAction SilentlyContinue | Select-Object -First 1; " +
    "[pscustomobject]@{BrowserPid=$gp.Id;BrowserSession=$gp.SessionId;ExplorerSession=if($e){$e.SessionId}else{$null};CommandLine=$p.CommandLine}|ConvertTo-Json -Compress";

  const raw = execFileSync(
    "powershell.exe",
    ["-NoProfile", "-NonInteractive", "-Command", command],
    { encoding: "utf8" },
  ).trim();
  const info = JSON.parse(raw);

  if (!runtime.isHeadless()) throw new Error("runtime_not_headless");
  if (!String(info.CommandLine ?? "").includes("--headless")) {
    throw new Error("browser_command_line_not_headless");
  }
  if (
    info.ExplorerSession !== null &&
    info.ExplorerSession !== undefined &&
    info.BrowserSession === info.ExplorerSession
  ) {
    throw new Error("browser_shares_operator_explorer_session");
  }

  console.log(
    JSON.stringify({
      isolation: "pass",
      browserPid: info.BrowserPid,
      browserSession: info.BrowserSession,
      explorerSession: info.ExplorerSession,
      headless: true,
      sharedOperatorSession: false,
    }),
  );
} finally {
  await runtime.stop();
  await rm(root, { recursive: true, force: true });
}
