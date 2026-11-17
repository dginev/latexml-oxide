//! elsart_support_core.sty — Elsevier journal article support (core)
//! Perl: elsart_support_core.sty.ltxml — 191 lines
//! Shared by elsart.cls and elsarticle.cls
use crate::{engine::base_utilities::reading_author_fields, prelude::*};

/// This binding's frontmatter conventions (the observer's, OXIDIZED_DESIGN_DIVERGENCES #837), declared at its load
/// (`declare_frontmatter_conventions`, user ruling 2026-10-28: each binding declares its own): the kernel's reading,
/// written (97e, OD #948), for the `\author[keys]{…}` of elsarticle, elsart and the classes keeping this binding's
/// (entics, arxbj; not autart, which loads OmniBus and its own `\address`, autart_cls.rs, and only this binding's keyword
/// markup): their title prints its notes in the body's arabic `\thefootnote` (elsarticle.cls:819, :836
/// `\gdef\thefootnote{\arabic{footnote}}`, no `\@fnsymbol` in its `\maketitle`; elsart.cls v2.18 :1107-1108
/// `\xarabic{footnote}` for the authors' notes), so `M.~Pesce-Rollins\footnotemark[1]` shows "1", not the kernel's "∗";
/// and an `\address[key]`/`\affiliation[key]` key is never printed — the class prints the entry's place, `\theaffn`
/// (elsarticle.cls:656-662), `\theaddress` (elsart.cls :1135-1164) —, so no footnote's mark answers it (`keyed_labels`;
/// 1108.0201's `\address[*]{current address}`, which pdflatex prints "bb current address" and no author requests); the
/// authors' keys do, and a mark typed by hand, as keys in order spell the places printed (0907.5014). Not handled (rare):
/// old elsart's proofing switch prints the key itself (`\if@Elproofing \def\@eltag{#4}`, elsart.cls v2.12 :1026), where
/// `keyed_labels` is wrong
pub const FRONTMATTER_CONVENTIONS: FmConventions = FmConventions {
  name: "elsart",
  binding: "elsart_support_core.sty",
  title_marks: FmTitleMarks::Body,
  keyed_labels: true,
  // (elsarticle's title page prints `\elsauthors`, never `\@author`, a setter of its own there: elsarticle.cls:247-288,
  // :805, :848 — a document's tail on `\@author` prints nowhere, FX27b; not old elsart.cls's, unverified)
  title_page: FmTitlePageDecl {
    author: FmAuthorPage::Own(&["elsarticle.cls"]),
    // (its `\maketitle` never runs `\@maketitle`, `\elsarticleprelims\pprintMaketitle` and its journal variants:
    // elsarticle.cls:918-989; a document's own prints nowhere, TP1b)
    maketitle_runs: &[(
      "elsarticle.cls",
      FmMaketitleRuns::Never,
      "elsarticle.cls:918-989",
    )],
    ..FM_KERNEL_CONVENTIONS.title_page
  },
  cite: "elsarticle.cls:656-662, :819, :836; elsart.cls v2.18 :1107-1108, :1135-1164; OD #948 (1108.0201)",
  ..FM_KERNEL_CONVENTIONS
};

