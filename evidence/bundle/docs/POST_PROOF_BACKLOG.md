# Post-Local-Proof Backlog

The project is not finished when the local proof passes. The local proof is the end of
the locally executable phase; the remaining work is local hardening followed by external
validation.

## Gate 0 — Complete the running full proof

- Wait for `./tools/local_proof.sh full` to finish.
- Fix every FAIL, not the test.
- Re-run the affected gates.
- Re-run the full proof on the final commit.
- Update `evidence/local-proof-2026-10-31-final/SUMMARY.md` and the evidence bundle.

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
- [x] Re-run the full local proof after all changes. All security gates pass at
  `8c41e4b`; see `evidence/local-proof-2026-10-31-final-audit/SUMMARY.md`.
- [x] Extend assembly evidence to an optimization/codegen/panic/LTO matrix.
- [x] Add Valgrind Memcheck and differential statistics validation.
- [x] Add a local signing dry run with an ephemeral test key.
- [x] Run a long fuzz campaign: 10M on the three fast targets and 1M on ChaCha20.
- [x] Replace the scalar masked path with a 64-block bitsliced batch and a fast
  64×64 bit-matrix transpose. The hard 10× ceiling is met at 7.15× in the final full proof (7.26× in an earlier run).
- [ ] Continue optimizing toward the 5× target. See `docs/PERFORMANCE_EVIDENCE.md`.

## Gate 2 — Freeze the local proof

- [x] Freeze the code and documentation.
- [x] Run the full proof on the frozen commit (`8c41e4b`).
- [x] Publish the evidence bundle and its hashes.
- [x] Record the supported platform and compiler matrix in `docs/SUPPORT_MATRIX.md`.
- [x] Record every remaining gap in `docs/GAP_REPORT.md`.

## Gate 3 — External second opinion

Status: the local proof is frozen at `8c41e4b`; none of these external gates has been
started or simulated.

Only after Gate 2:

- independent security audit;
- independent red team;
- hardware TVLA/CPA/DPA;
- production signing identity and SLSA release run.

These are not local tasks and must not be simulated or claimed as complete.

## Rule

Do not send the project for external review while a locally executable gate is failing or
while a locally provable claim lacks evidence.
