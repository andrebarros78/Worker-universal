"""F12 security and operational recovery controls. Python stdlib only."""
from __future__ import annotations

from contextlib import contextmanager, closing
import hashlib
import hmac
import json
import os
from pathlib import Path
import re
import shutil
import sqlite3
import subprocess
import time

class SecurityDenied(RuntimeError):
    pass

SENSITIVE_KEYS = frozenset({
    "password", "passwd", "api_key", "apikey", "access_token",
    "refresh_token", "client_secret", "authorization", "cookie",
    "private_key", "secret", "token", "credential", "credentials",
})
REFERENCE = re.compile(r"^env://TMA_[A-Z][A-Z0-9_]{0,95}$")
SAFE_ID = re.compile(r"^[A-Za-z0-9_.:-]{1,120}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
MANIFEST_VERSION = 1

def canonical_bytes(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=True, sort_keys=True, separators=(",", ":")).encode("ascii")

def digest(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()

MAX_DATABASE_BYTES = 1024 * 1024 * 1024

def hash_file(path: Path) -> tuple[int, str]:
    size = 0
    state = hashlib.sha256()
    with open(path, "rb") as src:
        while block := src.read(1024 * 1024):
            size += len(block)
            if size > MAX_DATABASE_BYTES:
                raise SecurityDenied("F12_DATABASE_SIZE_LIMIT")
            state.update(block)
    return size, state.hexdigest()

def hmac_sign(key: bytes, value: bytes) -> str:
    if len(key) < 32:
        raise SecurityDenied("F12_KEY_TOO_SHORT")
    return hmac.new(key, value, hashlib.sha256).hexdigest()

def validate_secret_refs(payload: object) -> None:
    """Only non-resolving references are permitted. Never log values."""
    def walk(value: object, path: str = "$") -> None:
        if isinstance(value, dict):
            for k, item in value.items():
                if not isinstance(k, str):
                    raise SecurityDenied("F12_CONFIG_KEY_INVALID")
                if k.lower().replace("-", "_") in SENSITIVE_KEYS:
                    raise SecurityDenied("F12_INLINE_SECRET_FIELD_DENIED")
                if k in ("secretRefs", "secret_refs"):
                    if not isinstance(item, list) or any(
                        not isinstance(v, str) or not REFERENCE.fullmatch(v) for v in item
                    ):
                        raise SecurityDenied("F12_SECRET_REFERENCE_INVALID")
                    continue
                walk(item, path + "." + k)
        elif isinstance(value, list):
            for v in value:
                walk(v, path)
        elif isinstance(value, str):
            if "://" in value:
                from urllib.parse import urlsplit
                parsed = urlsplit(value)
                if parsed.username is not None or parsed.password is not None:
                    raise SecurityDenied("F12_URL_USERINFO_DENIED")
    walk(payload)

def safe_identity(value: str) -> str:
    if not isinstance(value, str) or not SAFE_ID.fullmatch(value):
        raise SecurityDenied("F12_AUDIT_METADATA_UNSAFE")
    # Do not persist common raw token formats in audit metadata.
    if (re.search(r"(?i)(?:sk-[a-z0-9]{12,}|ghp_[a-z0-9]{12,}|"
                  r"akia[a-z0-9]{12,})",value) or value.lower().startswith("bearer")):
        raise SecurityDenied("F12_AUDIT_POSSIBLE_SECRET_DENIED")
    return value

def safe_rooted(root: Path, path: Path, *, must_exist: bool = False) -> Path:
    root = Path(root).absolute()
    path = Path(path).absolute()
    if not root.is_dir():
        raise SecurityDenied("F12_ROOT_NOT_DIRECTORY")
    try:
        rel = path.relative_to(root)
    except ValueError as e:
        raise SecurityDenied("F12_OUTSIDE_ROOT_DENIED") from e
    if rel == Path("."):
        raise SecurityDenied("F12_ROOT_AS_TARGET_DENIED")
    cursor = root
    if cursor.is_symlink():
        raise SecurityDenied("F12_ROOT_SYMLINK_DENIED")
    for part in rel.parts:
        if part in ("..", "."):
            raise SecurityDenied("F12_PATH_TRAVERSAL_DENIED")
        cursor = cursor / part
        if cursor.is_symlink():
            raise SecurityDenied("F12_SYMLINK_DENIED")
    if must_exist and not path.exists():
        raise SecurityDenied("F12_REQUIRED_PATH_MISSING")
    return path

def create_private_dir(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=False)
    if os.name == "nt":
        import csv
        import io
        result = subprocess.run(["whoami", "/user", "/fo", "csv", "/nh"],
            capture_output=True, text=True, check=True, timeout=10)
        row = next(csv.reader(io.StringIO(result.stdout)))
        sid = row[1].strip()
        if not re.fullmatch(r"S-1-\d+(?:-\d+)+", sid):
            raise SecurityDenied("F12_ACCOUNT_SID_INVALID")
        # Applies only to the newly created destination, never to an ancestor.
        subprocess.run(["icacls", str(path), "/inheritance:r"], capture_output=True,
                       text=True, check=True, timeout=10)
        grants = [f"*{sid}:(OI)(CI)F",
                  "*S-1-5-18:(OI)(CI)F", "*S-1-5-32-544:(OI)(CI)F"]
        subprocess.run(["icacls", str(path), "/grant:r", *grants], capture_output=True,
                       text=True, check=True, timeout=10)
    else:
        path.chmod(0o700)

@contextmanager
def transaction_connection(path: Path, timeout: float = 10):
    with closing(sqlite3.connect(path, timeout=timeout)) as con:
        with con:
            yield con

class AuditTrail:
    """SQLite append-only event log. Secret values are never accepted as metadata."""
    def __init__(self, db: Path):
        self.db = Path(db)
        self.db.parent.mkdir(parents=True, exist_ok=True)
        with self._conn() as con:
            con.execute("""CREATE TABLE IF NOT EXISTS audit_events (
               id INTEGER PRIMARY KEY AUTOINCREMENT,
               event_id TEXT NOT NULL UNIQUE,
               actor_id TEXT NOT NULL,
               action TEXT NOT NULL,
               outcome TEXT NOT NULL,
               resource_id TEXT NOT NULL,
               occurred_ms INTEGER NOT NULL,
               previous_hash TEXT NOT NULL,
               event_hash TEXT NOT NULL
            )""")
            con.execute("""CREATE TRIGGER IF NOT EXISTS audit_no_update
                BEFORE UPDATE ON audit_events BEGIN SELECT RAISE(ABORT,'audit_append_only'); END""")
            con.execute("""CREATE TRIGGER IF NOT EXISTS audit_no_delete
                BEFORE DELETE ON audit_events BEGIN SELECT RAISE(ABORT,'audit_append_only'); END""")
    @contextmanager
    def _conn(self):
        with transaction_connection(self.db, timeout=10) as con:
            con.execute("PRAGMA synchronous=FULL")
            yield con
    def record(self, *, event_id: str, actor_id: str, action: str,
               outcome: str, resource_id: str, occurred_ms: int) -> str:
        fields = (event_id, actor_id, action, outcome, resource_id)
        for field in fields:
            safe_identity(field)
        if not isinstance(occurred_ms, int) or occurred_ms < 0:
            raise SecurityDenied("F12_AUDIT_TIMESTAMP_INVALID")
        payload = {
            "event_id": event_id, "actor_id": actor_id, "action": action,
            "outcome": outcome, "resource_id": resource_id, "occurred_ms": occurred_ms,
        }
        with self._conn() as con:
            con.execute("BEGIN IMMEDIATE")
            row = con.execute(
                "SELECT previous_hash,event_hash,actor_id,action,outcome,resource_id,occurred_ms "
                "FROM audit_events WHERE event_id=?", (event_id,)
            ).fetchone()
            if row:
                previous, stored_hash, actor, act, out, res, at = row
                expected = digest(canonical_bytes({"prev": previous, "event": payload}))
                if stored_hash == expected and (actor, act, out, res, at) == (
                    actor_id, action, outcome, resource_id, occurred_ms
                ):
                    return "duplicate"
                raise SecurityDenied("F12_AUDIT_CONFLICT")
            last = con.execute(
                "SELECT event_hash FROM audit_events ORDER BY id DESC LIMIT 1").fetchone()
            prev = last[0] if last else "GENESIS"
            evhash = digest(canonical_bytes({"prev": prev, "event": payload}))
            con.execute(
                "INSERT INTO audit_events(event_id,actor_id,action,outcome,resource_id,"
                "occurred_ms,previous_hash,event_hash) VALUES(?,?,?,?,?,?,?,?)",
                (*fields, occurred_ms, prev, evhash))
        return "recorded"
    def verify(self) -> int:
        with self._conn() as con:
            rows = con.execute(
                "SELECT event_id,actor_id,action,outcome,resource_id,occurred_ms,"
                "previous_hash,event_hash FROM audit_events ORDER BY id"
            ).fetchall()
            prev = "GENESIS"
            for event_id, actor, action, outcome, resource, at, saved_prev, saved_hash in rows:
                if prev != saved_prev:
                    raise SecurityDenied("F12_AUDIT_CHAIN_PREV_MISMATCH")
                payload = {
                    "event_id": event_id, "actor_id": actor, "action": action,
                    "outcome": outcome, "resource_id": resource, "occurred_ms": at,
                }
                expected = digest(canonical_bytes({"prev": prev, "event": payload}))
                if expected != saved_hash:
                    raise SecurityDenied("F12_AUDIT_HASH_MISMATCH")
                prev = saved_hash
            return len(rows)

def sqlite_check(path: Path) -> None:
    try:
        with closing(sqlite3.connect(f"file:{path.as_posix()}?mode=ro", uri=True, timeout=5)) as con:
            result = con.execute("PRAGMA integrity_check").fetchone()
        if not result or result[0] != "ok":
            raise SecurityDenied("F12_DATABASE_CORRUPT")
    except (sqlite3.Error, OSError) as e:
        raise SecurityDenied("F12_DATABASE_UNAVAILABLE_OR_CORRUPT") from e

def backup_sqlite(*, root: Path, source: Path, destination: Path,
                  signing_key: bytes) -> dict:
    """Crash-consistent online snapshot; destination must not exist."""
    source = safe_rooted(root, source, must_exist=True)
    destination = safe_rooted(root, destination)
    if not source.is_file() or destination.exists():
        raise SecurityDenied("F12_BACKUP_INVALID_PATH")
    sqlite_check(source)
    # An authenticated manifest cannot be created without a key.
    hmac_sign(signing_key, b"preflight")
    create_private_dir(destination)
    try:
        target = destination / "database.sqlite3"
        with closing(sqlite3.connect(f"file:{source.as_posix()}?mode=ro", uri=True)) as src:
            with closing(sqlite3.connect(target)) as dst:
                src.backup(dst, pages=32, sleep=0.05)
        sqlite_check(target)
        length, checksum = hash_file(target)
        manifest = {
            "schema": MANIFEST_VERSION, "type": "sqlite-online-snapshot",
            "source_name": source.name, "artifact": target.name,
            "sha256": checksum, "bytes": length,
            "created_ms": int(time.time() * 1000),
        }
        envelope = {
            "manifest": manifest,
            "signature_hmac_sha256": hmac_sign(signing_key, canonical_bytes(manifest))
        }
        (destination / "manifest.json").write_bytes(canonical_bytes(envelope))
        return manifest
    except Exception:
        shutil.rmtree(destination, ignore_errors=True)
        raise

def restore_sqlite(*, root: Path, snapshot: Path, target: Path,
                   signing_key: bytes) -> str:
    """Restore into a fresh directory; never overwrite a live database."""
    snapshot = safe_rooted(root, snapshot, must_exist=True)
    target = safe_rooted(root, target)
    if not snapshot.is_dir() or target.exists():
        raise SecurityDenied("F12_RESTORE_DENIED_TARGET_EXISTS")
    meta_path = safe_rooted(root, snapshot/"manifest.json")
    if not meta_path.is_file():
        raise SecurityDenied("F12_BACKUP_MANIFEST_MISSING")
    if meta_path.stat().st_size > 64_000:
        raise SecurityDenied("F12_BACKUP_MANIFEST_TOO_LARGE")
    try:
        envelope = json.loads(meta_path.read_text(encoding="ascii"))
        manifest = envelope["manifest"]
        signature = envelope["signature_hmac_sha256"]
        if (manifest["schema"] != 1 or manifest["type"] != "sqlite-online-snapshot"
                or manifest["artifact"] != "database.sqlite3"
                or not HEX64.fullmatch(manifest["sha256"])
                or not HEX64.fullmatch(signature)):
            raise SecurityDenied("F12_BACKUP_MANIFEST_INVALID")
    except (KeyError, ValueError, TypeError) as e:
        raise SecurityDenied("F12_BACKUP_MANIFEST_INVALID") from e
    expected = hmac_sign(signing_key, canonical_bytes(manifest))
    if not hmac.compare_digest(signature, expected):
        raise SecurityDenied("F12_BACKUP_SIGNATURE_INVALID")
    artifact = safe_rooted(root, snapshot / "database.sqlite3", must_exist=True)
    if not artifact.is_file():
        raise SecurityDenied("F12_BACKUP_ARTIFACT_INVALID")
    length, checksum = hash_file(artifact)
    if length != manifest["bytes"] or checksum != manifest["sha256"]:
        raise SecurityDenied("F12_BACKUP_HASH_INVALID")
    sqlite_check(artifact)
    create_private_dir(target)
    try:
        dst = target / "restored.sqlite3"
        with open(artifact,"rb") as source:
            with open(dst,"xb") as stream:
                shutil.copyfileobj(source, stream, length=1024*1024)
                stream.flush()
                os.fsync(stream.fileno())
        sqlite_check(dst)
        if hash_file(dst) != (length, checksum):
            raise SecurityDenied("F12_RESTORE_VERIFICATION_FAILED")
        return str(dst)
    except Exception:
        shutil.rmtree(target, ignore_errors=True)
        raise

DEPENDENCY_FILES = (
    "core-rust/Cargo.lock", "foundation-rust/Cargo.lock",
    "planner-rust/Cargo.lock", "validation-rust/Cargo.lock",
    "qualification-rust/Cargo.lock", "economic-rust/Cargo.lock",
    "availability-rust/Cargo.lock", "security-rust/Cargo.lock",
    "supervisor-go/go.mod", "web-worker/package-lock.json",
    "web-worker/package.json", "platform-adapters/package.json",
    "formal-ada/tma_formal.gpr",
)
def dependency_inventory(project_root: Path) -> dict[str, str]:
    base = Path(project_root).absolute()
    result: dict[str,str] = {}
    for rel in DEPENDENCY_FILES:
        path = safe_rooted(base, base/rel)
        if not path.is_file():
            raise SecurityDenied("F12_DEPENDENCY_MANIFEST_MISSING:"+rel)
        result[rel] = digest(path.read_bytes())
    return result

def signed_release_inventory(*, project_root: Path, signing_key: bytes,
                             output: Path) -> dict:
    project_root = Path(project_root).absolute()
    output = safe_rooted(project_root, output)
    if output.exists():
        raise SecurityDenied("F12_RELEASE_INVENTORY_EXISTS")
    artifacts = dependency_inventory(project_root)
    manifest={"schema":1, "type":"dependency-inventory", "files":artifacts}
    envelope={"manifest":manifest,
              "signature_hmac_sha256":hmac_sign(signing_key,canonical_bytes(manifest))}
    output.write_bytes(canonical_bytes(envelope))
    return artifacts

def verify_release_inventory(*, project_root: Path, signing_key: bytes,
                             inventory_file: Path) -> int:
    project_root=Path(project_root).absolute()
    inventory_file=safe_rooted(project_root,inventory_file,must_exist=True)
    try:
        envelope=json.loads(inventory_file.read_bytes())
        manifest=envelope["manifest"]
        if manifest.get("schema")!=1 or manifest.get("type")!="dependency-inventory":
            raise SecurityDenied("F12_RELEASE_SCHEMA_INVALID")
        signature=envelope["signature_hmac_sha256"]
        if not isinstance(signature,str) or not hmac.compare_digest(
            hmac_sign(signing_key,canonical_bytes(manifest)),signature
        ):
            raise SecurityDenied("F12_RELEASE_SIGNATURE_INVALID")
        current=dependency_inventory(project_root)
        if manifest["files"]!=current:
            raise SecurityDenied("F12_RELEASE_DEPENDENCY_DRIFT")
        return len(current)
    except (json.JSONDecodeError,KeyError,TypeError) as e:
        raise SecurityDenied("F12_RELEASE_INVENTORY_INVALID") from e