#[rustfmt::skip]
LoadDefinitions!({
  // (this binding's frontmatter conventions, first: OXIDIZED_DESIGN_DIVERGENCES #837)
  declare_frontmatter_conventions(&FRONTMATTER_CONVENTIONS);
  // Frontmatter environment (Perl PR #2767)
  DefPrimitive!(T_CS!("\\begin{frontmatter}"), None, None);
  DefPrimitive!(T_CS!("\\end{frontmatter}"), None, None);
  // The environment's begin macro, which `\newenvironment{frontmatter}{}{\maketitle}` (elsarticle.cls:1303) defines
  // empty: a bare `\frontmatter` before `\maketitle` is a no-op (1111.1604, 2007.04268, 1309.1731), as OmniBus's.
  Let!("\\frontmatter", "\\@empty");

  // \author[mark]{name}
  // One \author per author! possibly with a mark to connect to affiliation
  // It either should be followed by \affiliation,
  // or both should get matching marks
  // (elsarticle prints the argument as written, so a name list in one `\author{Ann Able, Bob Baker}` is several people:
  // read as the per-author classes read theirs, each name given the marks; Perl's elsart_support_core.sty.ltxml:35-36
  // makes one creator. 2511.07560, 2401.02456)
  // (the keys an author lists and the `\corref`/`\cormark` keys its name holds, recorded for elsarticle's `\cortext` mark,
  // which depends on the route its names reach it by: `\lx@els@cormark`, 2512.04868)
  DefMacro!("\\author OptionalSemiverbatim {}",
    "\\lx@els@record@author{#1}{#2}\\if\\relax\\detokenize{#1}\\relax\\else\\gdef\\lx@els@labelled{}\\fi\\lx@els@author{#1}{#2}");
  // elsarticle.cls:126-131, :159-165, :188-190, :250-265: every route to a key gets the label form it was first read by
  // (`els_route`); the first name shows the outside mark read last (`ELS_FIRST`; 1501.03323, 1508.05262, 2201.04925,
  // 2606.17032, 2606.22682; OD #1067).
  // (keep in elsart_support_core: `policy_for` takes its conventions from here, OD #1067)
  DefMacro!("\\lx@els@author{}{}", sub[(keys, name)] {
    let listed: Vec<String> =
      keys.to_string().split(',').map(str::trim).filter(|k| !k.is_empty()).map(String::from).collect();
    let first = els_authors().len() == 1;
    let tokens = name.clone().unlist();
    let (correfs, fnrefs) = (
      ref_spans(&tokens, &T_CS!("\\corref"), &T_CS!("\\cormark")),
      ref_spans(&tokens, &T_CS!("\\fnref"), &T_CS!("\\fnmark")),
    );
    for key in &listed {
      set_els_route(key, "");
    }
    let mut labels: Vec<String> = listed.iter().map(|key| els_route_label(key, "")).collect();
    // (a `\corref`/`\fnref` naming a key a name listed first is that key: the request dropped, the key annotated)
    let mut dropped: Vec<std::ops::Range<usize>> = Vec::new();
    for (kind, spans) in [("cor", &correfs), ("fn", &fnrefs)] {
      for (span, refs) in spans.iter() {
        if refs.iter().any(|key| els_route(key).as_deref() == Some("")) {
          labels.extend(refs.iter().map(|key| els_route_label(key, kind)));
          dropped.push(span.clone());
        }
      }
    }
    for (kind, spans) in [("cor", &correfs), ("fn", &fnrefs)] {
      spans.iter().flat_map(|(_, refs)| refs).for_each(|key| set_els_route(key, kind));
    }
    if first {
      // (the outside marks read before it or right after its call; a later one, `els_first_note_label`)
      let ahead = peek_outside_marks()?;
      let mut took = String::new();
      for (kind, own) in [("cor", &correfs), ("fn", &fnrefs)] {
        // (its own `\corref`/`\fnref` replaces the outside mark; its own `\cormark` prints beside it, :178-180)
        if own.iter().any(|(span, _)| kind == "fn" || tokens[span.start] == T_CS!("\\corref")) {
          continue;
        }
        let read = ahead.iter().rev().find(|(k, _)| *k == kind).map(|(_, keys)| keys.clone());
        let read = read.unwrap_or_else(|| els_outside_mark(kind));
        labels.extend(read.iter().map(|key| els_route_label(key, kind)));
        took.push_str(&s!("{kind}={};", read.join(",")));
      }
      assign_value(ELS_FIRST, Stored::String(pin(&took)), Scope::Global);
    }
    let named: Vec<Token> = tokens
      .iter()
      .enumerate()
      .filter(|(at, _)| !dropped.iter().any(|span| span.contains(at)))
      .map(|(_, t)| *t)
      .collect();
    let mut call = mouth::tokenize_internal(TeXString::assembled(s!(
      "\\lx@add@authors@append[annotations={{{}}}]",
      labels.join(",")
    )))
    .unlist();
    call.push(T_BEGIN!());
    call.extend(named);
    call.push(T_END!());
    Ok(Tokens::new(call))
  });
  // (and the keys an `\address`/`\affiliation` labels: a key a note's text shares prints whichever `\elsLabel` wrote
  // it last, elsarticle.cls:126-131 — 1711.08299's `\cortext[a]` beside `\address[a]`, its names printing "a")
  DefPrimitive!("\\lx@els@record@place{}", sub[(key)] {
    record_els_keys(ELS_PLACE_KEYS, std::iter::once(key.to_string().as_str()));
  });
  DefPrimitive!("\\lx@els@record@author{}{}", sub[(keys, name)] {
    let keys = keys.to_string();
    record_els_keys(ELS_AUTHOR_KEYS, keys.split(','));
    let tokens = name.unlist_ref();
    let correfs = keys_in(tokens, &T_CS!("\\corref"), &T_CS!("\\cormark"));
    record_els_keys(ELS_CORREF_KEYS, correfs.iter().map(String::as_str));
    // (each author's keys and the `\fnref`s of its name: the marks it prints side by side; and the group
    // level the names are read at, an outside mark's)
    let fnrefs = keys_in(tokens, &T_CS!("\\fnref"), &T_CS!("\\fnmark"));
    let keys: Vec<&str> = keys.split(',').map(str::trim).filter(|k| !k.is_empty()).collect();
    let mut authors = lookup_string(ELS_AUTHORS);
    authors.push_str(&s!("{}|{}|{};", keys.join(","), fnrefs.join(","), correfs.join(",")));
    assign_value(ELS_AUTHORS, Stored::String(pin(&authors)), Scope::Global);
    assign_value(ELS_NAMES_LEVEL, i64::try_from(get_frame_depth()).unwrap_or(0), Scope::Global);
  });

  // (elsarticle puts an `\address` into the author list after the authors before it, elsarticle.cls:665-668, and an
  // `\affiliation` into the address list after all of them, :647-653; with no labels to link them, an unlabelled one is
  // read by authoring order: the run of authors with none yet, `annotate=run` as `\lx@els@shared` chooses (2609.06522,
  // 2609.07632). Once an author is labelled, an unlabelled one names no author — `corresponding author={…}`,
  // 2609.07780 — and stays the last's, as in Perl; a labelled one is found by its label, which `annotate` leaves alone)
  DefMacro!("\\lx@els@shared",
    "\\ifdefined\\lx@els@labelled\\expandafter\\@firstoftwo\\else\\expandafter\\@secondoftwo\\fi");
  DefMacro!("\\address OptionalSemiverbatim {}",
    "\\lx@els@record@place{#1}\\lx@els@shared{\\lx@add@contact[label={#1},role=address]}{\\lx@add@contact[label={#1},role=address,annotate=run]}{#2}");
  // \affiliation[label]{text-OR-keyvals} !!  (Perl PR #2767)
  DefMacro!("\\affiliation OptionalSemiverbatim {}",
    "\\lx@els@record@place{#1}\\lx@els@shared{\\lx@add@contact[label={#1},role=affiliation]}\
     {\\lx@add@contact[label={#1},role=affiliation,annotate=run]}{\\lx@els@parse@affiliation{#2}}");

  // The affiliation as the class prints it from its `stm/affiliation` keys ([`stm_affiliation`]).
  DefMacro!("\\lx@els@parse@affiliation {}", sub[(raw)] { stm_affiliation(raw, true) });
  // Redefine to account for the label, which we ignore for now!
  DefConstructor!("\\thanks[]{}", "<ltx:note role='thanks'>#2</ltx:note>", sizer => sub[_whatsit] { Ok(out_of_line_size()) });

  // Is this significantly different?
  // Perl elsart_support_core.sty.ltxml: body is `\author{#1}` but in
  // the `OptionalMatch:* {}` signature `#1` is the star flag and `#2` is
  // the content — the author name is silently dropped. Documented as a
  // Perl bug in docs/parity/KNOWN_PERL_ERRORS.md #16 (same root cause as
  // aipproc.cls.ltxml's \tablenote). Rust deliberately uses `#2` so the
  // author content reaches \author correctly.
  DefMacro!("\\collab OptionalMatch:* {}", "\\author{#2}");
  Let!("\\collaboration", "\\collab");

  // These pairs add various contact information to authors.  (Perl PR #2767)
  // The \<XXX>ref forms are used within the \author text to anchor the connection
  // the \<XXX>text forms supply the text (& and role) for the contact that will be attached.
  DefMacro!("\\thanksref{}",  "\\lx@request@frontmatter@annotation[thanks]{#1}");
  // (a `\corref` outside an `\author`'s name recorded where it is read, one inside it by `\lx@els@record@author`)
  // (an outside one recorded with the group level it is read at, for elsarticle's first name: `ELS_OUTSIDE`)
  DefMacro!("\\corref{}",     "\\lx@els@record@corref{#1}\\lx@request@frontmatter@annotation[cor]{#1}");
  DefPrimitive!("\\lx@els@record@corref{}", sub[(key)] {
    record_els_cor_key(&key.to_string());
    record_els_mark("cor", &key.to_string());
  });
  // (elsarticle's `\cormark[k]` records the key alone: it prints its ∗ where it is read and sets no `\@corref`
  // (elsarticle.cls:178-180), so one outside a name marks no name)
  DefPrimitive!("\\lx@els@record@cormark{}", sub[(key)] {
    record_els_cor_key(&key.to_string());
  });
  DefMacro!("\\corauthref{}", "\\lx@request@frontmatter@annotation[corauth]{#1}");
  DefMacro!("\\fnref{}",      "\\lx@els@record@fnref{#1}\\lx@request@frontmatter@annotation[fn]{#1}");
  DefPrimitive!("\\lx@els@record@fnref{}", sub[(keys)] {
    let keys = keys.to_string();
    record_els_mark("fn", &keys);
    keys.split(',').for_each(|key| set_els_route(key, "fn"));
  });
  DefMacro!("\\tnoteref{}",   "\\lx@request@frontmatter@annotation[tnote]{#1}"); // Possibly for title, not author?
  DefMacro!("\\thanks OptionalSemiverbatim {}",
    "\\lx@add@contact[label={thanks:#1},role=thanks]{#2}");
  DefMacro!("\\cortext OptionalSemiverbatim {}",
    "\\lx@add@contact[label={cor:#1},role=correspondent]{#2}");
  DefMacro!("\\corauth OptionalSemiverbatim {}",
    "\\lx@add@contact[label={corauth:#1},role=correspondent]{#2}");
  DefMacro!("\\fntext OptionalSemiverbatim {}",
    "\\lx@add@contact[label={fn:#1},role=note]{#2}");
  DefMacro!("\\tnotetext OptionalSemiverbatim {}",
    "\\lx@add@contact[label={tnote:#1},role=note]{#2}"); // title note?

  // Title/metadata (Perl PR #2767)
  // \runauthor / \runtitle are running-header SHORT forms (real elsart.cls
  // L1235 just stores them for `\@oddhead`; never typeset in the body). Perl
  // gobbles both — layout-only and redundant with `\author`/`\title`. Prior
  // over-preservation digested the running-head content, so an author typo
  // (stray `\` before a name) hit undefined-CS. Witness 1503.06349.
  def_macro_noop("\\runauthor{}")?;
  def_macro_noop("\\runtitle{}")?;
  DefMacro!("\\subtitle{}", "\\lx@add@subtitle{#1}");
  // \ead[label]{email} provides email address for the preceding \author (no marks used), set in typewriter type
  // (elsarticle.cls:215-219 `{\ttfamily\expandafter\strip@prefix\meaning\@@tmp}`, :232-235 a `[url]`'s;
  // 1601.03650 `vuongbv_56@…`)
  DefMacro!("\\ead Optional:email Semiverbatim",
    "\\lx@add@contact[role=#1]{\\lx@address@urlfont{#2}}");
  DefMacro!("\\sep", "\\unskip,\\space");
  DefMacro!("\\received{}", "\\lx@add@date[role=received]{#1}");
  DefMacro!("\\revised{}", "\\lx@add@date[role=revised]{#1}");
  DefMacro!("\\accepted{}", "\\lx@add@date[role=accepted]{#1}");
  DefMacro!("\\communicated{}", "\\lx@add@date[role=communicated]{#1}");
  // No label, as elsart.cls prints none (FIDELITY_CHARTER §4, user ruling 2026-10-30; OXIDIZED_DESIGN_DIVERGENCES #921) (quant-ph0004056's copy): :1165-1168 the
  // dedication centred in italics, :1131 the article's type, :1080-1081 and :1203 the volume in "Article published in
  // J V (year) pages", the issue stored, not printed (elsarticle.cls defines none of the four). `\journal` below keeps
  // the role's label: what the class prints around it depends on the version (SYNC_STATUS, the 92e row).
  DefMacro!("\\dedicated{}", "\\lx@add@pubnote[role=dedication,name={}]{#1}");
  DefMacro!("\\presented{}", "\\lx@add@date[role=presented]{#1}");
  DefMacro!("\\articletype{}", "\\lx@add@pubnote[role=type,name={}]{#1}");
  DefMacro!("\\issue{}", "\\lx@add@pubnote[role=issue,name={}]{#1}");
  // elsarticle.cls:1301-1302 (TL v3.4c; elsart.cls v2.18:1215 alike): `\journal` stores the journal's name in
  // `\@journal`, empty until set (elsarticle.cls:1302 `\let\@journal\@empty`, after :32's "Elsevier Ltd" served its
  // `\ProvidesClass` line; elsart.cls keeps its publisher, v2.18:28 "Elsevier Science", which elsart_cls.rs sets), which
  // the class's first-page footer prints ("Preprint submitted to \@journal") and papers' own footers and title code read (run-347 inventory: 48 elsarticle papers, 32 with a copy of their own, 13 elsart;
  // 1212.6083, 1007.3192, 0812.4577): the frontmatter keeps it as a note, the store as the class keeps it.
  RawTeX!(r"\let\@journal\@empty");
  DefMacro!("\\journal{}", "\\gdef\\@journal{#1}\\lx@add@pubnote[role=journal]{#1}");
  DefMacro!("\\volume{}", "\\lx@add@pubnote[role=volume,name={}]{#1}");
  DefMacro!("\\pubyear{}", "\\lx@add@date[role=publication]{#1}");
  def_macro_noop("\\FullCopyrightText")?;
  DefMacro!("\\copyear{}", "\\lx@add@copyrightyear{#1}");
  DefMacro!("\\copyrightholder{}", "\\lx@add@copyrightholder{#1}");
  Let!("\\copyrightyear", "\\copyear");
  def_macro_noop("\\RUNART")?;
  def_macro_noop("\\RUNDATE")?;
  def_macro_noop("\\RUNJNL")?;
  // Round-34 surpass-Perl: company/article-id are author metadata.
  DefMacro!("\\company{}",
    "\\lx@add@frontmatter{ltx:note}[role=company]{#1}");
  DefMacro!("\\aid{}",
    "\\lx@add@frontmatter{ltx:note}[role=article-id]{#1}");
  def_macro_noop("\\ssdi{}{}")?;
  def_macro_noop("\\readRCS Until:$ Until:$")?;
  def_macro_noop("\\RCSdate")?;
  def_macro_noop("\\RCSfile")?;
  def_macro_noop("\\RCSversion")?;
  DefMacro!("\\firstpage{}",
    "\\lx@add@frontmatter{ltx:note}[role=firstpage]{#1}");
  DefMacro!("\\lastpage{}",
    "\\lx@add@frontmatter{ltx:note}[role=lastpage]{#1}");
  def_macro_noop("\\preface")?;
  def_macro_noop("\\theHaddress")?;
  def_macro_noop("\\theaddress")?;
  Let!("\\ESpagenumber", "\\arabic");

  // Acknowledgements — Perl L123-125
  DefConstructor!("\\ack", "<ltx:acknowledgements>");
  DefConstructor!("\\endack", "</ltx:acknowledgements>");

  // Acknowledgements tag — Perl L125
  // ltx:acknowledgements Tag (autoClose + inlist=toc) is global — set in
  // latex_constructs.rs (arXiv-fork 23771504 removed the binding-local copies).

  // Keywords — Perl L130-153 (`define_keyword_markup`, which autart.cls's binding shares: autart.cls carries
  // elsart.cls's `{keyword}`, autart.cls:1248-1258 of 1106.2518's copy).
  define_keyword_markup()?;

  // Document structure — Perl L158-163
  DefMacro!("\\theparagraph", "\\thesubsubsection.\\arabic{paragraph}");
  DefMacro!("\\thesubparagraph", "\\theparagraph.\\arabic{subparagraph}");

  // Per-section equation numbering — Perl L161-163.
  // Emit `\@addtoreset{equation}{section}` + per-section
  // `\theequation` when the `seceqn` class option is active.
  if lookup_bool("@seceqn") {
    RawTeX!(r"\@addtoreset{equation}{section}");
    DefMacro!("\\theequation", "\\thesection.\\arabic{equation}");
  }

  // Theorems — Perl elsart_support_core.sty.ltxml L200-204: `{thm}` is elsart.cls's, declared by elsart_support_sty.rs;
  // elsarticle.cls has none (its own `\newdefinition` :1154, `\newproof` :1200), so an elsarticle preamble's
  // `\newcommand{\thm}` stands (2212.01438; Perl gives elsarticle the `{thm}` too, KNOWN_PERL_ERRORS #965).
  Let!("\\newdefinition", "\\newtheorem");
  Let!("\\newproof", "\\newtheorem");

  // Registers — Perl L180-183
  DefRegister!("\\eqnarraycolsep" => Dimension!("1pt"));
  DefRegister!("\\eqnbaselineskip" => Glue!("14pt"));
  DefRegister!("\\eqnlineskip" => Glue!("2pt"));
  DefRegister!("\\eqntopsep" => Glue!("12pt"));

  // Figures — Perl L186-191
  def_macro_noop("\\printfigures{}")?;
  def_macro_noop("\\printtables{}")?;
  def_macro_noop("\\MARK{}")?;
  def_macro_noop("\\mpfootnotemark")?;

  // Perl elsart_support_core.sty.ltxml L189:
  //   DefMacro('\note{}', "<ltx:note>#1</ltx:note>");    # ?
  // This is a *DefMacro* (token expansion), NOT a DefConstructor — so the
  // body tokenises to LITERAL TEXT (`<`, `>` are catcode-OTHER): `\note{X}`
  // expands to the characters `<ltx:note>` + X + `</ltx:note>`, NOT a real
  // <ltx:note> element. Perl's own `# ?` flags it as questionable, but it is
  // the ground truth and crucially is ERROR-FREE: a block argument such as
  // `\note{\begin{remark}…\end{remark}}` (a `\newtheorem`-based environment)
  // renders the remark as a normal <ltx:theorem> bracketed by stray literal
  // text, with no content-model violation. Porting it as a DefConstructor
  // (real <ltx:note>) instead made `ltx:theorem isn't allowed in <ltx:note>`
  // — papers that wrap a theorem-like env in `\note{…}` (common with a
  // user `\newcommand\note[1]{…}` that LaTeX/Perl *ignore* because elsart
  // already defined `\note`) then failed. Match Perl: use DefMacro.
  // Witness 2006.06087 (elsarticle, `\note{\begin{remark}…}`): 1 error → 0.
  DefMacro!("\\note{}", "<ltx:note>#1</ltx:note>");

  // Float environment
  DefEnvironment!("{esmark}",  "#body");
  def_macro_noop("\\figmark{}{}")?;
  def_macro_noop("\\tabmark{}{}")?;

  // \qed (proof end-of-proof marker). Previously only in elsart_support
  // (NOT loaded by elsarticle.cls — only by elsart.cls), so plain
  // elsarticle papers using \qed got `Error:undefined:\qed`. Move the
  // def here so all elsart_*-loading classes (elsarticle + elsart)
  // have it. Witness 2306.02411 (elsarticle Pfaffian paper).
  DefMacro!("\\qed", "\\ltx@qed");
  // Sized as elsart.cls:457's `$\Box$` and TL elsarticle.cls:1238 `\hbox{}\hfill$\Box$` (`\Box` from latexsym — lasy10
  // "32, 0.747225 × 0.444446 em — or amssymb — msam10 `\square`, .777781 × .675 em: a glyph box of comparable size), so
  // a cell or `\mbox` holding only it is not dropped as empty (Perl Alignment.pm:458-470). Synthetic: 1905.10621's
  // cells keep amsthm's `\qed` (krudces.sty:188 loads amsthm after the class), sized by `openbox_size`.
  DefConstructor!("\\ltx@qed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
    enter_horizontal => true,
    reversion => "\\qed",
    sizer => sub[whatsit] {
      Ok(engine::latex_constructs::symbol_box_size(
        whatsit,
        0.747225,
        0.444446,
        engine::latex_constructs::FontUnit::Em,
      ))
    });
});

