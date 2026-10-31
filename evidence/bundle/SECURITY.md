# Security Policy

## Current status

SiCaS is pre-release and has not been independently audited or hardware-validated.
Do not deploy it to protect real secrets.

## Reporting a vulnerability

Once the repository has a public security advisory channel, report vulnerabilities
privately through that channel. Until then, do not disclose suspected vulnerabilities
publicly; contact the maintainers directly.

Include:

- affected version or commit;
- a minimal reproduction;
- the affected security claim;
- observed impact and any proof-of-concept material;
- whether the issue is in scope for the claimed threat model.

## Disclosure

We aim to acknowledge reports within 7 days, provide a remediation plan within 30 days
for high-severity issues, and coordinate public disclosure after a fix is available.

## Scope

In scope for v1.0:

- `sicas-core` constant-time and masking behavior.
- Secret-handling API misuse that leads to disclosure.
- Entropy fail-closed behavior.
- Build and release integrity.

Out of scope until explicitly claimed:

- higher-order DPA, profiling template attacks, fault injection, invasive attacks,
  cache-timing and microarchitectural attacks, supply-chain attacks, side channels in
  the OS or I/O, and malicious hardware.
