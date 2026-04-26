// Perl: Base.pool.ltxml
//
// Mirrors `LaTeXML/blib/lib/LaTeXML/Engine/Base.pool.ltxml` 1:1.
//
// Loaded by `Core.pm::iniTeX` (default `mode = 'Base'`) before
// `DumpFile` or runtime processing. Establishes the TeX subsystem:
// schemas, parameter types, utilities, XMath, the TeX_* registers
// + primitives, eTeX + pdfTeX extensions, and Base_Deprecated.
//
// IMPORTANT: This pool deliberately does NOT include autoload triggers
// for LaTeX/expl3/AmSTeX, the `\documentstyle` macro, or
// `LoadFormat('plain')` — those live in TeX.pool.ltxml (`tex.rs`).
// Splitting them lets `ini_tex.rs` (dump-build) load only Base.pool
// before snapping, exactly like Perl's `iniTeX → DumpFile` flow.

use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl Base.pool.ltxml L26-29
  InnerPool!(base_schema);
  InnerPool!(base_parameter_types);
  InnerPool!(base_utilities);
  InnerPool!(base_xmath);

  // Perl Base.pool.ltxml L30-48 — TeX subsystem
  InnerPool!(tex_box);
  InnerPool!(tex_character);
  InnerPool!(tex_debugging);
  InnerPool!(tex_file_io);
  InnerPool!(tex_fonts);
  InnerPool!(tex_glue);
  InnerPool!(tex_hyphenation);
  InnerPool!(tex_inserts);
  InnerPool!(tex_job);
  InnerPool!(tex_kern);
  InnerPool!(tex_logic);
  InnerPool!(tex_macro);
  InnerPool!(tex_marks);
  InnerPool!(tex_math);
  InnerPool!(tex_page);
  InnerPool!(tex_paragraph);
  InnerPool!(tex_penalties);
  InnerPool!(tex_registers);
  InnerPool!(tex_tables);

  // Perl Base.pool.ltxml L49-50
  InnerPool!(etex);
  InnerPool!(pdftex);

  // Perl Base.pool.ltxml L52
  InnerPool!(base_deprecated);
});
