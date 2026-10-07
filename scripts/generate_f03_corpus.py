from __future__ import annotations

import json
from decimal import Decimal
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "benchmarks" / "f03_corpus"
FONT_PATH = Path(r"C:\Windows\Fonts\consola.ttf")


def cnpj_from_base(base: str) -> str:
    if len(base) != 12 or not base.isdigit():
        raise ValueError("base must contain 12 digits")

    def digit(value: str, weights: list[int]) -> str:
        total = sum(int(number) * weight for number, weight in zip(value, weights, strict=True))
        remainder = total % 11
        return "0" if remainder < 2 else str(11 - remainder)

    first = digit(base, [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2])
    second = digit(base + first, [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2])
    return base + first + second


def format_cnpj(value: str) -> str:
    return f"{value[:2]}.{value[2:5]}.{value[5:8]}/{value[8:12]}-{value[12:]}"


def money(value: Decimal) -> str:
    return f"{value:.2f}".replace(".", ",")


def render(case_id: str, fields: dict[str, str], variant: str, destination: Path) -> None:
    background = 255
    foreground = 0
    if variant == "low_contrast":
        background = 245
        foreground = 60

    image = Image.new("L", (1180, 720), background)
    draw = ImageDraw.Draw(image)
    font = ImageFont.truetype(str(FONT_PATH), 36)
    title_font = ImageFont.truetype(str(FONT_PATH), 42)

    draw.text((60, 45), "DOCUMENTO FISCAL - TESTE SINTETICO", font=title_font, fill=foreground)
    lines = [
        f"CNPJ: {fields['cnpj']}",
        f"Numero: {fields['numero']}",
        f"Data: {fields['data_emissao']}",
        f"Subtotal: {fields['subtotal']}",
        f"Desconto: {fields['desconto']}",
        f"Acrescimos: {fields['acrescimos']}",
        f"Total: {fields['total']}",
    ]
    y = 135
    for line in lines:
        draw.text((80, y), line, font=font, fill=foreground)
        y += 72

    if variant == "mild_blur":
        image = image.filter(ImageFilter.GaussianBlur(radius=0.45))
    elif variant == "compressed":
        temp = destination.with_suffix(".jpg")
        image.save(temp, "JPEG", quality=58, optimize=False)
        with Image.open(temp) as compressed:
            image = compressed.convert("L")
        temp.unlink()

    image.save(destination, "PNG")


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    cases: list[dict[str, object]] = []
    variants = ("clean", "low_contrast", "mild_blur", "compressed")

    for index in range(1, 17):
        base = f"73925184{index:04d}"
        cnpj = cnpj_from_base(base)
        subtotal = Decimal("100.00") + Decimal(index) * Decimal("13.25")
        desconto = Decimal(index % 4) * Decimal("2.50")
        acrescimos = Decimal(index % 3) * Decimal("1.75")
        total = subtotal - desconto + acrescimos
        fields = {
            "cnpj": format_cnpj(cnpj),
            "numero": f"{100000 + index}",
            "data_emissao": f"{index:02d}/10/2026",
            "subtotal": money(subtotal),
            "desconto": money(desconto),
            "acrescimos": money(acrescimos),
            "total": money(total),
        }
        case_id = f"invoice-{index:03d}"
        variant = variants[(index - 1) % len(variants)]
        filename = f"{case_id}-{variant}.png"
        render(case_id, fields, variant, OUT / filename)
        cases.append(
            {
                "id": case_id,
                "file": filename,
                "variant": variant,
                "expected_fields": fields,
            }
        )

    manifest = {
        "schema_version": 1,
        "dataset": "tma-f03-synthetic-invoices",
        "version": "1.0.0",
        "generator": "scripts/generate_f03_corpus.py",
        "license": "project-generated synthetic data",
        "cases": cases,
    }
    (OUT / "manifest.json").write_text(
        json.dumps(manifest, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    (OUT / "VERSION").write_text("1.0.0\n", encoding="utf-8")
    print(f"F03_CORPUS_GENERATED cases={len(cases)} version=1.0.0")


if __name__ == "__main__":
    main()
