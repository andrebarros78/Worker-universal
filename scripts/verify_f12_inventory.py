from pathlib import Path
import importlib.util
import secrets
import os
import uuid
r=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location("f12_security",r/"security-runtime/security.py")
assert spec and spec.loader
security=importlib.util.module_from_spec(spec)
spec.loader.exec_module(security)
folder=r/"runtime"
folder.mkdir(exist_ok=True)
out=folder/("f12-release-inventory-"+uuid.uuid4().hex+".json")
key=secrets.token_bytes(32)
try:
    result=security.signed_release_inventory(project_root=r,signing_key=key,output=out)
    count=security.verify_release_inventory(project_root=r,signing_key=key,inventory_file=out)
    if count!=len(result):raise RuntimeError("INVENTORY_MISMATCH")
    print(f"F12_RELEASE_INVENTORY=PASS dependencies={count} hmac_sha256=true")
finally:
    out.unlink(missing_ok=True)
