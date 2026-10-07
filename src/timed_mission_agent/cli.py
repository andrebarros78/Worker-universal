from __future__ import annotations

import argparse
import json
from pathlib import Path
from tempfile import TemporaryDirectory

from .extractors import TextInvoiceExtractor
from .ledger import EvidenceLedger
from .models import MissionTask
from .runner import TimedMissionRunner


def _simulate(args: argparse.Namespace) -> int:
    payload = Path(args.input).read_text(encoding="utf-8")
    runner = TimedMissionRunner(TextInvoiceExtractor(), EvidenceLedger(Path(args.ledger)))
    result = runner.run(
        MissionTask(args.task_id, args.mission_id, payload, args.deadline_ms)
    )
    print(json.dumps(
        {
            "status": result.status.value,
            "elapsed_ms": round(result.elapsed_ms, 3),
            "attempts": len(result.attempts),
            "confidence": round(result.confidence, 4),
            "fields": result.fields,
            "errors": result.errors,
        },
        ensure_ascii=False,
        indent=2,
        default=str,
    ))
    return 0 if result.status.value == "succeeded" else 2


def _self_test(_: argparse.Namespace) -> int:
    with TemporaryDirectory() as tmp:
        sample = "\n".join([
            "CNPJ: 04.252.011/0001-10",
            "Numero: 1001",
            "Data: 07/10/2026",
            "Subtotal: 120,00",
            "Desconto: 20,00",
            "Total: 100,00",
        ])
        runner = TimedMissionRunner(
            TextInvoiceExtractor(),
            EvidenceLedger(Path(tmp) / "evidence.sqlite3"),
        )
        result = runner.run(MissionTask("selftest", "selftest", sample, 1000))
        print(result.status.value)
        return 0 if result.status.value == "succeeded" else 1


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="timed-mission-agent")
    sub = parser.add_subparsers(dest="command", required=True)

    sim = sub.add_parser("simulate")
    sim.add_argument("--input", required=True)
    sim.add_argument("--deadline-ms", type=int, default=2000)
    sim.add_argument("--ledger", default="runtime/evidence.sqlite3")
    sim.add_argument("--task-id", default="invoice-001")
    sim.add_argument("--mission-id", default="simulation")
    sim.set_defaults(func=_simulate)

    self_test = sub.add_parser("self-test")
    self_test.set_defaults(func=_self_test)
    return parser


def main() -> int:
    args = build_parser().parse_args()
    return int(args.func(args))


if __name__ == "__main__":
    raise SystemExit(main())
