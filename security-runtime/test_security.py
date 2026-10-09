"""F12 focused operational security tests. Synthetic data only."""
from __future__ import annotations
import importlib.util
from contextlib import contextmanager, closing
import json
import os
import subprocess
import sys
from pathlib import Path
import shutil
import sqlite3
import tempfile
import threading
import unittest

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("tma_f12_security", ROOT/"security-runtime/security.py")
assert SPEC and SPEC.loader
security = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(security)
KEY = b"F12_SYNTHETIC_TEST_KEY_ONLY_" + b"A" * 32
OTHER_KEY = b"F12_SYNTHETIC_TEST_KEY_ONLY_" + b"B" * 32

@contextmanager
def test_db(path: Path):
    with closing(sqlite3.connect(path)) as con:
        with con:
            yield con

def create_db(p: Path) -> None:
    with test_db(p) as conn:
        conn.execute("CREATE TABLE sample (id INTEGER PRIMARY KEY, value TEXT NOT NULL)")
        conn.executemany("INSERT INTO sample(value) VALUES (?)", ((f"test-{i}",) for i in range(12)))

class F12Tests(unittest.TestCase):
    def setUp(self):
        base = Path(os.environ.get("TMA_F12_TEST_ROOT", tempfile.gettempdir()))
        base.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(prefix="tma-f12-test-", dir=base)
        self.root = Path(self.temp.name)
        self.source = self.root / "source.sqlite3"
        create_db(self.source)
    def tearDown(self):
        self.temp.cleanup()
    def backup(self):
        return security.backup_sqlite(
            root=self.root, source=self.source, destination=self.root/"snapshot",
            signing_key=KEY)
    def test_secret_ref_allowed_without_resolving(self):
        security.validate_secret_refs({"secretRefs": ["env://TMA_TEST_ONLY"],
            "mode": "simulation", "nested": [{"secret_refs": []}]})
    def test_password_inline_denied(self):
        with self.assertRaisesRegex(security.SecurityDenied, "INLINE_SECRET"):
            security.validate_secret_refs({"settings": {"password": "synthetic"}})
    def test_api_key_inline_denied(self):
        with self.assertRaises(security.SecurityDenied):
            security.validate_secret_refs({"api_key": "synthetic-only"})
    def test_reference_value_denied(self):
        for ref in ("synthetic_secret_literal", "env://PATH", "env://TMA_x", "file:///secret"):
            with self.subTest(ref=ref), self.assertRaises(security.SecurityDenied):
                security.validate_secret_refs({"secretRefs": [ref]})
    def test_embedded_url_credentials_denied(self):
        with self.assertRaisesRegex(security.SecurityDenied, "USERINFO"):
            security.validate_secret_refs({"url": "https://synthetic:secret@example.invalid/form"})
    def test_audit_hash_chain_append_only(self):
        trail = security.AuditTrail(self.root/"audit.sqlite3")
        for i in range(3):
            self.assertEqual(trail.record(event_id=f"e-{i}", actor_id="worker-1",
                action="mission.checkpoint", outcome="ok", resource_id=f"mission-{i}",
                occurred_ms=100+i), "recorded")
        self.assertEqual(trail.verify(), 3)
        with test_db(self.root/"audit.sqlite3") as conn:
            with self.assertRaises(sqlite3.DatabaseError):
                conn.execute("UPDATE audit_events SET outcome='bad' WHERE id=1")
            with self.assertRaises(sqlite3.DatabaseError):
                conn.execute("DELETE FROM audit_events WHERE id=1")
    def test_audit_idempotent_same_event(self):
        trail = security.AuditTrail(self.root/"audit.sqlite3")
        payload=dict(event_id="e",actor_id="w",action="health",outcome="ok",
                     resource_id="m",occurred_ms=123)
        self.assertEqual(trail.record(**payload),"recorded")
        self.assertEqual(trail.record(**payload),"duplicate")
        self.assertEqual(trail.verify(),1)
    def test_audit_conflicting_event_rejected(self):
        trail=security.AuditTrail(self.root/"audit.sqlite3")
        payload=dict(event_id="e",actor_id="w",action="health",outcome="ok",
                     resource_id="m",occurred_ms=123)
        trail.record(**payload)
        with self.assertRaisesRegex(security.SecurityDenied,"CONFLICT"):
            trail.record(**{**payload,"outcome":"fail"})
    def test_audit_sensitive_metadata_rejected(self):
        trail=security.AuditTrail(self.root/"audit.sqlite3")
        with self.assertRaisesRegex(security.SecurityDenied,"UNSAFE"):
            trail.record(event_id="e",actor_id="api key value",action="read",
                         outcome="ok",resource_id="m",occurred_ms=100)
        self.assertEqual(trail.verify(),0)
    def test_audit_manual_tamper_detected(self):
        trail=security.AuditTrail(self.root/"audit.sqlite3")
        trail.record(event_id="e",actor_id="w",action="read",
                     outcome="ok",resource_id="m",occurred_ms=1)
        with test_db(self.root/"audit.sqlite3") as conn:
            conn.execute("DROP TRIGGER audit_no_update")
            conn.execute("UPDATE audit_events SET outcome='failed' WHERE id=1")
        with self.assertRaisesRegex(security.SecurityDenied,"HASH_MISMATCH"):
            trail.verify()
    def test_audit_16_parallel_event_writers(self):
        trail=security.AuditTrail(self.root/"audit.sqlite3")
        errors=[]
        def writer(index):
            try:
                trail.record(event_id=f"e-{index:02}",actor_id="w",
                             action="checkpoint",outcome="ok",resource_id="mission",
                             occurred_ms=index)
            except Exception as exc:
                errors.append(str(exc))
        workers=[threading.Thread(target=writer,args=(i,)) for i in range(16)]
        for thread in workers:thread.start()
        for thread in workers:thread.join(timeout=10)
        self.assertFalse(errors,errors)
        self.assertEqual(trail.verify(),16)
    def test_online_backup_and_separate_restore(self):
        before=security.sqlite_check(self.source)
        self.assertIsNone(before)
        manifest=self.backup()
        self.assertEqual(manifest["type"],"sqlite-online-snapshot")
        restored=Path(security.restore_sqlite(
            root=self.root,snapshot=self.root/"snapshot",
            target=self.root/"restored",signing_key=KEY))
        self.assertEqual(restored.name,"restored.sqlite3")
        self.assertEqual(restored.read_bytes(),(self.root/"snapshot/database.sqlite3").read_bytes())
        with test_db(restored) as conn:
            self.assertEqual(conn.execute("SELECT COUNT(*) FROM sample").fetchone()[0],12)
    def test_corrupted_source_denied(self):
        self.source.write_bytes(b"not-a-sqlite-database")
        with self.assertRaisesRegex(security.SecurityDenied,"CORRUPT"):
            self.backup()
        self.assertFalse((self.root/"snapshot").exists())
    def test_corrupted_snapshot_hash_denied(self):
        self.backup()
        (self.root/"snapshot/database.sqlite3").write_bytes(b"tampered")
        with self.assertRaisesRegex(security.SecurityDenied,"BACKUP_HASH_INVALID"):
            security.restore_sqlite(root=self.root,snapshot=self.root/"snapshot",
                target=self.root/"restored",signing_key=KEY)
        self.assertFalse((self.root/"restored").exists())
    def test_bad_signature_denied(self):
        self.backup()
        with self.assertRaisesRegex(security.SecurityDenied,"SIGNATURE_INVALID"):
            security.restore_sqlite(root=self.root,snapshot=self.root/"snapshot",
                target=self.root/"restored",signing_key=OTHER_KEY)
    def test_restore_never_overwrites_live_target(self):
        self.backup()
        target=self.root/"live.sqlite3";target.write_bytes(b"do-not-replace")
        with self.assertRaisesRegex(security.SecurityDenied,"TARGET_EXISTS"):
            security.restore_sqlite(root=self.root,snapshot=self.root/"snapshot",
                target=target,signing_key=KEY)
        self.assertEqual(target.read_bytes(),b"do-not-replace")
    def test_backup_never_overwrites_existing_snapshot(self):
        self.backup()
        with self.assertRaisesRegex(security.SecurityDenied,"BACKUP_INVALID_PATH"):
            self.backup()
    def test_backup_outside_root_denied(self):
        external=Path(tempfile.gettempdir())/"outside-test-f12"
        with self.assertRaisesRegex(security.SecurityDenied,"OUTSIDE_ROOT"):
            security.backup_sqlite(root=self.root,source=self.source,destination=external,signing_key=KEY)
    def test_restore_missing_manifest_denied(self):
        folder=self.root/"broken";folder.mkdir()
        with self.assertRaisesRegex(security.SecurityDenied,"MANIFEST_MISSING"):
            security.restore_sqlite(root=self.root,snapshot=folder,
                target=self.root/"restore",signing_key=KEY)
    def test_secret_signing_key_too_short_denied(self):
        with self.assertRaisesRegex(security.SecurityDenied,"KEY_TOO_SHORT"):
            security.backup_sqlite(root=self.root,source=self.source,
                destination=self.root/"new",signing_key=b"weak")
    def test_symlink_backup_refused_when_supported(self):
        link=self.root/"linked.sqlite3"
        try:
            link.symlink_to(self.source)
        except (OSError,NotImplementedError):
            self.skipTest("symlink unavailable without developer privilege")
        with self.assertRaisesRegex(security.SecurityDenied,"SYMLINK"):
            security.backup_sqlite(root=self.root,source=link,
                destination=self.root/"snapshot",signing_key=KEY)
    def test_dependency_inventory_manifest_and_tamper_detection(self):
        fake=self.root/"project";fake.mkdir()
        for rel in security.DEPENDENCY_FILES:
            p=fake/rel;p.parent.mkdir(parents=True,exist_ok=True)
            p.write_text(f"synthetic:{rel}",encoding="utf-8")
        path=fake/"inventory.json"
        result=security.signed_release_inventory(
            project_root=fake,signing_key=KEY,output=path)
        self.assertEqual(len(result),len(security.DEPENDENCY_FILES))
        self.assertEqual(security.verify_release_inventory(
            project_root=fake,signing_key=KEY,inventory_file=path),len(result))
        (fake/security.DEPENDENCY_FILES[0]).write_text("changed")
        with self.assertRaisesRegex(security.SecurityDenied,"DEPENDENCY_DRIFT"):
            security.verify_release_inventory(
                project_root=fake,signing_key=KEY,inventory_file=path)
    def test_inventory_fails_closed_missing_lockfile(self):
        incomplete=self.root/"empty-project";incomplete.mkdir()
        with self.assertRaisesRegex(security.SecurityDenied,"MANIFEST_MISSING"):
            security.dependency_inventory(incomplete)
    def test_dependency_manifest_modified_signature_denied(self):
        fake=self.root/"project";fake.mkdir()
        for rel in security.DEPENDENCY_FILES:
            p=fake/rel;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(rel)
        manifest=fake/"inventory.json"
        security.signed_release_inventory(project_root=fake,signing_key=KEY,output=manifest)
        changed=json.loads(manifest.read_text())
        changed["manifest"]["files"]["core-rust/Cargo.lock"]="0"*64
        manifest.write_text(json.dumps(changed))
        with self.assertRaisesRegex(security.SecurityDenied,"RELEASE_SIGNATURE_INVALID"):
            security.verify_release_inventory(project_root=fake,signing_key=KEY,inventory_file=manifest)
    def test_database_failure_dependency_loss(self):
        self.source.unlink()
        with self.assertRaisesRegex(security.SecurityDenied,"REQUIRED_PATH_MISSING"):
            self.backup()
    def test_weak_audit_identity_rejected(self):
        for identity in ("../../windows", "a b", "synthetic\nvalue", "a"*121):
            with self.subTest(identity=repr(identity)):
                with self.assertRaises(security.SecurityDenied):
                    security.safe_identity(identity)
    def test_audit_rejects_token_looking_identifier(self):
        for sample in ("sk-"+"A"*24, "ghp_"+"B"*24):
            with self.assertRaises(security.SecurityDenied):
                security.safe_identity(sample)

    def _cli(self, *arguments, env=None):
        cmd=[sys.executable,str(ROOT/"security-runtime/cli.py"),*map(str,arguments)]
        return subprocess.run(cmd,capture_output=True,text=True,timeout=30,env=env)

    def test_cli_config_accepts_refs_and_rejects_inline_password(self):
        config=self.root/"policy.json"
        config.write_text('{"secretRefs":["env://TMA_SYNTHETIC_REF"],"mode":"simulation"}')
        result=self._cli("validate-config","--root",self.root,"--config",config)
        self.assertEqual(result.returncode,0,result.stderr)
        self.assertIn("F12_SECRET_POLICY=PASS",result.stdout)
        config.write_text('{"password":"test-secret-should-not-leak"}')
        denied=self._cli("validate-config","--root",self.root,"--config",config)
        self.assertEqual(denied.returncode,1)
        self.assertIn("F12_OPERATION=DENIED",denied.stderr)
        self.assertNotIn("test-secret-should-not-leak",denied.stderr)

    def test_cli_backup_without_key_fails_closed(self):
        env=os.environ.copy()
        env.pop("TMA_F12_SIGNING_KEY_HEX",None)
        result=self._cli("backup","--root",self.root,"--source",self.source,
                         "--destination",self.root/"backup",env=env)
        self.assertEqual(result.returncode,1)
        self.assertIn("SIGNING_KEY_NOT_CONFIGURED",result.stderr)
        self.assertFalse((self.root/"backup").exists())

    def test_cli_signed_backup_and_restore(self):
        env=os.environ.copy()
        env["TMA_F12_SIGNING_KEY_HEX"]=KEY.hex()
        backup=self._cli("backup","--root",self.root,"--source",self.source,
                         "--destination",self.root/"cli-backup",env=env)
        self.assertEqual(backup.returncode,0,backup.stderr)
        self.assertIn("F12_BACKUP=PASS",backup.stdout)
        restored=self._cli("restore","--root",self.root,"--snapshot",self.root/"cli-backup",
                           "--destination",self.root/"cli-restored",env=env)
        self.assertEqual(restored.returncode,0,restored.stderr)
        self.assertIn("F12_RESTORE=PASS",restored.stdout)
        self.assertTrue((self.root/"cli-restored/restored.sqlite3").is_file())

    def test_cli_audit_record_and_verify(self):
        db=self.root/"cli-audit.sqlite3"
        args=["audit-record","--root",str(self.root),"--db",str(db),
              "--event","e1","--actor","worker-1","--operation","recovery",
              "--outcome","ok","--resource","m1","--at","100"]
        result=self._cli(*args)
        self.assertEqual(result.returncode,0,result.stderr)
        duplicate=self._cli(*args)
        self.assertEqual(duplicate.returncode,0,duplicate.stderr)
        self.assertIn("F12_AUDIT_RECORD=duplicate",duplicate.stdout)
        proof=self._cli("audit-verify","--root",self.root,"--db",db)
        self.assertEqual(proof.returncode,0,proof.stderr)
        self.assertIn("events=1",proof.stdout)

    def test_acl_private_directory_smoke(self):
        destination=self.root/"private-only"
        security.create_private_dir(destination)
        self.assertTrue(destination.is_dir())
        if os.name=="nt":
            result = __import__("subprocess").run(["icacls",str(destination)],text=True,
                capture_output=True,check=True)
            self.assertNotIn("Everyone:(F)",result.stdout)
            self.assertNotIn("BUILTIN\\Users:(F)",result.stdout)

if __name__=="__main__":
    unittest.main(verbosity=2)