/// The separator slots of the `stm/affiliation` keys, in [`stm_affiliation`]'s order.
const STM_ORGANIZATION: usize = 0;
const STM_ADDRESS_LINE: usize = 1;
const STM_CITY: usize = 2;
const STM_POSTAL_CODE: usize = 3;
const STM_STATE: usize = 4;
const STM_COUNTRY: usize = 5;

/// `\affiliation`'s argument printed as elsarticle.cls:387-440 and cas-common.sty:1118-1160 print their
/// `stm/affiliation` keys (the value commands at elsarticle.cls:330-356, cas-common.sty:1061-1090): `organization`
/// (`o`, `or`, and in elsarticle `organisation`), `addressline` (`a`, `ad`), `city` (`c`, `ci`), `postcode` (`p`,
/// `pc`) and `state` (`s`, `st`) as the value, its separator — `,` unless `<key>sep` (`op`, `ap`, `cp`, `pp`, `sp`,
/// `orp`, …) gave another — and a space; `country` (`cy`) as the value and its own separator (`cyp`, none by
/// default); a key the set does not know as its value, or, given none, its own text, and a space. So
/// `organization={X}, city={Y}, citysep={}, postcode={1}` is "X, Y 1" (2609.01851), and a braced `{ARTORG Center,}`
/// part is kept (2609.04891). Three departures from the PDF (OXIDIZED_DESIGN_DIVERGENCES #472): an argument with no
/// key at all (`Univ, City, Country`) is kept as written, its commas with it, where the class reads each part as an
/// unknown key and drops them; the separator the class leaves after a last part that is not a country ("Univ A,") is
/// not kept; a key's own text is typeset, not printed as its string. Perl (PR #2767) joined the known values with
/// ", " and dropped the rest (KNOWN_PERL_ERRORS #545). `elsarticle` selects that class's set, which cas-common lacks
/// two keys of (`organisation`, `oraganisationsep`).
pub fn stm_affiliation(raw: Tokens, elsarticle: bool) -> Result<Tokens> {
  let items = top_level_items(raw.unlist_ref(), ',');
  if items
    .iter()
    .all(|item| top_level_items(item, '=').len() < 2)
  {
    return Ok(raw);
  }
  // `\__reset_affiliation:` (cas-common.sty:1164-1172) before each `\affiliation`
  let mut separators: [Vec<Token>; 6] = [
    vec![T_OTHER!(",")],
    vec![T_OTHER!(",")],
    vec![T_OTHER!(",")],
    vec![T_OTHER!(",")],
    vec![T_OTHER!(",")],
    Vec::new(),
  ];
  // Each value is set down with its separator's macro (`\ca_aff_city[\l_city_punc_tl]{…}`), which the class expands
  // only when it assembles the affiliation (cas-common.sty:1262 `\csxdef`): a `citysep={}` after the `city` still
  // applies. So the parts are kept with their separator slot, and the separators read at the end.
  let mut parts: Vec<(Vec<Token>, Option<usize>)> = Vec::new();
  for item in items {
    let (key, value) = match top_level_items(&item, '=').as_slice() {
      [key] => (unbraced(key), None),
      [key, value @ ..] => (unbraced(key), Some(unbraced(&value.join(&T_OTHER!("="))))),
      [] => continue,
    };
    if key.is_empty() && value.is_none() {
      continue;
    }
    let name = Tokens::new(key.clone()).to_string();
    let separator_key = match name.as_str() {
      "op" | "oraganizationsep" | "orp" => Some(STM_ORGANIZATION),
      "oraganisationsep" if elsarticle => Some(STM_ORGANIZATION),
      "ap" | "addresslinesep" | "adp" => Some(STM_ADDRESS_LINE),
      "cp" | "citysep" | "cip" => Some(STM_CITY),
      "pp" | "postcodesep" | "pcp" => Some(STM_POSTAL_CODE),
      "sp" | "statesep" | "stp" => Some(STM_STATE),
      "cyp" => Some(STM_COUNTRY),
      _ => None,
    };
    if let Some(slot) = separator_key {
      separators[slot] = value.unwrap_or_default();
      continue;
    }
    let value_key = match name.as_str() {
      "o" | "or" | "organization" => Some(STM_ORGANIZATION),
      "organisation" if elsarticle => Some(STM_ORGANIZATION),
      "a" | "ad" | "addressline" => Some(STM_ADDRESS_LINE),
      "c" | "ci" | "city" => Some(STM_CITY),
      "p" | "pc" | "postcode" => Some(STM_POSTAL_CODE),
      "s" | "st" | "state" => Some(STM_STATE),
      "cy" | "country" => Some(STM_COUNTRY),
      _ => None,
    };
    match value_key {
      Some(slot) => parts.push((value.unwrap_or_default(), Some(slot))),
      // the unknown key's value, or its own text (`\l_keys_key_tl`)
      None => parts.push((value.filter(|v| !v.is_empty()).unwrap_or(key), None)),
    }
  }
  // A separator separates: none follows a last part that is not the country, where the class, expecting a `country`
  // there, prints one with nothing after it ("Univ A," for a lone `organization`); the country's own (`cyp`) stays.
  let mut out: Vec<Token> = Vec::new();
  let count = parts.len();
  for (i, (value, slot)) in parts.into_iter().enumerate() {
    out.extend(value);
    if i + 1 == count && slot != Some(STM_COUNTRY) {
      break;
    }
    if let Some(slot) = slot {
      out.extend(separators[slot].iter().copied());
    }
    if slot != Some(STM_COUNTRY) {
      out.push(T_SPACE!());
    }
  }
  while out
    .last()
    .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
  {
    out.pop();
  }
  Ok(Tokens::new(out))
}

