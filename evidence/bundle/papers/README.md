# SiCaS Papers

This directory contains the LaTeX sources for the SiCaS white paper and yellow paper.

## Documents

| Document | Source | Purpose |
|---|---|---|
| White paper | [`whitepaper/SiCaS-Whitepaper.tex`](whitepaper/SiCaS-Whitepaper.tex) | Motivation, threat model, architecture, masking design, assessment methodology, evaluation, limitations, roadmap. |
| Yellow paper | [`yellowpaper/SiCaS-Yellowpaper.tex`](yellowpaper/SiCaS-Yellowpaper.tex) | Formal specification, data types, invariants, algorithms, data formats, security definitions, proof obligations, conformance. |
| Version log | [`VERSIONS.md`](VERSIONS.md) | Document and evidence version history. |

## Build

```bash
# Build both papers.
make -C papers

# Or build one paper.
make -C papers/whitepaper
make -C papers/yellowpaper
```

The build uses `pdflatex` and `bibtex`. The sources use only packages available in a
standard TeX Live installation: `amsmath`, `amssymb`, `amsthm`, `booktabs`,
`longtable`, `tabularx`, `enumitem`, `natbib`, `hyperref`, `microtype`, and `tikz`.

## Evidence alignment

Both papers are aligned with:

- proof code commit: `ef13675a9d44e881e1420f4cd5b9313043f13b73`;
- release build commit: `123eb69169199e3566980695d9723a61d5561481`;
- evidence tag: `final-audit-2026-10-31-dpa`;
- public label: **SAL-2 locally**.

The papers are documentation, not independent evidence. The authoritative evidence is
the hashed bundle in [`../evidence/bundle`](../evidence/bundle).
