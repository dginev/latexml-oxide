# Organization: the Rust layout for Perl readers

A map from Perl LaTeXML's files (`LaTeXML/lib/LaTeXML/` and `LaTeXML/bin/` in the main checkout) to the Rust crates
and files that port them: where the bindings, the dump, the constructs, the schema, the post-processor and the CLI
live. CLAUDE.md's "Key Concepts Mapping" (`CLAUDE.md`, repository root) names the concepts. This doc is the file-level
map (Bruce Miller Q&A QA85, Q8.1).

**Counts (2026-11-15):** 8 crates (`Cargo.toml` `[workspace] members`). The `.rs` count of each crate's `src/` is
in the first table.

## The crates

| Crate | Perl counterpart | Holds | `src/*.rs` |
|---|---|---|---|
| `latexml_core` | `Core.pm`'s machinery, `Core/*`, `Common/*`, most of `Util/*`, `Package.pm`'s definers and loaders | Mouth, Gullet, Stomach, State, Document, tokens, boxes, definitions, the model and schema, the dump reader and writer | 105 |
| `latexml_codegen` | — (does at compile time what `Core/Definition/Constructor/Compiler.pm` does at run time) | proc macros: constructor templates, the compiled model, parameters, tokens, test harnesses | 6 |
| `latexml_engine` | `Engine/*.pool.ltxml`, `Pre/BibTeX*.pm` | the TeX/LaTeX pools, the binding DSL macros (`DefMacro!` & co), the dump loaders, frontmatter | 165 |
| `latexml_package` | `Package/*.ltxml` | the bindings Perl ships (`*_sty.rs`, `*_cls.rs`, …) and their dispatch table | 552 |
| `latexml_contrib` | — | bindings Perl does not ship; the Rhai runtime bindings | 348 |
| `latexml_math_parser` | `MathParser.pm`, `MathGrammar` | the Marpa math grammar and its semantics | 15 |
| `latexml_post` | `Post.pm`, `Post/*`, `Util/Pack*`, `Util/ObjectDB*` | the post-processors, XSLT, CSS, JavaScript | 40 |
| `latexml_oxide` | `LaTeXML.pm`, `Core.pm`'s session, `bin/*` | the converter, iniTeX and dump building, the CLI binaries, the library API, the LSP server | 33 (+ 15 in `bin/`) |

## `latexml_core`: Core, Common, Util, and Package.pm's machinery

