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