/// `tokens` split at each brace-depth-0 character `c`, each part without its surrounding spaces.
fn top_level_items(tokens: &[Token], c: char) -> Vec<Vec<Token>> {
  let delimiter = pin(c.to_string());
  let mut parts: Vec<Vec<Token>> = vec![Vec::new()];
  let mut depth: i32 = 0;
  for t in tokens {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      Catcode::OTHER if depth == 0 && t.text == delimiter => {
        parts.push(Vec::new());
        continue;
      },
      _ => {},
    }
    parts.last_mut().unwrap().push(*t);
  }
  parts.into_iter().map(|part| trimmed(&part)).collect()
}

/// `tokens` without leading or trailing spaces.
fn trimmed(tokens: &[Token]) -> Vec<Token> {
  let start = tokens
    .iter()
    .position(|t| t.get_catcode() != Catcode::SPACE)
    .unwrap_or(tokens.len());
  let end = tokens
    .iter()
    .rposition(|t| t.get_catcode() != Catcode::SPACE)
    .map_or(start, |e| e + 1);
  tokens[start..end].to_vec()
}

/// `tokens` trimmed, and without the braces of a group that is all of it (`{ARTORG Center,}`), as the key-value
/// parser strips one level.
fn unbraced(tokens: &[Token]) -> Vec<Token> {
  let tokens = trimmed(tokens);
  if tokens.len() >= 2
    && tokens[0].get_catcode() == Catcode::BEGIN
    && tokens[tokens.len() - 1].get_catcode() == Catcode::END
  {
    // the first brace must close at the very end
    let mut depth = 0;
    for (i, t) in tokens.iter().enumerate() {
      match t.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => {
          depth -= 1;
          if depth == 0 && i + 1 < tokens.len() {
            return tokens;
          }
        },
        _ => {},
      }
    }
    return tokens[1..tokens.len() - 1].to_vec();
  }
  tokens
}

