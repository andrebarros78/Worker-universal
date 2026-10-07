# F03 — Vision/OCR & Document Intelligence Complete Map

Phase authority: F03
Status: IN_PROGRESS

Goal: make timed document work measurable, provider-neutral, recoverable at field level and guarded by deterministic validation.

| ID | Requirement | Deliverable | Proof |
|---|---|---|---|
| VIS-001 | Image ingestion | validated image path/size/hash/dimensions | unit test |
| VIS-002 | Image normalization | EXIF transpose + grayscale + autocontrast | unit/benchmark path |
| VIS-003 | Default preprocessing | 2x resampling + contrast | benchmark |
| VIS-004 | Selective reread preprocessing | 3x + sharpen + threshold | benchmark |
| VIS-005 | OCR provider contract | provider-neutral OCRProvider protocol | source purity gate |
| VIS-006 | Local OCR provider | Tesseract adapter | health/version + benchmark |
| VIS-007 | Remote vision extension point | provider-neutral RemoteVisionProvider protocol | source gate |
| VIS-008 | OCR confidence capture | TSV word/line confidence | unit/benchmark |
| VIS-009 | Invoice field parser | label-to-canonical-field parser | tests |
| VIS-010 | Multi-source reconciliation | normalized candidate consensus | tests |
| VIS-011 | Confidence calibration | binned monotonic calibrator | calibration tests |
| VIS-012 | Calibration measurement | ECE before/after | benchmark report |
| VIS-013 | Selective reread by confidence | uncertain field reread | unit test |
| VIS-014 | Selective reread by deterministic invalidity | invalid field reread/replacement | unit + benchmark |
| VIS-015 | CNPJ validation | deterministic check digits | regression |
| VIS-016 | Date validation | deterministic date parsing | regression |
| VIS-017 | Currency normalization | Brazilian decimal normalization | regression |
| VIS-018 | Invoice arithmetic validation | subtotal-discount+addition=total | regression |
| VIS-019 | Invalid-result confidence penalty | validator lowers invalid field trust | unit test |
| VIS-020 | Provider outage fallback | unavailable provider skipped | unit test |
| VIS-021 | Provider-neutral authority | document engine imports protocol, not vendor | purity gate |
| VIS-022 | Labeled corpus | versioned 16-case synthetic invoice corpus | manifest gate |
| VIS-023 | Degradation variants | clean/low-contrast/blur/compressed | corpus gate |
| VIS-024 | Field accuracy metric | normalized field correctness | benchmark |
| VIS-025 | Latency metrics | p50/p95/p99 | benchmark |
| VIS-026 | Accuracy threshold | >= 0.97 | benchmark gate |
| VIS-027 | Validation threshold | >= 0.95 | benchmark gate |
| VIS-028 | Latency threshold | p99 <= 5000 ms | benchmark gate |
| VIS-029 | Calibration non-regression | calibrated ECE <= raw ECE + 0.02 | benchmark gate |
| VIS-030 | Reread exercised | at least one case | benchmark gate |
| VIS-031 | F02 capability descriptor | versioned OCR capability fixture | contract gate |
| VIS-032 | F02 adapter manifest | versioned Tesseract adapter fixture | contract gate |
| VIS-033 | Offline operation | no remote API required | benchmark |
| VIS-034 | Reproducible generation | deterministic corpus generator | regenerate + hash structure |
| VIS-035 | Phase proof gate | one-command F03 verification | verify_f03.ps1 |

## Boundary

F03 does not implement browser actions, durable queues, autonomous planning, platform adapters or economic scheduling.

Tesseract is the first proven local OCR provider. It is not mission authority. The document engine accepts a provider list; field reconciliation plus deterministic validation determine whether an extraction is acceptable.
