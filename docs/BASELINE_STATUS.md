# Baseline Status

## Proven technical baseline

`POLYGLOT_V0_2`
- tag: `v0.2.0-polyglot-baseline`
- proof: `docs/PROOF_POLYGLOT_V0_2.md`

## Proven continuity baseline

`F01 CONCEPT_EXECUTIVE_CONTINUITY_BASELINE`
- tag: `v0.2.1-executive-baseline`
- proof: `docs/PROOF_F01_EXECUTIVE_CONTINUITY.md`

## Proven capability foundation

`F02 CAPABILITY_FOUNDATION`
- map: CAP-001..CAP-045;
- dedicated gate: `F02_VERIFY=PASS`;
- proof: `docs/PROOF_F02_CAPABILITY_FOUNDATION.md`;
- tag: `v0.2.2-capability-foundation`.

## Proven Vision/OCR foundation

`F03 VISION_OCR_DOCUMENT_INTELLIGENCE`
- map: VIS-001..VIS-035;
- local provider: Tesseract 5.4.0;
- Pillow 11.3.0;
- corpus: 16 cases / 4 variants / 112 fields;
- final benchmark: accuracy 1.0000, validation 1.0000;
- p99: 1427.96 ms;
- ECE: 0.0496 → 0.0250;
- reread cases: 4;
- dedicated gate: `F03_VERIFY=PASS`;
- full product gate: `BASELINE_VERIFY=PASS`;
- proof: `docs/PROOF_F03_VISION_OCR.md`;
- tag: `v0.3.0-vision-ocr`.

## Active construction lane

`F04 ISOLATED_BROWSER_COMPUTER_WORKER`

No other phase is authorized as the main construction lane until F04 closes.

## Formal proof status

Ada/SPARK sources exist, but GNATprove is not installed. Status remains:
`SOURCE_READY_NOT_PROVEN`.
