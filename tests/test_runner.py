import tempfile
import unittest
from pathlib import Path

from timed_mission_agent.extractors import StaticExtractor
from timed_mission_agent.ledger import EvidenceLedger
from timed_mission_agent.models import Extraction, MissionTask, TaskStatus
from timed_mission_agent.runner import RunnerConfig, TimedMissionRunner


GOOD = Extraction(
    fields={
        "cnpj": "04.252.011/0001-10",
        "numero": "10",
        "data_emissao": "07/10/2026",
        "subtotal": "100,00",
        "desconto": "10,00",
        "total": "90,00",
    },
    confidence={"cnpj": 0.99, "numero": 0.99, "data_emissao": 0.99, "total": 0.99},
)


class RunnerTests(unittest.TestCase):
    def ledger(self, tmp: str) -> EvidenceLedger:
        return EvidenceLedger(Path(tmp) / "ledger.sqlite3")

    def test_success_and_evidence(self):
        with tempfile.TemporaryDirectory() as tmp:
            ledger = self.ledger(tmp)
            runner = TimedMissionRunner(StaticExtractor(GOOD), ledger)
            result = runner.run(MissionTask("t1", "m1", {}, 500))
            self.assertEqual(result.status, TaskStatus.SUCCEEDED)
            events = ledger.events("m1", "t1")
            self.assertEqual(events[0]["event"], "task_started")
            self.assertEqual(events[-1]["event"], "task_finished")

    def test_retry_recovers(self):
        with tempfile.TemporaryDirectory() as tmp:
            runner = TimedMissionRunner(StaticExtractor(GOOD, fail_times=1), self.ledger(tmp))
            result = runner.run(MissionTask("t2", "m1", {}, 500))
            self.assertEqual(result.status, TaskStatus.SUCCEEDED)
            self.assertEqual(len(result.attempts), 2)

    def test_deadline_exceeded(self):
        with tempfile.TemporaryDirectory() as tmp:
            runner = TimedMissionRunner(
                StaticExtractor(GOOD, delay_ms=40),
                self.ledger(tmp),
                RunnerConfig(max_attempts=1),
            )
            result = runner.run(MissionTask("t3", "m1", {}, 10))
            self.assertEqual(result.status, TaskStatus.DEADLINE_EXCEEDED)

    def test_confidence_gate(self):
        low = Extraction(fields=GOOD.fields, confidence={k: 0.3 for k in GOOD.confidence})
        with tempfile.TemporaryDirectory() as tmp:
            runner = TimedMissionRunner(
                StaticExtractor(low),
                self.ledger(tmp),
                RunnerConfig(max_attempts=1),
            )
            result = runner.run(MissionTask("t4", "m1", {}, 500))
            self.assertEqual(result.status, TaskStatus.FAILED)
            self.assertIn("confidence_below_threshold", result.errors)


if __name__ == "__main__":
    unittest.main()
