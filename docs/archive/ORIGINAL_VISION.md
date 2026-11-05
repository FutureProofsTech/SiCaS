# Archived Original Vision (Superseded, Not a Security Claim)

> **Status: historical and superseded.** This document is the original project
> pitch. It contains architecture, performance, platform, and licensing claims
> that were rejected in [`PLAN.md`](../../PLAN.md): the LLVM pass, eBPF
> power-cancellation, universal jitter, weak entropy fallbacks, "perfectly flat"
> power, "mathematically useless" snooping, and "0% overhead" are **not** part
> of SiCaS and are **not** security claims. The current project is
> GPL-3.0-only, safe Rust, zero-runtime-dependency, first-order masking and
> leakage-assessment tooling. The current claims are in
> [`docs/SECURITY_CLAIMS.md`](../SECURITY_CLAIMS.md). Do not cite this file as
> evidence or as the current design.

---

PROJECT CODENAME: SiCaS
SPECIFICATION: Architecture for Universal Side-Channel Attenuation via Compiler-Level and Kernel-Level Polymorphism
AUTHOR: Autonomous System Architect
TARGET: Platform-Agnostic Physical Layer Security (ARM, x86_64, RISC-V)

================================================================================
1. EXECUTIVE SUMMARY & PROBLEM STATEMENT
================================================================================
Modern cryptography relies on mathematically proven algorithms (AES, RSA, ECC). 
However, when these algorithms execute on real, physical silicon, they leak 
information through physical phenomena known as Side-Channel Emissions. 

Every time a transistor switches state (0 to 1 or 1 to 0), micro-fluctuations 
occur in:
- Power Consumption (measured via Power Supply Lines - SPA/DPA attacks).
- Electromagnetic Radiation (measured via near-field RF probes - SEMA/DEMA attacks).
- Execution Timing (measured via high-resolution timers - Timing attacks).

Because cryptographic primitives (such as the XOR operation or modular 
multiplication) exhibit asymmetrical physical traits based on whether a processed 
bit is a 0 or a 1, an adversary with a $15 software-defined radio (SDR) or a 
near-field antenna can extract private cryptographic keys entirely out-of-band 
from a distance of up to several meters.

Current mitigations are either hardcoded into proprietary military-grade silicon 
or locked behind corporate closed-source compilers costing tens of thousands of 
dollars. Project Mirage provides a zero-cost, fully open-source, architecture-
agnostic software framework deployed at the compiler intermediate layer (LLVM 
Pass) and kernel runtime layer (eBPF) to completely flatten physical emissions, 
rendering hardware-level snooping mathematically useless.

================================================================================
2. TECHNICAL ARCHITECTURE & CORE PILLARS
================================================================================

Project Mirage implements defense-in-depth across three decoupled execution 
layers: Compiler (LLVM Intermediate Representation), Runtime Kernel (eBPF / 
OS Scheduler), and Environmental Entropy.

--------------------------------------------------------------------------------
Pillar A: Register Blinding via Dynamic IR Transformation (LLVM Pass)
--------------------------------------------------------------------------------
Instead of executing cryptographic operations on raw data keys, the Mirage LLVM 
compiler plugin intercepts the Intermediate Representation (IR) during compilation 
and transforms strict linear mathematical expressions into masked, multi-variable 
equations.

- Mechanism:
  For any identified cryptographic load instruction, the compiler injects a 
  True Random Number Generator (TRNG) fetch to retrieve a runtime mask ($M$).
  Before the sensitive variable ($V$) is loaded into physical CPU registers, it 
  is transformed into $V' = V \oplus M$. 
  
  All subsequent logical operations (e.g., S-Box lookups, shifts, rotations) 
  are executed using the masked variable $V'$ and a compensation loop for $M$.
  The mask is unmade only at the final memory write cycle inside an isolated register.

