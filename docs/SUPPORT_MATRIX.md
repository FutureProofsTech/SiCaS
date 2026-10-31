# Support Matrix

| Platform | v1.0 status | Mechanism | Evidence |
|----------|-------------|-----------|----------|
| Linux x86_64 | Tooling and core tests | `std` tooling, `no_std` core | Unit, Miri, Clippy |
| ARM Cortex-M4 | Core library target | `no_std` core | Cross-check and hardware TVLA pending |
| AArch64 Linux | Functional tests pass | `no_std` core + std tooling | QEMU user-mode: 141 workspace library tests pass; timing/hardware pending |
| x86 32-bit Linux | Functional tests pass | `no_std` core + std tooling | Native 32-bit: 154 all-target tests pass; timing/hardware pending |
| RISC-V 32/64 | Cross-check target | `no_std` core | Cross-check pending |
| iOS / Android | Not supported | Sandbox restrictions | None |
| eBPF / kernel modules | Not supported | Not feasible as described | None |
| Windows / macOS | Not supported in v1.0 | Runtime adapters planned for v1.1 | None |

The support matrix is a security claim. A platform is "supported" only when its tests,
evidence, and limitations are published.
