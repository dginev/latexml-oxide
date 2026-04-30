# expl3 Strict-Perl Parity Audit (2026-04-26)

> Triggered by user directives:
> - "wrong fix. aim for strict perl translation, find the root cause"
> - "audit this fully, we want complete parity in translation"
> - "focus the audit on the current examined call chain for expl3"

The recent commits `959d25e9d` / `e2ad374f7` were workarounds. They
were reverted in `a7cfa5827` / `cb406bd40`. This document captures
the strict-Perl baseline AND the engine-level root cause for the
49 boxing-group errors observed during `\usepackage{expl3}`.

---

## Strict-Perl baseline

`LaTeXML/lib/LaTeXML/Package/expl3.sty.ltxml` is **3 functional lines**:
```perl
LoadPool('LaTeX');
InputDefinitions('expl3', type => 'lua');
InputDefinitions('expl3', type => 'sty', noltxml => 1);
```

Perl converts `\usepackage{expl3}` cleanly: "Conversion complete: No
obvious problems". No errors. No warnings. The 3-line wrapper just
delegates to the raw expl3.sty file. That .sty has a TeX-level guard
`\expandafter\ifx\csname tex_let:D\endcsname\relax \expandafter\@firstofone\else\expandafter\@gobble\fi {\input expl3-code.tex }`
which detects that `\tex_let:D` is already defined (because
`latex_dump.pool.ltxml` includes the expl3 kernel state) and SKIPS
the `\input expl3-code.tex` step entirely.

---

## The boxing-error call chain (49 events per `\usepackage{expl3}`)

`expl3_sty.rs` adds compensatory `raw_tex(\msg_new:nnn{cmd}{define-command}{...})`
calls. Since the dump now contains `\msg text > cmd/define-command`,
each `\msg_new:nnn` call fires:

```
\msg_new:nnn → \msg_new:nnnn → \__msg_chk_free:nn
  → \msg_if_exist:nnT TRUE
    → \msg_error:nnnn{msg}{already-defined}
       → \msg_error:nnnnnn{msg}{already-defined}{...}{...}{}{}
         → \__msg_use:nnnnnnn{error}{msg}{already-defined}{...}
           → \cs_set_protected:Npe \__msg_use_code: { ... }
           → \__msg_use_code: → \__msg_error_code:nnnnnn{msg}{already-defined}{...}
             → \__msg_interrupt:NnnnN \msg_error_text:n {msg} ...
               → \__msg_interrupt_wrap:nnn → \__msg_interrupt:n
```

