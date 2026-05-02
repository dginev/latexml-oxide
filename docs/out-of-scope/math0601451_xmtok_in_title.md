# Out of scope (moved from SYNC_STATUS.md 2026-05-01)

Empirically verified: Perl LaTeXML on TL2025 with --preload=ar5iv.sty --path=~/git/ar5iv-bindings/bindings does NOT produce 0 errors on this paper, so it fails the in-scope predicate ("in scope iff Perl produces 0 errors").

Original SYNC_STATUS.md task content preserved below for future reference.

### 2. math0601451 — `XMTok` / `XMApp` leaking into `<ltx:title>`

1481× `Error:malformed:ltx:XMTok in <ltx:title>` (plus 54×
`XMApp in <ltx:text>`) on a single amsppt + amstex paper. Distinct
from the siunitx XMTok-in-text trigger. Math constructs inside
amsppt's `\title`/`\heading` need `XMText`-wrapped output, not raw
XMath tokens. Scope: `latexml_engine/src/amsppt*` + the digest path
that promotes XMath into text-context elements.

