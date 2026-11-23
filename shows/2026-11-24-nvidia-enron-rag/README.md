# 2026-11-24 Nvidia vs Enron: query the filings

`pmat query --raw --regex -i --limit 2000` over the Enron FY2000 10-K and the Nvidia FY2026 10-K.
This is cited keyword search (file:line), not semantic search: pmat's semantic index covers functions only.

- `rag-demo-test.sh CORPUS ENRON_10K NVDA_10K` asks the 4 show questions and fails unless each returns a cited passage from both filings.
- `spec.md` is the show spec; `scene/` is the terminal recipe.
- Filings are public SEC documents and are not stored here; fetch them into CORPUS first.
- Evidence, not accusation: a hit is a passage to read, not a verdict.
