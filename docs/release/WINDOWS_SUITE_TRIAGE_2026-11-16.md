# Windows full-suite triage — 2026-11-16 (brief for the Windows session)

Baseline: `windows-ci-manual-trigger.yml` with `require_guards=true` on main `f7e359fe7e1` (run 43793054474,
x86_64-pc-windows-msvc, TeX Live 2026 from setup-texlive-action). All 11 shards fail "run tests": 32 distinct tests, 1-6 per shard
of ~600-1100. PR runs of Windows CI pass because they only `cargo check`. Root-caused read-only from the shard logs.

## Scope of this branch (`fix/windows-suite`)
Fix classes **B, C (EPS only), D, E** below: 6 tests, all Windows-specific. Do **not** fix:
- **Class A (26 tests):** not Windows. It is being fixed on Linux in a separate PR, branch `fix/caption-subtypehook`. Once that merges into
  main, merge main into this branch; those 26 should go green with no Windows change.
- **The `~`-in-file-name engine gap** behind `subimport_absolute_path_resolves_like_real_latex`: it is SHARED (it reproduces on Linux) and
  changes Linux output. Here, only use the test-side alternative given under C.

Every change must leave Linux output byte-identical. Repo rules: `CLAUDE.md` (no `unsafe` in our code; portable tests skip visibly;
whole-element assertions; never delete a witness from a comment).

## Classes

### A. TL 2026's first aid for subcaption meets our caption binding — NOT Windows (26 tests; fixed elsewhere)
- `latex2e-first-aid-for-external-files.dtx` v1.1z (2026/09/27, latex2e gh/2172) runs `\g@addto@macro\caption@subtypehook{…}` when
  subcaption.sty v1.6b loads.
- Our `caption_sty.rs` lacks caption.sty's `\caption@subtypehook` (caption.sty:639, run at :314). Since #829, `\g@addto@macro` on an
  undefined macro is an Error.
- Evidence in every failing log: `First Aid for subcaption.sty applied!`, then
  `Error:undefined:\caption@subtypehook … Defining it with the addition`.
- Tests:
  - `114_streaming_cluster_regressions` (shard0), `114_streaming_cluster_regressions_b` (shard1), `114_streaming_complex`
  - `80_complex::figure_mixed_content_test`
  - `cluster_frontmatter_classes::{openmoss,selfevolagent,fairmeta}_*`
  - `cleveref_class_stubs::lipics_stub_requires_cleveref`
  - `perfect_kernel_batch65w::lipics_*` (4), `batch66h::lipics_*` (3), `batch68w::lipics_*` (3)
  - `batch67u::bmvc2k_class_copy_loads_its_subfigure_package`
  - `batch78v`, `batch92e`, `batchp25`, `batch61::the_2609_classes…`, `batchint23::mdpi_loads_cleveref…`
  - `batch56::bytedance_author_is_a_creator`, `batchsbx1::sandbox_2605_rows`

