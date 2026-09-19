# Branch red-tests triage — 2026-09-19

**Queued for a fix (user: "unrelated failures should still be queued for a fix,
we want complete improvement" + "tests must pass on TL 2023–2026, don't bake a
concrete date").** These are host-environment failures on `perfect_kernel`, not
test defects — they pass on a single-tree TL install and in CI.

## The big one: a wrong (vendor-built) dump — RESOLVED

**Measured baseline: 140 tests failed** with the on-disk dump; **16 with the
correct dump.** Root cause: `resources/dumps/latex.2025.dump.txt` had been built
against the **vendor** TL (`/usr/local/texlive/2025`, expl3 dated 2025-11-06 —
whatever was on `PATH` when `make_formats.sh` last ran), but the binary's
in-process libkpathsea reads the **distro** tree (`/usr/share/texlive`, expl3
2026-01-19) at runtime. expl3's dependency check (`expl3.sty:77-78` →
`expl3-code.tex:13096-13134`) errors when the format's baked expl3 date < the
tree's, emitting `Cannot run piped system commands` + `Mismatched LaTeX support
files detected` — **2 extra errors on every expl3-using test** (str/text/regex/
xparse/siunitx/most batch54–56 guards → their `error_count == 0` assertions trip).

**Fix (per-host, no code change): the dump must be built against the tree the
runtime reads.** The DEFAULT `tools/make_formats.sh` (ambient distro `kpsewhich`)
does exactly this — it builds a distro dump matching the runtime, and the 124
date-mismatch failures vanish. The vendor-built dump was a one-off anomaly (vendor
on `PATH` at dump time). Dumps are gitignored/per-checkout, so there is nothing to
commit; the discipline is **run `make_formats.sh` with the runtime tree's
`kpsewhich` on `PATH`** (single-TL hosts and CI do this automatically). No date is
baked into any test — the tests are TL-portable; only the dump/tree pairing was off.

## Residual 16 (fail under BOTH dumps — genuine pre-existing)

Being triaged (real bug vs host-artifact-needing-a-guard vs flaky):

- Font/host-dependent: `fontspec_file_names_and_family_names_resolve`
  (tex-gyre in `/usr/share/texmf`, a tree `coverage.rs texmf_trees()` doesn't
  scan — expanding it churns CJK goldens, so the fix is a test capability-guard),
  `iffontchar_bounds_unicodefonttable_to_font_coverage`, and the CJK/byte-mouth
  cluster (`cjk_octet_readers`, `dhucs_trivcj`, `japanese_otf_kanji_scanners`,
  `kanji_control_words_under_platex`, `kotexutf_runs_under_the_byte_mouth`).
- Other (cause TBD): `mathtools_test`, `new_ifnextchar_keeps_space`,
  `koma_declaresectioncommand_heading_is_a_subsection`,
  `beamer_section_names_slide_counter_and_patch_targets`,
  `installed_ldf_outranks_the_language_stub`, `xkeyval_sets_the_loaded_sentinel`,
  `newpsstyle_defines_the_custom_style_psset_consults`,
  `unbalanced_expansion_is_fatal`, `removed_subtrees_leave_no_c_heap_residue`
  (C-heap/timing — likely flaky on a loaded host).

**Status:** dump issue resolved (140→16); residual-16 triage in flight. Update
with fixes/guards and move to `docs/archive/` when the suite is green.