| Perl (`LaTeXML/lib/LaTeXML/…`) | Rust |
|---|---|
| `Core.pm` (`initializeState`, pre/postamble, `convertDocument`) | `latexml_oxide/src/core_interface.rs`, `latexml_oxide/src/converter.rs` |
| `Core/State.pm` | `latexml_core/src/state.rs` |
| `Core/Mouth.pm`, `Core/Mouth/{file,http,https}.pm` | `latexml_core/src/mouth.rs` |
| `Core/Mouth/Binding.pm` (the pseudo-mouth of a `.ltxml` being loaded) | — (a binding is compiled Rust; its `(Loading …)` note: `latexml_core/src/binding/content.rs`) |
| `Core/Gullet.pm` | `latexml_core/src/gullet.rs` |
| `Core/Stomach.pm` | `latexml_core/src/stomach.rs` |
| `Core/Document.pm` | `latexml_core/src/document.rs`, `latexml_core/src/document/` |
| `Core/Token.pm`, `Core/Tokens.pm` | `latexml_core/src/token.rs`, `latexml_core/src/tokens.rs` |
| `Core/Box.pm`, `Core/List.pm`, `Core/Whatsit.pm`, `Core/Comment.pm` | `latexml_core/src/tbox.rs`, `list.rs`, `whatsit.rs`, `comment.rs` (their shared interface: `digested.rs`) |
| `Core/Definition.pm`, `Core/Definition/{Expandable,Primitive,Constructor,Conditional,Register}.pm` | `latexml_core/src/definition.rs`, `latexml_core/src/definition/{expandable,primitive,constructor,conditional,register}.rs` |
| `Core/Definition/CharDef.pm`, `FontDef.pm` | `latexml_core/src/definition/register.rs` (`CharDefProps`), `definition/primitive.rs` (the `\font` id) |
| `Core/Definition/Constructor/Compiler.pm` | `latexml_core/src/binding/def/replacement.rs` (run time), `latexml_codegen/src/constructable.rs` (compile time) |
| `Core/Parameter.pm`, `Core/Parameters.pm` | `latexml_core/src/parameter.rs`, `latexml_core/src/definition/argument.rs` |
| `Core/KeyVal.pm`, `Core/KeyVals.pm` | `latexml_core/src/keyval.rs`, `latexml_core/src/keyvals.rs` |
| `Core/Alignment.pm`, `Core/Alignment/Template.pm` | `latexml_core/src/alignment.rs`, `latexml_core/src/alignment/` |
| `Core/Rewrite.pm` | `latexml_core/src/rewrite.rs`, `latexml_core/src/rewrite/declare.rs` |
| `Core/MuDimension.pm`, `Core/MuGlue.pm`, `Core/Pair.pm`, `Core/PairList.pm` | `latexml_core/src/common/{mudimension,muglue,pair}.rs` |
| `Core/Array.pm` | — (a token stream of `{…}` groups: `latexml_engine/src/base_parameter_types.rs` `CommaList`) |
| `Core/Dumper.pm` | `latexml_core/src/dump_writer.rs` (writes), `latexml_core/src/dump_reader.rs` (applies; see "The dump") |
| `Common/Config.pm` | `latexml_core/src/common.rs` (`Config`); the options: `latexml_oxide/bin/latexml_oxide.rs` |
| `Common/Color.pm`, `Common/Color/*` | `latexml_core/src/common/color.rs` |
| `Common/{Dimension,Glue,Number,Float}.pm` | `latexml_core/src/common/{dimension,glue,number,float}.rs` |
| `Common/Error.pm` | `latexml_core/src/common/error.rs`, `latexml_core/src/util/logger.rs` |
| `Common/Font.pm`, `Common/Font/{Metric,StandardMetrics}.pm` | `latexml_core/src/common/font.rs`, `latexml_core/src/common/font/` |
| `Common/Locator.pm`, `Common/Object.pm` | `latexml_core/src/common/locator.rs`, `latexml_core/src/common/object.rs` |
| `Common/Model.pm` | `latexml_core/src/common/model.rs` |
| `Common/Model/RelaxNG.pm` | `latexml_core/src/common/relaxng/` |
| `Common/Model/DTD.pm` | — (RelaxNG only, no DTD) |
| `Common/XML.pm`, `Common/XML/{Parser,XPath}.pm` | `latexml_core/src/common/xml.rs` |
| `Common/XML/XSLT.pm` | `latexml_post/src/xslt.rs` |
| `Common/XML/RelaxNG.pm` (load a schema document; validate) | loading: `latexml_core/src/common/relaxng/{embedded,scan}.rs`; validating: — (`latexml_post/src/document.rs` `validate` only reports the declared schema) |
| `Global.pm` (`$STATE`, the `TRACE_*` flags) | `latexml_core/src/state.rs` (`STATE`); the `TRACE_*` flags: — |
| `Version.pm` (`$REVISION`) | `latexml_oxide/src/identity.rs` (`GIT_REVISION`, set by `latexml_oxide/build.rs`) |
| `Util/Pathname.pm`, `Util/Image.pm`, `Util/Radix.pm` | `latexml_core/src/util/{pathname,image,radix}.rs` (`Util/Radix.pm` has a second port, `latexml_post/src/radix.rs`: a DRY candidate) |
| `Util/Test.pm` | `latexml_core/src/test.rs`, `latexml_oxide/src/util/test.rs`, `latexml_codegen/src/testable.rs` |
| `Util/Unicode.pm` | `latexml_post/src/unicode.rs` |
| `Util/Geometry.pm`, `Util/Transform.pm`, `Util/WWW.pm` | — (no Rust module cites them) |
| `resources/RelaxNG/*` | `latexml_core/resources/RelaxNG/` (embedded: `latexml_core/src/common/relaxng/embedded.rs`; our extensions: `docs/parity/SCHEMA_EXTENSIONS.md`) |
| `texmf/latexml.sty`, `texmf/lxRDFa.sty` (the TeX-side styles, for pdflatex) | — (not shipped; their bindings are `latexml_package/src/package/latexml_sty/` and `lxrdfa_sty.rs`) |

## Bindings: `Package.pm`'s definers, `Package/*.ltxml`, contrib, Rhai

