# rust-gui-from-zero

A Cargo workspace of side-by-side Rust GUI demos (iced, fltk, gtk4, relm4,
egui), each with a provable contract on its core state machine.

## Layout

```
crates/
  contracts/      -- assert_invariant! macro, ContractError type
  iced-demos/     -- iced 0.10 demos
  fltk-demos/     -- fltk 1.4 demos
  gtk-demos/      -- gtk4 0.8 demos
  relm4-demos/    -- relm4 0.8 demos
  egui-demos/     -- eframe 0.22 demos
```

Each crate splits **logic** (pure state machines in `src/*.rs`, fully tested,
covered to 100% lines) from **view** (event-loop bindings in `src/bin/*`,
excluded from coverage).

## Contract-first development

This project follows a **contract-first** development style. Every binary
embeds a `Provable contract:` doc and runs an
`assert_invariant!(NAME, check_*().is_ok())` at startup. The state machines
in `src/` carry exhaustive negative-path tests so the invariant assertions
can fire on synthetic-bad inputs as well as real walks.

**Authoring loop:**
1. State the invariant in prose at the top of `src/<topic>.rs` as a
   `//! Provable contract: <NAME> — <human description>` doc comment.
2. Write `validate_*` helpers that return `Result<(), ContractError>`
   for each post-state assertion. Each validator takes its inputs by
   value so synthetic test cases can exercise the error path directly.
3. Compose validators into `check_<topic>_invariants() -> Result<(), ContractError>`
   that walks a deterministic input sequence.
4. The binary in `src/bin/<demo>.rs` calls
   `assert_invariant!(<NAME>_CONTRACT_HOLDS, check_<topic>_invariants().is_ok())`
   at startup; failure aborts the program before any GUI is shown.

The `contracts` crate (`crates/contracts/`) provides the
`assert_invariant!` macro and the `ContractError { name, message }` type.
See `crates/relm4-demos/src/simon.rs` for the canonical reference.

## Code Search

**MANDATORY: Use `pmat query` for ALL code search. NEVER use grep, rg, find,
fd, Glob, or Grep tools for code search.**

`pmat query` returns quality-annotated, semantically ranked results with
TDG grades, complexity, fault patterns, and call graphs.

### Decision tree

| Task | Command |
|------|---------|
| Find by intent | `pmat query "press handler" --limit 10` |
| Find with faults | `pmat query "unwrap" --faults --exclude-tests` |
| Literal search | `pmat query --literal "PressOutcome" --limit 10` |
| Regex search | `pmat query --regex "fn check_\w+_invariants" --limit 10` |
| With source | `pmat query "validate_state" --include-source` |
| Coverage gaps | `pmat query --coverage-gaps --limit 30 --exclude-tests` |

When you need to read code found by `pmat query`, use
`pmat query <name> --include-source` (NOT the Read tool on the file).
Use Read only for non-code files (TOML, YAML, Markdown, JSON).

### Why

- Raw grep / rg returns lines without quality context
- `pmat query` ranks by PageRank, TDG grade, fault patterns
- Fault flags (`--faults`) surface unwrap/panic/unsafe at search time
- Coverage flags surface uncovered functions ranked by ROI

## Quality Gates

- `cargo fmt --all -- --check` — formatting
- `cargo clippy --workspace --all-targets --no-deps -- -D warnings` — strict
- `cargo test --workspace --all-targets` — full suite
- `cargo llvm-cov --workspace --lib --ignore-filename-regex '(src/bin/|target/)'`
  `--fail-under-lines 100 --fail-under-functions 100`
- `cargo deny check advisories licenses bans sources`

CI runs all six on the self-hosted intel runner; the `gate` job aggregates
them via `needs:` so the GitHub `Green Main` ruleset can require a single
literal "gate" check.

## Coverage policy

`src/bin/` is **excluded from coverage** by design: those files own the
GUI event-loop wiring, which we test live during framework upgrades, not
in unit tests. The state machines in `src/*.rs` carry the 100%
line+function gate.

## CB-081 (dependency health)

Five GUI frameworks pull ~354 transitive deps, exceeding the default
CB-081 cap of 250. We accept this as structural — dropping a framework
defeats the demo's purpose. Direct deps remain at 9.
