//! Single-shot package / engine regression guards.
//!
//! One test binary (one link unit, for CI economy) whose modules live one
//! per file beside this root: `cluster_package_guards/<module>.rs`. Test
//! paths are `<module>::<test>`, as when the modules were inline. All
//! members are subprocess- or few-conversion tests, so co-locating them in
//! one process stays far under the RSS fuse.

#[path = "../cluster/mod.rs"]
mod cluster;
#[path = "../common/mod.rs"]
mod common;

#[allow(unused_imports)]
pub(crate) use perfect_kernel_batch46::{
  convert, convert_files, convert_files_with, convert_with, convert_with_budget, error_count,
};

mod aastex_contribution_appendix;
mod accent_meaning_robust_shape;
mod acmart_description_aria;
mod aligned_overset_includestyles;
mod bib_field_digest_once;
mod biblatex_fallback_no_cite_loop;
mod bibliography_crossref;
mod bibliography_names_fields;
mod bibref_show;
mod binding_singletons_56;
mod braced_quantity_tail;
mod case_change_equivalents;
mod cleveref_class_stubs;
mod currsize_default;
mod deferred_load_retry;
mod defplain_skips_blanks_before_brace;
mod document_indirection;
mod dump_gate_init;
mod expanded_protected_brace_hunt;
mod expl3_nested_raw_load_catcodes;
mod fatal_salvages_partial_document;
mod filelist_letter_catcodes;
mod forest_chemnum;
mod frontespizio_inline;
mod graphics_asset_memo;
mod graphics_kpsewhich;
mod graphicx_internals;
mod greek_text;
mod href_edef_loop;
mod href_semiverbatim_loop;
mod hyperlink_bounded;
mod hyperref_colorlinks;
mod ifnextchar_scans;
mod lstinputlisting_range_crlf;
mod lua_state_mirror;
mod luacode_bridge;
mod luatex_babel_api;
mod luatex_direction_scan;
mod luatex_profile;
mod makeindex_allocates_indexfile;
mod memoir_output_streams;
mod natbib_label_dotless_i;
mod neurips_anonymous;
mod newtcblisting_verbatim;
mod nfss_font_state;
mod nicetabular_binding;
mod node_box_append;
mod noexpand_input_ends;
mod nul_byte_input;
mod openright_kernel_contract;
mod package_leads_56;
mod picture_makebox_offset;
mod picture_sizing;
mod preclass_kernel_autoload;
mod process_key_options_sees_load_options;
mod pstricks_drawing;
mod raw_classoptionslist_recorded;
mod raw_provides_version_survives;
mod rawclasses_binding_precedence_and_no_omnibus;
mod shipout_parskip;
mod silence_keeps_diagnostics;
mod stex_raw_ltxml;
mod subdir_dispatch_no_strip;
mod texinputs_usepackage;
mod unicode_caret_notation;

/// A no-dump (degraded raw-load) conversion of an expl3-using document must
/// still succeed. Witness issue #651: a bare `\usepackage{fvextra}` reported
/// "Conversion failed: 1 fatal error" under DEGRADED mode even though the output
/// (`<p>text</p>`) was correct. The failure was a benign expl3-code.tex
/// group-end codepoint cascade (L33074-33180) that the dump avoids and Perl's
/// own raw-load never produces, leaking a fatal into the conversion status.
/// `expl3_sty.rs` now snapshots the error report across the degraded raw expl3
/// load (gated on `raw_load_will_run`, so dump mode — which short-circuits the
/// re-load — is untouched). The fixture uses `\usepackage{expl3}` directly (the
/// l3kernel is always present, unlike a trimmed-TL `fvextra`), which triggers
/// the same raw-load path.
///
/// Linux-only: the degraded raw-load re-runs the whole ~33k-line expl3-code.tex,
/// which under the unoptimized `ci`/`dev` test profile takes ~2 min (measured
/// 124 s local `dev`); the behavior is OS-independent, so guarding one platform
/// keeps the ~2-min cost off all four macOS shards. It also needs an explicit
/// `--timeout` far above the 60 s CLI default (which is calibrated for the fast
/// dump path): with the default, the legitimate slow bootstrap is killed as a
/// `Fatal:timeout:wallclock` before the load can finish. Release-optimized
/// binaries — what real degraded users run — complete the same load well under
/// 60 s, so this large timeout is purely a slow-test-build accommodation.
#[cfg(target_os = "linux")]
mod expl3_degraded_no_dump;