- Physical Effect:
  The hardware registers never hold the actual binary pattern of the key. The 
  electromagnetic radiation emitted by the register buses reflects a fluctuating 
  random variable ($V'$), rendering Simple Power Analysis (SPA) completely blind.

--------------------------------------------------------------------------------
Pillar B: Core-Pair Execution Symmetry (Kernel Scheduler / eBPF)
--------------------------------------------------------------------------------
Even with masking, Differential Power Analysis (DPA) can bypass security by 
averaging thousands of power traces. Mirage thwarts this via strict hardware 
power balancing.

- Mechanism:
  Using an eBPF runtime monitor at the OS kernel level, Mirage intercepts 
  system threads executing marked cryptographic binaries. It instantly enforces 
  a strict CPU Affinity policy using the `sched_setaffinity` syscall, locking 
  execution to an isolated physical core pair (e.g., Core 0 and Core 1).
  
  While Core 0 executes the masked cryptographic instruction block, Core 1 is 
  simultaneously forced to execute a bitwise inverted dummy block. If Core 0 is 
  processing a payload with high Hamming Weight (lots of 1s), Core 1 
  instantaneously processes a payload with low Hamming Weight (lots of 0s).

- Physical Effect:
  The net power consumption drawn from the main device power rail remains perfectly 
  flat and constant. The physical system maintains near-zero net fluctuations 
  in current draw, destroying the signal-to-noise ratio needed for Differential 
  Power Analysis.

--------------------------------------------------------------------------------
Pillar C: Asynchronous Time Jittering via Inline Assembly Assembly Injection
--------------------------------------------------------------------------------
To prevent attackers from alignment-matching recorded EM traces across time, 
Mirage destroys the temporal predictability of the execution pipeline.

- Mechanism:
  The compiler pass automatically injects variable-latency delays into the basic 
  execution blocks of the binary. This is achieved by embedding specialized 
  architecture-independent dummy loops and conditional branch-prediction traps 
  driven by a fast, non-blocking Pseudo-Random Number Generator.
  
  These loops execute arbitrary instructions (e.g., adding zero, rotating non-
  allocated registers) that consume deterministic clock cycles without affecting 
  the program state.

- Physical Effect:
  The execution timeline of the exact same cryptographic function varies wildly 
  on every run. Traces cannot be aligned or stacked on an oscilloscope, making 
  mathematical statistical averaging impossible.

================================================================================
3. MULTI-ARCHITECTURE COMPATIBILITY MATRIX
================================================================================

To ensure absolute mass deployment without rewriting hardware drivers, Mirage operates 
at the LLVM Intermediate Representation (IR) layer. By targeting the IR, the 
security passes are applied *before* the compiler converts the code into machine-
specific assembly.

| Target Architecture | Implementation Mechanism | Low-Level Hardware Primitive |
|---------------------|--------------------------|------------------------------|
| **ARM (Android/iOS)**| LLVM IR + eBPF Kernel Hook| Read `CNTVCT_EL0` for jitter; Lock to LITTLE cores |
| **x86_64 (PC/Server)**| LLVM IR + Pthreads Layer | `RDTSC` timer trapping; SMT Hyperthread pairing |
| **RISC-V (IoT/Embedded)**| Compiler Inline Assembly| Direct Hardware CSR register masking (`rmask`) |

Cross-Platform Entropy Fallback (Universal TRNG HAL):
If hardware instructions like `RDRAND` (x86) or `MRS` from cryptographic registers 
(ARM) are locked by the host OS, Mirage falls back to an algorithmic software 
Harvesting Engine. It samples chaotic environmental hardware noise including:
- Nanosecond jitter variations in memory cache misses.
- Micro-thermal fluctuations from battery/CPU temperature sensors.
- Minor timing deltas between incoming network packet headers.

================================================================================
4. PRODUCTION IMPLEMENTATION PLAN (SINGLE DEVELOPER, ZERO BUDGET)
================================================================================

A lone engineer can construct this entire ecosystem using 100% open-source tools:

### Phase 1: The Compiler Pass (Weeks 1-3)
- Write an LLVM Custom Pass in C++ that targets the `ModulePass` or `FunctionPass` class.
- Scan for functions matching standard cryptographic signatures (e.g., blocks containing intensive bitwise shifts, `XOR` loops, or arrays named `sbox`).
- Programmatically rewrite the LLVM Abstract Syntax Tree (AST) to introduce basic `NOP`-equivalent loops driven by the system entropy source.

### Phase 2: Simulation & Physical Verification (Weeks 4-5)
- Utilize **ELMO** (an open-source, cycle-accurate power simulation framework for ARM Cortex-M0/M4) or **MAPS**.
- Compile a target binary without Mirage $\rightarrow$ Run through ELMO $\rightarrow$ Export power traces $\rightarrow$ Observe clear cryptographic key leakage spikes.
- Compile with Mirage $\rightarrow$ Run through ELMO $\rightarrow$ Observe a flat, uniform power distribution graph. This validates the mathematical efficacy of the code without owning a physical oscilloscope.

### Phase 3: Deployment & Distribution Strategy (Week 6+)
- Release the project on GitHub under a permissive license (MIT/Apache 2.0).
- Format the tool as an easy-to-use C/C++ SDK wrapper or an `npm`/`cargo`/`pip` plugin for developers.
- Market the project directly to decentralized privacy-centric communities (Signal protocol researchers, hardware wallet developers, open-source mobile OS maintainers).

================================================================================
5. SPECIFIC RISK MITIGATION & PERFORAMNCE FOOTPRINT
================================================================================

- **Battery & Power Overhead:** Continuous noise generation would drain mobile batteries. 
  *Mitigation:* Mirage remains dormant ($0\%$ overhead) during standard app runtime. It triggers execution hooks *only* when a target cryptoprimitive is active (e.g., for $0.05$ seconds during transaction signing), preserving battery life completely.
  
- **Compiler Optimization Bypass:** Modern optimization levels (`-O2`, `-O3`) will attempt to strip out dummy loops and dead code introduced by the time jittering engine.

