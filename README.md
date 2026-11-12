# batisseur

**v0.2.0 (experimental)** — a Rust CLI that routes typed software-build intents
to a small platform capability set. It explains scores, abstains on missing
information and ambiguous choices, and reports permission gaps.

The pinned [tisserand taxonomy](https://github.com/Tariq-Anjum/tisserand) is the
shared task-type contract (tt-1.1.0). Platform and interaction fields filter
within that type; they never choose a replacement type.

## Install

Prebuilt static binary for Linux x86_64 (checksum-verified, installs to `~/.local/bin`, no root):

```sh
curl -fsSL https://raw.githubusercontent.com/Tariq-Anjum/batisseur/main/install.sh | sh
batisseur --version
```

Or download the `.tar.gz` and `.sha256` from the GitHub Releases page and check them with `sha256sum -c`.

## Build from source

Build with a Rust toolchain and the dependencies in Cargo.lock available in
your Cargo cache. Repository configuration enforces offline dependency access.
For a fresh machine, deliberately provision the locked dependencies first;
this CLI does not install or download tools.

```sh
cargo build --locked --offline
cargo run -q -- packs list
cargo run -q -- packs show android-tv
cargo run -q -- validate examples/android-tv.json
cargo run -q -- route examples/android-tv.json --json
cargo run -q -- bench --json
```

`route` emits `needs_information` for incomplete typed input or close scores.
Check `risk_gate.status` separately: a selected route may still require
permissions. The CLI executes no plan and writes no run state.
`validate` checks complete typed input, not feasibility or authority.

## Layout

| Path | Purpose |
| --- | --- |
| src/ | Deterministic routing library and CLI |
| contracts/ | Pinned taxonomy, provenance, family/interaction maps, workflows |
| schemas/ | JSON Schema 2020-12 input and output contracts |
| packs/ | Nine platform/interaction packs and generic official references |
| bench/ | Frozen BuildBench v0 cases and expected routes |
| tests/ | Routing, schema, benchmark and CLI subprocess checks |
| skill/batisseur/ | Thin optional skill; not installed by this project |
| docs/SPEC.md | Routing cascade, pack contract, state layout and boundaries |

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -q -- bench
```

No network, database or daemon is required. Framework entries are conservative
candidates, and verifier lists are requirements; neither is proof of a built
or verified application. Unsupported resource limits require pack evidence.
