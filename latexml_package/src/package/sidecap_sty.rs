use crate::{
  engine::latex_constructs::{after_float, before_xfloat},
  prelude::*,
};

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: sidecap.sty.ltxml
  // First draft stub; ignore sidecaption-ness and just make a regular table/figure
  RequirePackage!("ifthen");

  // sidecap.sty:177-182 `\SC@float[<vpos>]{figure}`/`\SC@dblfloat`: the figure or table float itself, built by
  // `\@float{figure}[<placement>]` (:225-226, `\SC@zfloat`), never through `\figure`; the first optional is the
  // caption's relative width, the second the placement. Perl's `\SCfigure[][]` is `\figure[#2]`, the class's own
  // where a class redefined it: solarphysics.cls:1714 boxes the body, "<ltx:caption> isn't allowed in <ltx:block>"
  // (2006.04669; 1510.03282, 1511.05217, 2202.13838). OXIDIZED_DESIGN_DIVERGENCES #1156; guard
  // `perfect_kernel_batchregtaild::sidecap_float_is_the_kernel_float`.
  DefEnvironment!("{SCfigure}[][]",
    "^^<ltx:figure xml:id='#id' inlist='#inlist' ?#2(placement='#2')>#tags#body</ltx:figure>",
    properties => { stored_map!("layout" => "vertical") },
    tex_group => { TexGroup::Box },
    before_digest => { before_xfloat("figure", false)?; },
    after_digest => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical");
  DefEnvironment!("{SCfigure*}[][]",
    "^^<ltx:figure xml:id='#id' inlist='#inlist' ?#2(placement='#2')>#tags#body</ltx:figure>",
    properties => { stored_map!("layout" => "vertical") },
    tex_group => { TexGroup::Box },
    before_digest => { before_xfloat("figure", true)?; },
    after_digest => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical");
  DefEnvironment!("{SCtable}[][]",
    "^^<ltx:table xml:id='#id' inlist='#inlist' ?#2(placement='#2')>#tags#body</ltx:table>",
    properties => { stored_map!("layout" => "vertical") },
    tex_group => { TexGroup::Box },
    before_digest => { before_xfloat("table", false)?; },
    after_digest => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical");
  DefEnvironment!("{SCtable*}[][]",
    "^^<ltx:table xml:id='#id' inlist='#inlist' ?#2(placement='#2')>#tags#body</ltx:table>",
    properties => { stored_map!("layout" => "vertical") },
    tex_group => { TexGroup::Box },
    before_digest => { before_xfloat("table", true)?; },
    after_digest => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical");

  // \sidecaptionvpos{<float type>}{<t|c|b>} — real sidecap.sty:
  // `\newcommand*\sidecaptionvpos[2]` — configures the vertical alignment of a
  // side caption. Purely a layout hint with no bearing on the logical HTML
  // output (we already ignore side-caption-ness above), so consume both args
  // and expand to nothing. Witness 2408.08435 (ar5iv #555).
  DefMacro!("\\sidecaptionvpos{}{}", "");

  DefEnvironment!("{wide}", "#body");

  // sidecap.sty:39-40 `\sidecaptionsep`, the gap between a float and its side caption (`\marginparsep`; 1302.0875).
  RawTeX!(r"\newcommand*\sidecaptionsep{}\edef\sidecaptionsep{\the\marginparsep}");
  // sidecap.sty:85-153 the options, layout choices the binding ignores but `origragged` (:74, :118); :156-172 at
  // `\begin{document}`, unless `origragged`, loads ragged2e (`[NewParameters]`) when no package did, and lets
  // `\RaggedRight`, `\RaggedLeft`, `\Centering` be LaTeX's own when they are still undefined. A document reaches them
  // through sidecap alone (1112.5588, 1507.07568, 1610.00850, 1812.02282, 1812.03320 — 21 papers). Perl's binding
  // lacks both (KNOWN_PERL_ERRORS #822). Guard: `perfect_kernel_batch68j::sidecap_loads_ragged2e_and_its_sep`.
  RawTeX!(r"\newif\ifSC@origRagged");
  for option in ["innercaption", "outercaption", "rightcaption", "leftcaption", "wide", "raggedright", "raggedleft",
    "ragged", "margincaption", "innerbody", "outerbody", "centerbody", "rightbody", "leftbody"] {
    DeclareOption!(option, None);
  }
  DeclareOption!("origragged", "\\SC@origRaggedtrue");
  ProcessOptions!();
  RawTeX!(r"\AtBeginDocument{\ifSC@origRagged\else\@ifpackageloaded{ragged2e}{}{\RequirePackage[NewParameters]{ragged2e}}\fi
\@ifundefined{RaggedRight}{\let\RaggedLeft\raggedleft\let\RaggedRight\raggedright\let\Centering\centering}{}}");
});