/// elsart.cls's `{keyword}` markup: the keywords, then the `\PACS`, `\MSC`, `\JEL` and `\UK` classifications that
/// cut them, `\sep` between the entries (Perl elsart_support_core.sty.ltxml:130-153). autart.cls carries elsart.cls's
/// (autart.cls:1248-1258 of 1106.2518's copy), so its binding defines the same (autart_cls.rs).
#[rustfmt::skip]
pub fn define_keyword_markup() -> Result<()> {
  // keyword environment and macros with XUntil pattern
  // Perl L135-152: \begin{keyword}/\end{keyword} use DefMacroI with begingroup/endgroup,
  // NOT DefEnvironment!, to properly scope the XUntil delimiter reading.
  DefMacro!(T_CS!("\\begin{keyword}"), None, "\\begingroup\\@keyword");
  DefMacro!(T_CS!("\\end{keyword}"), None, "\\@keyword@cut\\endgroup");
  // Perl elsart_support_core.sty.ltxml L138-141: `\keyword{...}` (1-arg
  // form) reads a balanced group and wraps with `\@keyword <arg>
  // \@keyword@cut` so XUntil terminates at the inserted token.
  //
  // History — DO NOT switch this back to `read_balanced(_, _, false)`:
  // commit 09bc60c2 ("\keyword absorbs trailing unbalanced }") tried to
  // mirror Perl's `$gullet->readBalanced` with no args (require_open=
  // false → level starts at 1) to absorb the legacy unbalanced-`}`
  // idiom from arXiv:1710.03688 / hep-ph0702114, where the abstract
  // ends:
  //     \keyword{Keyword1; ...} \vskip ...\noindent{...e3}}
  // (a trailing literal `}` past the keyword arg). For that input
  // alone, `require_open=false` correctly consumes `{...} }` and stops.
  //
  // BUT for the common balanced form — `\keyword{Higgs; Boson}` with
  // no trailing extra `}` (the test fixture
  // `tests/babel/elsart_keyword_brace_form.tex` and the vast majority
  // of real-world usage) — `require_open=false` reads past the
  // matching `}`, walks through `\end{abstract}` (raw `\end` is a CS
  // and CC_END count is unchanged across that expansion), then
  // `\end{frontmatter}`, … all the way to EOF. The reader's pushback
  // grows unboundedly on the way; in CI we observed a single
  // `memory allocation of 21743271936 bytes failed` (21 GB) → SIGABRT
  // before the test could even report. The cost of one paper's
  // 1-error reduction is the entire test suite OOM-aborting at
  // `81_babel::elsart_keyword_brace_form_test`.
  //
  // RESOLVED 2026-05-30 via the design note's first suggestion: keep the
  // strict balanced read of the keyword argument, then peek past spaces and
  // gobble an OPTIONAL trailing `}` (catcode END). This reproduces Perl's
  // lenient `readBalanced` *result* (it absorbs the legacy unbalanced `}`)
  // for the trailing-`}` idiom — abstract bodies that end
  //     \keyword{Kw1; Kw2; …}   <comments/spaces>   }   \end{abstract}
  // (the stray `}` is the orphaned close of a commented-out brace group;
  // witness 1601.01227, also 1710.03688 / hep-ph0702114) — WITHOUT the
  // speculative read-to-EOF that OOM'd the balanced fixture: for the common
  // `\keyword{Higgs; Boson}\end{abstract}` form the token after the strict
  // read is `\end` (a CS, not `}`), so nothing is gobbled. The peek is
  // bounded to one token, never crossing `\end`.
  // Skip leading spaces and blank-line `\par`s, then gobble a single trailing
  // `}` (catcode END) if one is there — Perl's `readBalanced` reads straight
  // through the `\par` from a blank line before the stray brace (witness
  // 1705.01354 separates `\keyword{…}` and the orphaned `}` by a blank line).
  // The skipped whitespace/`\par` tokens are buffered and FULLY RESTORED when
  // no trailing `}` follows, so the common `\keyword{…}\end{abstract}` form
  // (and `\keyword{…}<blank>\end{abstract}`) is left byte-for-byte untouched.
  // Bounded: stops at the first non-space, non-`\par` token; never crosses
  // `\end`.
  // Perl's `readBalanced` (L140) reads tokens up to the next UNMATCHED `}`
  // (tracking brace depth), absorbing that stray `}` as its terminator —
  // regardless of what non-brace material (spaces, `\par`, `\vskip <dimen>`,
  // `\noindent`, …) sits between `\keyword{…}` and the orphaned brace. The
  // earlier version only skipped spaces/`\par`s, so it stopped at the first
  // real token (e.g. `\vskip`) and left the stray `}` to hit the abstract's
  // mode-switch frame (`ltx:para`-free `}` → "close a group that switched to
  // mode internal_vertical"; witness 1604.00855:
  // `\keyword{…} \vskip 0.5\baselineskip}`).
  //
  // Walk forward tracking brace depth: an unmatched `}` (depth 0) is the
  // stray terminator — DROP it, re-inject everything else (so the `\vskip`
  // etc. still render, content-preserving). Bound the scan at `\end` (depth
  // 0) — the env terminator — and a hard token cap, so the common
  // `\keyword{Higgs; Boson}\end{abstract}` form (no stray `}`) reads one
  // token (`\end`), restores it, and is left untouched (this is also what
  // averts the read-to-EOF OOM a naive `readBalanced` hit). Witnesses:
  // 1604.00855, 1601.01227, 1705.01354, 1710.03688, hep-ph0702114.
  DefPrimitive!("\\lx@elsart@gobble@optbrace", {
    let mut skipped: Vec<Token> = Vec::new();
    let mut depth: i32 = 0;
    let mut count: usize = 0;
    while let Some(tok) = read_token()? {
      count += 1;
      if count > 4096 {
        skipped.push(tok); // safety cap: restore everything, gobble nothing
        break;
      }
      let cc = tok.get_catcode();
      if cc == Catcode::BEGIN {
        depth += 1;
        skipped.push(tok);
      } else if cc == Catcode::END {
        if depth == 0 {
          break; // unmatched stray `}` — drop it (absorbed, like readBalanced)
        }
        depth -= 1;
        skipped.push(tok);
      } else if depth == 0 && tok == T_CS!("\\end") {
        skipped.push(tok); // env end reached before any stray `}` — restore all
        break;
      } else {
        skipped.push(tok);
      }
    }
    // Re-inject everything we buffered, in order. The only token ever removed
    // is the stray `}` (which is never pushed onto `skipped`).
    for tok in skipped.into_iter().rev() {
      unread_one(tok);
    }
  });
  DefMacro!("\\keyword{}", "\\@keyword #1 \\@keyword@cut\\lx@elsart@gobble@optbrace");
  DefMacro!("\\endkeyword", "\\@keyword@cut");
  DefMacro!("\\PACS", "\\@keyword@cut\\@PACS");
  DefMacro!("\\MSC[]", "\\@keyword@cut\\@MSC{#1}");
  DefMacro!("\\JEL", "\\@keyword@cut\\@JEL");
  DefMacro!("\\UK", "\\@keyword@cut\\@UK");

  // Perl L148-152: @keyword reads until @keyword@cut delimiter using XUntil.
  // XUntil expands tokens while reading, so \end{keyword} → \@keyword@cut is found.
  DefConstructor!("\\@keyword@cut", "");
  DefMacro!("\\@keyword XUntil:\\@keyword@cut", "\\lx@add@keywords{#1}");
  DefMacro!("\\@PACS XUntil:\\@keyword@cut", "\\lx@add@classification[scheme=PACS,name={PACS:~}]{#1}");
  DefMacro!("\\@MSC{} XUntil:\\@keyword@cut", "\\lx@add@classification[scheme={#1 MSC},name={MSC:~}]{#2}");
  DefMacro!("\\@JEL XUntil:\\@keyword@cut", "\\lx@add@classification[scheme=JEL,name={JEL:~}]{#1}");
  DefMacro!("\\@UK XUntil:\\@keyword@cut", "\\lx@add@classification[scheme=UK,name={UK:~}]{#1}");
  Ok(())
}

