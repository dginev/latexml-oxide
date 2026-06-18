# Perl Dumper — On-Disk Record Format and Engine Hookup

> **Reference note (refreshed 2026-04-30).** Strict-Perl LoadFormat
> mutual-exclusivity remains the intended model, and the structured
> Parameter encoding documented here is still the relevant dump-format
> design. Current acceptance status, dump line counts, and sandbox
> numbers live in [`SYNC_STATUS.md`](SYNC_STATUS.md). Treat exact line
> references and bisection context below as Apr 26 audit data unless
> revalidated on current `HEAD`.

Scope: a close reading of `LaTeXML/lib/LaTeXML/Core/Dumper.pm` plus the
shipped `LaTeXML/blib/lib/LaTeXML/Engine/{plain,latex}_dump.pool.ltxml`,
comparing line-for-line against our `dump_writer.rs` / `dump_reader.rs`
to keep the record format faithful.

## The three concrete problems we hit

1. **`Until:<delim>` / `Match:<delim>` with brace-containing delimiters
   livelock at runtime.** Our `Parameters::stringify()` emits
   `"Until:\\end{verbatim}"`; `parse_parameters` re-parses that as
   `Until:\\end` followed by a `{verbatim}` *nested param spec*, which
   both loses the catcoded delimiter tokens and triggers
   `Unrecognized parameter type with name "verbatim"`.

2. **`DefToken`/`Optional`/`Semiverbatim`/custom types get flattened
   to `Plain`** whenever only `nargs` is known. This was partially
   fixed in commit fc45e068 (the `<proto>` 5th field), but only for
   specs `Parameters::stringify` round-trips cleanly. Anything more
   structured than `{}` / `[]` / `DefToken` still breaks.

3. **00_tokenize hangs for 34 minutes** when mutual-exclusivity is
   enabled against a format-v2 dump, because `\@ifnextchar` is
   declared `DefToken {}{}` in `_base.rs` but round-trips as
   `{}{}{}` — three plain args — so user code hitting
   `\@ifnextchar[{...}{...}` tries to read `[` as a balanced group
   and the tokenize pipeline livelocks.

All three trace to the same root cause: **we serialize the
*prototype string* instead of the *parsed Parameter struct*.**

## What Perl serializes instead — by example

From `latex_dump.pool.ltxml` L2835 (one of the expl3 `\::N` macros):

```perl
I(E(C('\\::N'),
    Ps(P('Until','Until:\\:::',extra=>[T(C('\\:::'))]),
       $P,
       $P),
    T(A(1),C('\\:::'),$TB,A(2),A(3),$TE),
    isLong=>1));
```

Decoded:

| Piece | Meaning |
|---|---|
| `C('\\::N')` | `Token('::N', CC_CS)` — the CS being defined |
| `Ps(...)` | `Parameters` wrapping three `Parameter` structs |
| `P('Until', 'Until:\\:::', extra=>[T(C('\\:::'))])` | **structured** Parameter: type=`Until`, spec=`Until:\\:::`, delimiter = the single-token list `[T_CS('\\:::')]` |
| `$P` | the singleton `LaTeXML::Core::Parameter->new('Plain', '{}')` — reused for every `{}` arg |
| `T(A(1), C('\\:::'), $TB, A(2), A(3), $TE)` | expansion: `#1 \::: { #2 #3 }` as catcoded tokens |
| `isLong=>1` | trait flag |

## Why this round-trips where ours cannot

Two independent design choices at the Perl side:

### (a) Parameters are dumped structurally, not as prototype strings.

Perl's `dump_parameter` (`Dumper.pm` L299-311):

```perl
sub dump_parameter {
  my ($parameter) = @_;
  my $type = $$parameter{type};
  my $spec = $$parameter{spec};
  if ($type eq 'Plain') { return '$P'; }       # Plain {} → singleton
  my $options = '';
  $options .= ',novalue=>1' if $$parameter{novalue};
  $options .= ',extra=>' . dump_rec($$parameter{extra})
    if $$parameter{extra};
  return 'P(' . dump_rec($type) . ',' . dump_rec($spec) . $options . ')';
}
```

The load-time reconstructor (`Dumper.pm` L296):