The `\__msg_interrupt:n` body (verified bit-equivalent in Rust dump
and Perl `latex_dump.pool.ltxml`) is:
```
\iow_term:n {<text>}
\__kernel_iow_with:Nnn \tex_newlinechar:D {`\^^J}
{ \__kernel_iow_with:Nnn \tex_errorcontextlines:D {-1}
  { \group_begin:                 ← OPENS non-boxing group
    \cs_set_protected:Npn \  {<padding-spaces>}
    \tex_errmessage:D {#1}
    \use_none:n {<padding>}
    \exp_after:wN \group_end:     ← Closes non-boxing group
  } }
```

Token count audit of the body: **8 catcode-1 (BEGIN) tokens, 52
catcode-2 (END) tokens**. UNBALANCED at the LITERAL level — but
balanced at the SEMANTIC level because most catcode-2 tokens are
**space characters with catcode 2** that participate in TeX's
spacing/error-message rendering, NOT real group closes.

In Perl's encoding: `Token(' ',1)` = space with catcode 1, `Token(' ',2)`
= space with catcode 2. They're literal SPACES that happen to have
group-catcodes — TeX's traditional way of laying out error message
indentation/padding.

In Rust's encoding: `1: ` and `2: ` (single space content with
catcode 1 and 2 respectively).

---

## ENGINE ROOT CAUSE (Rust-side bug)

When the gullet/stomach pipeline encounters a catcode-2 token,
it ALWAYS treats it as a real group-close, even if the token
is a SPACE-with-catcode-2 (literal text content).

The Perl side handles this distinction: `\tex_errmessage:D` writes
its arg to terminal; the catcoded-spaces inside `{...}` are part
of the message TEXT, not actual group syntax. Perl's primitive
`\errmessage` reads its arg via `readBalanced` which understands
catcoded-space-as-content vs catcoded-space-as-group.

In Rust, the gullet/stomach uses `Catcode::END` (catcode 2) as the
sole indicator that a `}` is closing the current group. There's no
distinction between "literal `}` in text content" and "structural
`}` closing a `\begingroup`-frame".

**The bug**: when `\tex_errmessage:D {<msg-with-catcoded-spaces>}`
is being x-expanded for arg-reading, the catcode-2 spaces inside
the message body are interpreted as group-closes, hitting the
"Attempt to close boxing group; current frame is non-boxing" error.

This is the same **eager group-close** bug pattern as the gullet
DEFERRED_COMMANDS issue (commit e3d4f8532). Both stem from the
gullet's unconditional treatment of catcode-1/2 as group syntax,
ignoring context-dependent semantic distinctions.

---

## Audit conclusion

The strict-Perl translation requires:

1. **DELETE** all compensatory `raw_tex` blocks in `expl3_sty.rs`
   (lines 41-225). They were workarounds for engine-level
   deficiencies that, on subsequent fixes, surface as
   "already-defined" cascades.

2. **FIX the engine** in `gullet.rs` / `stomach.rs` so that
   catcode-1/2 tokens are correctly treated as TEXT CONTENT when
   they're inside the body of an Expandable (not as new group
   begins/closes).

3. **OR** fix the engine so that the dump's serialized expl3 state
   is sufficient that the raw expl3.sty load gates correctly via
   `\tex_let:D` (which it should already do — verify the
   `\csname tex_let:D \endcsname` lookup actually finds the
   dump-loaded `\tex_let:D` PA-alias).

The strict-Perl mission per CLAUDE.md L1-3:
> Every translated entry must follow tightly the original
> semantics and nuances of the Perl source. Do not invent new
> abstractions, rename concepts, or simplify behavior unless
> explicitly marked as an intentional divergence. The Perl code
> is the ground truth.

---

## read_balanced/read_arg trace finding (2026-04-26 iteration B)

User directive: "instrument read_balanced/read_arg directly and isolate
where cc-2 leaks into the unconsumed pushback."

Added `LATEXML_TRACE_BAL=1`-gated [BAL-IN/BAL-BEG/BAL-END/BAL-OUT/ARG-IN/
ARG-FT/ARG-OUT] tracer in `gullet.rs` (committed temporarily, then
reverted — see git history for the patch). Trace confirmed:

**read_balanced consumes cc-1/cc-2 SPACE correctly as group syntax.**
Body's 8 cc-1 + 8 cc-2 SPACE pairs are read CLEANLY by the read_arg
calls of `\iow_now:Nn`'s n-arg, `\__kernel_iow_with:Nnn`'s n-arg, etc.
No leak via read_balanced.

The 7 boxing-mismatch errors fire on REAL `}` tokens (cc-2 with text
`}`, NOT cc-2 with text SPACE). Trace immediately before first
boxing error:
```
[ARG-IN]  d=1 exp=Off
[ARG-FT]  d=1 first_tok=BEGIN/{       ← regular `{`
[BAL-IN]  d=1 exp=Off mdef=false ropen=false pb=150
[BAL-END] d=1 tok=END/} level: 1->0   ← regular `}`
[BAL-OUT] d=1 ntokens=11 unbal_level=0
   pb_top=["END/}", "OTHER/E", "OTHER/r", "OTHER/r", "OTHER/o",
           "OTHER/r", "OTHER/:", "LETTER/:"]   ← `}` then "Error:..."
[ARG-OUT] d=1 ntokens=11

Error:unexpected:} Attempt to close boxing group; current frame is
   non-boxing group due to T_CS[\group_begin:]
