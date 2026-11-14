# Upstream sync — brucemiller/LaTeXML master #2833 → 0db6f47e (2026-11-14)

The 14 upstream master commits after #2833, through `0db6f47e` (#2875), against latexml-oxide main `0340aff82e0`.
Classified by the audit UPSAUD (2026-11-14); the four missing ones ported by UPS1 on branch `sync/upstream-0db6f47e`,
one commit per upstream PR. The method is the last sync's (`docs/archive/UPSTREAM_SYNC_2767_to_2833_2026-06-26.md`):
read the Perl diff (`git -C LaTeXML show <commit>`), port faithfully, move the goldens to Perl master's.

| commit | PR | change | state | where / guard |
|---|---|---|---|---|
| bc90e36c | #2832 | `\relax` after the altcolumn template in `\lx@alignment@multicolumn` | superseded | the template is spliced in place (`tex_tables.rs`, OD #474/#559) |
| 705fe887 | #2835 | multline alignment, `\shoveleft`/`\shoveright`, mathtools shove `\kern` | ported | `8d9189f7e46`; `amsdisplay.tex` = upstream; `mathtools_sty.rs` |
| 9f8d313a | — | `\plpaarsep` → `\plparsep` | ported | `paralist_sty.rs` |
| b17cc621 | — | rewrite `replace` math unwraps a one-item List | ported | `8d9189f7e46` |
| d666adf8 | #2829 | framing: `framedProperties`, `\lx@framed` keyvals, CSS padding dropped | ported | `tex_box.rs` `framed_properties`; `LookupDimension` of a `\def`-ized length diverges (KPE #41) |
| 9f3fa9fc | #2837 | `\hdotsfor[]{N}` spans N columns | ported | `amsmath_sty.rs` |
| abd6adb9 | #2846 | clear `inPreamble` before `\AtBeginDocument` | superseded | reverted upstream by #2848 (KPE #43) |
| 27db0d96 | #2845 | lozenge code points, `ifgeo` fallback, StandardMetrics regenerated | **UPS1** | code points `e5a443de933`; metrics: `standard_metrics.rs` msam5-10 key U+25CA/U+29EB, `ifgeo10` added; guard `perfect_kernel_batchups1::lozenge_widths_are_msam_glyph_widths` |
| 0d02309d | — | llncs `\toctitle{}` → `\toctitle` | ported | `llncs_cls.rs` (witness 2112.13058) |
| 1718f89e | #2855 | archive output keeps an existing log; the zip's log member is the `--log` name | **UPS1** | `latexml_oxide.rs` `external_log`; EPUB keeps its log outside the container (OD #1037); guard `cluster_cli::archive_output_keeps_an_existing_log` |
| 5f6b9fd0 | #2852 | standalone subfile `\documentclass` options are not packages | ported (ours first) | `6dabff6d189`, OD #63 |
| 7412d564 | #2820 | `\fboxsep` padding for `\fcolorbox`, `\boxed`, listings frames; listings `frame=` from `LST@frame`/`LST@framesep` | **UPS1** | color/xcolor/amsmath `framed_properties(margin \fboxsep)`; listings `lst_frame_sides`; goldens colors/xcolors/listing/sampler; guard `perfect_kernel_batchups1::fboxsep_pads_fcolorbox_boxed_and_listing_frames` |
| 9baaf966 | #2848 | `\par` a no-op only in the raw preamble | ported (ours the origin) | `tex_paragraph.rs` |
| 0db6f47e | #2875 | `<mtable align>`: top → `baseline1`, bottom → `baseline-1` | **UPS1** | `presentation.rs` `pmml_array_inner`; guard `perfect_kernel_batchups1::mtable_align_is_the_first_or_last_baseline` |

Departures recorded with the ports: a listing `frame=` naming no side (empty, `xyz`) draws no frame, as listings does
(Perl frames a rectangle); an EPUB's log stays outside the container and never over an existing file (OD #1037);
MathML Core keeps no `align` on `<mtable>` — the attribute is ported as Perl writes it. Metric.pm's `$em` guard
(#2845) applies when a TFM is read, at StandardMetrics generation; this port reads no TFM at run time (the metrics are
the generated tables), so it has no counterpart. The upstream CSS commits ee4b3640 and 95a77fc0 are taken with PR #844.
