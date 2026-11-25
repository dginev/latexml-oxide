# latexml-oxide 0.8.1: the full list of changes

The release summary will be the `## [0.8.1]` section of [`CHANGELOG.md`](../../CHANGELOG.md). This file keeps every
entry the release's pull requests wrote, with its guard tests and design references (`OXIDIZED_DESIGN_DIVERGENCES #N`
is [`docs/parity/OXIDIZED_DESIGN_DIVERGENCES.md`](../parity/OXIDIZED_DESIGN_DIVERGENCES.md)); a new entry goes at the
top of its group.

## Changes

### Unreleased

  - **Tests: guards read whole elements.** The shared test helpers (`latexml_oxide/src/util/test.rs`) gain
    `xml_elements`, `element_count`, `assert_all_elements`, `assert_no_element` and `markup_text`, which match an
    attribute at its start (`id="p1"` is not `xml:id="p1"`), skip comments, CDATA and processing instructions, and read
    an element whole; `assert_no_element` also fails on a matching start tag left open. Listing guards pin whole
    `<listingline>` elements. `LATEXML_TEST_SHADOW=<absolute dir>` logs every `xml_element` call that disagrees with the
    new reading (0 on the whole suite) and fails when it cannot write. No conversion output changes.
  - **amsppt (AMS-TeX preprint) references keep their fields.** Each `\ref` field is digested as amsppt's `\makerefbox`
    boxes it, so field macros that assign (math0610862's `\noOf`) no longer misfire: math0610862 and math0205237 no
    longer end in `Fatal:TooManyErrors` (101 Errors → 2; +1,985 and +2,144 words), and math9204232's 5 references are
    back. A field that prints only a rule (`\bysame`) is kept; a `\ref` left open is closed by its next `\par`, `\ref`
    or `\endRefs` with AMS-TeX's Error and every field kept; `\newline` in a reference is a line break; a field marker
    inside a group the field opened no longer invents a picture; the `\paper` italic no longer leaks over the journal
    and year. Guards: `perfect_kernel_batchamsppt`.
  - **hyperref keeps the kernel's `\refstepcounter` on a format from 2024-11-01, as hyperref.sty does, and sets
    cleveref's label data through its `hyperref/fixcleveref` plug.** A document's
    `\let\H@refstepcounter\refstepcounter` no longer loops (2609.30372: a `Timeout:Recursion` Fatal, now no Fatal);
    with cleveref loaded before hyperref only cleveref's own load-order Error remains, as in pdflatex (1811.08194,
    1909.08399), and its typed `\refstepcounter[thm]{myc}` no longer prints `thm]myc`. Such documents now run cleveref's
    own `\refstepcounter`, as pdflatex does (1811.08194: +7.3 % instructions, the cost the other load order already
    paid). Guard: `perfect_kernel_batchloopfix::hyperref_leaves_refstepcounter_to_the_kernel`.
  - **Fewer `unsafe` blocks (issue 937, phase 2 step 1).** The glibc heap statistics (`mallinfo2`) and mimalloc's page
    purge (`mi_collect`) are each called from one safe function in `heap_trim.rs` (`c_heap_mb`, `collect_rust_heap`);
    their other call sites are safe code: 69 → 65 `unsafe` keywords. Output byte-identical; instructions and syscalls
    unchanged.
  - **An `\item[label]` in `{thebibliography}` is an entry labelled as written,** as LaTeX prints it (asl.cls author
    groups, backref's `\item[\@biblabel{1}]`, `\item[]`, footnote marks): the label was printed twice ("[1] [[A1]] …")
    and each later entry, and its citations, numbered one too high, as in Perl (KNOWN_PERL_ERRORS #1335). Witnesses
    1005.4193, 0705.0395, 1211.3435, quant-ph0409153 (`\cite{audenaert}` now [25], as the PDF).
  - **manyfoot, nccfoots and bigfoot are bound.** A declared series' `\footnote<X>` is a footnote marked as the series
    prints it (its counter and `\thefootnote<X>` style, `[<n>]`, a minipage too); `\footnotetext<X>` and
    `\Footnotetext<X>{<mark>}` are footnote texts marked as given, never taken for the text of a main `\footnotemark` of
    the same number; bigfoot's series too. The `\endminipage` Warning on every manyfoot or bigfoot load is gone, and
    manyfoot's begin-document definitions (`\MFL@fudgefactor`, `\MFL@paraskip`, `\defaultfootnoterule`) are kept for the
    packages that read them. Witnesses 2511.01971 (an e-mail line in `\author`, lost before), 2609.01974. Guard:
    `footnote_series::footnote_series_rows`.
  - **Identical images keep their own paths.** Two `\includegraphics` of different files with the same bytes each
    link their own file, as Perl does (`Post/Graphics.pm` L289-300), also under `--sitedirectory`; before, both linked
    the first file's path, so a page silently depended on another folder's file. Converted images and generated `xN`
    copies still share one file per content (OXIDIZED_DESIGN_DIVERGENCES #24).
  - **One longtable caption.** A `\caption[]{… (continued)}` in `\endhead` no longer shows as a second caption at the
    table's foot, and a `\endfoot` caption after `\endlastfoot` no longer moves to the head (Perl does both;
    KNOWN_PERL_ERRORS #1333).
  - **`\usepackage[svgpath={figs/}]{svg}` finds its SVGs,** reading the option as svg.sty's `\svgpath{{…}}`
    (svg.sty:546-556). As in pdflatex, the option holds one path: `svgpath={{a/}{b/}}` is not a list.
  - **Split pages no longer overwrite each other or leave the destination.** Under `--splitnaming=label`/
    `labelrelative`, two labels that name the same file (`dup` and `dup/`, `./x` and `x`, or `index` beside
    `index.html`) wrote both pages to one file, losing one (1502.05726 lost two appendix subsections); the second page
    now takes its id (`A1.SS2.html`), with a Warning. A label that would put its page outside the destination
    (`\label{../esc}`) is an Error, and the page is written inside under its id. Under `labelrelative` an unlabelled
    page is named relative to its parent's id, as in LaTeXML (`sec/SS1.html`, was `sec/S1.SS1.html`). One page namer
    serves both split paths (OXIDIZED_DESIGN_DIVERGENCES #1164).
  - **Split pages are named in Perl's order, each page before its children,** on both split paths: when two labels
    give the same file, the earlier page in the document keeps it and the later one takes its id (Perl writes both to
    one file, which keeps the later page). Under `--splitnaming=labelrelative`, a page whose label climbs out of its
    parent's folder (`\label{../up}`) now gets links that resolve. Collision and escape messages print page paths
    relative to the destination, EPUB included, and print a bare destination's directory as `.`.
  - **Eleven fewer `unsafe` sites (issue 937, phases 1 and 3):** the cortex log mirror, the crash-cause record, `getpid`
    and the watchdog's stderr line use safe `rustix` calls; `getrusage` has one call site; `--init` no longer writes
    `LATEXML_INI_MODE` into the process environment (an in-process switch; the variable is still read). 69 `unsafe`
    keywords remain (80 before). Output is byte-identical; user instructions −0.4/−0.5 %, task-clock and system-call
    counts unchanged.
  - **`\graphicspath` and `svgpath` entries that are absolute on Windows (`C:/figs/`) are searched as written.** They
    were recognized by a leading `/`, so a drive-letter entry was joined under the source directory and its figures
    went missing; both now ask `pathname::is_absolute`, Perl's `pathname_is_absolute`. Guard:
    `perfect_kernel_batch79g::an_absolute_graphics_path_is_searched_as_written` (red on Windows).
  - **The graphics cache delivers its images by copy, not hard link, and prunes to 90 % of its cap.** An output image
    edited in place no longer rewrites the cache entry and every other document's copy; an insertion no longer walks
    the whole cache (only past the cap, by a usage tally). Output byte-identical. Guards:
    `graphics_cache::tests::{an_output_edited_in_place_leaves_the_cache_and_its_siblings,
    a_prune_leaves_the_low_water_mark_and_a_later_insertion_walks_nothing}`.
  - **Three fewer `unsafe` sites (issue 937, phase 5):** the `unsafe impl Send/Sync` markers on `State` and `Stored` are
    gone; the ratchet `unsafe_sites_ratchet::unsafe_sites_only_fall` falls 80 → 77. Output byte-identical.
  - **cortex_worker: `--max-rss-mb` sets the cooperative memory fuse and the streaming watermarks for standalone and
    pool workers, as it already did for harness children; `LATEXML_RSS_CAP_BYTES` no longer overrides the flag.** A
    worker's memory messages name `--max-rss-mb`, or no flag, where they advised `--max-memory`, which `cortex_worker`
    does not accept; `tools/benchmark_canvas.sh` passes the fleet child's 5376 MiB. Guards: `cortex_worker`
    `cli_tests::max_rss_mb_sets_the_fuse_and_the_watermarks_in_every_worker_mode`,
    `cortex_worker_untrusted_bindings::a_standalone_workers_max_rss_mb_sets_its_fuse`, `stomach::memory_cap_tests::
    {the_fuse_fatal_names_the_knob_that_set_it, the_box_cap_fatals_name_the_knob_that_set_it}`,
    `watchdog::tests::the_oom_fatal_names_the_knob_that_set_it`.
