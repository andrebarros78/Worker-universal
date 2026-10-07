from __future__ import annotations

import json
import sqlite3
import threading
from contextlib import closing
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


class EvidenceLedger:
    def __init__(self, path: str | Path) -> None:
        self.path = str(path)
        self._lock = threading.Lock()
        self._initialize()

    def _connect(self) -> sqlite3.Connection:
        return sqlite3.connect(self.path, timeout=10)

    def _initialize(self) -> None:
        Path(self.path).parent.mkdir(parents=True, exist_ok=True)
        with closing(self._connect()) as conn:
            conn.execute("PRAGMA journal_mode=WAL")
            conn.execute("PRAGMA synchronous=NORMAL")
            conn.execute(
                """
                CREATE TABLE IF NOT EXISTS events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    ts TEXT NOT NULL,
                    mission_id TEXT NOT NULL,
                    task_id TEXT NOT NULL,
                    event TEXT NOT NULL,
                    payload_json TEXT NOT NULL
                )
                """
            )
            conn.commit()

    def record(self, mission_id: str, task_id: str, event: str, payload: dict[str, Any]) -> None:
        ts = datetime.now(timezone.utc).isoformat()
        data = json.dumps(payload, ensure_ascii=False, sort_keys=True, default=str)
        with self._lock, closing(self._connect()) as conn:
            conn.execute(
                "INSERT INTO events(ts, mission_id, task_id, event, payload_json) VALUES (?, ?, ?, ?, ?)",
                (ts, mission_id, task_id, event, data),
            )
            conn.commit()

    def events(self, mission_id: str, task_id: str) -> list[dict[str, Any]]:
        with closing(self._connect()) as conn:
            rows = conn.execute(
                "SELECT ts,event,payload_json FROM events WHERE mission_id=? AND task_id=? ORDER BY id",
                (mission_id, task_id),
            ).fetchall()
        return [{"ts": ts, "event": event, "payload": json.loads(payload)} for ts, event, payload in rows]
