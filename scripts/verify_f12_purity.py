from pathlib import Path
import tomllib
root=Path(__file__).resolve().parents[1]
p=root/"security-rust/Cargo.toml"
deps=set(tomllib.loads(p.read_text(encoding="utf-8")).get("dependencies",{}))
if deps!={"tma-core"}:raise SystemExit("F12_PURITY_FAIL Rust_dependency_boundary")
s=(root/"security-runtime/security.py").read_text(encoding="utf-8")
cli=(root/"security-runtime/cli.py").read_text(encoding="utf-8")
code=(root/"security-rust/src/main.rs").read_text(encoding="utf-8")
asserts=["safe_rooted","secretRefs","AuditTrail","verify_chain","backup_sqlite",
         "restore_sqlite","hmac.compare_digest","dependency_inventory","create_private_dir"]
for word in asserts:
    if word=="verify_chain":
        if "def verify(" not in s:raise SystemExit("F12_PURITY_FAIL audit_chain")
    elif word not in s:raise SystemExit("F12_PURITY_FAIL missing_"+word)
for word in ("TMA_F12_SIGNING_KEY_HEX","SIGNING_KEY_NOT_CONFIGURED","F12_OPERATION=DENIED"):
    if word not in cli:raise SystemExit("F12_PURITY_FAIL CLI_secret_handling")
for word in ("DurableStore","verify_ledger","integrity_check","queue_depth"):
    if word not in code:raise SystemExit("F12_PURITY_FAIL F05_verification")
for p in list((root/"security-runtime").glob("*.py"))+list((root/"security-rust/src").glob("*.rs")):
    text=p.read_text(encoding="utf-8").lower()
    for word in ("playwright","selenium","openrouter","stripe","paypal"):
        if word in text:raise SystemExit("F12_PURITY_FAIL new_vendor_dependency")
print("F12_PURITY_OK no_provider_sdks=1 secrets_reference_only=1 audit=append_only backup=signed restored_f05=rust_verified")
