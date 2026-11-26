//! The TeX and LaTeX engine definitions of latexml-oxide: one Rust module per Perl
//! `LaTeXML/Engine/*.pool.ltxml`, defining the same control sequences in the same file.
//!
//! Loading follows Perl. [`tex`] (`TeX.pool`) loads [`base`] (`Base.pool`: the `Base_*` and `TeX_*` pools, eTeX,
//! pdfTeX) and then the plain format; [`latex`] (`LaTeX.pool`) loads `TeX.pool` and then the LaTeX format. Each format
//! loads in one of two ways, never both (Perl `Package.pm` `LoadFormat`):
//!
//! - `bootstrap → dump → constructs` when a format dump built from this TeX tree is found and `LATEXML_NODUMP` is
//!   unset (e.g. [`plain_bootstrap`],
//!   [`plain_dump`], then `plain_constructs`, which also loads `math_common`), or
//! - `bootstrap → base → constructs` otherwise (`plain_base`, [`latex_base`] in place of the dump).
//!
//! [`dump_paths`] and [`embedded_dumps`] find the dumps; [`lua_bridge`] runs `\directlua` in a sandboxed `texlua`.
//! The definer macros (`DefMacro!`, `DefConstructor!`, …) live in `setup_binding_language` and reach binding modules
//! through [`prelude`]. The crate is split from `latexml_package` to keep compile-time memory and rebuilds small.
//!
//! Read next: [ORGANIZATION.md](https://github.com/dginev/latexml-oxide/blob/main/docs/parity/ORGANIZATION.md) (the loading hierarchy) and
//! [DUMP_DESIGN.md](https://github.com/dginev/latexml-oxide/blob/main/docs/parity/DUMP_DESIGN.md) (the dumps).

#[macro_use]
extern crate latexml_codegen;

// Auxiliary compile-time macros (compile_*: prototype/replacement/expansion
// helpers used by setup_binding_language). #[macro_export] makes them
// crate-public; #[macro_use] here makes them visible to engine modules.
#[macro_use]
pub mod macros;

// Main macros (DefMacro!, DefConstructor!, LoadDefinitions!, ...) live in
// setup_binding_language. They use `#[macro_export]` for cross-crate
// reach and `#[macro_use]` here to also be visible to engine's own
// modules without `crate::name!` path prefixes.
#[macro_use]
pub mod setup_binding_language;

// Shared prelude (`crate::prelude::*`) used by every engine module and
// re-exported to `latexml_package` so the original `crate::prelude::*`
// paths in `package/*.rs` keep resolving (via re-export chain).
pub mod prelude;

// Base.pool.ltxml — loaded via Base.pool (a pure loader-of-pools)
pub mod base; // Perl: Base.pool.ltxml (LoadPool('Base_Schema'), ..., 'Base_Deprecated')
mod base_deprecated;
mod base_parameter_types; // Perl: Base_ParameterTypes.pool.ltxml
pub use base_parameter_types::with_unexpandable_protect;
mod base_schema; // Perl: Base_Schema.pool.ltxml
pub mod base_utilities; // Perl: Base_Utility.pool.ltxml
pub mod base_xmath;
pub mod frontmatter_stores; // K11: raw classes' title-page stores → frontmatter API // Perl: Base_XMath.pool.ltxml // Perl: Base_Deprecated.pool.ltxml
// The native answer of l3fp's doors, OXIDIZED_DESIGN_DIVERGENCES #977
pub mod l3fp_native;

