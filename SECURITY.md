# Security Policy

## Supported Versions

| Version | Supported |
| ------- | --------- |
| latest  | Yes       |
| older   | No        |

Only the latest published release receives security updates.

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Report privately via [GitHub Security Advisories](https://github.com/Sigilweaver/OpenMassSpec/security/advisories/new).

Include:

- A description of the vulnerability and its potential impact.
- Steps to reproduce or a proof of concept (a small input file is
  ideal).
- The affected crate (`openmassspec-io`, `openmassspec-io-cli`, or
  `openmassspec-io-py`).
- The OS, Rust toolchain, and crate version you were running.

Expect an initial acknowledgment within 7 days.

## Scope

In scope:

- **Parser correctness on malicious input.** `openmassspec-io` dispatches
  to vendor parsers and runs the canonical mzML writer from
  `openmassspec-core`. Crashes (panics, OOB reads, infinite loops),
  arbitrary file writes, or memory corruption triggered by a crafted
  vendor file or mzML input are in scope.
- **Path-traversal or arbitrary-file-write bugs** in
  `vendor2mzml` (CLI) and `openmassspec-io-py` (Python wheel).
- **Supply-chain integrity** of published artifacts on crates.io and
  PyPI: tampered manifests, missing provenance, unsigned releases.

Out of scope:

- Denial of service via legitimately oversized vendor files. Mass-spec
  acquisitions can be hundreds of GB by design.
- Vulnerabilities in third-party crates with no demonstrated exploit
  path through this stack. Forward those upstream.
- Issues that require write access to the parser source tree (this is
  a library, not a sandboxed service).

This repository is the **umbrella** of the OpenMassSpec stack. Reports
about a specific vendor parser are usually better routed to the
relevant repo - but the umbrella is a fine entry point and we will
forward as needed:

- Thermo `.raw`: [OpenTFRaw](https://github.com/Sigilweaver/OpenTFRaw)
- Bruker `.d/` (timsTOF): [OpenTimsTDF](https://github.com/Sigilweaver/OpenTimsTDF)
- Waters MassLynx `.raw/`: [OpenWRaw](https://github.com/Sigilweaver/OpenWRaw)
- Shared core: [openmassspec-core](https://github.com/Sigilweaver/OpenMassSpecCore)

## Disclosure

We follow coordinated disclosure. Reporters are credited in the
release notes unless they prefer to remain anonymous. We aim to ship
a fix within 30 days of confirming a high or critical issue.

## Note on reverse-engineered formats

The vendor parsers in this stack were developed by clean-room
reverse engineering of public-domain artifacts (PRIDE deposits,
published specifications). They do not depend on any vendor SDK or
binary blob. Bug reports about parser inaccuracy or unsupported
acquisition modes are welcome but are not security issues - file
those as regular GitHub issues on the relevant parser repo.
