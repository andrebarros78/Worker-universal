import tempfile
import unittest
from pathlib import Path

from timed_mission_agent.benchmark import build_report
from timed_mission_agent.extractors import StaticExtractor
from timed_mission_agent.ledger import EvidenceLedger
from timed_mission_agent.models import Extraction, MissionTask, TaskStatus
from timed_mission_agent.orchestrator import MissionOrchestrator
from timed_mission_agent.runner import TimedMissionRunner


FIELDS = {
    "cnpj": "04.252.011/0001-10",
    "numero": "55",
    "data_emissao": "07/10/2026",
    "subtotal": "100,00",
    "total": "100,00",
}


class OrchestratorTests(unittest.TestCase):
    def test_parallel_and_benchmark(self):
        with tempfile.TemporaryDirectory() as tmp:
            ledger_path = Path(tmp) / "ledger.sqlite3"

            def factory():
                extraction = Extraction(
                    fields=dict(FIELDS),
                    confidence={
                        "cnpj": 0.99,
                        "numero": 0.99,
                        "data_emissao": 0.99,
                        "total": 0.99,
                    },
                )
                return TimedMissionRunner(
                    StaticExtractor(extraction, delay_ms=15),
                    EvidenceLedger(ledger_path),
                )

            tasks = [
                MissionTask(f"t{i}", "m-batch", {}, 500, expected_fields=FIELDS)
                for i in range(8)
            ]
            results = MissionOrchestrator(factory, max_workers=4).run_many(tasks)
            self.assertEqual(len(results), 8)
            self.assertTrue(all(r.status == TaskStatus.SUCCEEDED for r in results))
            report = build_report(tasks, results)
            self.assertEqual(report.field_accuracy, 1.0)
            self.assertEqual(report.success_rate, 1.0)
            self.assertEqual(report.sla_pass_rate, 1.0)
            self.assertGreater(report.p95_ms, 0.0)
            self.assertLess(report.p95_ms, 500.0)


if __name__ == "__main__":
    unittest.main()
