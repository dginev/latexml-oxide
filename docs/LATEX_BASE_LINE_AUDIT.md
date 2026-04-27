# `latex_base.pool.ltxml` ↔ `latex_base.rs` line audit

Strict line-by-line walk of the 865-line Perl `latex_base.pool.ltxml`
against `latex_base.rs`. Goal: confirm every Perl entry is in the
matching Rust file, in the same source order, with the same shape.

**Status legend**:
* ✅ PARITY — Perl entry has Rust counterpart in expected location.
* ↻ ORDER — entry exists in Rust but in a different sibling file.
* 📁 FILE — entry placed correctly relative to file structure.
* ⚠ DIVERGE — entry differs in semantics or shape.
* ❌ MISSING — Perl entry has no Rust counterpart.
* 🔵 RUST_ONLY — Rust entry without Perl source.

## Phase 1 — Perl L1-150 (C.0 Preliminaries & Shorthands)

| Perl L | Symbol | Rust file:line | Status |
|--------|--------|----------------|--------|
| 35 | `Let '\@pushfilename' '\lx@pushfilename'` | latex_base.rs:58 | ✅ |
| 36 | `Let '\@popfilename' '\lx@popfilename'` | latex_base.rs:59 | ✅ |
| 38 | `\@ehc` "I can't help" | latex_base.rs:62 | ✅ |
| 40 | `\@gobble{}` (Tokens()) | latex_base.rs:65 | ✅ |
| 41 | `\@gobbletwo{}{}` | latex_base.rs:66 | ✅ |
| 42 | `\@gobblefour{}{}{}{}` | latex_base.rs:67 | ✅ |
| 43 | `\@firstofone{}` | latex_base.rs:73 (token-list "#1" form, see comment L68-72) | ✅ ⚠ shape |
| 44 | `Let '\@iden' '\@firstofone'` | latex_base.rs:74 | ✅ |
| 45 | `\@firstoftwo{}{}` | latex_base.rs:75 | ✅ ⚠ shape |
| 46 | `\@secondoftwo{}{}` | latex_base.rs:76 | ✅ ⚠ shape |
| 47 | `\@thirdofthree{}{}{}` | latex_base.rs:77 | ✅ ⚠ shape |
| 48-49 | `\@expandtwoargs{}{}{}` (closure) | latex_base.rs:82-90 | ✅ |
| 50-52 | `\@makeother{}` (closure) | latex_base.rs:93-104 | ✅ |
| 55-64 | RawTeX block: `\@namedef`/`\@nameuse`/`\@cons`/`\@car`/`\@cdr`/`\@carcube`/`\nfss@text`/`\@sect` | latex_base.rs:25-37 (TeX!) | ✅ ↻ position |
| 66-72 | RawTeX: `\obeycr`/`\@gobblecr`/`\restorecr` | latex_base.rs:107-113 (TeX!) | ✅ |
| 73-90 | RawTeX: `\rem@pt`/`\strip@pt`/`\strip@prefix`/`\@sanitize`/`\@onelevel@sanitize`/`\dospecials` | latex_base.rs:115-133 (TeX!) | ✅ |
| 92-114 | `\nfss@catcodes` | latex_base.rs:135-160 | ✅ |
| 116 | `\@height` ("height") | latex_base.rs:163 | ✅ |
| 117 | `\@width` ("width") | latex_base.rs:164 | ✅ |
| 118 | `\@depth` ("depth") | latex_base.rs:165 | ✅ |
| 119 | `\@minus` ("minus") | latex_base.rs:166 | ✅ |
| 120 | `\@plus` ("plus") | latex_base.rs:167 | ✅ |
| 121 | `\hb@xt@` ("\hbox to") | latex_base.rs:168 | ✅ |
| 122 | `\hmode@bgroup` ("\leavevmode\bgroup") | latex_base.rs:169 | ✅ |
| 124 | `\@backslashchar` (T_OTHER('\\')) | latex_base.rs:171 | ✅ |
| 125 | `\@percentchar` (T_OTHER('%')) | latex_base.rs:172 | ✅ |
| 126 | `\@charlb` (T_LETTER('{')) | latex_base.rs:173 | ✅ |
| 127 | `\@charrb` (T_LETTER('}')) | latex_base.rs:174 | ✅ |
| 129 | `\@vpt` (T_OTHER('5')) | latex_base.rs:177 | ✅ |
| 130 | `\@vipt` (T_OTHER('6')) | latex_base.rs:178 | ✅ |
| 131 | `\@viipt` (T_OTHER('7')) | latex_base.rs:179 | ✅ |
| 132 | `\@viiipt` (T_OTHER('8')) | latex_base.rs:180 | ✅ |
| 133 | `\@ixpt` (T_OTHER('9')) | latex_base.rs:181 | ✅ |
| 134 | `\@xpt` ("10") | latex_base.rs:182 | ✅ |
| 135 | `\@xipt` ("10.95") | latex_base.rs:183 | ✅ |
| 136 | `\@xiipt` ("12") | latex_base.rs:184 | ✅ |
| 137 | `\@xivpt` ("14.4") | latex_base.rs:185 | ✅ |
| 138 | `\@xviipt` ("17.28") | latex_base.rs:186 | ✅ |
| 139 | `\@xxpt` ("20.74") | latex_base.rs:187 | ✅ |
| 140 | `\@xxvpt` ("24.88") | latex_base.rs:188 | ✅ |
| 142-153 | `\vpt`/`\vipt`/`\viipt`/`\viiipt`/`\ixpt`/`\xpt`/`\xipt`/`\xiipt`/`\xivpt`/`\xviipt`/`\xxpt`/`\xxvpt` (LaTeX 209 size aliases) | latex_base.rs:190-201 | ✅ |

### Phase 1 findings

* **Strong PARITY**. All Perl L31-153 entries have Rust counterparts
  in proper source order at latex_base.rs L57-201.
* **⚠ shape** divergence on `\@firstofone`/`\@firstoftwo`/`\@secondoftwo`/
  `\@thirdofthree`: Rust uses token-list form `"#1"` etc. instead of
  Perl's closure `sub { $_[1] }`. Documented in latex_base.rs:68-72:
  matches Perl latex.ltx's `\long\def\@firstofone#1{#1}` end-state
  (via raw-load), AND lets these CSes survive dump-only mode dump
  loading. Validated as intentional.
* **🔵 Rust-only entry**: `Let!("\\@empty", "\\lx@empty")` at
  latex_base.rs:22 is not in Perl latex_base.pool.ltxml directly —
  the alias is for `\lx@empty` from Base_Schema (TeX pool). `\@empty`
  is also defined via raw-load of latex.ltx in Perl. Functionally
  equivalent.
* **↻ position**: The Perl L55-64 RawTeX block (with `\@namedef` etc.)
  is at latex_base.rs L25-37 — placed BEFORE the L40-49 macro block
  (Rust L65+). Perl has them after L40-52. This is a minor ordering
  divergence; doesn't affect semantics since the entries are
  independent.

## Phase 2+ (TODO)

* Phase 2: Perl L150-300 (C.1.3 Fragile Commands, C.3 Sentences/Paragraphs)
* Phase 3: Perl L300-500 (C.4 Sectioning, fontenc, etc.)
* Phase 4: Perl L500-700 (\loggingall, math chardefs, etc.)
* Phase 5: Perl L700-865 (final block — to be discovered)