/// The global value holding the keys `\author[…]` calls list (comma-separated).
pub(crate) const ELS_AUTHOR_KEYS: &str = "@elsarticle@authorkeys";
/// The global value holding the keys `\corref`/`\cormark` name (comma-separated).
pub(crate) const ELS_CORREF_KEYS: &str = "@elsarticle@correfs";
/// The global value holding the keys `\address[…]`/`\affiliation[…]` label (comma-separated).
pub(crate) const ELS_PLACE_KEYS: &str = "@elsarticle@placekeys";
/// The global value holding each author's `\author[…]` keys and its name's `\fnref` and `\corref` keys, in reading order
/// (`<keys>|<fnrefs>|<correfs>;` each).
pub(crate) const ELS_AUTHORS: &str = "@elsarticle@authors";
/// The global value holding the group level the `\author` calls are read at.
pub(crate) const ELS_NAMES_LEVEL: &str = "@elsarticle@nameslevel";
/// The global value holding each `\corref`/`\fnref` as read, with its group level (`<kind>:<level>:<keys>;`): one read
/// outside every name, at the names' level, is the first name's (`\lx@els@author`).
pub(crate) const ELS_OUTSIDE: &str = "@elsarticle@marks";
/// The global value holding the route each key was first read by (`<key>=<route>;`, [`els_route`]).
pub(crate) const ELS_ROUTES: &str = "@elsarticle@routes";

/// The route `key` was first read by: `""` an `\author[…]` list, `"cor"` a `\corref`, `"fn"` an `\fnref` — the label form
/// elsarticle_cls.rs gives every later route of the key, so all of them meet.
pub(crate) fn els_route(key: &str) -> Option<String> {
  let key = key.trim();
  lookup_string(ELS_ROUTES).split(';').find_map(|entry| {
    entry
      .split_once('=')
      .filter(|(k, _)| *k == key)
      .map(|(_, route)| route.to_string())
  })
}

