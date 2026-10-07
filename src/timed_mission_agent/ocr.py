from __future__ import annotations

import csv
import io
import shutil
import subprocess
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Protocol

from .vision import preprocess_image


@dataclass(frozen=True, slots=True)
class OCRLine:
    text: str
    confidence: float


@dataclass(frozen=True, slots=True)
class OCRResult:
    provider: str
    profile: str
    lines: tuple[OCRLine, ...]
    elapsed_ms: float


class OCRProvider(Protocol):
    @property
    def provider_id(self) -> str: ...

    def available(self) -> bool: ...

    def read(self, image_path: Path, profile: str = "default") -> OCRResult: ...


class RemoteVisionProvider(Protocol):
    @property
    def provider_id(self) -> str: ...

    def available(self) -> bool: ...

    def read(self, image_path: Path, profile: str = "default") -> OCRResult: ...


class TesseractOCRProvider:
    DEFAULT_WINDOWS_PATH = Path(r"C:\Program Files\Tesseract-OCR\tesseract.exe")

    def __init__(self, executable: str | Path | None = None, language: str = "eng") -> None:
        resolved = Path(executable) if executable else self._resolve_executable()
        self.executable = resolved
        self.language = language

    @property
    def provider_id(self) -> str:
        return "vision.ocr.tesseract"

    @classmethod
    def _resolve_executable(cls) -> Path:
        from_path = shutil.which("tesseract")
        if from_path:
            return Path(from_path)
        return cls.DEFAULT_WINDOWS_PATH

    def available(self) -> bool:
        return self.executable.is_file()

    def version(self) -> str:
        if not self.available():
            return "unavailable"
        completed = subprocess.run(
            [str(self.executable), "--version"],
            capture_output=True,
            text=True,
            check=False,
            timeout=10,
        )
        first = (completed.stdout or completed.stderr).splitlines()
        return first[0].strip() if first else "unknown"

    def read(self, image_path: Path, profile: str = "default") -> OCRResult:
        if not self.available():
            raise RuntimeError(f"tesseract unavailable: {self.executable}")
        if profile not in {"default", "reread"}:
            raise ValueError(f"unsupported OCR profile: {profile}")

        started = time.perf_counter_ns()
        with tempfile.TemporaryDirectory(prefix="tma-ocr-") as tmp:
            prepared = Path(tmp) / "prepared.png"
            preprocess_image(image_path, prepared, profile=profile)
            psm = "6" if profile == "default" else "11"
            completed = subprocess.run(
                [
                    str(self.executable),
                    str(prepared),
                    "stdout",
                    "-l",
                    self.language,
                    "--psm",
                    psm,
                    "tsv",
                ],
                capture_output=True,
                text=True,
                check=False,
                timeout=30,
                encoding="utf-8",
                errors="replace",
            )
            if completed.returncode != 0:
                raise RuntimeError(
                    "tesseract failed: "
                    + (completed.stderr.strip() or f"exit={completed.returncode}")
                )
            lines = _parse_tsv_lines(completed.stdout)

        elapsed = (time.perf_counter_ns() - started) / 1_000_000
        return OCRResult(
            provider=self.provider_id,
            profile=profile,
            lines=tuple(lines),
            elapsed_ms=elapsed,
        )


def _parse_tsv_lines(tsv_text: str) -> list[OCRLine]:
    reader = csv.DictReader(io.StringIO(tsv_text), delimiter="\t")
    grouped: dict[tuple[str, str, str, str], list[tuple[str, float]]] = {}
    for row in reader:
        text = (row.get("text") or "").strip()
        if not text:
            continue
        try:
            confidence = float(row.get("conf") or "-1")
        except ValueError:
            confidence = -1.0
        if confidence < 0:
            continue
        key = (
            row.get("page_num") or "0",
            row.get("block_num") or "0",
            row.get("par_num") or "0",
            row.get("line_num") or "0",
        )
        grouped.setdefault(key, []).append((text, confidence))

    lines: list[OCRLine] = []
    for words in grouped.values():
        line_text = " ".join(word for word, _ in words).strip()
        if not line_text:
            continue
        confidence = sum(conf for _, conf in words) / (100.0 * len(words))
        lines.append(OCRLine(text=line_text, confidence=max(0.0, min(1.0, confidence))))
    return lines


class StaticOCRProvider:
    def __init__(
        self,
        provider_id: str,
        default_lines: tuple[OCRLine, ...],
        reread_lines: tuple[OCRLine, ...] | None = None,
    ) -> None:
        self._provider_id = provider_id
        self.default_lines = default_lines
        self.reread_lines = reread_lines if reread_lines is not None else default_lines

    @property
    def provider_id(self) -> str:
        return self._provider_id

    def available(self) -> bool:
        return True

    def read(self, image_path: Path, profile: str = "default") -> OCRResult:
        lines = self.reread_lines if profile == "reread" else self.default_lines
        return OCRResult(
            provider=self.provider_id,
            profile=profile,
            lines=lines,
            elapsed_ms=0.0,
        )