### B. Windows `\` path separator (2 tests)
- **`latexml_contrib script_bindings::tests::api_reference_is_up_to_date`.**
  - Cause: the reference records `DefMathLigature!` as `D:\a\…\latexml_contrib\..\latexml_engine\src\setup_binding_language.rs`.
    `workspace_macros` builds `CARGO_MANIFEST_DIR/../<krate>/src` with `Path::join` (`latexml_contrib/src/script_bindings/api_doc.rs:3461`),
    then strips at `"/../"` (:3451), which never matches `\..\`.
  - Fix: `path.strip_prefix(&src)`, then join the components with `/` behind `"{krate}/src/"`. Byte-identical on Linux.
- **`cluster_cli whatsinout::format_epub_ar5iv_preload_carries_the_css_param`.** A real EPUB bug on Windows.
  - Cause: the OPF writes `href="sub\x.css"` (id `_sub_x5C_x.css`) where it should write `sub/x.css`. `latexml_post/src/manifest/epub.rs:126`
    uses `to_string_lossy()`.
  - Fix: apply the `.replace(std::path::MAIN_SEPARATOR, "/")` that `add_document` (:171) and `pack.rs:232` already use.

### C. 8.3 short name vs long name (1 test), and a test-side workaround for `~` (1 test)
- **`perfect_kernel_batch56::eps_bounding_boxes_are_read_as_bytes`.**
  - Symptom: `candidates="eps_bbox_latin1.eps,../../../../../RUNNER~1/AppData/Local/Temp/.tmpC55hzZ/eps_bbox_latin1.eps"`.
  - Cause: `SEARCHPATHS` seeds `.` lexically through `pathname::absolute(current_dir)` (`latexml_core/src/state.rs:805-811`), which keeps
    `RUNNER~1`. `SOURCEDIRECTORY` comes from `pathname::canonicalize(request)` (`latexml_oxide/src/core_interface.rs:1235`), which gives the
    long `runneradmin`. `image_candidates` finds the file in both, relativizes the short-name hit against the long name
    (`latexml_core/src/util/image.rs:136`), and its string dedup (:249) keeps both.
  - Fix: seed `.` with `pathname::canonicalize(current_dir)`, falling back to `absolute`. Linux is unchanged because `getcwd` is already
    physical. Do NOT canonicalize `--path` dirs or image hits: that would resolve symlinks and change Linux output. `dunce` is not needed;
    `pathname::canonicalize` already strips `\\?\` (`pathname.rs:568`).
- **`06_cluster_standalone_subfiles::subimport_absolute_path_resolves_like_real_latex`.**
  - Cause: the test writes the absolute short path, containing `RUNNER~1`, into the TeX source. Our `~` becomes `\lx@NBSP` inside file names
    (`plain_constructs.rs:472`; `import_sty.rs:81-104` uses `Expand!`), whereas LaTeX makes it a literal `~` (latex.ltx:9413-9419, 19727-19730).
  - This is the shared engine gap (out of scope here). Branch fix: the test canonicalizes the temp directory (long form, no `~`) before writing
    the path.

### D. `in_tex_live` on Windows (1 test, test-only)
- **`perfect_kernel_batch68d::author_file_stand_ins_are_bindings_outside_tex_live`.**
  - Symptom: `AUTHOR_FILE_STAND_INS: ieeeconf.cls is a TeX Live file`. Windows kpathsea finds TL's `IEEEconf.cls` on a case-insensitive file
    system.
  - Fix (`latexml_oxide/tests/cluster_package_guards/perfect_kernel_batch68d.rs:152-172`): require that a kpsewhich hit's `file_name()` equals
    `file` exactly. Split the expand-path output on `;` on Windows and `:` elsewhere (:164).
  - Unverified: whether `\documentclass{ieeeconf}` on Windows treats TL's `IEEEconf.cls` as the paper's copy. Check it.

### E. The xslt test builds a malformed Windows file URL (1 test, test-only)
- **`latexml_post xslt::local_stylesheet_tests::only_a_named_file_is_read_from_disk`.**
  - Cause: `file://C:\Users\RUNNER~1\…\100%25\LaTeXML-html5.xsl` gives `left: None`. The test concatenates `file://` with a `C:\…` path
    (`latexml_post/src/xslt.rs:294`, :297-306). Production stylesheet loading works.
  - Fix: build `file:///` plus the path with `\` turned into `/`.
  - The next sub-check (`file://server…` expects `is_some() == cfg!(windows)`) needs a real share. Factor the URL-to-path mapping out of
    `disk_path` (:1053) so it is testable without the file-exists check, or assert `None` for a non-existent server on every platform.

## History
- Run 43565541569 (2026-11-14, branch ci/windows-latest-texlive) failed exactly the 4 tests in C, D and E. So these predate the perf merge.
- Run 43517730516 (2026-11-13) already printed "First Aid for subcaption.sty applied!" but no error. Class A became an Error with #829.
- Class B is new: the cheat-sheet file cell came in 5bf0bc52d5f, the EPUB assertion in #840.

## Verify
- On Windows: `cargo nextest run --workspace` (TL 2026 on PATH), or the targeted tests above.
- Then dispatch `gh workflow run windows-ci-manual-trigger.yml --ref fix/windows-suite -f require_guards=true` (~40 min).
- Expect green on B/C/D/E now, and on A once main (with `fix/caption-subtypehook`) is merged in.
- On Linux the full gates must stay green with byte-identical output.
