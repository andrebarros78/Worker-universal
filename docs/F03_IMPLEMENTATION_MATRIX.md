# F03 Implementation Matrix

| Requirements | Implementation | Proof |
|---|---|---|
| VIS-001..004 | `vision.py` | image unit tests + benchmark |
| VIS-005..008 | `ocr.py` | provider purity/version + OCR benchmark |
| VIS-009..014 | `document_intelligence.py` | 9 focused tests + benchmark |
| VIS-015..019 | `validators.py` + document engine | validator regression + confidence penalty |
| VIS-020..021 | provider list/fallback + purity scanner | fallback test + source gate |
| VIS-022..023 | `benchmarks/f03_corpus/` | manifest and variant gate |
| VIS-024..030 | `run_f03_benchmark.py` | report thresholds |
| VIS-031..032 | F02 capability/adapter JSON fixtures | F03 contract gate |
| VIS-033 | local Tesseract + Pillow | benchmark with no remote provider |
| VIS-034 | `generate_f03_corpus.py` | deterministic regeneration gate |
| VIS-035 | `scripts/verify_f03.ps1` | F03_VERIFY=PASS |
