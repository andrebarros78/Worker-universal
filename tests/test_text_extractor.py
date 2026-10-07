import unittest

from timed_mission_agent.extractors import TextInvoiceExtractor


class TextExtractorTests(unittest.TestCase):
    def test_extracts_known_labels(self):
        sample = """
        CNPJ: 04.252.011/0001-10
        Número: 1234
        Data: 07/10/2026
        Total: 99,90
        """
        result = TextInvoiceExtractor().extract(sample)
        self.assertEqual(result.fields["numero"], "1234")
        self.assertEqual(result.fields["total"], "99,90")


if __name__ == "__main__":
    unittest.main()
