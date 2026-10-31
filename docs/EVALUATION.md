# Leakage Evaluation

`sicas-harness` is the evidence layer. It does not make a security claim by itself;
it produces measurements that can be published with the raw traces, parameters, tool
version, commit hash, and target details.

## Trace format

```text
magic:             8 bytes, b"SiCaSTR1"
version:           u16 little-endian, currently 1
samples_per_trace: u32 little-endian
trace_count:       u32 little-endian
samples:           trace_count * samples_per_trace i16 little-endian
```

Limits: 1,000,000 traces, 1,000,000 samples per trace, 100,000,000 total samples.

The parser is fuzzed with `cargo fuzz run trace_parser`. Invalid magic, unsupported
versions, zero dimensions, ragged traces, oversized inputs, and truncated files are
rejected with typed errors.

## Synthetic validation

Generate a fixed-vs-random pair with a known leak at sample zero:

```bash
cargo run -p sicas-harness -- synthetic \
  --fixed-out fixed.bin --random-out random.bin \
  --fixed-count 128 --random-count 128 --samples 4 \
  --secret 0x42 --noise 10 --leakage 100 --seed 11
```

Run TVLA:

```bash
cargo run -p sicas-harness -- tvla --fixed fixed.bin --random random.bin
```

The command exits non-zero when any sample exceeds `|t| > 4.5`.

Generate a single trace set with plaintexts for CPA:

```bash
cargo run -p sicas-harness -- synthetic-cpa \
  --out traces.bin --plaintexts-out plaintexts.bin \
  --count 128 --samples 4 --secret 0x42 --noise 0 --leakage 100 --seed 7
```

Run CPA:

```bash
cargo run -p sicas-harness -- cpa --traces traces.bin --plaintexts plaintexts.bin
```

The synthetic CPA validation recovers `0x42` with correlation `1.0` at sample zero.

## Reference integration

`sicas-reference` provides a first-order Boolean masked ChaCha20 keystream block.
It is validated against the RFC 8439 keystream vector and against an independent
unmasked reference for randomized keys, nonces, and counters.

The masked block function is a declassification boundary: it returns plaintext
keystream bytes. Hardware capture of this target is still required for a TVLA/CPA
claim; the synthetic harness validates the analysis path, not the physical device.

## Preprocessing

```bash
sicas-harness trim --in traces.bin --out trimmed.bin --start 100 --end 900
sicas-harness filter --in trimmed.bin --out filtered.bin --window 5
sicas-harness align --in filtered.bin --out aligned.bin --reference 0 --max-shift 20
```

`align` uses normalized cross-correlation against a reference trace and zero-fills
samples outside the trace. `filter` applies a centered moving average with clipped
edges. `trim` keeps a half-open sample range.

## TVLA

Fixed-vs-random TVLA compares a group with a fixed input against a group with random
inputs using Welch's t-test per sample. `|t| > 4.5` is the default leakage indicator.

Report fields:

- `max_abs_t` — largest absolute t-statistic;
- `max_sample` — sample index of that statistic;
- `exceeding_samples` — number of samples above the threshold;
- `sample_count`, `fixed_traces`, `random_traces`, `threshold`.

A passing TVLA run is evidence for the tested target, trace count, and acquisition
setup only. It is not a proof of absence of leakage.

## CPA

The default CPA model is `HW(plaintext XOR key)` for a single byte. It is intentionally
simple and is intended for harness validation and first-order experiments. The library
also exposes `cpa_with_model` for algorithm-specific models such as an AES S-box. It is
not a replacement for a validated leakage model.

The Hamming-weight model is invariant under key complement: `HW(x XOR k)` and
`HW(x XOR k XOR 0xff)` are perfectly negatively correlated, so CPA identifies the pair
`{k, k XOR 0xff}` and the tie-break returns the lower key. The XOR model has the same
symmetry. Use an asymmetric model such as an S-box when the key pair must be
distinguished.

## First-order leakage model

The harness includes a deterministic first-order model check:

```bash
cargo run -p sicas-harness -- leakage-check --samples 50000 --threshold 0.02
```

It measures the Pearson correlation between individual shares and the secret for a
masked byte, a masked AND composition, and a masked `u32` addition. A negative control
correlates the secret with itself and must be detected; the command fails if the control
is missed or if any masked check exceeds the threshold.

This validates the masking model and the analysis code. It is not hardware evidence and
does not replace TVLA/CPA on a physical target. The reference crate also runs a
statistical check that the masked ChaCha20 key shares are uncorrelated with the key
words. The published run is in
[`evidence/leakage/2026-10-31-x86_64.md`](../evidence/leakage/2026-10-31-x86_64.md).

## DPA

The harness includes a first-order difference-of-means DPA over one bit of the AES
S-box output `SBOX[plaintext XOR key]`:

```bash
cargo run -p sicas-harness -- dpa --traces traces.bin --plaintexts plaintexts.bin
```

The S-box supplies the nonlinearity that makes the partition key-dependent. The
synthetic tests recover the exact key from a noise-free single-bit leak at bit 0 and at
bit 3, including the sign of the difference. A single-bit partition may not uniquely
recover the full key from noisy traces; combine several bits or samples before treating
a key as recovered.

The DPA command is offline analysis code. It is not constant-time and must not be used
to process live secret material.

## Timing

The timing command runs a dudect-style fixed-vs-random comparison of `ct::eq_u32`:

```bash
cargo run -p sicas-harness -- timing --samples 100000 --warmup 10000
```

Timing results are environment-sensitive. Use a quiet machine, pin the process, run
many samples, and publish raw measurements. The timing command is evidence tooling,
not a CI gate by itself.

## Limitations

- First-order analysis only.
- Alignment uses normalized cross-correlation with zero-fill; pathological traces
  may require manual alignment.
- `i16` samples only.
- The CPA model is byte-oriented and Hamming-weight based.
- TVLA can produce false positives and false negatives; use multiple tests, multiple
  devices, and a known-leaky control.
- No claim is valid without published raw data and reproduction steps.