// TeX_*.pool.ltxml
pub mod tex_box; // Perl: TeX_Box.pool.ltxml
mod tex_character; // Perl: TeX_Character.pool.ltxml
mod tex_debugging; // Perl: TeX_Debugging.pool.ltxml
mod tex_file_io; // Perl: TeX_FileIO.pool.ltxml
mod tex_fonts; // Perl: TeX_Fonts.pool.ltxml
pub use tex_fonts::read_font_ident;
mod tex_glue; // Perl: TeX_Glue.pool.ltxml
mod tex_hyphenation; // Perl: TeX_Hyphenation.pool.ltxml
mod tex_inserts; // Perl: TeX_Inserts.pool.ltxml
pub mod tex_job; // Perl: TeX_Job.pool.ltxml
mod tex_kern; // Perl: TeX_Kern.pool.ltxml
mod tex_logic; // Perl: TeX_Logic.pool.ltxml
mod tex_macro; // Perl: TeX_Macro.pool.ltxml
mod tex_marks; // Perl: TeX_Marks.pool.ltxml
pub mod tex_math; // Perl: TeX_Math.pool.ltxml (includes tex_scripts content)
mod tex_page; // Perl: TeX_Page.pool.ltxml
mod tex_paragraph; // Perl: TeX_Paragraph.pool.ltxml
mod tex_penalties; // Perl: TeX_Penalties.pool.ltxml
mod tex_registers; // Perl: TeX_Registers.pool.ltxml
pub mod tex_tables; // Perl: TeX_Tables.pool.ltxml

// eTeX + pdfTeX extensions
pub mod etex; // Perl: eTeX.pool.ltxml
pub mod lua_bridge;
pub mod pdftex; // Perl: pdfTeX.pool.ltxml

// plain TeX format — LoadFormat('plain') chain called by tex.rs
mod math_common;
mod plain_base; // Perl: plain_base.pool.ltxml
pub mod plain_bootstrap; // Perl: plain_bootstrap.pool.ltxml
mod plain_constructs; // Perl: plain_constructs.pool.ltxml
pub mod plain_dump; // Rust: precompiled plain.ltx state (auto-generated) // Perl: math_common.pool.ltxml

// Versioned dump filename conventions + ambient-TeXLive year detection.
pub mod dump_paths;

// `cargo:rerun-if-changed` path selection for the dump embed. The build script
// pulls this file in directly via `#[path] mod`; declared here under cfg(test)
// only, so its unit tests run under `cargo test` without adding a public item
// or a dead-code warning to normal library builds (#676).
#[cfg(test)]
mod build_rerun;

// Compile-time-bundled fallback for plain.dump.txt + latex.dump.txt.
// Used when no on-disk dump is resolvable (single-file binary distribution).
pub mod embedded_dumps;

// LaTeX format — LoadFormat('latex') chain called by latex.rs
pub mod latex_base; // Perl: latex_base.pool.ltxml
pub mod latex_bootstrap; // Perl: latex_bootstrap.pool.ltxml
pub mod latex_constructs;
pub mod latex_constructs_rust_only; // Rust-only overrides loaded LAST in latex.rs after latex_constructs.
pub mod latex_dump; // Rust: precompiled latex.ltx state (auto-generated) // Perl: latex_constructs.pool.ltxml (C.1-C.15)
pub mod latex_kernel; // Rust-only: on-demand LaTeX format load for kernel CSes used before \documentclass

// Top-level entry points
pub mod latex;
pub mod tex; // Perl: TeX.pool.ltxml (loads Base + LoadFormat('plain')) // Perl: LaTeX.pool.ltxml (loads TeX + LoadFormat('latex'))

// AmSTeX format — LoadPool('AmSTeX'), routed via `\input amstex`.
pub mod amstex; // Perl: AmSTeX.pool.ltxml

// BibTeX format — LoadPool('BibTeX'), preloaded for `--type=BibTeX`
// or via `LoadPool('BibTeX')` from amsrefs.sty.ltxml.
// Skeleton: full bib defs port pending (see docs/POOL_PARITY_AUDIT.md).
pub mod bibtex; // Perl: BibTeX.pool.ltxml
pub mod pre_bibtex; // Perl: Pre/BibTeX.pm (+ Pre/BibTeX/Entry.pm) — low-level .bib parser

/// Reset all per-conversion thread-local state owned by `latexml_engine` so a
/// persistent worker (`cortex_worker`, LSP) starts every paper clean and never
/// carries BibTeX entries, Lua sessions, hook snapshots, or `SymStr` caches
/// across conversions.
pub fn reset_conversion_state() {
  base_utilities::reset_fm_reading();
  base_utilities::reset_title_page_check();
  pdftex::reset_pdftex_random();
  lua_bridge::reset_session();
  bibtex::reset();
  pre_bibtex::clear_wanted_keys();
  tex_box::reset_splice_state();
  frontmatter_stores::reset_net_hooks_before();
  latex_constructs::reset_conversion_caches();
}
