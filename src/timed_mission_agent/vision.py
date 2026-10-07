from __future__ import annotations

import hashlib
from dataclasses import dataclass
from pathlib import Path

from PIL import Image, ImageEnhance, ImageFilter, ImageOps


@dataclass(frozen=True, slots=True)
class ImageAsset:
    path: Path
    sha256: str
    width: int
    height: int
    mode: str


class ImageIngestor:
    def __init__(self, max_bytes: int = 20 * 1024 * 1024) -> None:
        self.max_bytes = max_bytes

    def ingest(self, payload: object) -> ImageAsset:
        path = Path(payload) if isinstance(payload, (str, Path)) else None
        if path is None or not path.is_file():
            raise ValueError("image payload must be an existing file")
        size = path.stat().st_size
        if size <= 0:
            raise ValueError("image file is empty")
        if size > self.max_bytes:
            raise ValueError("image exceeds configured size limit")
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        with Image.open(path) as image:
            image.verify()
        with Image.open(path) as image:
            width, height = image.size
            mode = image.mode
        if width <= 0 or height <= 0:
            raise ValueError("invalid image dimensions")
        return ImageAsset(path=path, sha256=digest, width=width, height=height, mode=mode)


def preprocess_image(source: Path, destination: Path, profile: str = "default") -> Path:
    destination.parent.mkdir(parents=True, exist_ok=True)
    with Image.open(source) as raw:
        image = ImageOps.exif_transpose(raw).convert("L")
        image = ImageOps.autocontrast(image)

        if profile == "reread":
            scale = 3
            image = image.resize(
                (image.width * scale, image.height * scale),
                Image.Resampling.LANCZOS,
            )
            image = ImageEnhance.Contrast(image).enhance(1.35)
            image = image.filter(ImageFilter.SHARPEN)
            image = image.point(lambda value: 255 if value > 180 else 0)
        else:
            scale = 2
            image = image.resize(
                (image.width * scale, image.height * scale),
                Image.Resampling.LANCZOS,
            )
            image = ImageEnhance.Contrast(image).enhance(1.15)

        image.save(destination, format="PNG")
    return destination
