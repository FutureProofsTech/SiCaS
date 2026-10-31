# Security Test Matrix

Use this table to jump directly from a security property to the test or command that
exercises it.

| Property | Test / evidence | Command |
|---|---|---|
| Masking round-trip | `masked::tests::mask_and_unmask_round_trip` | `cargo test -p sicas-core mask_and_unmask` |
| Linear masked ops | `masked::tests::linear_operations_match_unmasked_reference` | `cargo test -p sicas-core linear_operations` |
| Masked AND / XOR / refresh / select | `masked::tests::all_integer_widths_round_trip_and_operate`, Kani `masked_*_logic_ops_match_reference` | `cargo test -p sicas-core`; `cargo kani --workspace` |
| Masked addition | `masked::tests::masked_addition_matches_reference`, Kani `masked_u32_add_matches_reference` | `cargo test -p sicas-core masked_addition` |
| Share uniformity | Kani `masked_u8_share0_covers_all_values`, `masked_u32_share0_covers_all_values`, `masked_u8_and_share0_covers_all_values` | `cargo kani --workspace` |
| Bitsliced lane independence | `chacha20::tests::bitsliced_addition_is_lane_independent` | `cargo test -p sicas-reference bitsliced_addition` |
| Bitsliced key-plane balance | `chacha20::tests::bitsliced_key_planes_are_balanced` | `cargo test -p sicas-reference bitsliced_key_planes` |
| Bitsliced batch correctness | `chacha20::tests::bitsliced_blocks_match_the_reference` | `cargo test -p sicas-reference bitsliced_blocks` |
| Bitsliced counter advance | `chacha20::tests::bitsliced_counter_advances_by_a_batch` | `cargo test -p sicas-reference bitsliced_counter` |
| Entropy failure in batch | `chacha20::tests::bitsliced_entropy_failure_is_propagated` | `cargo test -p sicas-reference bitsliced_entropy` |
| RFC 8439 vector | `chacha20::tests::rfc8439_keystream_block_is_correct` | `cargo test -p sicas-reference rfc8439` |
| Scalar masked ChaCha20 | `chacha20::tests::masked_blocks_match_the_reference` | `cargo test -p sicas-reference masked_blocks` |
| Key-share first-order correlation | `chacha20::tests::masked_key_shares_have_no_first_order_correlation` | `cargo test -p sicas-reference masked_key_shares` |
| First-order leakage model | `leakage-check` with negative control | `cargo run --release -p sicas-harness -- leakage-check --samples 50000` |
| Constant-time assembly | 69-configuration matrix | `python3 tools/ct_asm_check.py --matrix --lto` |
| Dudect timing | `sicas-harness ct` | `cargo run --release -p sicas-harness -- ct --samples 1000000 --warmup 100000 --trials 3` |
| TVLA correctness | `tvla::tests::*` | `cargo test -p sicas-harness tvla` |
| CPA correctness | `cpa::tests::*` | `cargo test -p sicas-harness cpa` |
| DPA correctness | `dpa::tests::*` (AES S-box key recovery, sign, tie-break) | `cargo test -p sicas-harness dpa` |
| Trace parser | `trace::tests::*`, `fuzz/targets/trace_parser.rs` | `cargo test -p sicas-harness trace`; `cargo +nightly-2026-08-03 fuzz run trace_parser -- -runs=1000000` |
| Bitsliced batch fuzz | `fuzz/fuzz_targets/chacha20_batch.rs` | `cargo +nightly-2026-08-03 fuzz run chacha20_batch -- -runs=1000000` |
| Preprocessing bounds | `preprocess::tests::*` | `cargo test -p sicas-harness preprocess` |
| Statistics differential | `tools/check_stats.py` | `python3 tools/check_stats.py` |
| Zeroization | volatile stores, drop test, assembly check | `cargo test -p sicas-zeroize`; `python3 tools/zeroize_asm_check.py` |
| Memory safety | Miri, ASan, Valgrind | `cargo miri test --workspace`; ASan/Valgrind scripts |
| Mutation strength | `cargo-mutants` | `cargo mutants --workspace --no-shuffle` |
| Coverage | `cargo-llvm-cov` | `cargo llvm-cov --workspace --all-features --fail-under-lines 90` |
| Release integrity | manifest validator | `python3 tools/validate_release_manifest.py --manifest release` |
| Reproducibility | three-build hash comparison | `./tools/local_proof.sh full` |
