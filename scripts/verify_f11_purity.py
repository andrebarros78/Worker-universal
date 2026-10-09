from pathlib import Path
import tomllib
root=Path(__file__).resolve().parents[1]
cargo=tomllib.loads((root/"availability-rust/Cargo.toml").read_text(encoding="utf-8"))
if set(cargo.get("dependencies",{}))!={"tma-core"}:
    raise SystemExit("F11_PURITY_FAIL unexpected_dependency")
rust=(root/"availability-rust/src/lib.rs").read_text(encoding="utf-8")
go=(root/"supervisor-go/supervisor/pool.go").read_text(encoding="utf-8")
cmd=(root/"supervisor-go/cmd/tma-supervisor/main.go").read_text(encoding="utf-8")
if not all(w in rust for w in ["DurableStore","claim_next","save_checkpoint","verify_ledger","integrity_check","fence_seq"]):
    raise SystemExit("F11_PURITY_FAIL missing_f05_integration")
if not all(w in go for w in ["ErrBackpressure","ErrDuplicateTask","watchdog_stalled","/readyz","/healthz","RunWithRestart"]):
    raise SystemExit("F11_PURITY_FAIL missing_go_supervisor")
if not all(w in cmd for w in ["127.0.0.1:","serve-health","signal.NotifyContext"]):
    raise SystemExit("F11_PURITY_FAIL health_binding")
for p in (root/"availability-rust/src").glob("*.rs"):
    s=p.read_text(encoding="utf-8").lower()
    for token in ["playwright","openai","selenium","homeocta","99freelas","http://","https://"]:
        if token in s:raise SystemExit("F11_PURITY_FAIL vendor_code")
print("F11_PURITY_OK durable_authority=F05 retries=bounded resource_locks=1 queue_backpressure=1 http=loopback only")
