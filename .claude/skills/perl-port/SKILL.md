---
name: perl-port
description: >
  Faithfully translate or fix a LaTeXML binding (macro / primitive / constructor
  / column type / package) from the Perl source to Rust. Use whenever you write
  or change engine/package code that mirrors Perl LaTeXML — porting a new .pool
  entry, binding a package, or aligning a definition to Perl semantics. Enforces
  "read the Perl source first" and the divergence policy. Invoke for "port \foo",
  "translate this .pool entry", "bind package X", "make Y match Perl",
  "/perl-port".
---

> The Perl code is the **ground truth**. This is a translation project: preserve
> the original semantics, control flow, edge cases, and naming. Do not invent
> abstractions, rename concepts, or simplify behavior unless the difference is a
> documented intentional divergence.

## 1 — Read the Perl source FIRST (always, before writing Rust)

| Kind | Perl source |
|---|---|
| Engine definitions (`\cs` in the kernel) | `LaTeXML/lib/LaTeXML/Engine/<file>.pool.ltxml` |
| Package bindings | `LaTeXML/lib/LaTeXML/Package/<name>.sty.ltxml` (and `.cls.ltxml`) |
| Core machinery (Mouth/Gullet/Stomach/Document/State) | `LaTeXML/lib/LaTeXML/Core/*.pm` |
| The `Def*` API itself | `LaTeXML/lib/LaTeXML/Package.pm` |
| TeX ground truth (when Perl is itself emulating TeX) | `background/tex.web`, `background/texbook.tex` |

Quote the Perl line range in a comment next to the Rust port (the codebase does
this consistently, e.g. `// Perl L2734-2752`) so the next reader can diff against
the source.

## 2 — Find the Rust home: `docs/ORGANIZATION.md`

It maps each Perl `.pool.ltxml` → `latexml_engine/src/<file>.rs` (38 of 40 are
1:1). **Same-file rule:** every `\foo` defined in
`Engine/<file>.pool.ltxml` must be defined in `latexml_engine/src/<file>.rs`. The
Perl→Rust crate map (Mouth→mouth, Gullet→gullet, Stomach→stomach, …) is in
CLAUDE.md "Key Concepts Mapping".

## 3 — Translate faithfully

- Match the **macro kind**: Perl `DefMacro`/`DefPrimitive`/`DefConstructor`/
  `DefColumnType`/`DefRegister` → the corresponding Rust `Def*!`. (`tools/
  audit_def_parity.py` cross-checks kinds.)
- Where Perl uses `RawTeX` / raw `\def`-style bodies, port them as **Token
  bodies**, not opaque Rust closures — so the kernel dump captures them as
  serializable records (CLAUDE.md strict-LoadFormat rule #3).
- Preserve attribute semantics (`locked`, `scope`, `bounded`, `requireMath`,
  `robust`); `tools/audit_attrs.sh` / `audit_locked.sh` verify parity.
- Reach for meaningful Rust types where Perl was untyped — but only when it does
  not change behavior.

## 4 — Check three docs BEFORE finalizing

1. **`docs/WISDOM.md`** — known internal traps. Don't re-introduce a fixed bug
   (compile-time vs runtime token packing, `Font::merge`/`specialize`, catcode CS
   vs ESCAPE, `RegisterType` `PartialEq` trap, `at_letter` restore, the libxml
   shared-Node `try_borrow_mut` guard, …).
2. **`docs/KNOWN_PERL_ERRORS.md`** — is the behavior you're "fixing" actually an
   upstream Perl bug? If so, keep parity (don't out-correct it) and record it
   there with a minimal trigger.
3. **`docs/OXIDIZED_DESIGN.md`** — is the difference a *sanctioned* divergence
   (e.g. `%\n` not emitted, comments off by default, `\cdots` ELIDEOP, color
   visual-equivalence, no `tex=` on `<picture>`)? If not, you must match Perl.

## 5 — Divergence policy

Diverge **only** when documented in `OXIDIZED_DESIGN.md`. In particular, a change
that makes Rust emit *fewer* errors than Perl on the same input is a divergence,
not a fix — it needs same-host parity proof or explicit surpass-Perl
authorization (see the `canvas-triage` golden rules). When you do diverge with
approval, add the entry to `OXIDIZED_DESIGN.md` in the same change.

## 6 — Validate, then ship

- `cargo test --tests --no-fail-fast` (true count; baseline ~1467/0 — see
  `perf-check` for the RSS-fuse `MemoryBudget` cascade gotcha). New `.tex`/`.xml`
  fixture ⇒ `cargo clean` first so the plugin rediscovers it.
- `cargo +nightly clippy --workspace --all-targets -- -D warnings` and
  `cargo +nightly fmt --all` — the pre-push hook enforces both (it does NOT run
  tests, so run them yourself).
- Confirm Perl parity on the witness paper (verbose, same host).
- **Before pushing**: build current HEAD green; never `push --no-verify` on a
  shared branch without it (a co-agent's commit can ride your push → red CI).
  Push to the working **feature branch**, not a fast-forward to `main`.
