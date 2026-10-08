from pathlib import Path
import tomllib

root=Path(__file__).resolve().parents[1]
src=root/"economic-rust/src"
cargo=tomllib.loads((root/"economic-rust/Cargo.toml").read_text(encoding="utf-8"))
expected={"rusqlite","sha2","tma-core","tma-foundation"}
actual=set(cargo.get("dependencies",{}))
if actual!=expected:raise SystemExit(f"F10_PURITY_FAIL dependencies={sorted(actual)}")
for path in src.glob("*.rs"):
 content=path.read_text(encoding="utf-8").lower()
 for token in ["playwright","openai","anthropic","tesseract","selenium","homeocta","99freelas","http://","https://","reqwest"]:
  if token in content:raise SystemExit(f"F10_PURITY_FAIL vendor_or_io_import={path.name}:{token}")
controller=(src/"controller.rs").read_text(encoding="utf-8")
ledger=(src/"ledger.rs").read_text(encoding="utf-8")
scheduler=(src/"scheduler.rs").read_text(encoding="utf-8")
if not all(x in controller for x in ["CapabilityRegistry","PromotionState::Production","ReadinessState::Operational","MissionState::Succeeded"]):
 raise SystemExit("F10_PURITY_FAIL F02_F05_authority_not_consumed")
if not all(x in ledger for x in ["terminal_result_hash","single_earning_per_mission","single_payment_per_settlement","economic_events_append_only","verify_chain"]):
 raise SystemExit("F10_PURITY_FAIL accounting_integrity_missing")
if "f64" in scheduler or "f32" in scheduler:raise SystemExit("F10_PURITY_FAIL floating_point_money")
print("F10_PURITY_OK vendor_sdks=0 f02_registry=1 f05_validated_success=1 append_only_ledger=1 fixed_point=1")
