# Mini Plan: Round 3

## Status: 217 pass, 0 fail, 62 ignored

## Three most connected work packets

### Selection rationale
Looking at the remaining 62 ignored tests, most are blocked by the math parser (text= diffs)
or equation numbering (tags diffs). I'll focus on tests that have structural diffs fixable
without math parser changes, and crashes that can be resolved.

### Packet 1: Fix `cd_test` — port amscd.sty binding (56_ams)
- **Current state**: 199 diffs after panic fix. Most diffs are structural — missing `ltx:XMApp` elements from the CD (commutative diagram) environment.
- **Root cause**: amscd.sty binding not ported. Perl has amscd.sty.ltxml.
- **Approach**: Port the amscd.sty.ltxml binding. It's a small package (~100 lines) defining `\CD`, `\@>`, `\@<`, `\@A`, `\@V`, `\@=`.
- **Expected**: Many diffs should resolve with the binding ported.

### Packet 2: Fix `mathtools_test` — port mathtools.sty binding (56_ams)
- **Current state**: TooManyErrors (>100 undefined tokens like \radical, \ext@arrow, \arrowfill@).
- **Root cause**: mathtools.sty.ltxml not ported. It redefines many amsmath commands.
- **Approach**: Port mathtools.sty.ltxml. Focus on the core definitions — the package is moderate size.
- **Expected**: Reduce errors, see actual test output.

### Packet 3: Fix `matrix_test` — afterConstruct + equation numbering (56_ams)
- **Current state**: 176 diffs. afterConstruct is ported. Diffs likely from matrix environment structure.
- **Root cause**: ams matrix/cases environments need proper MathFork generation.
- **Expected**: Assess remaining diffs after tex= fix from previous round.

### Execution order
1. Port amscd.sty binding → cd_test structural fix
2. Port mathtools.sty binding → mathtools_test error reduction
3. Analyze matrix_test remaining diffs → targeted fixes
