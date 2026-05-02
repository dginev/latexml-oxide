# math0010241 — `\emph{... $$display math$$ ...}` malformed

**Status: out-of-scope** (Perl=19, Rust=33; both engines fail).

## Trigger pattern
The paper has multiple `\begin{EG}\emph{ ... $$Q=\left[\begin{array}{...
\end{array}\right]$$ ... }\end{EG}` blocks where display math
`$$...$$` appears inside an `\emph{...}` text wrapper.

This is fundamentally malformed input — `<ltx:emph>` is an inline-text
element and cannot legally contain `<ltx:equation>` (display math) or
`<ltx:XMArray>` (matrices). Both engines correctly reject it.

## Counts
- Perl=19: 8 × `Error:unexpected:^/_ Script ^/_ can only appear in math mode` + 9 × `Error:malformed:ltx:XMArray <ltx:XMArray> isn't allowed in <ltx:emph>` + 2 × `XMApp`
- Rust=33: 13 × `XMTok-in-emph` + 8 × `XMArray-in-emph` + 7 × `^` + 3 × `_` + 1 × `XMApp` + 1 × inline `XMTok`

The +14 delta is verbosity divergence in malformed-XML reporting:
Rust reports per-position malformed-XMTok inside the offending element
where Perl consolidates them. Same family as `0901.2408_emph_dollar`
out-of-scope entry.

## Verdict
Author error in source TeX. Out-of-scope per the documented
in-scope predicate (Perl LaTeXML on TL2025 with ar5iv preset
produces 0 errors). Both engines correctly reject the input.

## Witnesses
- math0010241 (R=33, P=19)
- See also: 0901.2408 (R=4, P=4) — similar `\emph{...$$...$$...}` pattern.
