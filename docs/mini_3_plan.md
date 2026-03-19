# Mini Plan: Round 9

## Status: 225 pass, 0 fail, 94 ignored

## Analysis
All remaining tests need substantial work. The most impactful areas:

## Three most connected work packets

### Packet 1: Implement adjustMathstyle for \over fractions
- **Impact**: Fixes fracs_test (86 diffs), partially helps sizes_test
- **Effort**: Port adjustMathstyle() from Perl — recursive walk of digested boxes
- **Files**: tex_math.rs (add function), constructor afterDigest

### Packet 2: Implement rearrangeEqnarray afterConstruct
- **Impact**: Fixes badeqnarray (151), eqnarray (727), split (2523), amsdisplay (1708)
- **Effort**: Large — DOM manipulation converting _Capture_ to MathFork/MathBranch
- **Files**: latex_ch7, base_xmath.rs, document.rs

### Packet 3: Implement \lxDeclare / math notation system
- **Impact**: Fixes declare_test, simplemath_test, many parse tests (~20+)
- **Effort**: Large — new math declaration infrastructure
- **Files**: math_parser, latexml_sty.rs

### Expected outcome
These are all multi-session tasks. Focus on Packet 1 first (most contained).
