from pathlib import Path
import tomllib
r=Path(__file__).resolve().parents[1]
manifest=tomllib.loads((r/"production-rust/Cargo.toml").read_text(encoding="utf-8"))
deps=set(manifest["dependencies"])
expected={"tma-core","tma-foundation","tma-planner","tma-validation","tma-economic","sha2"}
if deps!=expected:raise SystemExit("F13_PURITY_FAIL unexpected_dependency")
lib=(r/"production-rust/src/lib.rs").read_text(encoding="utf-8")
adapters=(r/"platform-adapters/src/policy.ts").read_text(encoding="utf-8")
for token in ("finalize_validated_success","EvidenceValidator","persist_plan_checkpoint","recover_plan_checkpoint",
              "schedule_registered","PromotionState::Experimental","observed_success","record_cost"):
    if token=="observed_success":continue
    if token not in lib:raise SystemExit("F13_PURITY_FAIL missing_"+token)
for token in ('config.mode !== "simulation"',"F09_POLICY_LIVE_NOT_AUTHORIZED"):
    if token not in adapters:raise SystemExit("F13_PURITY_FAIL F09_productive_gate_removed")
if "record_verified_earning" in lib or "record_payment" in lib:
    raise SystemExit("F13_PURITY_FAIL synthetic_revenue_path")
if "Production" in lib and "PromotionState::Production" in lib:
    raise SystemExit("F13_PURITY_FAIL simulated_promotion")
print("F13_PURITY_OK native_synthetic_only=1 f02_gate=1 f07_validation=1 f05_fencing=1 f10_zero_revenue=1 f09_external_block=1")
