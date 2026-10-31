# Zeroization Design Review

## Scope

This review covers `Secret<T>`, `Masked<T>`, and the key material held by
`MaskedChaCha20`.

## Current guarantee

There is **no zeroization guarantee**. Dropping any of these types performs an ordinary
Rust drop. The compiler may keep copies in registers, stack slots, or spilled memory, and
it may remove stores that it considers dead. Swap, hibernation, crash dumps, and physical
memory access are outside the current model.

This limitation is documented on the public types and is tracked in
[`GAP_REPORT.md`](GAP_REPORT.md). It is not counted as a locally proven claim.

## Why it is not implemented locally

A trustworthy zeroization primitive needs at least:

- a volatile write that the optimizer cannot remove;
- a compiler fence around the write;
- coverage of every copy, including temporaries and moved values;
- handling of error paths and panics;
- optionally, page locking (`mlock`) and fork/exec hygiene.

In Rust, the volatile write requires `unsafe` (`core::ptr::write_volatile`). The published
crates use `#![forbid(unsafe_code)]` and have zero third-party runtime dependencies. A
safe wrapper around an unsafe primitive would still need an audited unsafe boundary and a
platform-specific implementation. That boundary is an external gate, not a local
implementation shortcut.

`compiler_fence` is safe, but a fence without a volatile write does not guarantee that
the value is erased. A normal write followed by a fence can still be optimized away.

## Threat model

Zeroization matters after a key or share is no longer needed, especially when:

- the process can be crashed and a core dump captured;
- memory can be swapped or hibernated;
- the attacker has physical memory access or can trigger a cold-boot attack;
- a library returns early on an error path and leaves a copy behind.

These threats are outside the current first-order passive side-channel model. They are
still relevant to a production deployment and must be addressed before any production
claim.

## Future adapter review checklist

An audited platform adapter must demonstrate:

1. Every secret-bearing buffer is overwritten with volatile writes on drop and on every
   error path.
2. A compiler fence follows the final write.
3. No copies survive in registers, temporaries, or moved values that the adapter does not
   control.
4. The adapter documents its memory-locking and fork/exec behavior.
5. Tests cover success, error, and panic paths, and the generated assembly is inspected
   to confirm the writes survive optimization.
6. The unsafe boundary is small, reviewed, and covered by Miri or a documented
   alternative where Miri cannot model the operation.
7. The adapter is optional; the default build remains safe Rust with no runtime
   dependencies.

Until that adapter exists and is independently reviewed, SiCaS must not claim volatile
zeroization, secure erasure, or protection against memory-remnant attacks.

## Status

- Experimental implementation: `crates/sicas-zeroize` uses volatile byte writes and a
  compiler fence, with unit tests. It is `publish = false` and is not used by any
  published crate.
- Local proof: the crate passes its tests, and the x86_64 assembly contains explicit
  zero stores rather than a `memset` call (`evidence/zeroize/2026-10-31-x86_64.md`).
  Independent review is still required.
- External gate: independent review of the unsafe boundary or a platform guarantee with
  equivalent evidence.