| Perl | Rust |
|---|---|
| `Package.pm` definers (`DefMacro`, `DefConstructor`, `DefEnvironment`, `DefPrimitive`, …) as functions | `latexml_core/src/binding/def/dialect.rs` (`def_macro` & co), `latexml_core/src/binding/def/builder.rs` |
| … and as the call-site DSL | the macros `DefMacro!`, `DefConstructor!`, `DefEnvironment!` in `latexml_engine/src/setup_binding_language.rs`; helper macros in `latexml_core/src/binding/def/macros.rs` (`docs/parity/BINDING_DSL_ARCHITECTURE.md`) |
| `Package.pm` loaders (`InputDefinitions`, `RequirePackage`, `LoadClass`, `LoadPool`, `FindFile`) | `latexml_core/src/binding/content.rs` |
| `Package.pm` counters (`NewCounter`, `RefStepCounter`, …) | `latexml_core/src/binding/counter/dialect.rs` (`new_counter`, `ref_step_counter`), `latexml_core/src/binding/counter.rs` |
| `Package/<name>.<ext>.ltxml` (455: 365 sty, 43 cls, 17 fontmap, 14 tex, 11 def, 3 ldf, 1 ltx, 1 lua) | `latexml_package/src/package/<name>_<ext>.rs`, lowercased, with non-alphanumerics → `_` (`IEEEtran.cls` → `ieeetran_cls.rs`; 544 files). 450 of the 455 have a file of that name. The other 5 dispatch to a sibling: `epsf.tex` → `epsf_sty.rs`, `psfig.tex` → `epsfig_sty.rs`, `xy.tex` → `xy_sty.rs`, `utf8x.def` → `utf8_def.rs`, `latexml.sty` → the module `latexml_package/src/package/latexml_sty/` |
| `FindFile` of a binding (`<name>.<ext>.ltxml` on the path) | the table `BINDINGS` (509 entries) and `dispatch` in `latexml_package/src/lib.rs`; the modules are declared in `latexml_package/src/package.rs` |
| — (bindings Perl does not ship) | `latexml_contrib/src/<name>_<ext>.rs` (342 files: 196 sty, 130 cls, 7 tex, and 9 others: `lib.rs` and shared modules); table `BINDINGS` (363 entries) and `dispatch` in `latexml_contrib/src/lib.rs`, passed to the converter as `extra_bindings_dispatch` (`latexml_oxide/src/api.rs`); catalogued in `docs/parity/OXIDIZED_DESIGN_DIVERGENCES.md` "Bindings beyond Perl's tree" |
| a user's `.ltxml` (Perl code loaded at run time) | `<name>.<ext>.rhai`, loaded by `latexml_oxide/src/converter.rs` through `latexml_contrib/src/script_bindings/` (the only module that embeds Rhai; API: `latexml_contrib/src/script_bindings/API.md`; guide: `docs/parity/RHAI_BINDINGS_GUIDE.md`; examples: `docs/examples/`) |

## The dump

Strict `LoadFormat` mutual exclusivity (Perl `Package.pm:LoadFormat` L2734-2752): bootstrap → dump → constructs where
a dump built from this TeX tree is found, otherwise bootstrap → base → constructs. Design:
`docs/parity/DUMP_DESIGN.md`; the parity audit: [`PERL_LOADFORMAT_AUDIT.md`](../archive/PERL_LOADFORMAT_AUDIT.md).