/// Regression: minted's `\newmintinline`/`\newminted`/`\newmint` take an optional
/// `[env-name]` before the two mandatory `{language}{options}` args (real
/// minted.sty: `\newcommand{..}[3][]`). The Rust binding declared them with a
/// two-mandatory `#1#2` signature, so `\newminted[leancode]{lean4}{...}` captured
/// `#1 = "["` and ran `\expandafter\def\csname [\endcsname{...}` — and `\csname
/// [\endcsname` IS the control sequence `\[`, silently redefining display-math open
/// to `\begin{lstlisting}`. Every later `\[` then opened a listing that ran to
/// end-of-file, swallowing the body and `\bibliography` with no `Error:` — a silent
/// bibliography loss. Witness `arXiv:2606.05629` (issue #520). The reporter's first
/// hypothesis (the comment-wrapped `leancode` env runs away) is not the cause: that
/// block sits inside `\begin{comment}` and is discarded; the runaway starts at the
/// first `\[` far earlier.
mod minted_newminted_optional_env_name;

mod alignment_ledger_expansion_pushback;
mod autoload_trigger_identity;
mod expl3_state_and_param_replay;
mod input_routing_and_bbx;
mod kernel_language_and_part_contracts;
mod math_text_font_restore;
/// minted binding quality (follow-up to #520): the inline `\mintinline{lang}{code}`
/// brace form must render without erroring or swallowing following content, and
/// `\begin{minted}{language}` must activate listings' syntax highlighting.
mod minted_inline_and_highlighting;
mod overpic_renders_graphic_and_overlays;
mod perfect_kernel_batch40_43;
pub(crate) mod perfect_kernel_batch46;
mod perfect_kernel_batch47;
mod perfect_kernel_batch48;
mod perfect_kernel_batch49;
mod rtoken_patchcmd;
mod scanner_status;
mod token_kernel_gaps;
mod unicode_format_encoding;
mod vfs_file_end;
mod xkeyval_internals;

mod accent_composite_expansion;
mod beamer_includegraphics_overlay;
mod counter_id_formatters_from_the_dump;
mod datatool_native_load;
mod etex_logo;
mod glossary_refs_post;
mod kernel_fallbacks_never_block_newcommand;
mod latex_via_exemplos_residue;
mod listings_virtual_file;
mod mbox_argument_is_bounded;
mod noexpand_marker_texweb;
mod nomencl_inline;
mod nomencl_nomentbl;
/// Red/green guards for perfect-kernel batch 50 (PLANS P50 …).
mod perfect_kernel_batch50;
mod perfect_kernel_batch51;
mod perfect_kernel_batch52;
mod perfect_kernel_batch53;
mod perfect_kernel_batch54;
mod perfect_kernel_batch55;
mod perfect_kernel_batch56;
mod perfect_kernel_gemini;
mod pgfkeys_native_accessors;
mod raw_class_stores_reroute_to_frontmatter;
mod svg_nested_picture;
mod wrapstuff_inline_float;

/// Guards for the sandbox-arxiv-2605 rerun regression clusters (2026-09-19).
/// Each reproduces a cluster of arXiv papers that regressed against the
/// 2026-08-23 baseline and asserts the fix. Root-cause notes:
/// `~/data/pk_agents/w23/regress_2605/CLUSTERS.md`.
#[cfg(test)]
mod regress_2605_clusters;

/// The TeX Live 2025 class census (2026-09-24, `docs/perfect_kernel/LEDGER.md`):
/// every TL class in a hello-world document, raw-loaded. Each guard is the
/// minimal repro of one failure cluster among the usable classes.
mod class_census;