/// Record `route` for `key` unless a route was read for it before ([`els_route`]).
pub(crate) fn set_els_route(key: &str, route: &str) {
  let key = key.trim();
  if key.is_empty() || els_route(key).is_some() {
    return;
  }
  let mut routes = lookup_string(ELS_ROUTES);
  routes.push_str(&s!("{key}={route};"));
  assign_value(ELS_ROUTES, Stored::String(pin(&routes)), Scope::Global);
}

/// The global value holding the keys `\cortext[…]` labels (comma-separated).
pub(crate) const ELS_CORTEXT_KEYS: &str = "@elsarticle@cortextkeys";
/// The global value holding the keys `\fntext[…]` labels (comma-separated).
pub(crate) const ELS_FNTEXT_KEYS: &str = "@elsarticle@fntextkeys";

/// An `\author` call as [`ELS_AUTHORS`] records it.
pub(crate) struct ElsAuthor {
  pub keys:    Vec<String>,
  pub fnrefs:  Vec<String>,
  pub correfs: Vec<String>,
}

/// The `\author` calls read so far, in order.
pub(crate) fn els_authors() -> Vec<ElsAuthor> {
  let list = |part: &str| -> Vec<String> {
    part
      .split(',')
      .map(str::trim)
      .filter(|k| !k.is_empty())
      .map(String::from)
      .collect()
  };
  lookup_string(ELS_AUTHORS)
    .split(';')
    .filter(|entry| !entry.is_empty())
    .map(|entry| {
      let mut parts = entry.split('|');
      let mut next = || list(parts.next().unwrap_or_default());
      ElsAuthor {
        keys:    next(),
        fnrefs:  next(),
        correfs: next(),
      }
    })
    .collect()
}

/// The keys of the authors whose `\author[…]` list holds `key`, each author's in its own list (its `\author[…]` keys
/// and its name's `\fnref`s: the marks it prints side by side).
pub(crate) fn els_carriers(key: &str) -> Vec<Vec<String>> {
  let key = key.trim();
  els_authors()
    .into_iter()
    .map(|author| {
      author
        .keys
        .into_iter()
        .chain(author.fnrefs)
        .collect::<Vec<_>>()
    })
    .filter(|keys| keys.iter().any(|k| k == key))
    .collect()
}

/// Record a `\corref` (`kind` "cor") or `\fnref` ("fn") as read, with its group level ([`ELS_OUTSIDE`]).
fn record_els_mark(kind: &str, keys: &str) {
  warn_moved_notes(kind, keys);
  let mut marks = lookup_string(ELS_OUTSIDE);
  marks.push_str(&s!("{kind}:{}:{};", get_frame_depth(), keys.trim()));
  assign_value(ELS_OUTSIDE, Stored::String(pin(&marks)), Scope::Global);
}

/// The keys of the last `\corref` (`kind` "cor") or `\fnref` ("fn") read outside every name, at the level the names are
/// read at: the class's `\@corref`/`\@fnmark`, set where it is read and shown by the first name printed (elsarticle.cls
/// :159-165, :188-190, :256-264); one in a brace group of its own sets it for that group alone (1909.01114).
pub(crate) fn els_outside_mark(kind: &str) -> Vec<String> {
  let level = lookup_int(ELS_NAMES_LEVEL).to_string();
  lookup_string(ELS_OUTSIDE)
    .rsplit(';')
    .find_map(|entry| {
      let mut parts = entry.splitn(3, ':');
      (parts.next()? == kind && parts.next()? == level).then(|| parts.next().unwrap_or_default())
    })
    .map(|keys| {
      keys
        .split(',')
        .map(str::trim)
        .filter(|k| !k.is_empty())
        .map(String::from)
        .collect()
    })
    .unwrap_or_default()
}

/// The global value holding, for each kind (`cor`, `fn`) the first name sets no mark of itself, the outside keys it took
/// at its call (`<kind>=<keys>;`): the class's `\@corref`/`\@fnmark` as then read ([`els_first_note_label`]).
pub(crate) const ELS_FIRST: &str = "@elsarticle@first";
/// The global value holding the notes [`els_first_note_label`] found moved where they were read (`<kind>:<key>`, comma-separated).
const ELS_MOVED: &str = "@elsarticle@movednotes";
/// The global flag elsarticle sets: its `\cortext`/`\fntext` label their notes by [`els_first_note_label`] too.
pub(crate) const ELS_NOTE_LABELS: &str = "@elsarticle@notelabels";

/// The outside keys of kind `kind` the first name took at its call, or `None` where it sets its own (or none is read).
fn els_first_took(kind: &str) -> Option<Vec<String>> {
  lookup_string(ELS_FIRST).split(';').find_map(|entry| {
    let (k, keys) = entry.split_once('=')?;
    (k == kind).then(|| {
      keys
        .split(',')
        .filter(|k| !k.is_empty())
        .map(String::from)
        .collect()
    })
  })
}

/// Whether the outside mark the first name shows now (the last read, [`els_outside_mark`]) differs from the one it took
/// on `key`: the first name holds `key`'s route label and no longer shows it, or shows it without holding it.
fn els_first_moved(kind: &str, key: &str, shown: &[String]) -> bool {
  els_first_took(kind)
    .is_some_and(|took| took.iter().any(|k| k == key) != shown.iter().any(|k| k == key))
}

/// The label of elsarticle's note of kind `kind` keyed `key` where an outside mark read after the first name's call moved
/// its mark (elsarticle.cls:159-165, :188-190, :256-264): the first name's annotations are queued, so the note it took
/// and no longer shows is the block's, `<kind>@<key>`, and the one it now shows stays its key's (the block's where no
/// other name reaches it) — a Warning, the PDF's mark on the first name lost (fail-safe, OD #1067). `None`: unmoved.
pub(crate) fn els_first_note_label(kind: &str, key: &str) -> Option<String> {
  let key = key.trim();
  let shown = els_outside_mark(kind);
  if key.is_empty() || !els_first_moved(kind, key, &shown) {
    return None;
  }
  let elsewhere = els_authors().iter().any(|author| {
    let refs = if kind == "cor" {
      &author.correfs
    } else {
      &author.fnrefs
    };
    author.keys.iter().chain(refs).any(|k| k == key)
  });
  let shows = shown.iter().any(|k| k == key);
  record_els_keys(ELS_MOVED, std::iter::once(s!("{kind}:{key}").as_str()));
  Warn!(
    "frontmatter",
    "elsarticle_outside_mark",
    s!(
      "elsarticle: an outside mark read after the first author's call moved its mark {} the note {kind}:{key}; the note is {}",
      if shows { "to" } else { "from" },
      if elsewhere {
        "kept on the names listing its key"
      } else {
        "the block's, its mark on the first author lost"
      }
    )
  );
  (!shows && !elsewhere).then(|| s!("{kind}@{key}"))
}

