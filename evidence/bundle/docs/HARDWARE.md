# Hardware Leakage Evidence Protocol

This document defines the minimum protocol for producing hardware evidence. It is a
protocol, not evidence. No hardware claim is valid until the raw traces and acquisition
metadata are published.

## Target

- A supported Cortex-M4 board (for example, an STM32F3/F4 target) or an equivalent
  target with a clean trigger pin.
- ChipWhisperer-Lite/Nano or a comparable oscilloscope/EM acquisition setup.
- Isolated lab power supply, stable clock, and temperature monitoring.
- Firmware built from `sicas-reference` with a deterministic test interface.

## Firmware interface

The target must expose:

- a GPIO trigger asserted immediately before the measured block;
- a fixed key mode and a random-key mode selected by the host;
- a fixed nonce and counter for the baseline experiment;
- a UART or equivalent channel for reporting the input and output hashes.

Do not print key material, masks, or intermediate shares over the reporting channel.

## Acquisition

1. Build the firmware with the pinned toolchain and record the commit hash.
2. Flash the target and verify the RFC 8439 keystream vector on-device.
3. Capture a small pilot set and verify trigger alignment.
4. Capture fixed-key and random-key groups with the same acquisition settings.
5. Record sample rate, gain, probe position, target voltage, temperature, and firmware
   hash in the evidence metadata.
6. Capture multiple devices and repeat at least one acquisition on a second day.

Initial target: 1,000,000 traces per group for first-order TVLA; 100,000 traces for
CPA. Exact counts must be reported with the results.

## Analysis

Export traces in the `sicas-harness` binary format and run:

```bash
sicas-harness tvla --fixed fixed.bin --random random.bin --threshold 4.5
sicas-harness cpa --traces traces.bin --plaintexts plaintexts.bin
```

Publish:

- raw traces and acquisition metadata;
- the exact analysis commands and tool versions;
- max `|t|`, sample index, trace counts, and confidence intervals;
- CPA correlation and key rank;
- known-leaky control results;
- limitations and failed runs.

## Current status

Hardware acquisition has not been performed. The synthetic harness validates the
analysis path only. The project cannot claim hardware leakage resistance until this
protocol is executed and independently reviewed.
