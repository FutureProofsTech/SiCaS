# Threat Model (v1.0 Draft)

## Assets

- Long-term and session cryptographic keys.
- Plaintext and intermediate secret values.
- Mask shares and entropy-source state.
- Control flow and memory-access patterns derived from secrets.

## Adversary

- Passive physical attacker with a power or EM probe, an oscilloscope, known or
  selected plaintext, and a trigger signal.
- Can collect a bounded number of traces from the same device under normal operating
  conditions. The v1.0 budget is up to 1,000,000 traces for first-order TVLA/DPA and
  up to 100,000 traces for CPA; exact per-target numbers are published with results.
- No invasive probing, no fault injection, no firmware modification, no malicious OS
  or toolchain. Red-team engagements may exceed this model; findings outside it are
  reported as hardening evidence, not as violations of the claimed threat model.

## Out of scope as security claims

- Higher-order DPA and profiling template attacks.
- Fault-injection resistance.
- Invasive attacks.
- Cache-timing and microarchitectural attacks.
- Supply-chain attacks.
- Side channels in the OS, I/O, or hardware.
- Malicious hardware.

## Assumptions

- The OS CSPRNG or injected entropy source is secure when it reports success.
- The compiler and toolchain are not malicious.
- The target device is not physically modified.
- The attacker cannot observe the device during a trusted setup phase.

## Claim policy

A release may claim only the protections listed in
[`SECURITY_CLAIMS.md`](SECURITY_CLAIMS.md), with the evidence referenced in
[`PROOF_OBLIGATIONS.md`](PROOF_OBLIGATIONS.md). No absolute or unverifiable claims.
