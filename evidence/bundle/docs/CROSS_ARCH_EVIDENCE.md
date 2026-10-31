# Cross-Architecture Functional Evidence

## AArch64 Linux under QEMU

Date: 2026-10-31

Commit: `ef13675` (AES S-box DPA commit)

Target: `aarch64-unknown-linux-gnu`  
Runner: `qemu-aarch64` user mode  
Command: `./tools/run-aarch64-tests.sh`

| Crate | Tests | Result |
|---|---:|---|
| `sicas-core` | 20 | pass |
| `sicas-harness` | 98 | pass |
| `sicas-platform` | 4 | pass |
| `sicas-reference` | 15 | pass |
| `sicas-zeroize` | 4 | pass |
| **Total** | **141** | **pass** |

The run covers all workspace library tests, including masked addition, masked
ChaCha20 RFC 8439 vectors and differential checks, trace parsing, preprocessing, TVLA,
CPA, DPA AES S-box key recovery, timing logic, the first-order leakage model, and the
OS entropy adapter.

## What this proves

- The workspace compiles and runs functionally on AArch64 Linux.
- The masked reference output matches the unmasked reference on AArch64.
- The trace parser and analysis code behave correctly on a 64-bit little-endian
  architecture.

## What this does not prove

- It is not runtime timing evidence. QEMU timing is not representative.
- It is not hardware power/EM evidence.
- It does not cover 32-bit ARM, RISC-V runtime, or Cortex-M hardware.
- It does not replace an independent audit.

The AArch64 timing and hardware gates remain in `docs/GAP_REPORT.md`.

## 32-bit x86 Linux

Date: 2026-10-31

Commit: `ef13675` (AES S-box DPA commit)

Target: `i686-unknown-linux-gnu`  
Runner: native 32-bit execution  
Command: `./tools/run-i686-tests.sh`

| Crate | Tests | Result |
|---|---:|---|
| `sicas-core` | 20 | pass |
| `sicas-harness` library | 98 | pass |
| `sicas-harness` main | 8 | pass |
| `sicas-harness` CLI | 5 | pass |
| `sicas-platform` | 4 | pass |
| `sicas-reference` | 15 | pass |
| `sicas-zeroize` | 4 | pass |
| **Total** | **154** | **pass** |

This proves functional behavior on a 32-bit little-endian target, including masked
addition, masked ChaCha20, trace parsing, preprocessing, TVLA, CPA, DPA, CLI workflows,
and entropy adaptation. It is not timing or hardware leakage evidence.

An i686 timing run was attempted but is not used as evidence: the 32-bit timer
resolution and release optimization make the measurements unreliable. The x86_64
timing evidence remains the only dudect evidence in this report.
