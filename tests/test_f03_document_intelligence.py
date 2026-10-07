from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from PIL import Image

from timed_mission_agent.document_intelligence import (
    BinnedConfidenceCalibrator,
    CandidateReconciler,
    FieldCandidate,
    InvoiceDocumentEngine,
    expected_calibration_error,
)
from timed_mission_agent.ocr import OCRLine, StaticOCRProvider
from timed_mission_agent.vision import ImageIngestor
from timed_mission_agent.f03_adapter import TesseractCapabilityAdapter


class F03DocumentIntelligenceTests(unittest.TestCase):
    def test_image_ingestion_hashes_and_reads_dimensions(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "sample.png"
            Image.new("L", (120, 80), 255).save(path)
            asset = ImageIngestor().ingest(path)
            self.assertEqual(asset.width, 120)
            self.assertEqual(asset.height, 80)
            self.assertEqual(len(asset.sha256), 64)

    def test_reconciler_prefers_multi_provider_consensus(self):
        candidates = [
            FieldCandidate("numero", "123", 0.90, "a", "default"),
            FieldCandidate("numero", "123", 0.88, "b", "default"),
            FieldCandidate("numero", "999", 0.99, "c", "default"),
        ]
        fields, confidence, evidence = CandidateReconciler().reconcile(candidates)
        self.assertEqual(fields["numero"], "123")
        self.assertGreater(confidence["numero"], 0.85)
        self.assertEqual(len(evidence["numero"]), 2)

    def test_reconciler_normalizes_equivalent_money_values(self):
        candidates = [
            FieldCandidate("total", "123,45", 0.90, "a", "default"),
            FieldCandidate("total", "123.45", 0.88, "b", "default"),
        ]
        fields, _, evidence = CandidateReconciler().reconcile(candidates)
        self.assertIn(fields["total"], {"123,45", "123.45"})
        self.assertEqual(len(evidence["total"]), 2)

    def test_selective_reread_repairs_low_confidence_field(self):
        default_lines = (
            OCRLine("CNPJ: 11.222.333/0001-81", 0.99),
            OCRLine("Numero: 100001", 0.99),
            OCRLine("Data: 01/10/2026", 0.99),
            OCRLine("Subtotal: 100,00", 0.99),
            OCRLine("Desconto: 0,00", 0.99),
            OCRLine("Acrescimos: 0,00", 0.99),
            OCRLine("Total: 10,00", 0.40),
        )
        reread_lines = (
            OCRLine("Total: 100,00", 0.99),
        )
        provider = StaticOCRProvider("static", default_lines, reread_lines)
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "sample.png"
            Image.new("L", (120, 80), 255).save(path)
            extraction = InvoiceDocumentEngine(
                [provider],
                reread_threshold=0.80,
            ).extract(path)
        self.assertEqual(extraction.fields["total"], "100,00")
        self.assertTrue(extraction.raw["validation"]["ok"])
        self.assertTrue(
            any(run["profile"] == "reread" for run in extraction.raw["providers"])
        )

    def test_validation_failure_triggers_reread_even_when_confidence_is_high(self):
        default_lines = (
            OCRLine("CNPJ: 11.222.333/0001-81", 0.99),
            OCRLine("Numero: 100001", 0.99),
            OCRLine("Data: 01/10/2026", 0.99),
            OCRLine("Subtotal: 100,00", 0.99),
            OCRLine("Desconto: @,00", 0.99),
            OCRLine("Acrescimos: 0,00", 0.99),
            OCRLine("Total: 100,00", 0.99),
        )
        reread_lines = (OCRLine("Desconto: 0,00", 0.81),)
        provider = StaticOCRProvider("static", default_lines, reread_lines)
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "sample.png"
            Image.new("L", (120, 80), 255).save(path)
            extraction = InvoiceDocumentEngine(
                [provider],
                reread_threshold=0.80,
            ).extract(path)
        self.assertEqual(extraction.fields["desconto"], "0,00")
        self.assertTrue(extraction.raw["validation"]["ok"])
        self.assertTrue(
            any(run["profile"] == "reread" for run in extraction.raw["providers"])
        )

    def test_unavailable_provider_falls_back_to_available_provider(self):
        class Unavailable:
            provider_id = "unavailable"

            def available(self):
                return False

            def read(self, image_path, profile="default"):
                raise AssertionError("must not be called")

        lines = (
            OCRLine("CNPJ: 11.222.333/0001-81", 0.99),
            OCRLine("Numero: 1", 0.99),
            OCRLine("Data: 01/10/2026", 0.99),
            OCRLine("Subtotal: 10,00", 0.99),
            OCRLine("Desconto: 0,00", 0.99),
            OCRLine("Acrescimos: 0,00", 0.99),
            OCRLine("Total: 10,00", 0.99),
        )
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "sample.png"
            Image.new("L", (120, 80), 255).save(path)
            extraction = InvoiceDocumentEngine(
                [Unavailable(), StaticOCRProvider("fallback", lines)]
            ).extract(path)
        self.assertEqual(extraction.fields["numero"], "1")

    def test_invalid_arithmetic_reduces_total_confidence(self):
        lines = (
            OCRLine("CNPJ: 11.222.333/0001-81", 0.99),
            OCRLine("Numero: 1", 0.99),
            OCRLine("Data: 01/10/2026", 0.99),
            OCRLine("Subtotal: 100,00", 0.99),
            OCRLine("Desconto: 0,00", 0.99),
            OCRLine("Acrescimos: 0,00", 0.99),
            OCRLine("Total: 50,00", 0.99),
        )
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "sample.png"
            Image.new("L", (120, 80), 255).save(path)
            extraction = InvoiceDocumentEngine(
                [StaticOCRProvider("static", lines)]
            ).extract(path)
        self.assertIn("mismatch:total", extraction.raw["validation"]["errors"])
        self.assertLessEqual(extraction.confidence["total"], 0.49)

    def test_calibrator_is_monotonic(self):
        calibrator = BinnedConfidenceCalibrator.fit(
            [(0.1, False), (0.2, False), (0.8, True), (0.9, True)],
            bins=4,
        )
        values = [calibrator.calibrate(value) for value in (0.1, 0.3, 0.6, 0.9)]
        self.assertEqual(values, sorted(values))

    def test_tesseract_capability_adapter_runs_real_corpus_case(self):
        root = Path(__file__).resolve().parents[1]
        image = root / "benchmarks" / "f03_corpus" / "invoice-001-clean.png"
        if not image.is_file():
            self.skipTest("F03 corpus has not been generated yet")
        adapter = TesseractCapabilityAdapter()
        health = adapter.health()
        self.assertEqual(health["lifecycle"], "healthy")
        manifest = adapter.manifest()
        self.assertIn("vision.ocr.tesseract", manifest["capabilities"])
        result = adapter.invoke(image)
        self.assertEqual(result["status"], "succeeded")
        self.assertEqual(result["fields"]["numero"], "100001")

    def test_ece_is_zero_for_perfect_certainty(self):
        self.assertEqual(
            expected_calibration_error([(1.0, True), (0.0, False)], bins=2),
            0.0,
        )


if __name__ == "__main__":
    unittest.main()