```perl
sub P { return LaTeXML::Core::Parameter->new(@_); }
```

At load time Perl calls `Parameter->new($type, $spec, extra=>[...])`
**with** the already-parsed delimiter tokens. No `parse_parameters`
is invoked. The stringified spec is carried along for diagnostics,
not for reconstruction. `Until:\end{verbatim}` never gets
re-fragmented.

Our equivalent `latexml_core::parameter::Parameter::new(name, spec,
extra)` exists (L127 of `parameter.rs`) and takes the same triple.
The gap is that our **writer** only emits `spec` and `nargs`; we never
serialize `type`, `name`, or `extra`.

### (b) Delimiter tokens are emitted catcode-explicit.

For `Until:\end{verbatim}`, Perl dumps
`extra=>[T(C('\\end'),$TB,L('v'),L('e'),L('r'),L('b'),L('a'),L('t'),L('i'),L('m'),$TE)]`
— a `Tokens` list with the full catcode info for each token. The
Huffman-short constructors are:

| Perl | Catcode | Rust equivalent |
|---|---|---|
| `C(s)` | `CC_CS` | `Token { text: arena::pin(s), code: Catcode::CS }` |
| `L(s)` | `CC_LETTER` | `Catcode::LETTER` |
| `O(s)` | `CC_OTHER` | `Catcode::OTHER` |
| `TA(s)` | `CC_ACTIVE` | `Catcode::ACTIVE` |
| `A(n)` | `CC_ARG` | `Catcode::ARG` (`#n`) |
| `$TB / $TE / $TM / $TP / $TS / $TSB / $TSP` | begin, end, math, param, space, sub, super | singletons |
| `$CR` | space + `\n` | singleton |
| `T(...)` | `Tokens` list | `Tokens::new(vec![...])` |

Our `parse_token` / `parse_token_list` in `dump_reader.rs` already
handles all these catcodes — the infrastructure exists on the reader
side. We just need the writer to emit them in the `extra` position
of each Parameter record.

### (c) `$P` singleton shortcut.

A scan of `latex_dump.pool.ltxml` shows `$P` is used 8,300+ times —
every `{}` parameter reuses the pre-declared singleton. This is
purely a file-size optimization (our dump already reuses one interned
spec, but the issue is the prototype-level reuse, not the
per-parameter reuse).

## Structural differences we need to close

Ordered by blocker-status for mutual-exclusivity:

| # | Gap | Fix |
|---|---|---|
| 1 | Writer emits prototype string, not Parameter structs | **Emit one sub-line per Parameter**: type + spec + extras |
| 2 | `extra` Tokens lost entirely | Encode catcoded tokens (already on the reader side) |
| 3 | `Plain {}` singleton not shared | File-size optimization; defer |
| 4 | No `Ps(...)` wrapper, just a flat "nargs" number | Parameters is a sequence; v3 needs a count-then-list encoding |
| 5 | E-entry trait flags limited to L/P | Perl also dumps `isOuter`; we don't store outer on Expandable |
| 6 | Register `extra` fields like `default`/`address` round-trip | We already do this via the R entry's extra fields; keep |

## Proposed format v3 on-disk layout

Tab-separated, parseable line-by-line, keeping the **invariant one CS
= one primary line**. Parameters serialized as sub-lines indented
under their E entry, read eagerly before the next `\n<toplevel>`:

```
E\t<cs>\t<nargs>\t<flags>\t<body>\n
\tP\t<name>\t<spec>\t<extra-token-count>\t<token1>\t<token2>\t…\n
\tP\t…\n
```

Where `<tokenN>` uses the same compact encoding as the expansion body
tokens (we already have this in `parse_token`).

`\tP\t` sub-lines attach to the most recent E-entry. This keeps
streaming parsing simple: the reader gathers `\tP\t` sub-lines until
it sees a non-indented line, then hands the full `(cs, nargs, flags,
body, Vec<Parameter>)` tuple to `Expandable::new`. No prototype string
is re-parsed. The Parameter ctor takes `(name, spec, extra)` — exactly
Perl's `sub P`.

Writer-side: `serialize_expandable` walks the Parameters slice, and
for each `Parameter { name, spec, extra, novalue, … }` emits one
indented line. The existing token-emitter handles `extra`'s
`Vec<Tokens>`.

