import unittest
from decimal import Decimal

from timed_mission_agent.validators import parse_decimal, validate_cnpj, validate_invoice


class ValidatorTests(unittest.TestCase):
    def test_valid_cnpj(self):
        self.assertTrue(validate_cnpj("04.252.011/0001-10"))

    def test_invalid_cnpj(self):
        self.assertFalse(validate_cnpj("11.111.111/1111-11"))

    def test_brazilian_decimal(self):
        self.assertEqual(parse_decimal("1.234,56"), Decimal("1234.56"))

    def test_invoice_arithmetic(self):
        result = validate_invoice({
            "cnpj": "04.252.011/0001-10",
            "numero": "1",
            "data_emissao": "07/10/2026",
            "subtotal": "100,00",
            "desconto": "10,00",
            "total": "90,00",
        })
        self.assertTrue(result.ok, result.errors)

    def test_invoice_mismatch(self):
        result = validate_invoice({
            "cnpj": "04.252.011/0001-10",
            "numero": "1",
            "data_emissao": "07/10/2026",
            "subtotal": "100,00",
            "total": "80,00",
        })
        self.assertFalse(result.ok)
        self.assertIn("mismatch:total", result.errors)


if __name__ == "__main__":
    unittest.main()
