from pathlib import Path
root=Path(__file__).resolve().parents[1]
src=root/"platform-adapters/src"
runtime=(src/"runner.ts").read_text(encoding="utf-8")
policy=(src/"policy.ts").read_text(encoding="utf-8")
adapters=(src/"adapters.ts").read_text(encoding="utf-8")
pkg=(root/"platform-adapters/package.json").read_text(encoding="utf-8")
if 'BrowserRuntime' not in runtime or 'web-worker/src/browser_runtime.ts' not in runtime:
 raise SystemExit("F09_PURITY_FAIL F04_isolation_not_consumed")
if 'sha256(buffer)' not in runtime or 'requiredEvidence' not in runtime:
 raise SystemExit("F09_PURITY_FAIL evidence_integrity_missing")
if "SIMULATION_LOOPBACK_ONLY" not in policy or "LIVE_NOT_AUTHORIZED" not in policy:
 raise SystemExit("F09_PURITY_FAIL live_gate_missing")
if "QUALIFICATION_DENIED" not in policy or "READ_ONLY" not in policy:
 raise SystemExit("F09_PURITY_FAIL platform_policies_missing")
if '"experimental"' not in adapters or '.set_promotion(' in adapters:
 raise SystemExit("F09_PURITY_FAIL F08_quarantine_missing")
if 'playwright' in pkg.lower() or '"dependencies"' in pkg:
 raise SystemExit("F09_PURITY_FAIL duplicate_browser_runtime")
for p in src.glob("*.ts"):
 text=p.read_text(encoding="utf-8").lower()
 if any(x in text for x in ["stealth", "captcha bypass", "fingerprint spoof", "mfa bypass", "kyc bypass"]):
  raise SystemExit("F09_PURITY_FAIL forbidden_evasion")
bridge=(src/"sdk_bridge.ts").read_text(encoding="utf-8")
if '../../adapter-sdk/typescript/sdk.ts' not in bridge or '"needs_review"' not in bridge:
 raise SystemExit("F09_PURITY_FAIL F02_sdk_bridge_missing")
if (root/"platform-adapters/node_modules").exists():
 raise SystemExit("F09_PURITY_FAIL duplicated_node_modules")
print("F09_PURITY_OK browser_authority=F04 ocr_authority=F03 validation_authority=F07 qualification_authority=F08 external_effects=0")
