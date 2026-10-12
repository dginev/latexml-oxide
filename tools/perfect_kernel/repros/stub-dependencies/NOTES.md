# stub-dependencies

A stub binding replaces a package that is not interpreted raw, so the packages the real one loads, and its public
names, are what documents and classes rely on without loading them. All shapes are SHARED with Perl, whose ar5iv stubs
load a subset (KNOWN_PERL_ERRORS #618, #619); fixed in batch 65k (OXIDIZED_DESIGN_DIVERGENCES #547, #548).

| repro | mechanism | fix site |
|---|---|---|
| `mdframed_framemethod_tikz_loads_tikz` | mdframed.sty:154-169 `framemethod` key loads tikz/pst-node/the named package | `mdframed_sty.rs` `\lx@mdf@framemethod` |
| `mdframed_loads_its_dependencies` | mdframed.sty:72-77 loads zref-abspage (→ iftex `\ifpdf`), needspace, color | `mdframed_sty.rs` |
| `pnas_new_dates_dropcap` | pnas-new.cls:270/284 setters, :53-55 `[framemethod=tikz]{mdframed}`, :173 jabbrv | `pnas_new_cls.rs` |
| `mciteplus_bbl_switches` | mciteplus.sty:72-87, 182-184 switches a `.bbl` sets; :170 sublist setter | `mciteplus_sty.rs` |
| `achemso_section_abstract_switches` | achemso.cls:1091-1102, 1160-1169 switches, :1266-1269 lengths; :309-315 caption etc. | `achemso_cls.rs` |
| `arydshln_shorthand_environments` | arydshln.sty:697-715 `{Array}`/`{Tabular}`/`{Tabular*}`/`{Longtable}` | `arydshln_sty.rs` |
| `nag_public_api` | nag.sty:82-168 rule commands and their arguments | `nag_sty.rs` |
| `mathpartir_premise_separators` | mathpartir.sty:62-68 `\mpr@andcr` and siblings (ottalt.sty) | `mathpartir_sty.rs` |
| `epstopdf_loads_pdftexcmds` | epstopdf.sty:149-152 loads pdftexcmds (→ iftex `\ifpdf`) | `epstopdf_sty.rs` |
| `hyperref_pagebackref_loads_backref` | hyperref.sty:2762-2810, 4587-4589 option-driven backref load; backref.sty:95-148, 207-208 | `hyperref_sty.rs`, `backref_sty.rs` |
| `hyperref_passoptions_pagebackref_info` | the option via `\PassOptionsToPackage`; `\ifHy@backref` true (:2801); backref's message an Info | `hyperref_sty.rs`, `backref_sty.rs` |
| `backref_direct_load_warns` | a direct `\usepackage[ref]{backref}` keeps the Warning; backref.sty:83-85 `\backref` | `backref_sty.rs` |
| `hyperref_backref_none_loads_nothing` | hyperref.sty:2762-2810 `backref=none`, `pagebackref=false` load nothing | `hyperref_sty.rs` |
| `mdframed_body_framemethod_loads_nothing` | a body `framemethod=tikz` loads nothing (latex.ltx:18628), with an Info | `mdframed_sty.rs` |
| `pnas_new_show_matmethods_acknow` | pnas-new.cls:402-428 `\showmatmethods`, `\showacknow` | `pnas_new_cls.rs` |
| `arydshln_longtable_shorthand` | arydshln.sty:697-715 `{Longtable}` after longtable | `arydshln_sty.rs` |

| `amsrefs_loads_mathscinet_url` | amsrefs.sty:45-46, 213-217 loads url, rkeyval, textcmds, mathscinet (65r, KPE #627) | `amsrefs_sty.rs` |
| `amsrefs_lite_rkeyval` | `[lite]` (:214-218) skips textcmds/mathscinet; `{biblist}[\resetbiblist{99}]` (:569, :651) | `amsrefs_sty.rs` |
| `hyperref_colorlinks_after_class_preload` | a class binding's hyperref load; the repeat load's options via `\hypersetup` (65r, KPE #628) | `binding/content.rs`, `hyperref_sty.rs` |
| `thm_restate_loads_thmtools` | thm-restate.sty:26 loads thmtools; the cycle ends with an Info (65r, KPE #629) | `thm_restate_sty.rs`, `binding/content.rs` |
| `elsarticle_3p_loads_geometry` | elsarticle.cls:1252-1300 1p/3p/5p load geometry (65r, KPE #629) | `elsarticle_cls.rs` |
| `xypic_curved_arrow` | xyv2 frame → curve (xypdf-fr.tex:60), `\crvi` (65r) | `xy_sty.rs` |
| `revtex_sanitize_url` | revtex4-1.cls:2378, revtex4-2.cls:2399 `\sanitize@url` (65r) | `revtex4_support_sty.rs` |
| `pos_loads_color` | PoS.cls:162 (v1.2.6) loads color (65r, KPE #629) | `pos_cls.rs` |

`\bibselect` (amsrefs.sty:1291-1340) is guarded with post-processing: `index-bib/amsrefs_bibselect_{database,star_database}`
(the fixtures and `amsrefs_bibselect_db.ltb` in `latexml_oxide/tests/cluster_regressions/`).

Dead end (one line): a `framemethod` set by `\mdfsetup` after the package is loaded makes pdflatex error too (the
`md-frame-1.mdf` re-input: "Command \mdf already defined"), so pdflatex has no clean oracle for it; the binding loads the package in the preamble
and gives an Info in the body (`mdframed_body_framemethod_loads_nothing`, validated by `\typeout`).