| Perl | Rust |
|---|---|
| `latexml --init=latex.ltx` (`Core.pm` iniTeX, then `Core/Dumper.pm`) | `latexml_oxide/src/ini_tex.rs`; a dump for the TeX tree in use is built on demand by `latexml_oxide/src/format_dumps.rs` (OD #449) |
| `Core/Dumper.pm` writing `blib/…/Engine/{plain,latex}_dump.pool.ltxml` | `latexml_core/src/dump_writer.rs` → `resources/dumps/{plain,latex}.YYYY.dump.txt` (gitignored; `tools/make_formats.sh`) |
| loading the generated `*_dump.pool.ltxml` (every record `assign_internal('global')`, Dumper.pm L59-67) | `latexml_core/src/dump_reader.rs`, called by `latexml_engine/src/plain_dump.rs` and `latexml_engine/src/latex_dump.rs` |
| — | the embedding of dumps in the binary: `latexml_engine/build.rs` → `latexml_engine/src/embedded_dumps.rs`; year and tree matching: `latexml_engine/src/dump_paths.rs`; dump → Rust source: `latexml_core/src/dump_codegen.rs` |

## `latexml_engine`: the pools

Perl `lib/LaTeXML/Engine/` holds 38 `.pool.ltxml`; `make` adds the two generated dump pools to `blib/` (40). All 38
have a Rust module of the same name in `latexml_engine/src/`: 36 files and two module directories (`base_utilities/`,
`latex_constructs/`). Two of them are partial ports (AmSTeX, BibTeX). The 50 top-level `.rs` files are those 36,
`pre_bibtex.rs` (Perl `Pre/BibTeX.pm`), the two dump loaders, and 11 Rust-only files (listed below). With the module directories
(`base_utilities/`, `latex_constructs/`, `l3fp_native/`) the crate has 165 `.rs` files.

### Loading hierarchy

Perl:
```
TeX.pool
├── Base.pool          (loads all Base_* + TeX_* + eTeX + pdfTeX + Base_Deprecated)
│   ├── Base_Schema
│   ├── Base_ParameterTypes
│   ├── Base_Utility
│   ├── Base_XMath
│   ├── TeX_Box .. TeX_Tables  (19 files)
│   ├── eTeX
│   ├── pdfTeX
│   └── Base_Deprecated
└── LoadFormat('plain')
    ├── plain_bootstrap
    ├── plain_base
    ├── plain_dump
    └── plain_constructs
        └── math_common

LaTeX.pool
├── LoadPool('TeX')        (everything above)
└── LoadFormat('latex')
    ├── latex_bootstrap
    ├── latex_base
    ├── latex_dump
    └── latex_constructs
```

Rust (dump XOR base, never both):
```
tex.rs                     (≈ TeX.pool)
├── base                   (≈ Base.pool: base_schema … base_deprecated, as Perl, plus the Rust-only frontmatter_stores)
├── plain_bootstrap
├── if dump available && !LATEXML_NODUMP:
│     plain_dump           (runtime loader for the plain dump)
│   else:
│     plain_base
└── plain_constructs
    └── math_common

latex.rs                   (≈ LaTeX.pool)
├── LoadPool!("TeX")       (tex.rs above)
├── latex_bootstrap
├── if dump available && !LATEXML_NODUMP:
│     latex_dump           (runtime loader for the latex dump)
│   else:
│     latex_base
├── latex_constructs       (module directory, sections C.1–C.15)
└── latex_constructs_rust_only
```

`--init` (dump building) stops after the bootstrap (`latexml_engine/src/latex.rs`), so the snapshot holds what
`latex.ltx` defines, as Perl's iniTeX → `DumpFile` flow does.

### File-by-file

| Perl file | Rust (`latexml_engine/src/`) | Notes |
|---|---|---|
| `TeX.pool.ltxml` | `tex.rs` | autoload triggers, `LoadFormat('plain')` |
| `LaTeX.pool.ltxml` | `latex.rs` | `LoadFormat('latex')` |
| `Base.pool.ltxml` | `base.rs` | loader only |
| `Base_Schema.pool.ltxml` | `base_schema.rs` | |
| `Base_ParameterTypes.pool.ltxml` | `base_parameter_types.rs` | |
| `Base_Utility.pool.ltxml` | `base_utilities/` | also the frontmatter machinery (`base_utilities/frontmatter/`, `base_utilities/defs/`) |
| `Base_XMath.pool.ltxml` | `base_xmath.rs` | |
| `TeX_{Box,Character,Debugging,FileIO,Fonts,Glue,Hyphenation,Inserts,Job,Kern,Logic,Macro,Marks,Math,Page,Paragraph,Penalties,Registers,Tables}.pool.ltxml` (19) | `tex_{box,character,debugging,file_io,fonts,glue,hyphenation,inserts,job,kern,logic,macro,marks,math,page,paragraph,penalties,registers,tables}.rs` | `tex_math.rs` includes sub/superscripts (once `tex_scripts.rs`) |
| `eTeX.pool.ltxml`, `pdfTeX.pool.ltxml` | `etex.rs`, `pdftex.rs` | |
| `Base_Deprecated.pool.ltxml` | `base_deprecated.rs` | |
| `plain_bootstrap.pool.ltxml` | `plain_bootstrap.rs` | `\TeX`, `\newif`, `\leavevmode`, `\alloc@` |
| `plain_base.pool.ltxml` | `plain_base.rs` | Appendix B: registers, allocation, spacing, formatting |
| `plain_constructs.pool.ltxml` | `plain_constructs.rs` | font commands, accents, alignment, footnotes |
| `math_common.pool.ltxml` | `math_common.rs` | Greek, operators, relations, arrows, delimiters, log functions |
| `latex_bootstrap.pool.ltxml` | `latex_bootstrap.rs` | stubs for font/counter internals |
| `latex_base.pool.ltxml` | `latex_base.rs` | infrastructure: `DefMacro`, `Let`, `DefRegister`, `RawTeX` |
| `latex_constructs.pool.ltxml` (5,969 lines) | `latex_constructs/` (21,305 lines in `sect01.rs`–`sect13.rs`, 9,229 in `mod.rs`) | sections below |
| `plain_dump.pool.ltxml`, `latex_dump.pool.ltxml` (generated, `blib/` only) | `plain_dump.rs`, `latex_dump.rs` | loaders of the text dump |
| `AmSTeX.pool.ltxml` | `amstex.rs` (1,208 lines) | partial port |
| `BibTeX.pool.ltxml`, `Pre/BibTeX.pm`, `Pre/BibTeX/Entry.pm` | `bibtex.rs` (3,873 lines), `pre_bibtex.rs` (1,924 lines) | `.bib → .xml` end to end; field-synthesis polish open (`docs/archive/BIBTEX_PORT_PLAN_2026-06-20.md`) |

### `latex_constructs`: the sections

| Section (Lamport) | Perl from line | Rust (`latexml_engine/src/latex_constructs/`) |
|---|---|---|
| preamble: compatibility, `\documentclass` (Perl L73) | 1 | `mod.rs` |
| C.1 Commands and Environments | 162 | `sect01.rs` |
| C.2 The Structure of the Document | 287 | `sect02.rs` |
| C.3 Sentences and Paragraphs | 400 | `sect03.rs` |
| C.4 Sectioning and Table of Contents | 544 | `sect04.rs` |
| C.5 Classes, Packages and Page Styles (the title page and abstract: C.5.4, Perl 1060) | 771 | `sect05.rs`; the title-page reading: `title_page_reading.rs` |
| C.6 Displayed Paragraphs | 1206 | `sect06.rs` |
| C.7 Mathematical Formulas | 1870 | `sect07.rs`, `reference_math_slots.rs` |
| C.8 Definitions, Numbering and Programming | 2509 | `sect08.rs` |
| C.9 Figures and Other Floating Bodies | 3125 | `sect09.rs`; the panel grid (`annotate_panel_grid`) in `mod.rs` |
| C.10 Lining It Up in Columns | 3501 | `sect10.rs` |
| C.11 Moving Information Around | 3774 | `sect11.rs` |
| C.12 Line and Page Breaking; C.13 Lengths, Spaces and Boxes | 4553, 4590 | `sect12.rs` |
| C.14 Pictures and Color; C.15 Font Selection | 4815, 5146 | `sect13.rs` |
| (auxiliary) other stuff, case changing, semi-undocumented, sundry, aux-file stubs, language declarations (to the end, L5969) | 5463 | `sect13.rs` |

### Rust-only files in `latexml_engine/src/`

| Rust file | Purpose |
|---|---|
| `lib.rs`, `prelude.rs`, `macros.rs` | crate root, prelude, engine-internal macros (`InnerPool!`, …) |
| `setup_binding_language.rs` | the binding DSL macros (`DefMacro!`, `DefConstructor!`, …) every binding crate uses |
| `dump_paths.rs`, `embedded_dumps.rs`, `build_rerun.rs` | dump file names and TeX Live year detection; the embedded-dump manifest; what `build.rs` tracks |
| `latex_kernel.rs` | loads the LaTeX format on demand (the safety net of the eager trigger list) |
| `latex_constructs_rust_only.rs` | LaTeX-format overrides with no Perl counterpart, loaded after `latex_constructs` |
| `frontmatter_stores.rs` | reroutes a raw class's title-page stores to the frontmatter API |
| `lua_bridge.rs` | evaluates `\directlua` bodies in an external `texlua` |
| `l3fp_native/` | a native evaluator for l3fp's `\__fp_parse:n` |

## `latexml_math_parser`

| Perl | Rust (`latexml_math_parser/src/`) |
|---|---|
| `MathParser.pm` (driver, lexing, semantics) | `parser.rs`, `util.rs`, `semantics.rs`, `semantics/`, `pragmatics.rs` |
| `MathGrammar` (Parse::RecDescent) | `grammar.rs`, `grammar/builder.rs`, `grammar/macros.rs` (Marpa: an ambiguous grammar pruned in the semantics) |
| — | `asf_traverser.rs` (the parse forest), `data.rs`, `diag.rs` |

Design: `docs/math/MATH_PARSER_AND_ASF.md`.

## `latexml_post`

| Perl (`LaTeXML/lib/LaTeXML/…`) | Rust (`latexml_post/src/`) |
|---|---|
| `Post.pm` (driver, `LaTeXML::Post::Document`, the processor base) | `lib.rs`, `document.rs`, `processor.rs`; the pipeline API: `latexml_oxide/src/post.rs` |
| `Post/{Collector,CrossRef,Graphics,LaTeXImages,LexMath,MakeBibliography,MakeIndex,MathImages,OpenMath,PictureImages,Scan,SVG,TeXMath,UnicodeMath,Writer,XMath,XSLT}.pm` | `collector.rs`, `crossref.rs`, `graphics.rs`, `latex_images.rs`, `lex_math.rs`, `make_bibliography.rs`, `make_index.rs`, `math_images.rs`, `open_math.rs`, `picture_images.rs`, `scan.rs`, `svg.rs`, `tex_math.rs`, `unicode_math.rs`, `writer.rs`, `xmath.rs`, `xslt.rs` |
| `Post/Split.pm` | `split.rs`, `stream_split.rs` |
| `Post/MathML.pm`, `Post/MathML/{Presentation,Content,Linebreaker,OperatorDictionary}.pm` | `mathml/mod.rs`, `mathml/{presentation,content,linebreaker,operator_dictionary}.rs` |
| `Post/Manifest.pm`, `Post/Manifest/Epub.pm` | `manifest/mod.rs`, `manifest/epub.rs` |
| `Util/Pack.pm`, `Util/Pack/{Dir,Zip}.pm` | `pack.rs`, `extract.rs` |
| `Util/ObjectDB.pm`, `Util/ObjectDB/Entry.pm` | `object_db.rs` |
| `Util/Radix.pm`, `Util/Unicode.pm` | `radix.rs` (a second port of `Util/Radix.pm`, beside `latexml_core/src/util/radix.rs`: a DRY candidate), `unicode.rs` |
| `resources/{XSLT,CSS,javascript}/` | `latexml_post/resources/{XSLT,CSS,javascript}/` (embedded) |
| — | `graphics_cache.rs`, `math_processor.rs`, `postscript_fonts.rs`, `schema_docs.rs`, `diag.rs`, `test_env.rs` |

## `latexml_oxide`: the converter and the CLI

| Perl | Rust |
|---|---|
| `LaTeXML.pm` (the converter: `convert`, the ambles, the status) | `latexml_oxide/src/converter.rs` |
| `Core.pm` (`initializeState`, pre/postamble) | `latexml_oxide/src/core_interface.rs` |
| `bin/latexml`, `bin/latexmlc`, `bin/latexmlpost` | one binary, `latexml_oxide/bin/latexml_oxide.rs` (post-processing included) |
| `bin/latexmlmath` | `latexml_oxide/bin/latexmlmath_oxide.rs` |
| `doc/manual/genschema`, `tools/compileschema` (Perl repository) | `latexml_oxide/bin/genschema_oxide.rs`, `tools/compileschema.sh` |
| `bin/latexmlfind` | — |
| — | `latexml_oxide/bin/cortex_worker.rs` (the arXiv fleet worker), `latexml_oxide/bin/binding_audit.rs`, `latexml_oxide/bin/ab_audit/` |
| — | `latexml_oxide/src/api.rs` (the library API), `latexml_oxide/src/lsp_server/`, `main_tex.rs` (finding a directory's main `.tex`), `multidoc.rs`, `bib_session.rs`, `render_workers.rs`, `streaming_restart.rs`, `identity.rs`, `source_date.rs`, `conformance/` |
| `latexml --init` | `latexml_oxide/src/ini_tex.rs`, `latexml_oxide/src/format_dumps.rs` (see "The dump") |
