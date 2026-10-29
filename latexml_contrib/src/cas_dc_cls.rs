//! Stub for Elsevier cas-dc.cls / cas-sc.cls (CAS journals double-column).
//!
//! The cas-* classes load cas-common.sty which uses xparse/expl3
//! NewDocumentCommand to define many frontmatter helpers. Our raw load
//! may not invoke them; provide gobble stubs for the most common
//! frontmatter macros.
use latexml_package::prelude::*;

use crate::shipped_class::class_option_given;

LoadDefinitions!({
  LoadClass!("OmniBus");
  // cas-sc.cls:144, cas-dc.cls:142 `\RequirePackage[T1]{fontenc}`: text `_`, `<`, `>`, `|` print as themselves.
  RequirePackage!("fontenc", options => vec![s!("T1")]);
  RequirePackage!("amsmath");
  // cas-common.sty:1500, :2264 `\RequirePackage{moreverb}` and `{wrapfig}`. moreverb has no binding (raw loading is off
  // by default, a warning in every cas paper), so the verbatim it loads, whose `{comment}` the papers use
  // (2506.19439), stands for it.
  RequirePackage!("verbatim");
  RequirePackage!("wrapfig");
  // cas-dc.cls:64 `\RequirePackage{xspace,xstring,footmisc}` (the samples'
  // `\def\tsc#1{\textsc{#1}\xspace}`); xstring has no binding and would
  // warn under the default (non-raw) load, so only the bound two.
  RequirePackage!("xspace");
  RequirePackage!("footmisc");
  RequirePackage!("amsthm");
  // amssymb supplies \lesssim, \gtrsim, \nleq, \ngeq, \square, \blacksquare
  // and other binary-relation/blackboard-bold symbols that CAS authors
  // routinely use without explicit \usepackage{amssymb}. Witness
  // 2312.12523 (\lesssim undefined).
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  // cas-dc.cls L63: \RequirePackage{booktabs,makecell,multirow,array,colortbl,dcolumn,stfloats}.
  RequirePackage!("booktabs");
  RequirePackage!("multirow");
  RequirePackage!("array");
  RequirePackage!("colortbl");
  RequirePackage!("makecell");
  // cas-dc.cls L: `\RequirePackage{etoolbox,balance}` — pulls in the
  // balance package which defines `\balance` (column-balancing in
  // 2-col docs). Without it, papers using `\balance` near the end
  // trip Error:undefined. Witness 2303.04712.
  RequirePackage!("balance");
  // cas-sc.cls:49, :67 and cas-dc.cls:49, :67: the `review` option loads setspace (the double spacing itself, :136-138,
  // is layout); a paper's `\DeclareCaptionFont{mysingle}{\setstretch{1}}` under `[review]` calls it (2512.10464).
  if class_option_given("review")? {
    RequirePackage!("setspace");
  }
  // cas-dc.cls (Elsevier CAS) aliases \newdefinition and \newproof to \newtheorem
  // (elsarticle convention), used to declare definition/proof-like theorem envs.
  // Were undefined (Perl defines them). Witness 2306.04212.
  Let!("\\newdefinition", "\\newtheorem");
  Let!("\\newproof", "\\newtheorem");
  // cas-common.sty:247-425, 1634, 2033-2042: the name parsers, the e-mail, URL, ORCID and first-page-note printers its
  // `\author`/`\ead` machinery calls, and its page styles, which a paper's own `\RenewDocumentCommand` or `\ps@cas`
  // needs defined — ltcmd errors on renewing an undefined command (2609.16168, 16199, 36345 `\firstname`,
  // `\emailauthor`, `\ps@cas`; 2609.00634, 20010 `\printorcid`, `\printemails`). This binding's frontmatter, which
  // keeps the e-mails and notes itself, does not call them; the page styles are page furniture.
  // cas-common.sty:1723-1732: the title-page layout keys (`\pprintMaketitle[<keys>]`, `\MaketitleBox[…]`, :1748-1829,
  // or a paper's own `\keys_set:nn {stm/mktitle}{nologo}`, 2609.00281) — layout only here.
  RawTeX!(
    r"\ExplSyntaxOn
\bool_if_exist:NF \g_stm_blind_bool { \bool_new:N \g_stm_blind_bool }
\bool_if_exist:NF \g_stm_longtitle_bool { \bool_new:N \g_stm_longtitle_bool }
\bool_if_exist:NF \g_stm_nologo_bool { \bool_new:N \g_stm_nologo_bool }
\tl_if_exist:NF \g_stm_footer_default_tl { \tl_new:N \g_stm_footer_default_tl }
\tl_if_exist:NF \g_stm_footer_custom_tl { \tl_new:N \g_stm_footer_custom_tl }
\clist_if_exist:NF \g_stm_breakafter_clist { \clist_new:N \g_stm_breakafter_clist }
\keys_define:nn { stm / mktitle }
  {
    blind .bool_gset:N = \g_stm_blind_bool,
    footer .multichoice:,
    footer/default .tl_gset:N = \g_stm_footer_default_tl,
    footer/custom .tl_gset:N = \g_stm_footer_custom_tl,
    longtitle .bool_gset:N = \g_stm_longtitle_bool,
    longabstract .bool_gset:N = \g_stm_longtitle_bool,
    breakafter .clist_gset:N = \g_stm_breakafter_clist,
    nologo .bool_gset:N = \g_stm_nologo_bool,
  }
\ExplSyntaxOff"
  );
  for (cs, args) in [
    ("\\parsename", "{}"),
    ("\\invparsename", "{}"),
    ("\\surname", ""),
    ("\\firstname", ""),
    ("\\eadauthor", ""),
    ("\\urlauthor", "{}{}"),
    ("\\emailauthor", "{}{}"),
    ("\\printorcid", ""),
    ("\\printemails", ""),
    ("\\printurls", ""),
    ("\\printFirstPageNotes", ""),
    ("\\ps@first", ""),
    ("\\ps@cas", ""),
  ] {
    def_macro_noop(&s!("{cs}{args}"))?;
  }

  // cas-common.sty:1599-1610 the classification lines a `{keywords}` block ends with, `\MSC[<year>]{<codes>}`,
  // `\JEL{<codes>}`, `\PACS{<codes>}`; :415-423 `\orcidauthor{<id>}{<name>}`, an author's ORCID, which the class
  // prints in its ORCID list (`\printorcid`): kept as the contact of the author just given; :774 `\stmAuthorSetup`, the
  // author-type keys; :2067-2068 the `\FullWidth` dimension of its wide floats. 35 cas-sc and 14 cas-dc papers of run
  // 353 had one of them undefined (1909.01564, 2012.11920, 2108.00185, 2407.03849 `\MSC`; 2104.12510, 2304.13458,
  // 2407.18715 `\orcidauthor`, each after its own `\author`; 2106.08963 swaps the two arguments, `{Andrew S.
  // Nencka}{0000-…}`, the author's error, so its link is orcid.org/Andrew S. Nencka).
  RawTeX!(
    r"\NewDocumentCommand\MSC{O{}m}{\par\noindent\textit{#1 MSC: }#2}
\NewDocumentCommand\JEL{m}{\par\noindent\textit{JEL: }#1}
\NewDocumentCommand\PACS{m}{\par\noindent\textit{PACS: }#1}
\NewDocumentCommand\orcidauthor{mm}{\lx@add@orcid{#1}}
\DeclareDocumentCommand\stmAuthorSetup{m}{}
\newdimen\FullWidth \FullWidth=\textwidth"
  );
  // cas-common.sty dynamically defines `\tblwidth` via
  // `\csgdef{tblwidth}{\dim_use:N \l_tbl_width_dim}` only inside its
  // own table-float wrapper. Authors use `\tblwidth` directly as the
  // width argument of `\begin{tabular*}{\tblwidth}{...}` even outside
  // that wrapper. Provide a \linewidth fallback so the tabular still
  // renders with reasonable width. Witness 2209.06932 (cas-dc table
  // cascade — 136 errors).
  DefMacro!("\\tblwidth", "\\linewidth");
  // cas-common.sty L2070-2072: \newcolumntype{L|R|C}{...}. Authors use
  // L/R/C column types in `\begin{tabular*}{...}{@{} LLLLLL @{}}`.
  // Without these, the unrecognized column letters trigger one
  // "Extra alignment tab '&'" per & cell. Define matching short-hand
  // raggedright/etc. column types.
  RawTeX!(
    r"\newcolumntype{L}{@{\extracolsep{\fill}}l}\newcolumntype{R}{@{\extracolsep{\fill}}r}\newcolumntype{C}{@{\extracolsep{\fill}}c}"
  );

  // cas-common frontmatter — gobble cleanly.
  // Title-note text — author prose. Preserve as ltx:note frontmatter.
  DefMacro!(
    "\\tnotetext[]{}",
    "\\@add@frontmatter{ltx:note}[role=titlenote]{#2}"
  );
  def_macro_noop("\\tnotemark[]")?; // mark only, no body
  DefMacro!(
    "\\tnoteref[]{}",
    "\\@add@frontmatter{ltx:note}[role=titlenote-ref]{#2}"
  );
  def_macro_noop("\\fnmark[]")?;
  DefMacro!(
    "\\fnref[]{}",
    "\\@add@frontmatter{ltx:note}[role=footnote-ref]{#2}"
  );
  // Footnote / author-note text — preserve as ltx:note rather than
  // gobble (content-preserving). `\fntext` / `\nonumnote` carry the
  // actual note prose, `\cortext` is the corresponding-author byline.
  DefMacro!(
    "\\fntext[]{}",
    "\\@add@frontmatter{ltx:note}[role=footnote]{#2}"
  );
  DefMacro!(
    "\\nonumnote{}",
    "\\@add@frontmatter{ltx:note}[role=note]{#1}"
  );
  DefMacro!(
    "\\nonumtnotetext{}",
    "\\@add@frontmatter{ltx:note}[role=note]{#1}"
  );
  DefMacro!(
    "\\cortext[]{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#2}"
  );
  def_macro_noop("\\cormark[]")?; // mark only, no body
  // cas-common.sty:224 `\NewDocumentCommand \corref { m }`: the label of the `\cortext` it marks, no text (an optional
  // `\corref[]` left `cor1` typeset after the name or as a paragraph, 2303.00352's "Enrique García-Macíascor1"; the
  // mark's link to its `\cortext` is not built)
  def_macro_noop("\\corref{}")?;
  // cas-common.sty:895 `\RenewDocumentCommand\author{O{} m O{}}` —
  // `\author[affil-marks]{name}[type=editor, auid=…, orcid=…]`, once per
  // author (cas-sc-sample.tex:58-97: four calls). The inst_support shape
  // `[marks]{name}` left every `[keyvals]` block in the stream, typeset as a
  // `<para>` before the title (els-cas-templates/cas-sc-sample, cas-dc-sample:
  // three such paragraphs, schema-invalid). Same name splitting and
  // accumulation as inst_support; the keyvals go through the shared
  // `\lx@add@author@keyvals` (base_utilities.rs: `orcid=` → contact, the
  // production keys dropped).
  // The marks link the author to the affiliations labelled with them (cas-common.sty:895-940 `\author[1,2]{…}`,
  // :1252-1266 `\affiliation[1]{…}`); unmarked, an affiliation goes to the author before it.
  // (the names are split as the kernel splits an author list, at commas, `\and` and the word "and": cas prints the
  // argument as written, `\author[a]{Cy Cee and Dee Dee}` two people; review of 63u)
  // (the marks are read as the letters cas prints, `cas_mark_labels`: `\author[1,2]` under `\address[a]…\address[b]`
  // is "a,b", 2207.12381; under `\address[b]{Univ X}\address[a]{Univ Y}`, which print "a Univ X", "b Univ Y", it is
  // Univ X and Univ Y, review of 88b r1)
  DefMacro!("\\author[]{}[]", sub[(marks, name, keys)] {
    let name: Tokens = name;
    let mut out = mouth::tokenize_internal("\\lx@add@authors@append[annotations=").unlist();
    out.push(T_BEGIN!());
    out.extend(cas_mark_labels(marks));
    out.extend([T_END!(), T_OTHER!("]"), T_BEGIN!()]);
    out.extend(name.unlist());
    out.extend([T_END!(), T_CS!("\\lx@add@author@keyvals"), T_BEGIN!()]);
    out.extend(keys.map(Tokens::unlist).unwrap_or_default());
    out.push(T_END!());
    Ok(Tokens::new(out))
  });
  // \affiliation[id]{keys}: cas-common.sty:1118-1160 reads the argument as nearly the `stm/affiliation` keys
  // elsarticle reads (elsarticle.cls:387-440), printed as the class prints them (`stm_affiliation`): `organization={X},
  // city={Y}, country={Z}` is "X, Y, Z". The keys were typeset as text (160 of 200 cas papers of 2609: 2609.00047,
  // 2609.04891) and every affiliation went to the last author.
  // cas-common.sty:1252 `\NewDocumentCommand \affiliation { O{} m O{} }`: the trailing optional is the `stm/address`
  // keys (`foot`, `type`, `name`: where and under what heading the class prints the entry, :1178-1190), no text of it.
  DefMacro!("\\affiliation[]{}[]", sub[(label, keys, options)] {
    let keys: Tokens = keys;
    let mut out = cas_labelled_entry(label, options);
    out.extend([T_BEGIN!(), T_CS!("\\lx@cas@affiliation"), T_BEGIN!()]);
    out.extend(keys.unlist());
    out.extend([T_END!(), T_END!()]);
    Ok(Tokens::new(out))
  });
  // cas-common.sty:1216 `\NewDocumentCommand \address { O{} m O{} }` files the address in the affiliation list
  // `\affiliation` files into (`g_stm_aff<n>_seq`), labelled as it labels (`\textsuperscript{\itshape\@currentlabel}`
  // from `#1`, :1222-1242), its text printed as written (no keys read), the trailing optional the same `stm/address`
  // keys. OmniBus's `\address[]{}` (Perl OmniBus.cls.ltxml:91) dropped the label, so `\author[1]{…}[orcid=…]
  // \author[2]{…}\address[1]{X}\address[2]{Y}` gave both addresses to the last author (2011.00505). Both commands'
  // labels are read as cas prints them (`cas_labelled_entry`).
  DefMacro!("\\address[]{}[]", sub[(label, text, options)] {
    let text: Tokens = text;
    let mut out = cas_labelled_entry(label, options);
    out.push(T_BEGIN!());
    out.extend(text.unlist());
    out.push(T_END!());
    Ok(Tokens::new(out))
  });
  DefMacro!("\\lx@cas@affiliation {}", sub[(raw)] {
    elsart_support_core_sty::stm_affiliation(raw, false)
  });
  // \ead[type]{address} — author email/url, preserve as contact. The address is read verbatim, as cas-common.sty
  // prints it, stringified (`\ttfamily \tl_to_str:n`, :362; the `[url]` form through `\url`, :338): 2410.07921's
  // `arash_khajooei@…`, its `_` an error. The type is `email` by default, as elsarticle's, through the same contact
  // API (elsart_support_core_sty.rs; an empty `role` before), in the typewriter type cas-common.sty sets it in; the
  // HTML link is made of the address text inside it (LaTeXML-structure-xhtml.xsl `contact-address`).
  DefMacro!(
    "\\ead Optional:email Semiverbatim",
    "\\lx@add@contact[role=#1]{\\lx@address@urlfont{#2}}"
  );
  // \sep — author/affil separator that cas-common defines.
  DefMacro!("\\sep", ",");

  // cas-common credit-tagging macros (CRediT taxonomy). \credit{role}
  // attaches an author contribution (Conceptualization, Investigation,
  // etc.); \printcredits emits the credit list. Round-34 surpass-Perl:
  // preserve the role text as a frontmatter note so the CRediT
  // taxonomy is retained for downstream JATS conversion. Witness
  // 2405.20972.
  DefMacro!(
    "\\credit{}",
    "\\@add@frontmatter{ltx:note}[role=credit]{#1}"
  );
  def_macro_noop("\\printcredits")?;

  // Elsevier highlights / biography environments (cas-common.sty).
  // `{highlights}` collects bullet points for the dedicated highlights
  // section; `{bio}` / `\bio` defines an author biography. Render as
  // ltx:note (frontmatter) so author content is preserved.
  // Witness 2503.16816, 2502.18516 (cas-dc).
  // Use internal_vertical mode so the body can contain paragraphs,
  // lists, and other block-level content. Without the explicit mode,
  // the env infers restricted_horizontal and trips
  // `Error:unexpected:\endbio Attempt to end mode restricted_horizontal
  // in internal_vertical` when the bio contains \par-separated prose.
  // Witness 2503.16816.
  DefEnvironment!("{highlights}",
    "<ltx:note role='highlights'>#body</ltx:note>",
    mode => "internal_vertical");
  // cas-common.sty:1560 `\DeclareDocumentEnvironment{graphicalabstract}
  // {O{Graphical~Abstract}}`; same shape as elsarticle. Witness cas-sc/cas-dc.
  DefEnvironment!("{graphicalabstract}[]",
    "<ltx:note role='graphicalabstract'>#body</ltx:note>",
    mode => "internal_vertical");
  // cas-common.sty:2402 `\NewDocumentCommand \bio { O{} m }`: the options and the photo's file, which `\WrapFigure`
  // includes (`\includegraphics{#2}`, :2376) and never typesets: `\bio{Figures/chamara_sandeepa.png}` raised "Script _
  // can only appear in math mode" (2203.04264; OXIDIZED_DESIGN #551), and `\bio[width=…]{…}` read `[` as the file.
  // The photo is not carried: `ltx:note` has no attribute for it (the schema drops a `name`). Guard
  // `perfect_kernel_batch65m::file_arguments_are_not_typeset`.
  DefEnvironment!("{bio}[] Semiverbatim",
    "<ltx:note role='biography'>#body</ltx:note>",
    mode => "internal_vertical");
  // \newproof{env}{display-name} — cas-common's environment factory
  // for proof-like environments. Define a basic environment that
  // wraps content in ltx:proof.
  DefMacro!(
    "\\newproof{}{}",
    "\\newenvironment{#1}{\\par\\noindent\\textbf{#2.}\\hspace{0.5em}}{\\par}"
  );
});

/// The entry counter cas prints its affiliation-list labels by (`\g_stm_aff_int`, cas-common.sty:1175), global and
/// never reset, which `\address` and `\affiliation` share.
const CAS_ENTRY_COUNTER: &str = "lx_cas_aff_int";

/// The key of the letter a cas entry's label prints (`\stmLabel`'s `X@<label>:`, cas-common.sty:184-196), in the
/// deferred labels' table (`set_label_ref_once`, read as `\lx@label@ref`: cas's `\stmRef` reads it from the `.aux` of
/// the run before, where the first entry under a label keeps it).
fn cas_label_key(label: &str) -> String { s!("cas:{label}") }

/// The key of the letter the last entry of cas's affiliation list printed (`\@currentlabel`, cas-common.sty:1233,
/// :1240), in the same table (no label's key: those are `cas:<label>`): the letter a lettered `[foot]` entry is printed
/// under (:1310 `\tex_def:D \thefootnote{\itshape\@currentlabel}`), the last the entries printed, as the title page
/// prints its footnoted entries after them.
const CAS_CURRENT_LABEL: &str = "cas@current";

/// The marks a cas `\author[marks]` requests, as the letters cas-common.sty:907-914 prints them: a list opening with a
/// letter reads each item as an entry's label, the letter that entry prints (`\processAffRef`, `\stmRef`, :197-201,
/// :278-286; the letter when the entry came first, else `\lx@label@ref`, read at the flush once the entries are known,
/// as cas reads it from the `.aux` of the run before; nothing for a label no entry has, where cas prints "0"), any
/// other each number N as the N-th letter (`\processAffNum`, `\int_to_alph:n`, :288-296), the letter
/// the entry `\address[N]` and the N-th lettered entry print (`cas_labelled_entry`). (an item cas cannot read so, a
/// `*` among letters or a template's `<num>` among numbers, as written)
fn cas_mark_labels(marks: Option<Tokens>) -> Vec<Token> {
  let marks = marks.map(Tokens::unlist).unwrap_or_default();
  let by_label = cas_letter_headed(&marks);
  let mut out: Vec<Token> = Vec::new();
  for item in marks.split(|t| *t == T_OTHER!(",")) {
    let text = Tokens::new(item.to_vec()).to_string();
    let text = text.trim();
    if text.is_empty() {
      continue;
    }
    if !out.is_empty() {
      out.push(T_OTHER!(","));
    }
    if by_label && cas_letter_headed(item) {
      // (the letter, when the entry came first; else read once the entries are known)
      let key = cas_label_key(text);
      match label_ref_text(&key) {
        Some(letter) => out.extend(mouth::tokenize_internal(TeXString::assembled(letter)).unlist()),
        None => out.extend(label_ref_tokens(&key)),
      }
    } else {
      let letter = (!by_label)
        .then(|| cas_number(text))
        .flatten()
        .filter(|n| *n > 0);
      let text = letter.map_or_else(|| text.to_string(), radix::radix_alpha);
      out.extend(mouth::tokenize_internal(TeXString::assembled(text)).unlist());
    }
  }
  out
}

/// `\lx@add@contact[label={…},role=affiliation]` for a cas affiliation-list entry (`\affiliation[label]{…}[keys]`,
/// `\address[label]{…}[keys]`), labelled by the letter it prints, as cas-common.sty:1228-1242 (and :1274-1290) prints
/// it: a label opening with a letter steps the entry counter and prints its letter, the letter the label stands for
/// (`\int_gincr:N \g_stm_aff_int`, `\stmLabel`); a number N sets the counter and prints the N-th letter
/// (`\int_gset:Nn`). So `\address[b]{Univ X}\address[a]{Univ Y}` prints "a Univ X", "b Univ Y", the entries an
/// `\author[1]` and an `\author[2]` ask for, and `\address[1]{X}\affiliation[a]{…}` prints X and the affiliation as
/// "a" and "b". An entry the `foot` key sets as a footnote (:1294-1326, :1328-1373) leaves the counter and labels
/// nothing: a number N prints the N-th letter, a lettered one the letter the entries before it printed last
/// (`\@currentlabel`, `\lx@label@ref{cas@current}`). (a label cas cannot read so, a `*` or `1a`, is kept as written and leaves
/// the counter; no label, none)
fn cas_labelled_entry(label: Option<Tokens>, keys: Option<Tokens>) -> Vec<Token> {
  let label = label.map(Tokens::unlist).unwrap_or_default();
  let text = Tokens::new(label.clone()).to_string();
  let text = text.trim();
  let lettered = !text.is_empty() && cas_letter_headed(&label);
  let foot = keys.is_some_and(|keys| cas_foot_key(&keys));
  let printed = if text.is_empty() {
    None
  } else if foot {
    if lettered {
      Some(label_ref_tokens(CAS_CURRENT_LABEL))
    } else {
      cas_number(text).map(|n| cas_letter_tokens(n, &label))
    }
  } else {
    let counter = if lettered {
      Some(lookup_int(CAS_ENTRY_COUNTER) + 1)
    } else {
      cas_number(text)
    };
    counter.map(|counter| {
      assign_value(CAS_ENTRY_COUNTER, Stored::Int(counter), Some(Scope::Global));
      let letter = radix::radix_alpha(counter);
      set_label_ref(CAS_CURRENT_LABEL, &letter);
      if lettered {
        // (the first entry under a label keeps it: `\NewLabel` defines `X@<label>:` globally only when it is free,
        // and LaTeX reads the `.aux` in a group, latex.ltx:9486-9490, so the second's local definition goes)
        set_label_ref_once(&cas_label_key(text), &letter);
      }
      cas_letter_tokens(counter, &label)
    })
  };
  let mut out = mouth::tokenize_internal("\\lx@add@contact[label=").unlist();
  out.push(T_BEGIN!());
  out.extend(printed.unwrap_or(label));
  out.push(T_END!());
  out.extend(mouth::tokenize_internal(",role=affiliation]").unlist());
  out
}

/// The tokens of the letter `\int_to_alph:n` prints for `number`, else (0, which prints none) `label` as written.
fn cas_letter_tokens(number: i64, label: &[Token]) -> Vec<Token> {
  let letter = radix::radix_alpha(number);
  if letter.is_empty() {
    label.to_vec()
  } else {
    mouth::tokenize_internal(TeXString::assembled(letter)).unlist()
  }
}

/// Whether an entry's `stm/address` keys set it as a footnote: `foot` (a `.bool_gset:N`, cas-common.sty:1180, true
/// when given no value) set true, the last setting read. Each entry reads its own keys from `foot = false`
/// (`\__reset_addr:`, :1200).
fn cas_foot_key(keys: &Tokens) -> bool {
  let text = keys.to_string();
  let mut foot = false;
  for item in text.split(',') {
    let (key, value) = item.split_once('=').unwrap_or((item, "true"));
    if key.trim() == "foot" {
      foot = value.trim().trim_matches(|c| c == '{' || c == '}').trim() == "true";
    }
  }
  foot
}

/// Whether a cas label opens with a letter (`\tl_if_head_eq_catcode:nNTF {#1} a`, spaces skipped as the head is read).
fn cas_letter_headed(label: &[Token]) -> bool {
  label
    .iter()
    .find(|t| **t != T_SPACE!())
    .is_some_and(|t| t.get_catcode() == Catcode::LETTER)
}

/// A cas label written as a number in digits, the value `\int_gset:Nn` and `\int_to_alph:n` read.
fn cas_number(text: &str) -> Option<i64> {
  if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
    return None;
  }
  text.parse().ok()
}
