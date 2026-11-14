---
name: latexml-perl-port
description: Faithfully port, repair, or root-cause LaTeXML engine and package behavior from Perl to Rust. Use when changing a macro, primitive, constructor, register, column type, package binding, load behavior, or core mechanism that is expected to match Perl LaTeXML.
---

# Port Perl LaTeXML behavior faithfully

The Perl implementation is the translation oracle. Read it before writing Rust.

## Locate ground truth

| Behavior | Primary source |
|---|---|
| Engine definition | `LaTeXML/lib/LaTeXML/Engine/*.pool.ltxml` |
| Package or class binding | `LaTeXML/lib/LaTeXML/Package/*.ltxml` |
| Mouth, Gullet, Stomach, Document, State | `LaTeXML/lib/LaTeXML/Core/*.pm` |
| Binding API | `LaTeXML/lib/LaTeXML/Package.pm` |
| Real LaTeX or package definition | `kpsewhich latex.ltx` or `kpsewhich <pkg>.sty` |
| TeX mechanism | `background/tex.web` and `background/texbook.tex` |

Use `docs/parity/ORGANIZATION.md` to find the Rust home. Keep a definition in the
Rust file corresponding to its Perl pool, and cite a durable Perl symbol or line
range beside a non-obvious translation.

## Translate the mechanism

- Match `DefMacro`, `DefPrimitive`, `DefConstructor`, `DefColumnType`, and
  `DefRegister` with the corresponding Rust definition mechanism.
- Preserve flags and semantics such as locking, scope, boundedness, robustness,
  and math requirements.
- Port Perl `RawTeX` bodies as serializable token bodies so dump generation can
  capture them. Do not replace them with opaque closures for convenience.
- Follow existing types and naming. Add an abstraction only when it preserves the
  model and removes real duplication.
- Instrument the state gate when a primitive appears not to fire. A correctly
  false gate usually points to the missing upstream setter or mode transition.
- When a byte-faithful `.pool` binding still differs, compare it with the real
  kernel or package definition early. Perl core machinery can compensate for a
  simplification that Rust does not yet implement.

## Check before deciding

Read:

- `docs/parity/WISDOM.md` for known implementation traps;
- `docs/parity/KNOWN_PERL_ERRORS.md` for upstream defects;
- the `docs/parity/OXIDIZED_DESIGN*` family for approved divergences;
- `docs/parity/DUMP_DESIGN.md` when load phase or serialization is involved.

Do not suppress or downgrade a diagnostic to obtain a clean run. Do not add a
no-op binding for semantic behavior. A new divergence requires user approval,
documentation, a code-site reference, and tests; use `latexml-surpass-perl`.

## Validate

1. Add or update the narrow regression guard at the layer where behavior fails.
2. Confirm the reduced/current witness against verbose same-host Perl.
3. Re-run every arXiv witness named in a touched comment.
4. Run focused tests, then `cargo nextest run --workspace` when the change is
   ready. A new `.tex`/`.xml` pair requires one `cargo clean`.
5. Before publication, run nightly clippy with `-D warnings` and formatting.

For a paper whose classification is not established, use
`latexml-canvas-triage` first. For dump-only behavior, use
`latexml-dump-debug` before editing load code.

