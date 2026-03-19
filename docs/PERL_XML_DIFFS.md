# Perl vs Rust Test XML Differences

> Comprehensive comparison of `LaTeXML/t/*.xml` (Perl ground truth) and `latexml_oxide/tests/*.xml` (Rust expected output). Last audited 2026-03-19.

## A. `%&#10;` only — intentional Rust divergence (30 files)

Rust does not emit `%\n` (TeX comment-newline separator) in `tex=` attributes. Documented in `CLAUDE.md`. No action needed.

1. `fonts/acc.xml`
2. `fonts/mathaccents.xml`
3. `fonts/stmaryrd.xml`
4. `fonts/wasysym.xml`
5. `math/arrows.xml`
6. `math/declare.xml`
7. `math/fracs.xml`
8. `math/not.xml`
9. `math/sampler.xml`
10. `math/testscripts.xml`
11. `alignment/diagboxtest.xml`
12. `alignment/ncases.xml`
13. `theorem/amstheorem.xml`
14. `theorem/ntheorem.xml`
15. `ams/cd.xml`
16. `ams/mathtools.xml`
17. `graphics/colors.xml`
18. `graphics/picture.xml`
19. `graphics/xytest.xml`
20. `parse/kludge.xml`
21. `parse/operators.xml`
22. `complex/figure_mixed_content.xml`
23. `complex/physics.xml`
24. `complex/si.xml`
25. `pgf/stress_pgfmath.xml`
26–30. `tikz/` (all 10 files: 3d-cone, ac-drive-components, ac-drive-voltage, atoms-and-orbitals, consort-flowchart, cycle, dominoes, tikz_figure, unit_tests_by_silviu, various_colors)

## B. Intentional Rust divergences (no action needed)

31. **`tokenize/mathtokens.xml`** — `\cdots` uses `role="ELIDEOP"` in Rust vs `role="ID"` in Perl. Documented in `CLAUDE.md` and `OXIDIZED_DESIGN.md`.

32. **`complex/xii.xml`** — DTD-based output (`<song><verse><line>`) vs standard `<document>` wrapper. Rust has no DTD support (documented).

33. **`parse/parser_speculate.xml`** — Rust produces `f@(x)` (speculative application active), Perl produces `f * x`. Both use `[mathparserspeculate]` option. Perl XML appears outdated.

## C. Rust improvements over Perl (no action needed)

34. **`fonts/mathbbol.xml`** — Rust parses successfully where Perl marks as `ltx_math_unparsed`. Rust has better math parsing here.

## D. Known minor divergences in passing tests

35. **`keyval/keyvalstyle.xml`** — Comma-space in serialized keyval options: Rust `"width=100, height=200"` vs Perl `"width=100,height=200"`.

36. **`parse/multirelations.xml`** — xml:id numbering differences on NUMBER tokens. Rust adds `{}` in `tex=` attribute for parameterless DefMath macros.

37. **`parse/sequences_and_lists.xml`** — xml:id numbering offset by 1 in formulae section.

## E. Perl updated — Rust needs code fixes (tracked in SYNC_STATUS.md)

38. **`fonts/ding.xml`** — Perl updated `guessTableHeaders`: `<thead>` wrapper with `thead="column"`, empty `<td/>` for undefined dingbats. Rust has `thead="row"` and `?` placeholders. (~250 line diff)

39. **`structure/figure_grids.xml`** — Perl now adds `ltx_figure_panel` CSS class to `<graphics>` inside figures. Missing in Rust. (~470 line diff, all from this one class)

40. **`alignment/tabular.xml`** — Header detection differences: Perl has `<thead>` wrapper with `thead="column"`, different border attributes, spacing differences.

41. **`ams/dots.xml`** — Perl recognizes `d` as `meaning="differential-d" role="DIFFOP"` in integral contexts (`\int ... dx`). Rust treats `d` as plain UNKNOWN. Smart dots (⋯ vs …) now implemented.

42. **`graphics/framed.xml`** — Perl generates titled frame heading text (`<text>A Titled Frame</text>`). Missing in Rust.

43. **`graphics/xcolors.xml`** — Multiple issues (~688 line diff):
    - Color complement/wheel computation errors in Rust
    - Missing `pt` units in calc output
    - `colortbl` row cycling broken (all "row 0" instead of cycling 1–9)
    - Missing `ltx_guessed_headers` class on named colors tabular

44. **`complex/aliceblog.xml`** — RDFa support missing. Perl has `<rdf>` elements, `property=`, `typeof=`, `resource=` attributes. Rust produces `<ERROR class="undefined"/>` nodes. (~88 line diff)

## F. Daemon/format differences (low priority, post-processing)

45. **`daemon/fatals/fatal_100.xml`** — Dimension formatting: Rust "0.0pt" vs Perl "0".

46. **`daemon/formats/citationraw.xml`** — Missing `lang="en"` on `<html>`, `Content-Type` casing, LaTeXML logo styling differences.

47. **`daemon/formats/citation.xml`** — Same as citationraw.

48. **`daemon/formats/jats.xml`** — Missing MathML namespace declarations (`xmlns:mml`, `xmlns:svg`, `xmlns:xlink`), minus sign U+2212 vs ASCII hyphen, alignment classes on `<mml:mtd>`.

49. **`daemon/formats/lexmath.xml`** — `<tbody>` wrapper missing, MathML attribute differences (`lspace`, `rspace`, `largeop`/`symmetric`), `class` vs `mathvariant`.

50. **`daemon/formats/mixedmath.xml`** — Same patterns as lexmath plus `%&#10;`.

51. **`daemon/formats/tei.xml`** — `<tbody>` wrapper, MathML minus sign, alignment classes.

## G. Missing files

52. **Missing in Rust** (Perl-only): `daemon/complex/testlocks.xml`, `daemon/formats/dir.xml`, `daemon/formats/whatdir.xml`

53. **Missing in Perl** (Rust-only): `expansion/simple_dimen.xml`, `fonts/mathbbol_perl.xml`, `fonts/mixed_perl.xml`, 7× `alignment/min_listing*.xml`, `graphics/keyval.xml`, `graphics/simplekv.xml`, `daemon/profiles/stex/stex.xml`
