# Post-Local-Proof Backlog

The project is not finished when the local proof passes. The local proof is the end of
the locally executable phase; the remaining work is local hardening followed by external
validation.

## Gate 0 — Complete the running full proof

- Wait for `./tools/local_proof.sh full` to finish.
- Fix every FAIL, not the test.
- Re-run the affected gates.
- Re-run the full proof on the final commit.
- Update `evidence/local-proof-final/SUMMARY.md` and the evidence bundle.

## Gate 1 — Local proof hardening

- [x] Add Kani proofs for every `Masked` linear operation, logical operation,
  `select`, refresh, equality, and bit extraction across all widths.
- [x] Add property tests for trace trimming, filtering, alignment, TVLA ties and
  thresholds, and CPA key recovery.
- [x] Add a first-order leakage-model checker for masked bytes, masked AND, masked
  addition, and ChaCha20 key shares, with a negative control.
- [x] Improve mutation coverage where missed mutants are not equivalent; record
  remaining equivalent mutants.
- [x] Add a zeroization design review; the adapter itself remains external because it
  requires an audited unsafe boundary.
- [x] Add release-manifest/SBOM/provenance consistency validation with tests.
- [x] Re-run the full local proof after all changes. All gates pass at `c349329`;
  see `evidence/local-proof-2026-10-31/SUMMARY.md`.

## Gate 2 — Freeze the local proof

- [x] Freeze the code and documentation.
- [x] Run the full proof on the frozen commit (`c349329`).
- [x] Publish the evidence bundle and its hashes.
- [x] Record the supported platform and compiler matrix in `docs/SUPPORT_MATRIX.md`.
- [x] Record every remaining gap in `docs/GAP_REPORT.md`.

## Gate 3 — External second opinion

Only after Gate 2:

- independent security audit;
- independent red team;
- hardware TVLA/CPA/DPA;
- production signing identity and SLSA release run.

These are not local tasks and must not be simulated or claimed as complete.

## Rule

Do not send the project for external review while a locally executable gate is failing or
while a locally provable claim lacks evidence.
