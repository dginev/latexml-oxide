//! JHEP.cls — Journal of High Energy Physics document class
//! Perl: JHEP.cls.ltxml — 314 lines (mostly journal abbreviation macros)
use crate::engine::latex_constructs::{after_float, before_float};
use crate::prelude::*;

/// DEP-NEW (2026-05-19): data-drive helper for the ~94 `\<cs>{}{}{}`
/// journal-abbreviation macros that all expand to
/// `\@spires{<CODE>\%2C#1\%2C#3}{{\it <Name>}{\bf #1} (#2) #3}`.
/// Body text differs per journal but the prototype and shape are
/// identical. Replacing the 94 `DefMacro!` macro arms with this
/// runtime helper collapses the repeated macro expansion to a
/// single inlined function body.
/// Per [[wisdom_data_drive_min_call_sites]]: 94 ≫ 5 threshold.
fn def_jhep_journal(cs: &str, body: &str) -> Result<()> {
  let proto = format!("{cs}{{}}{{}}{{}}");
  let (cs_tok, params) = parse_prototype(&proto, true)?;
  let body_toks = mouth::tokenize_internal(body);
  def_macro(cs_tok, params, ExpansionBody::Tokens(body_toks), None)?;
  Ok(())
}

#[rustfmt::skip]
LoadDefinitions!({
  // Perl L26-35: Class options
  DeclareOption!("proceedings", {});
  DeclareOption!("published", {});
  DeclareOption!("hyper", {});
  DeclareOption!("nohyper", {});
  DeclareOption!("notoc", {});
  ProcessOptions!();
  load_class("article", Vec::new(), Tokens!())?;
  RequirePackage!("amssymb");
  // The raw JHEP.cls loads hyperref, giving authors \href / \url / \hypertarget
  // without an explicit \usepackage{hyperref}. Perl's JHEP.cls.ltxml omits
  // this, but the real-world paper corpus (e.g. arxiv 1010.4240 via PoS →
  // JHEP) depends on hyperref being active. Load it here so the arxiv sandbox
  // resolves \href without an Error:undefined cascade.
  RequirePackage!("hyperref");

  // Perl L40-58: Frontmatter
  DefMacro!("\\speaker{}", "\\lx@add@creator[role=speaker]{#1}");
  DefMacro!("\\abstract{}", "\\lx@add@abstract[name={\\abstractname~}]{#1}");
  DefMacro!("\\email Semiverbatim", "\\lx@add@email{#1}");
  // Perl L50-52: each date macro ships `name={\<role>name}` so the localized
  // label is attached to the frontmatter date entry. Restored to match.
  DefMacro!("\\received{}", "\\lx@add@date[role=received,name={\\receivedname}]{#1}");
  DefMacro!("\\revised{}", "\\lx@add@date[role=revised,name={\\revisedname}]{#1}");
  DefMacro!("\\accepted{}", "\\lx@add@date[role=accepted,name={\\acceptedname}]{#1}");
  DefMacro!("\\JHEPcopydate{}", "\\lx@add@date[role=copydate]{#1}");
  DefMacro!("\\dedicated{}", "\\lx@add@pubnote[role=dedication]{#1}");
  DefMacro!("\\conference{}", "\\lx@add@pubnote[role=conference]{#1}");
  DefMacro!("\\preprint{}", "\\lx@add@pubnote[role=preprint]{#1}");
  DefMacro!("\\keywords{}", "\\lx@add@keywords[name={\\keywordsname~}]{#1}");

  // Perl L61-64: Acknowledgements environment. Perl uses
  //   properties => sub { (name => Digest(T_CS('\acknowlname'))) }
  // — i18n via the user's current \acknowlname binding. Hardcoded
  // "Acknowledgments" was breaking non-English JHEP submissions.
  DefConstructor!("\\acknowledgments", "<ltx:acknowledgements name='#name'>",
    properties => {
      let name_toks = DigestIf!(T_CS!("\\acknowlname"))?;
      stored_map!("name" => name_toks)
    });
  DefConstructor!("\\endacknowledgments", "</ltx:acknowledgements>");
  // Perl L64: explicit autoClose so a stray paragraph or sectioning command
  // can flush an unclosed `\acknowledgments` block (no \endacknowledgments).
  Tag!("ltx:acknowledgements", auto_close => true);

  // Perl L67-76: Misc macros.
  // \hash: Perl `DefPrimitiveI('\hash', undef, '#')` emits "#" text.
  // Rust delegates to `\#` — observationally equivalent (both yield "#").
  // DP-flag: DefPrimitive → DefMacro, WISDOM #44; safe as `\hash` is a
  // user-facing text macro, never `\edef`-observed in JHEP documents.
  DefMacro!("\\hash", "\\#");
  DefMacro!("\\secstyle", "\\bfseries");
  DefMacro!("\\militarytime", "\\time");
  Let!("\\textref", "\\ref");
  def_macro_noop("\\tocsecs")?;
  DefMacro!("\\logo", "JHEP");
  // \JHEP{volume/issue} and \PrHEP{volume/issue} carry journal-issue
  // metadata. Perl L73-74 gobbles with `?` (uncertain); we surpass
  // by preserving as ltx:note for downstream JATS metadata.
  DefMacro!("\\JHEP{}", "\\@add@frontmatter{ltx:note}[role=jhep-issue]{#1}");
  DefMacro!("\\PrHEP{}", "\\@add@frontmatter{ltx:note}[role=prhep-issue]{#1}");
  DefMacro!("\\Proof", "\\emph{Proof.}\\ ");

  // Perl L80-83: Figure/table macros (map to environments)
  // Perl wraps into `{floatingfigure}` / `{floatingtable}` so the nested
  // `\caption` sees a proper `\@captype`. Previously Rust expanded to bare
  // `#2` which dumped the caption into text-mode and triggered
  // `Error:unexpected:\caption (outside any known float)`.
  DefMacro!("\\FIGURE[]{}", "\\begin{floatingfigure}[#1]#2\\end{floatingfigure}");
  DefMacro!("\\TABLE[]{}",  "\\begin{floatingtable}[#1]#2\\end{floatingtable}");
  DefMacro!("\\EPSFIGURE[]{}{}", "\\begin{floatingfigure}[#1]\\epsfig{file=#2}\\caption{#3}\\end{floatingfigure}");
  DefMacro!("\\TABULAR[]{}{}{}",
    "\\begin{floatingtable}[#1]\\begin{tabular}{#2}#3\\end{tabular}\\caption{#4}\\end{floatingtable}");

  // Perl JHEP.cls.ltxml L85-89: \DOUBLEFIGURE[pos]{img1}{img2}{cap1}{cap2}
  DefMacro!("\\DOUBLEFIGURE[]{}{}{}{}",
    "\\begin{figure}[#1]\
     \\begin{@half@doublefigure}\\epsfig{file=#2}\\caption{#4}\\end{@half@doublefigure}\
     \\begin{@half@doublefigure}\\epsfig{file=#3}\\caption{#5}\\end{@half@doublefigure}\
     \\end{figure}");
  DefEnvironment!("{@half@doublefigure}",
    "<ltx:figure xml:id='#id' inlist='#inlist' width='0.45%'>#body</ltx:figure>#tags",
    before_digest => { before_float("figure", None); },
    after_digest  => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical");

  // Perl JHEP.cls.ltxml L96-100: \DOUBLETABLE[pos]{tab1}{tab2}{cap1}{cap2}
  DefMacro!("\\DOUBLETABLE[]{}{}{}{}",
    "\\begin{table}[#1]\
     \\begin{@half@doubletable}#2\\caption{#4}\\end{@half@doubletable}\
     \\begin{@half@doubletable}#3\\caption{#5}\\end{@half@doubletable}\
     \\end{table}");
  DefEnvironment!("{@half@doubletable}",
    "<ltx:table xml:id='#id' inlist='#inlist' width='0.45%'>#body</ltx:table>#tags",
    before_digest => { before_float("table", None); },
    after_digest  => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical");

  // Perl JHEP.cls.ltxml L109-117: JHEP-specific {floatingfigure} without
  // the `{Dimension}` width arg that the standalone floatfig package uses.
  DefEnvironment!("{floatingfigure}[]",
    "<ltx:figure xml:id='#id' inlist='#inlist' float='#float'>#tags#body</ltx:figure>",
    before_digest => { before_float("figure", None); },
    after_digest  => sub[whatsit] { after_float(whatsit); },
    properties    => sub[args] {
      let pos = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
      let float = if pos.starts_with('v') || pos.starts_with('r') { "right" } else { "left" };
      Ok(stored_map!("float" => float))
    },
    mode => "internal_vertical");
  DefEnvironment!("{floatingtable}[]",
    "<ltx:table xml:id='#id' inlist='#inlist' float='#float'>#tags#body</ltx:table>",
    before_digest => { before_float("table", None); },
    after_digest  => sub[whatsit] { after_float(whatsit); },
    properties    => sub[args] {
      let pos = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
      let float = if pos.starts_with('v') || pos.starts_with('r') { "right" } else { "left" };
      Ok(stored_map!("float" => float))
    },
    mode => "internal_vertical");

  // Perl JHEP.cls.ltxml L133-136 — JHEP redefines `\href` as a 2-arg
  // `Semiverbatim Semiverbatim` Constructor. The crucial difference from
  // hyperref.sty's `\href HyperVerbatim {}` is that the SECOND arg is
  // also `Semiverbatim`: catcode-neutralized so `^` / `_` in the body
  // become OTHER tokens and do NOT fire script_handler when digested in
  // math mode.
  //
  // Why this matters: JHEP defines journal-citation macros like
  //   \am{}{}{} → \@spires{ANMAA\%2C#1\%2C#3}{...#3}
  // where `\@spires{URL}{BODY}` expands to `\href{URL}{BODY}`. Papers
  // call them inside math (`\beq … \am\mgr M^2S …`), grabbing `^` as
  // the third arg. The `^` then ends up at the end of `\href`'s body
  // — and `\href`'s body MUST treat it as Semiverbatim text, not as
  // a SUPER catcode token that would fire script_handler. The earlier
  // Rust port omitted this override, so `\href` stayed bound to
  // hyperref's `HyperVerbatim {}` form and the trailing `^` errored.
  //
  // Witness: arXiv:2602.22473 (Pallis et al.) line 1019
  //   \beq … -2\ld\am\mgr
  //   M^2S, … \eeq
  // Rust=1, Perl=0 → 0/0 with this binding.
  DefConstructor!("\\href Semiverbatim Semiverbatim",
    "<ltx:ref href='#href'>#2</ltx:ref>",
    enter_horizontal => true,
    properties => sub[args] {
      let url = args.first().and_then(|a| a.as_ref()).map(|t| t.to_string()).unwrap_or_default();
      let href = compose_url(&state::lookup_string("BASE_URL"), &url, None);
      Ok(stored_map!("href" => href))
    });

  // Perl L138-140: Stubs.
  def_macro_noop("\\JHEPspecialurl Semiverbatim")?;
  def_macro_noop("\\base Semiverbatim")?;
  def_macro_noop("\\name Semiverbatim")?;

  // Perl L143: SPIRES URL generator
  DefMacro!("\\@spires{}", "\\href{http://www-spires.slac.stanford.edu/spires/find/hep/www?j=#1}");

  // Journal abbreviation macros with SPIRES links — Perl L145-238.
  // Faithful 1:1 list of every journal alias. All follow the same
  // `\@spires{CODE%2C#1%2C#3}{{\it Name }{\bf #1} (#2) #3}` shape
  // (JHEP and the `\href`-direct ones are the only exceptions).
  def_jhep_journal("\\apa", "\\@spires{APASA\\%2C#1\\%2C#3}{{\\it Acta Phys.\\ Austriaca }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\apas", "\\@spires{APAUA\\%2C#1\\%2C#3}{{\\it Acta Phys.\\ Austriaca, Suppl.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\appol", "\\@spires{APPOA\\%2C#1\\%2C#3}{{\\it Acta Phys.\\ Polon.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\advm", "\\@spires{ADMTA\\%2C#1\\%2C#3}{{\\it Adv.\\ Math.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\adnp", "\\@spires{ANUPB\\%2C#1\\%2C#3}{{\\it Adv.\\ Nucl.\\ Phys.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\adp", "\\@spires{ADPHA\\%2C#1\\%2C#3}{{\\it Adv.\\ Phys.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\atmp", "\\@spires{00203\\%2C#1\\%2C#3}{{\\it Adv.\\ Theor.\\ Math.\\ Phys.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\am", "\\@spires{ANMAA\\%2C#1\\%2C#3}{{\\it Ann.\\ Math.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\ap", "\\@spires{APNYA\\%2C#1\\%2C#3}{{\\it Ann.\\ Phys.\\ (NY) }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\araa", "\\@spires{ARAAA\\%2C#1\\%2C#3}{{\\it Ann.\\ Rev.\\ Astron.\\ \\& Astrophys.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\arnps", "\\@spires{ARNUA\\%2C#1\\%2C#3}{{\\it Ann.\\ Rev.\\ Nucl.\\ Part.\\ Sci.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\asas", "\\@spires{AAEJA\\%2C#1\\%2C#3}{{\\it Astron.\\ Astrophys.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\asj", "\\@spires{ANJOA\\%2C#1\\%2C#3}{{\\it Astron.\\ J.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\app", "\\@spires{APHYE\\%2C#1\\%2C#3}{{\\it Astropart.\\ Phys.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\apj", "\\@spires{ASJOA\\%2C#1\\%2C#3}{{\\it Astrophys.\\ J. }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\baas", "\\@spires{AASBA\\%2C#1\\%2C#3}{{\\it Bull.\\ Am.\\ Astron.\\ Soc.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\bams", "\\@spires{BAMOA\\%2C#1\\%2C#3}{{\\it Bull.\\ Am.\\ Math.\\ Soc.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\blms", "\\@spires{LMSBB\\%2C#1\\%2C#3}{{\\it Bull.\\ London Math.\\ Soc.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\cjm", "\\@spires{CJMAA\\%2C#1\\%2C#3}{{\\it Can.\\ J.\\ Math.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\cqg", "\\@spires{CQGRD\\%2C#1\\%2C#3}{{\\it Class.\\ and Quant.\\ Grav.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\cmp", "\\@spires{CMPHA\\%2C#1\\%2C#3}{{\\it Commun.\\ Math.\\ Phys.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\ctp", "\\@spires{CTPMD\\%2C#1\\%2C#3}{{\\it Commun.\\ Theor.\\ Phys.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\cag", "\\@spires{00142\\%2C#1\\%2C#3}{{\\it Commun.\\ Anal.\\ Geom.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\cpam", "\\@spires{CPAMA\\%2C#1\\%2C#3}{{\\it Commun.\\ Pure Appl.\\ Math.\\ }{\\bf #1} (#2) #3}")?;
  // Perl JHEP.cls.ltxml L169 — Computer Physics Communications journal alias
  def_jhep_journal("\\cpc", "\\@spires{CPHCB\\%2C#1\\%2C#3}{{\\it Comput.\\ Phys.\\ Commun.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\dmj", "\\@spires{DUMJA\\%2C#1\\%2C#3}{{\\it Duke Math.\\ J. }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\epjc", "\\@spires{EPHJA\\%2CC#1\\%2C#3}{{\\it Eur.\\ Phys.\\ J. }{\\bf C #1} (#2) #3}")?;
  def_jhep_journal("\\epjd", "\\@spires{EPHJD\\%2CC#1\\%2C#3}{{\\it Eur.\\ Phys.\\ J. Direct.\\ }{\\bf C #1} (#2) #3}")?;
  def_jhep_journal("\\epl", "\\@spires{EULEE\\%2C#1\\%2C#3}{{\\it Europhys.\\ Lett. }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\forp", "\\@spires{FPYKA\\%2C#1\\%2C#3}{{\\it Fortschr.\\ Phys.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\faa", "\\@spires{FAAPB\\%2C#1\\%2C#3}{{\\it Funct.\\ Anal.\\ Appl.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\grg", "\\@spires{GRGVA\\%2C#1\\%2C#3}{{\\it Gen.\\ Rel.\\ Grav.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\hpa", "\\@spires{HPACA\\%2C#1\\%2C#3}{{\\it Helv.\\ Phys.\\ Acta }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\ijmpa", "\\@spires{IMPAE\\%2CA#1\\%2C#3}{{\\it Int.\\ J.\\ Mod.\\ Phys.\\ }{\\bf A #1} (#2) #3}")?;
  def_jhep_journal("\\ijmpb", "\\@spires{IMPAE\\%2CB#1\\%2C#3}{{\\it Int.\\ J.\\ Mod.\\ Phys.\\ }{\\bf B #1} (#2) #3}")?;
  def_jhep_journal("\\ijmpc", "\\@spires{IMPAE\\%2CC#1\\%2C#3}{{\\it Int.\\ J.\\ Mod.\\ Phys.\\ }{\\bf C #1} (#2) #3}")?;
  def_jhep_journal("\\ijmpd", "\\@spires{IMPAE\\%2CD#1\\%2C#3}{{\\it Int.\\ J.\\ Mod.\\ Phys.\\ }{\\bf D #1} (#2) #3}")?;
  def_jhep_journal("\\ijtp", "\\@spires{IJTPB\\%2CB#1\\%2C#3}{{\\it Int.\\ J.\\ Theor.\\ Phys.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\invm", "\\@spires{INVMB\\%2C#1\\%2C#3}{{\\it Invent.\\ Math.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\jag", "\\@spires{00124\\%2C#1\\%2C#3}{{\\it J.\\ Alg.\\ Geom.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\jams", "\\@spires{00052\\%2C#1\\%2C#3}{{\\it J.\\ Am.\\ Math.\\ Soc.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\jap", "\\@spires{JAPIA\\%2C#1\\%2C#3}{{\\it J.\\ Appl.\\ Phys.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\jdg", "\\@spires{JDGEA\\%2C#1\\%2C#3}{{\\it J.\\ Diff.\\ Geom.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\jgp", "\\@spires{JGPHE\\%2C#1\\%2C#3}{{\\it J.\\ Geom.\\ Phys.\\ }{\\bf #1} (#2) #3}")?;
  DefMacro!("\\jhep{}{}{}",   "\\href{http://jhep.sissa.it/stdsearch?paper=#1\\%28#2\\%29#3}{{\\it J. High Energy Phys.\\ }{\\bf #1} (#2) #3}");
  def_jhep_journal("\\jmp", "\\@spires{JMAPA\\%2C#1\\%2C#3}{{\\it J.\\ Math.\\ Phys.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\joth", "\\@spires{JOTHE\\%2C#1\\%2C#3}{{\\it J.\\ Operator Theory }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\jpha", "\\@spires{JPAGB\\%2CA#1\\%2C#3}{{\\it J. Phys.\\ }{\\bf A #1} (#2) #3}")?;
  def_jhep_journal("\\jphc", "\\@spires{JPAGB\\%2CC#1\\%2C#3}{{\\it J. Phys.\\ }{\\bf C #1} (#2) #3}")?;
  def_jhep_journal("\\jphg", "\\@spires{JPAGB\\%2CG#1\\%2C#3}{{\\it J. Phys.\\ }{\\bf G #1} (#2) #3}")?;
  def_jhep_journal("\\lmp", "\\@spires{LMPHD\\%2CA#1\\%2C#3}{{\\it Lett.\\ Math.\\ Phys.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\ncl", "\\@spires{NCLTA\\%2C#1\\%2C#3}{{\\it Lett.\\ Nuovo Cim.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\matan", "\\@spires{MAANA\\%2CA#1\\%2C#3}{{\\it Math.\\ Ann.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\mussr", "\\@spires{MUSIA\\%2CA#1\\%2C#3}{{\\it Math.\\ USSR Izv.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\mams", "\\@spires{MAMCA\\%2CA#1\\%2C#3}{{\\it Mem.\\ Am.\\ Math.\\ Soc.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\mpla", "\\@spires{MPLAE\\%2CA#1\\%2C#3}{{\\it Mod.\\ Phys.\\ Lett.\\ }{\\bf A #1} (#2) #3}")?;
  def_jhep_journal("\\mplb", "\\@spires{MPLAE\\%2CB#1\\%2C#3}{{\\it Mod.\\ Phys.\\ Lett.\\ }{\\bf B #1} (#2) #3}")?;
  def_jhep_journal("\\nature", "\\@spires{NATUA\\%2C#1\\%2C#3}{{\\it Nature }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\nim", "\\@spires{NUIMA\\%2C#1\\%2C#3}{{\\it Nucl.\\ Instrum.\\ Meth.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\npa", "\\@spires{NUPHA\\%2CA#1\\%2C#3}{{\\it Nucl.\\ Phys.\\ }{\\bf A #1} (#2) #3}")?;
  def_jhep_journal("\\npb", "\\@spires{NUPHA\\%2CB#1\\%2C#3}{{\\it Nucl.\\ Phys.\\ }{\\bf B #1} (#2) #3}")?;
  def_jhep_journal("\\npps", "\\@spires{NUPHZ\\%2C#1\\%2C#3}{{\\it Nucl.\\ Phys.\\ }{\\bf #1} {\\it(Proc.\\ Suppl.)} (#2) #3}")?;
  def_jhep_journal("\\nc", "\\@spires{NUCIA\\%2C#1\\%2C#3}{{\\it Nuovo Cim.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\ncs", "\\@spires{NUCUA\\%2C#1\\%2C#3}{{\\it Nuovo Cim.\\ Suppl.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\pan", "\\@spires{PANUE\\%2C#1\\%2C#3}{{\\it Phys.\\ Atom.\\ Nucl.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\pla", "\\@spires{PHLTA\\%2CA#1\\%2C#3}{{\\it Phys.\\ Lett.\\ }{\\bf A #1} (#2) #3}")?;
  def_jhep_journal("\\plb", "\\@spires{PHLTA\\%2CB#1\\%2C#3}{{\\it Phys.\\ Lett.\\ }{\\bf B #1} (#2) #3}")?;
  def_jhep_journal("\\pr", "\\@spires{PHRVA\\%2C#1\\%2C#3}{{\\it Phys.\\ Rev.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\pra", "\\@spires{PHRVA\\%2CA#1\\%2C#3}{{\\it Phys.\\ Rev.\\ }{\\bf A #1} (#2) #3}")?;
  def_jhep_journal("\\prb", "\\@spires{PHRVA\\%2CB#1\\%2C#3}{{\\it Phys.\\ Rev.\\ }{\\bf B #1} (#2) #3}")?;
  def_jhep_journal("\\prc", "\\@spires{PHRVA\\%2CC#1\\%2C#3}{{\\it Phys.\\ Rev.\\ }{\\bf C #1} (#2) #3}")?;
  def_jhep_journal("\\prd", "\\@spires{PHRVA\\%2CD#1\\%2C#3}{{\\it Phys.\\ Rev.\\ }{\\bf D #1} (#2) #3}")?;
  def_jhep_journal("\\pre", "\\@spires{PHRVA\\%2CE#1\\%2C#3}{{\\it Phys.\\ Rev.\\ }{\\bf E #1} (#2) #3}")?;
  def_jhep_journal("\\prep", "\\@spires{PRPLC\\%2C#1\\%2C#3}{{\\it Phys.\\ Rept.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\prl", "\\@spires{PRLTA\\%2C#1\\%2C#3}{{\\it Phys.\\ Rev.\\ Lett.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\phys", "\\@spires{PHYSA\\%2CA#1\\%2C#3}{{\\it Physica }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\plms", "\\@spires{PHLTA\\%2CB#1\\%2C#3}{{\\it Proc.\\ London Math.\\ Soc.\\ }{\\bf B #1} (#2) #3}")?;
  def_jhep_journal("\\pnas", "\\@spires{PNASA\\%2C#1\\%2C#3}{{\\it Proc.\\ Nat.\\ Acad.\\ Sci.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\ppnp", "\\@spires{PPNPD\\%2C#1\\%2C#3}{{\\it Prog.\\ Part.\\ Nucl.\\ Phys.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\ptp", "\\@spires{PTPKA\\%2C#1\\%2C#3}{{\\it Prog.\\ Theor.\\ Phys.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\ptps", "\\@spires{PTPSA\\%2C#1\\%2C#3}{{\\it Prog.\\ Theor.\\ Phys.\\ Suppl.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\rmp", "\\@spires{RMPHA\\%2C#1\\%2C#3}{{\\it Rev.\\ Mod.\\ Phys.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\sjnp", "\\@spires{SJNCA\\%2C#1\\%2C#3}{{\\it Sov.\\ J.\\ Nucl.\\ Phys.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\sjpn", "\\@spires{SJPNA\\%2C#1\\%2C#3}{{\\it Sov.\\ J.\\ Part.\\ Nucl.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\jetp", "\\@spires{SPHJA\\%2C#1\\%2C#3}{{\\it Sov.\\ Phys.\\ JETP\\/ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\jetpl", "\\@spires{JTPLA\\%2C#1\\%2C#3}{{\\it Sov.\\ Phys.\\ JETP Lett.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\spu", "\\@spires{SOPUA\\%2C#1\\%2C#3}{{\\it Sov.\\ Phys.\\ Usp.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\tmf", "\\@spires{TMFZA\\%2C#1\\%2C#3}{{\\it Teor.\\ Mat.\\ Fiz.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\tmp", "\\@spires{TMPHA\\%2C#1\\%2C#3}{{\\it Theor.\\ Math.\\ Phys.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\ufn", "\\@spires{UFNAA\\%2C#1\\%2C#3}{{\\it Usp.\\ Fiz.\\ Nauk.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\ujp", "\\@spires{00267\\%2C#1\\%2C#3}{{\\it Ukr.\\ J.\\ Phys.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\yf", "\\@spires{YAFIA\\%2C#1\\%2C#3}{{\\it Yad.\\ Fiz.\\ }{\\bf #1} (#2) #3}")?;
  def_jhep_journal("\\zpc", "\\@spires{ZEPYA\\%2CC#1\\%2C#3}{{\\it Z.\\ Physik }{\\bf C #1} (#2) #3}")?;
  def_jhep_journal("\\zetf", "\\@spires{ZETFA\\%2C#1\\%2C#3}{{\\it Zh.\\ Eksp.\\ Teor.\\ Fiz.\\ }{\\bf #1} (#2) #3}")?;
  DefMacro!("\\newjournal{}{}{}{}{}", "\\@spires{#2\\%2C#3\\%2C#5}{{\\it #1 }{\\bf #3} (#4) #5}");
  DefMacro!("\\ibid{}{}{}",   "{\\it ibid.\\ }{\\bf #1} (#2) #3");

  // arXiv category links — Perl L244-261
  DefMacro!("\\hepth{}",   "\\href{http://xxx.lanl.gov/abs/hep-th/#1}{\\tt hep-th/#1}");
  DefMacro!("\\hepph{}",   "\\href{http://xxx.lanl.gov/abs/hep-ph/#1}{\\tt hep-ph/#1}");
  DefMacro!("\\heplat{}",  "\\href{http://xxx.lanl.gov/abs/hep-lat/#1}{\\tt hep-lat/#1}");
  DefMacro!("\\hepex{}",   "\\href{http://xxx.lanl.gov/abs/hep-ex/#1}{\\tt hep-ex/#1}");
  DefMacro!("\\nuclth{}",  "\\href{http://xxx.lanl.gov/abs/nucl-th/#1}{\\tt nucl-th/#1}");
  DefMacro!("\\nuclex{}",  "\\href{http://xxx.lanl.gov/abs/nucl-ex/#1}{\\tt nucl-ex/#1}");
  DefMacro!("\\grqc{}",    "\\href{http://xxx.lanl.gov/abs/gr-qc/#1}{\\tt gr-qc/#1}");
  DefMacro!("\\qalg{}",    "\\href{http://xxx.lanl.gov/abs/q-alg/#1}{\\tt q-alg/#1}");
  DefMacro!("\\accphys{}", "\\href{http://xxx.lanl.gov/abs/accphys/#1}{\\tt accphys/#1}");
  DefMacro!("\\alggeom{}", "\\href{http://xxx.lanl.gov/abs/alg-geom/#1}{\\tt alg-geom/#1}");
  DefMacro!("\\astroph{}", "\\href{http://xxx.lanl.gov/abs/astro-ph/#1}{\\tt astro-ph/#1}");
  DefMacro!("\\chaodyn{}", "\\href{http://xxx.lanl.gov/abs/chao-dyn/#1}{\\tt chao-dyn/#1}");
  DefMacro!("\\condmat{}", "\\href{http://xxx.lanl.gov/abs/cond-mat/#1}{\\tt cond-mat/#1}");
  DefMacro!("\\nlinsys{}", "\\href{http://xxx.lanl.gov/abs/nlin-sys/#1}{\\tt nlin-sys/#1}");
  DefMacro!("\\quantph{}", "\\href{http://xxx.lanl.gov/abs/quant-ph/#1}{\\tt quant-ph/#1}");
  DefMacro!("\\solvint{}", "\\href{http://xxx.lanl.gov/abs/solv-int/#1}{\\tt solv-int/#1}");
  DefMacro!("\\Math{}{}", "\\href{http://xxx.lanl.gov/abs/math.#1/#2}{\\tt math.#1/#2}");

  // Conditionals — Perl L267-291
  TeX!(r"
  \newif\if@preprint\@preprinttrue
  \newif\if@draft\@draftfalse
  \newif\if@hyper\@hypertrue
  \newif\if@proc\@procfalse
  \newif\if@author\@authorfalse
  \newif\if@abstract\@abstractfalse
  \newif\if@keywords\@keywordsfalse
  \newif\if@todotoc\@todotocfalse
  \newif\if@rece\@recefalse
  \newif\if@revi\@revifalse
  \newif\if@acce\@accefalse
  \newif\if@conf\@conffalse
  \newif\if@speaker\@speakerfalse
  ");

  // Perl L293-308: Names
  DefMacro!("\\acknowlname", "Acknowledgments");
  DefMacro!("\\receivedname", "Received:");
  DefMacro!("\\revisedname", "Revised:");
  DefMacro!("\\acceptedname", "Accepted:");
  DefMacro!("\\keywordsname", "Keywords:");
  DefMacro!("\\abstractname", "Abstract:");
  def_macro_noop("\\JHEP@todaysname")?;
  DefMacro!("\\preprintname", "PREPRINT");
  DefMacro!("\\appendixname", "Appendix");
  DefMacro!("\\contentsname", "Contents");
  DefMacro!("\\listfigurename", "List of figures");
  DefMacro!("\\listtablename", "List of tables");
  DefMacro!("\\refname", "References");
  DefMacro!("\\figurename", "Figure");
  DefMacro!("\\tablename", "Table");
  DefMacro!("\\partname", "Part");
});