Reader-side: extend the E-arm state machine. When the next line
starts with `\tP\t`, peel off a Parameter sub-line, build a
`Parameter::new(name, spec, Some(extras))`, append. Stop when the
next line isn't indented. Result list is the `paramlist` passed to
`Expandable::new` — bypassing `parse_parameters` entirely.

## What this does NOT fix

Format v3 alone is **necessary but not sufficient** for the
mutual-exclusivity flip. The 00_tokenize hang had multiple gates; the
`DefToken` round-trip was one. Others suspected but not verified:

- **PA-consumption chain** for expl3 `:`-style macros — requires `M`
  bodies referenced by PAs to be loaded (currently gated to
  `@`-internal only). A PA pointing at a `:`-style target whose M
  body is filtered out leaves an undefined-alias.

- **`@currname` preservation** through the dump load path. Fixed for
  user code in commit 56b0c35d2, but the dump loader doesn't run
  `input_definitions`, so the fix doesn't apply — the dump may
  still leak internal names. Low risk; flag if any regression is
  traceable.

- **Registers with closure getters/setters** — Perl's `dump_register`
  (L369-381) skips any Register with a getter/setter. Our writer
  needs the same guard (currently we emit them, and the reader
  tries to reconstruct state that only makes sense with the live
  closure).

## Recommended implementation sequence

Each step is a separately-committable unit that ships value without
flipping mutual-exclusivity:

1. **(v3.a) Add writer-side Parameter sub-lines.** [DONE, commit 3e1f89eb2].
   Backward-compatible: v2 dumps still load via the existing `<proto>`
   5th-field fallback; v3 dumps carry richer data that the reader can
   use. Test-suite unaffected.

2. **(v3.b) Add reader-side Parameter sub-line parser.** [DONE, same
   commit as v3.a]. Prefers v3 structured records when present, falls
   back to v2 proto-parsing, then v1 nargs-repeat. One code path.

3. **(v3.c) Wire `Until:`/`Match:` fixtures.** [DONE, commit 0be9641bf].
   Six roundtrip unit tests cover Plain, Until-with-braces, novalue
   flag, optional flag, empty, multi-Tokens extras. Lock the encoding
   layer so future refactors don't silently break round-trip.

4. **(v3.d) Filter closure-backed Registers** on the writer side.
   [ALREADY DONE pre-v3]. `dump_writer.rs` L254-256 already matches
   Perl's `dump_register` L371 — returns `None` when `getter` or
   `setter` is present.

5. **(v3.e/v3.f) Mutual-exclusivity REVIVED (2026-04-26 strict-Perl
   pivot).** A 2026-04-18 bisection of `LATEXML_DUMP_ONLY=1` left this
   path on the back-burner under the older "unified design" (always run
   bootstrap → _base → dump → _constructs). User directive 2026-04-26
   reversed that decision: the Rust port now mirrors Perl's mutual
   exclusivity exactly — `bootstrap → dump → constructs` when the dump
   is on disk, `bootstrap → base → constructs` otherwise. See
   [`PERL_LOADFORMAT_AUDIT.md`](archive/PERL_LOADFORMAT_AUDIT.md) for the
   parity audit and [`SYNC_STATUS.md`](SYNC_STATUS.md) for
   the rationale (the dump must be a faithful Perl translation, not a
   Rust-flavored alternate path).

   The v3 structured Parameter encoding (v3.a-v3.c) remains the active
   on-disk format.

## References

- `LaTeXML/lib/LaTeXML/Core/Dumper.pm` — the complete Perl dumper
  (395 lines)
- `LaTeXML/blib/lib/LaTeXML/Engine/latex_dump.pool.ltxml` L2835-2849
  — expl3 `\::N`-family entries (rich `Until`/`Match` use)
- `latexml_core/src/dump_writer.rs` L207-211 — current
  `stringify()`-based writer call-site
- `latexml_core/src/dump_reader.rs` L396-453 — current E-arm parser
  with the proto-parsing fallback and silent-degrade path
- `latexml_core/src/parameter.rs` L127-135 — `Parameter::new(name,
  spec, extra)` constructor — identical signature to Perl's `sub P`,
  no port work needed for the load-time side
