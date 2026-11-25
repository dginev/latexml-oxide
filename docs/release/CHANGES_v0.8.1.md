# latexml-oxide 0.8.1: the full list of changes

The release summary will be the `## [0.8.1]` section of [`CHANGELOG.md`](../../CHANGELOG.md). This file keeps every
entry the release's pull requests wrote, with its guard tests and design references (`OXIDIZED_DESIGN_DIVERGENCES #N`
is [`docs/parity/OXIDIZED_DESIGN_DIVERGENCES.md`](../parity/OXIDIZED_DESIGN_DIVERGENCES.md)); a new entry goes at the
top of its group.

## Changes

### Unreleased

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
