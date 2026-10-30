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

- Add Kani proofs for every `Masked` linear operation and `select`.
- Add property tests for trace alignment, filtering, and custom CPA models.
- Add a first-order leakage-model checker for the masked adder and ChaCha20
  composition.
- Improve mutation coverage where missed mutants are not equivalent.
- Add a zeroization design review; implement only inside an audited unsafe boundary.
- Add release-manifest/SBOM consistency validation.
- Re-run quick proof after each change.

## Gate 2 — Freeze the local proof

- Freeze the code and documentation.
- Run the full proof on the frozen commit.
- Publish the evidence bundle and its hashes.
- Record the exact supported platform and compiler matrix.
- Record every remaining gap in `docs/GAP_REPORT.md`.

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