/// Before an outside `\corref`/`\fnref` read at the names' level after the first name is recorded: a note read already
/// whose first-name mark it moves keeps the label it was given where it was read, with a Warning (not the one the first
/// name took right after its call).
fn warn_moved_notes(kind: &str, keys: &str) {
  if !lookup_bool(ELS_NOTE_LABELS)
    || reading_author_fields()
    || i64::try_from(get_frame_depth()).ok() != Some(lookup_int(ELS_NAMES_LEVEL))
  {
    return;
  }
  let Some(took) = els_first_took(kind) else {
    return;
  };
  let notes = if kind == "cor" {
    ELS_CORTEXT_KEYS
  } else {
    ELS_FNTEXT_KEYS
  };
  let after: Vec<String> = keys
    .split(',')
    .map(str::trim)
    .filter(|k| !k.is_empty())
    .map(String::from)
    .collect();
  // (a note read already whose label the move would now decide otherwise than it was decided where it was read)
  let before = els_outside_mark(kind);
  let mut moved: Vec<&String> = before
    .iter()
    .chain(&after)
    .chain(&took)
    .filter(|k| recorded_els_key(notes, k))
    .filter(|k| {
      recorded_els_key(ELS_MOVED, &s!("{kind}:{k}")) != (took.contains(k) != after.contains(k))
    })
    .collect();
  moved.sort_unstable();
  moved.dedup();
  if !moved.is_empty() {
    Warn!(
      "frontmatter",
      "elsarticle_outside_mark",
      s!(
        "elsarticle: an outside \\{} read after the note(s) {} moves the first author's mark, which the note keeps as read",
        if kind == "cor" { "corref" } else { "fnref" },
        moved
          .iter()
          .map(|k| k.as_str())
          .collect::<Vec<_>>()
          .join(", ")
      )
    );
  }
}

/// Record a key a `\corref`/`\cormark` names, and its route ([`els_route`]).
fn record_els_cor_key(key: &str) {
  record_els_keys(ELS_CORREF_KEYS, std::iter::once(key));
  set_els_route(key, "cor");
}

/// Add each of `keys` (trimmed, non-empty) to the comma list kept in the global value `list`.
pub(crate) fn record_els_keys<'a>(list: &str, keys: impl Iterator<Item = &'a str>) {
  let mut recorded = lookup_string(list);
  for key in keys.map(str::trim).filter(|key| !key.is_empty()) {
    if !recorded.split(',').any(|k| k == key) {
      if !recorded.is_empty() {
        recorded.push(',');
      }
      recorded.push_str(key);
    }
  }
  assign_value(list, Stored::String(pin(&recorded)), Scope::Global);
}

/// Whether the comma list kept in the global value `list` holds `key`.
pub(crate) fn recorded_els_key(list: &str, key: &str) -> bool {
  let key = key.trim();
  lookup_string(list).split(',').any(|k| k == key)
}

/// The keys `<braced>{<keys>}` and `<bracketed>[<keys>]` name in `tokens`, an `\author`'s argument (`\corref` and
/// `\cormark`, `\fnref` and `\fnmark`), each comma-separated list split.
pub(crate) fn keys_in(tokens: &[Token], braced: &Token, bracketed: &Token) -> Vec<String> {
  ref_spans(tokens, braced, bracketed)
    .into_iter()
    .flat_map(|(_, keys)| keys)
    .collect()
}

/// Each `<braced>{<keys>}` and `<bracketed>[<keys>]` in `tokens`: its token range and its keys.
pub(crate) fn ref_spans(
  tokens: &[Token],
  braced: &Token,
  bracketed: &Token,
) -> Vec<(std::ops::Range<usize>, Vec<String>)> {
  let mut spans = Vec::new();
  for (at, t) in tokens.iter().enumerate() {
    let (open, close) = if t == braced {
      (Catcode::BEGIN, Catcode::END)
    } else if t == bracketed {
      (Catcode::OTHER, Catcode::OTHER)
    } else {
      continue;
    };
    let mut i = at + 1;
    while tokens
      .get(i)
      .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
    {
      i += 1;
    }
    if !tokens
      .get(i)
      .is_some_and(|t| t.get_catcode() == open && (open == Catcode::BEGIN || t.to_string() == "["))
    {
      continue;
    }
    let start = i + 1;
    let mut depth = 0;
    let mut end = start;
    while let Some(t) = tokens.get(end) {
      depth += t.group_step();
      let closed = if close == Catcode::END {
        depth < 0
      } else {
        depth == 0 && t.to_string() == "]"
      };
      if closed {
        break;
      }
      end += 1;
    }
    let key = Tokens::new(tokens[start..end.min(tokens.len())].to_vec()).to_string();
    let keys = key
      .split(',')
      .map(str::trim)
      .filter(|k| !k.is_empty())
      .map(String::from)
      .collect();
    spans.push((at..(end + 1).min(tokens.len()), keys));
  }
  spans
}

/// The label a route of kind `kind` (`""` an `\author[…]` key, `"cor"`, `"fn"`) gives `key`: the form the key was first
/// read by ([`els_route`]), so that all its routes meet.
fn els_route_label(key: &str, kind: &str) -> String {
  let key = key.trim();
  match els_route(key).as_deref().unwrap_or(kind) {
    "" => key.to_string(),
    route => s!("{route}:{key}"),
  }
}

/// The `\corref{…}`, `\fnref{…}` and `\fnmark[…]` right after an `\author` call (past spaces, not an `\ead`'s Semiverbatim),
/// each kind with its keys, left in the input: the first name's outside marks (1501.03323; OD #1067).
fn peek_outside_marks() -> Result<Vec<(&'static str, Vec<String>)>> {
  let mut read: Vec<Token> = Vec::new();
  let mut marks = Vec::new();
  // (one argument, braced or bracketed, read whole)
  let argument = |read: &mut Vec<Token>, open: &str, close: &str| -> Result<Option<String>> {
    let mut text = Vec::new();
    loop {
      let Some(t) = read_token()? else {
        return Ok(None);
      };
      read.push(t);
      if t.get_catcode() != Catcode::SPACE {
        if t.to_string() != open {
          return Ok(None);
        }
        break;
      }
    }
    let mut depth = 0;
    while let Some(t) = read_token()? {
      read.push(t);
      depth += t.group_step();
      if (open == "{" && depth < 0) || (open == "[" && depth == 0 && t.to_string() == close) {
        return Ok(Some(Tokens::new(text).to_string()));
      }
      text.push(t);
    }
    Ok(None)
  };
  while let Some(t) = read_token()? {
    read.push(t);
    let kind = if t == T_CS!("\\corref") {
      "cor"
    } else if t == T_CS!("\\fnref") || t == T_CS!("\\fnmark") {
      "fn"
    } else if t.get_catcode() == Catcode::SPACE {
      continue;
    } else {
      break;
    };
    let bracketed = t == T_CS!("\\fnmark");
    let keys = if bracketed {
      argument(&mut read, "[", "]")?
    } else {
      argument(&mut read, "{", "}")?
    };
    let Some(keys) = keys else { break };
    marks.push((
      kind,
      keys
        .split(',')
        .map(str::trim)
        .filter(|k| !k.is_empty())
        .map(String::from)
        .collect(),
    ));
  }
  unread(Tokens::new(read));
  Ok(marks)
}