```

The leftover `}` followed by literal "Error: foo" text is the
**already-defined error message** template, expanded into pushback
via `\msg_error_text:n` / `\msg_format_*` chain. Stomach reads the
text from pushback for digestion; on encountering each `}`, calls
egroup() which fails against the `\group_begin:`-frame.

## Perl baseline comparison

Same probe (`\documentclass{article}\usepackage{expl3}\ExplSyntaxOn
\msg_new:nnn{cmd}{define-command}{x}\ExplSyntaxOff\begin{document}
hi\end{document}`) in Perl LaTeXML produces:
- **1 error** total: `LaTeX Error: Message 'define-command' for module
  'cmd' already defined.`
- **0 boxing-mismatch errors**

Perl's `\errmessage{}` (DefPrimitive) reads its `{...}` arg via the
`{}` parameter type → `read_balanced(0, 0, 1)`. Inside that read,
ALL cc-1/cc-2 SPACE tokens AND nested `{...}` get consumed as group
syntax / content. The ENTIRE message (including its trailing braces)
is consumed by `\errmessage`'s arg. Then `\group_end:` closes the
non-boxing frame cleanly.

Rust's `\errmessage{}` is structurally identical
(`latexml_engine/src/tex_debugging.rs:59`). So why does Rust
produce the cascade? Hypothesis: a divergent flow control PATH or
something earlier in the chain emits text-content `}` tokens that
escape `\errmessage`'s read_balanced scope.

## Updated next-steps

The bug is NOT in read_balanced. The bug is in the macro-expansion
chain that produces the error-message text — somewhere in the
`\msg_error → \msg_error:nnnn → \__msg_use:nnnnnnn → \__msg_use_code:
→ \__msg_error_code → \__msg_interrupt:NnnnN → \__msg_interrupt:n`
chain, an extra `}` is being emitted to the input stream that sits
OUTSIDE the `\errmessage`-arg's brace-balanced scope.

Next iteration: trace the message-text expansion (search for where
"Error:" text gets pushed) — likely via instrumentation of `unread*`
or `pushback` writes during `\__msg_*` macro expansion.

## Original (2026-04-26 iteration A)

Single duplicate `\msg_new:nnn{cmd}{define-command}{x}` in `\ExplSyntaxOn`
context produces:
- **7 boxing-mismatch errors per `\__msg_interrupt:n` call** (linear:
  1→7, 2→14, 3→21).

Instrumented `bgroup()`/`egroup()` shows pattern per call:
- 5 `BGROUP-OPEN depth=2->3 tok={` events (cc-1 SPACE tokens that DO
  reach stomach as proper open-group)
- 7 `BOXING-MISMATCH depth=2 cur_tok=} cur_tok_cc=END initiator=\group_begin:` events

Body decode (`\__msg_interrupt:n`):
- 8 catcode-1 SPACE tokens (`Token(' ',1)` in Perl Dumper notation)
- 8 catcode-2 SPACE tokens (`Token(' ',2)`)
- 1 catcode-13 ACTIVE space (`TA(' ')`) — NOT T_ALIGN, the `TA` SUB
  in Dumper.pm L243 is `CC_ACTIVE`, not `T_ALIGN` (the `$TA` VAR)
- 44 catcode-12 OTHER spaces (`O(' ')`)

Expected: cc-1 / cc-2 SPACE pairs structurally pair within macro
arg-reading (Nn = N + n parameter spec consuming the cc-1 + content +
cc-2 of the n-arg). Only 1 cc-2 should reach `\group_end:` to close the
explicit `\group_begin:`-frame.

Observed: 5 cc-1 leak past arg-reading and reach `bgroup()` as
boxing-frame opens. 7 cc-2 leak and reach `egroup()` against a
non-boxing frame, firing the boxing-mismatch error.

The 5 vs 7 asymmetry is the bug: the `n` parameter type's
`read_balanced` (gullet.rs L825-839) tracks cc-1/cc-2 levels for arg
boundary detection. Either it consumes cc-1 SPACE but leaves the cc-2
SPACE in pushback, OR macro-body unread reverses the order, leaking
cc-2 first.

`\iow_term:n` in dump (`E ... 0 LP \iow_now:Nn \c_term_iow`) is
verified Perl-faithful — body has NO `#1`, expl3-code.tex L11133 reads
`\cs_new_protected:Npn \iow_term:n { \iow_now:Nn \c_term_iow }`.
The pass-through pattern relies on `\iow_now:Nn` reading its 2nd arg
from input, which would consume the cc-1 / cc-2 properly IF
`\iow_now:Nn`'s `n` parameter reading is correct.

---

## Cross-references

- `LaTeXML/blib/lib/LaTeXML/Package/expl3.sty.ltxml` — 3-line baseline
- `LaTeXML/blib/lib/LaTeXML/Package/expl3.lua.ltxml` — 124-line lua intarray
- `latexml_package/src/package/expl3_sty.rs` — 229-line current state (76× bloat)
- `latexml_package/src/package/expl3_lua.rs` — 171-line current state
- Token encoding in dump — see `latexml_core/src/dump_writer.rs` /
  `dump_reader.rs` for catcode-prefixed token format
  (`1: ` = BEGIN with space, `2: ` = END with space)
- Reverted commits: `cb406bd40`, `a7cfa5827`
- Previous fix attempts (now reverted): `959d25e9d`, `e2ad374f7`
- The DEFERRED_COMMANDS alias fix `e3d4f8532` shows a similar pattern:
  catcode/identity-based gates should also handle aliases and
  context-dependent token semantics.
