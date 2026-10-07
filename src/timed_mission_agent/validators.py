from __future__ import annotations

from datetime import datetime
from decimal import Decimal, InvalidOperation

from .models import ValidationResult


def _digits(value: object) -> str:
    return "".join(ch for ch in str(value) if ch.isdigit())


def validate_cnpj(value: object) -> bool:
    digits = _digits(value)
    if len(digits) != 14 or digits == digits[0] * 14:
        return False

    def calc(base: str, weights: list[int]) -> str:
        total = sum(int(d) * w for d, w in zip(base, weights, strict=True))
        rem = total % 11
        return "0" if rem < 2 else str(11 - rem)

    d1 = calc(digits[:12], [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2])
    d2 = calc(digits[:12] + d1, [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2])
    return digits[-2:] == d1 + d2


def parse_decimal(value: object) -> Decimal:
    text = str(value).strip().replace("R$", "").replace(" ", "")
    if "," in text and "." in text:
        text = text.replace(".", "").replace(",", ".")
    elif "," in text:
        text = text.replace(",", ".")
    return Decimal(text)


def normalize_field(key: str, value: object) -> object:
    if key == "cnpj":
        return _digits(value)
    if key in {"subtotal", "desconto", "acrescimos", "total"}:
        return parse_decimal(value).quantize(Decimal("0.01"))
    if key == "data_emissao":
        text = str(value).strip()
        for fmt in ("%d/%m/%Y", "%Y-%m-%d"):
            try:
                return datetime.strptime(text, fmt).date().isoformat()
            except ValueError:
                pass
        return text
    return str(value).strip()


def validate_invoice(fields: dict[str, object], tolerance: Decimal = Decimal("0.02")) -> ValidationResult:
    errors: list[str] = []
    for key in ("cnpj", "numero", "data_emissao", "total"):
        if key not in fields or str(fields[key]).strip() == "":
            errors.append(f"missing:{key}")

    if "cnpj" in fields and not validate_cnpj(fields["cnpj"]):
        errors.append("invalid:cnpj")

    if "data_emissao" in fields:
        normalized = normalize_field("data_emissao", fields["data_emissao"])
        try:
            datetime.strptime(str(normalized), "%Y-%m-%d")
        except ValueError:
            errors.append("invalid:data_emissao")

    numeric: dict[str, Decimal] = {}
    for key in ("subtotal", "desconto", "acrescimos", "total"):
        if key not in fields:
            continue
        try:
            numeric[key] = parse_decimal(fields[key])
        except (InvalidOperation, ValueError):
            errors.append(f"invalid:{key}")

    if "subtotal" in numeric and "total" in numeric:
        expected = numeric["subtotal"] - numeric.get("desconto", Decimal("0")) + numeric.get("acrescimos", Decimal("0"))
        if abs(expected - numeric["total"]) > tolerance:
            errors.append("mismatch:total")

    return ValidationResult(ok=not errors, errors=errors)
