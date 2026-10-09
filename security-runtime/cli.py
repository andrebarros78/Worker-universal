"""CLI for F12 security operations. Signing keys are read only from an environment variable."""
from __future__ import annotations
import argparse
import json
import os
from pathlib import Path
import sys
from security import (
    AuditTrail, SecurityDenied, safe_rooted, validate_secret_refs,
    backup_sqlite, restore_sqlite,
    signed_release_inventory, verify_release_inventory,
)

def key_from_env() -> bytes:
    # No secrets in command-line history or process arguments.
    raw=os.environ.get("TMA_F12_SIGNING_KEY_HEX")
    if raw is None:
        raise SecurityDenied("F12_SIGNING_KEY_NOT_CONFIGURED")
    try:
        value=bytes.fromhex(raw)
    except ValueError as exc:
        raise SecurityDenied("F12_SIGNING_KEY_INVALID") from exc
    if len(value)<32:
        raise SecurityDenied("F12_SIGNING_KEY_TOO_SHORT")
    return value

def main(argv:list[str]|None=None)->int:
    parser=argparse.ArgumentParser(prog="tma-f12-security")
    parser.add_argument("action",choices=[
        "validate-config","backup","restore","audit-record","audit-verify",
        "inventory-create","inventory-verify",
    ])
    parser.add_argument("--root",required=True)
    parser.add_argument("--source")
    parser.add_argument("--destination")
    parser.add_argument("--snapshot")
    parser.add_argument("--config")
    parser.add_argument("--db")
    parser.add_argument("--event")
    parser.add_argument("--actor")
    parser.add_argument("--operation")
    parser.add_argument("--outcome")
    parser.add_argument("--resource")
    parser.add_argument("--at",type=int)
    args=parser.parse_args(argv)
    root=Path(args.root).absolute()

    def must(value:str|None, option:str)->Path:
        if not value:raise SecurityDenied("F12_REQUIRED_ARGUMENT_"+option)
        return safe_rooted(root,Path(value))
    if args.action=="validate-config":
        p=must(args.config,"CONFIG")
        validate_secret_refs(json.loads(p.read_text(encoding="utf-8")))
        print("F12_SECRET_POLICY=PASS inline_credentials=denied references_only=true")
    elif args.action=="backup":
        manifest=backup_sqlite(root=root,source=must(args.source,"SOURCE"),
                               destination=must(args.destination,"DESTINATION"),
                               signing_key=key_from_env())
        print(f"F12_BACKUP=PASS bytes={manifest['bytes']} signed=true")
    elif args.action=="restore":
        path=restore_sqlite(root=root,snapshot=must(args.snapshot,"SNAPSHOT"),
                            target=must(args.destination,"DESTINATION"),
                            signing_key=key_from_env())
        print(f"F12_RESTORE=PASS integrity=ok separate_target=true")
    elif args.action=="audit-record":
        p=must(args.db,"DB")
        status=AuditTrail(p).record(event_id=args.event,actor_id=args.actor,
            action=args.operation,outcome=args.outcome,resource_id=args.resource,
            occurred_ms=args.at)
        print(f"F12_AUDIT_RECORD={status}")
    elif args.action=="audit-verify":
        n=AuditTrail(must(args.db,"DB")).verify()
        print(f"F12_AUDIT_VERIFY=PASS events={n}")
    elif args.action=="inventory-create":
        count=len(signed_release_inventory(project_root=root,
            output=must(args.destination,"DESTINATION"),signing_key=key_from_env()))
        print(f"F12_INVENTORY=PASS manifests={count} signed=true")
    elif args.action=="inventory-verify":
        count=verify_release_inventory(project_root=root,signing_key=key_from_env(),
            inventory_file=must(args.source,"SOURCE"))
        print(f"F12_INVENTORY_VERIFY=PASS manifests={count}")
    return 0

if __name__=="__main__":
    try:sys.exit(main())
    except (SecurityDenied, OSError, ValueError, json.JSONDecodeError, TypeError) as exc:
        # Expose only an error code/class, never user data or credential text.
        label=str(exc) if isinstance(exc,SecurityDenied) else type(exc).__name__
        print(f"F12_OPERATION=DENIED code={label}",file=sys.stderr)
        sys.exit(1)
