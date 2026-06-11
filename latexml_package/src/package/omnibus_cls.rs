//! OmniBus.cls — fallback class for documents with unknown document classes.
//! Port of LaTeXML/Package/OmniBus.cls.ltxml (312 lines).
//!
//! Defines common frontmatter commands, theorem environments, natbib autoloads,
//! and various compatibility macros encountered in real-world arxiv submissions.
use crate::prelude::*;

/// Push the digested body of a `{keyword}`/`{keywords}` env directly into
/// the `frontmatter` map under `ltx:classification[scheme=keywords]`, the
/// way Perl LaTeXML's after_digest_keywords does it
/// (OmniBus.cls.ltxml `sub after_digest_keywords`). Avoids the
/// raw_tex(`#body`) workaround that mistakenly tokenized `#` as PARAM
/// and dumped the literal text "#body" into the output element.
fn push_keyword_body_to_frontmatter(whatsit: &mut Whatsit) -> Result<Vec<Digested>> {
  use latexml_core::{BoxOps, common::store::Stored};
  if let Some(body) = whatsit.get_body()? {
    let mut attrs: rustc_hash::FxHashMap<String, String> = rustc_hash::FxHashMap::default();
    attrs.insert("scheme".to_string(), "keywords".to_string());
    let entry = document::tag::TagData {
      tag:     "ltx:classification".to_string(),
      attr:    attrs,
      content: vec![document::tag::TagContent::Box(body)],
    };
    with_value_mut("frontmatter", |val_opt| {
      if let Some(Stored::HashTagData(frnt)) = val_opt {
        frnt
          .entry("ltx:classification".to_string())
          .or_insert_with(Vec::new)
          .push(entry);
      }
    });
  }
  Ok(Vec::new())
}

#[rustfmt::skip]
LoadDefinitions!({
  // Perl L33: LoadClass('article');
  LoadClass!("article");
  // Perl L34: ProcessOptions();
  ProcessOptions!();

  // Perl L37-45: Various common packages
  RequirePackage!("inst_support");
  RequirePackage!("epsf");
  RequirePackage!("graphicx");
  RequirePackage!("aas_macros");

  // Perl L48-51: natbib autoloads — load natbib when any of its macros is used
  for trigger in [
    "\\citet", "\\citep", "\\citealt", "\\citealp", "\\citenum",
    "\\citeauthor", "\\citefullauthor", "\\citeyear", "\\citeyearpar",
    "\\citeauthoryear", "\\setcitestyle", "\\bibpunct",
  ] {
    let cs = T_CS!(trigger);
    if !IsDefined!(&cs) {
      let cs_clone = cs;
      // Lazy-load natbib on first use of any cite trigger. After
      // require_package returns, natbib's own DefMacro for \citet etc.
      // is in scope, but the closure persists at OmniBus's load
      // frame so re-emitting cs_clone could fire THIS closure again
      // (infinite loop on every \citet — witness 2207.14344 timeout
      // with 8K+ require_package(natbib) calls). Clear the closure
      // GLOBALLY before re-emitting so the next lookup of cs_clone
      // finds natbib's binding-loaded def, not us. Task #260.
      def_macro(cs, None,
        ExpansionBody::Closure(Rc::new(move |_args| {
          require_package("natbib", RequireOptions::default())?;
          // After require_package, natbib's LoadDefinitions has overlaid
          // `\citep`/`\citet`/etc. at LOCAL scope on the current frame
          // stack — so a fresh lookup of `cs_clone` will resolve to
          // natbib's real def (the local overlay sits ABOVE this
          // closure on the meaning stack). No need to "clear" the
          // closure; just re-emit. If we cleared with Global scope
          // (previous behavior), assign_internal walks down to the
          // locked frame and removes ALL overlays of `cs_clone` —
          // including natbib's freshly-installed local def — leaving
          // `\citep` undefined after the load. Witness: 1403.6801
          // (paper-class wlpeerj.cls → OmniBus fallback → user calls
          // `\citep{foo}` → natbib loads, but `\citep` immediately
          // reverts to undefined → 101 errors + fatal). The recursion
          // concern (cited in original comment: 2207.14344 with 8K
          // require_package(natbib) calls) is moot when natbib's
          // local def shadows this closure correctly — re-emission
          // resolves to natbib's def, not back to here.
          Ok(Tokens::new(vec![cs_clone]))
        })), None)?;
    }
  }

  // Perl L52-57: save original \bibitem; redefine to auto-load natbib if the
  // argument uses \protect\citeauthoryear.
  Let!("\\lx@OmniBus@saved@bibitem", "\\bibitem");
  DefPrimitive!("\\lx@late@usepackage Semiverbatim", sub[(pkg)] {
    require_package(&pkg.to_string(), RequireOptions::default())?;
  });
  DefMacro!("\\bibitem",
    "\\@ifnext@n{[\\protect\\citeauthoryear}{\\lx@late@usepackage{natbib}\\bibitem}{\\lx@OmniBus@saved@bibitem}");

  // Perl L58-60: frontmatter section environments
  DefEnvironment!("{frontmatter}", "#body");
  DefEnvironment!("{mainmatter}",  "#body");
  DefEnvironment!("{backmatter}",  "#body");
  // Override the bare-`\frontmatter` / `\mainmatter` / `\backmatter`
  // macros to noops AFTER the DefEnvironment registration. Modern
  // book.cls-style papers (e.g. memo-l / amsmemo "Memoirs" templates
  // — paper-bundled, no binding) call bare `\frontmatter` as a
  // noop marker (book.cls semantics: sets page numbering style,
  // increments chapter counter style). Our DefEnvironment binds
  // `\frontmatter` to the env-opener, which pushes a structural
  // frame that blocks subsequent display math (`$$...$$` triggers
  // "Script _ can only appear in math mode" cascades; witness
  // 1102.3639 — book-style paper using memo-l). LaTeX's
  // `\begin{frontmatter}` / `\end{frontmatter}` env-bracket
  // tracking still functions when the opener is a noop, so
  // elsart/JHEP-style `\begin{frontmatter}...\end{frontmatter}`
  // usage continues to work (env body flows as plain text).
  Let!("\\frontmatter", "\\@empty");
  Let!("\\mainmatter",  "\\@empty");
  Let!("\\backmatter",  "\\@empty");

  // Perl L62-63
  DefMacro!("\\shorttitle{}", "\\lx@add@toctitle{#1}");
  DefMacro!("\\subtitle{}",   "\\lx@add@subtitle{#1}");

  // Perl L65-76: ignored/running title/author variants
  def_macro_noop("\\shortauthor{}")?;
  DefRegister!("\\titlerunning",  Tokens!());
  DefRegister!("\\authorrunning", Tokens!());
  Let!("\\runningauthor", "\\authorrunning");
  Let!("\\runauthor",     "\\authorrunning");
  // Running title / short authors — author metadata; preserve.
  DefMacro!("\\runningtitle{}",
    "\\lx@add@toctitle{#1}");
  Let!("\\runninghead", "\\runningtitle");
  // Perl `OmniBus.cls.ltxml` L75: `DefMacro('\shortauthors{}', Tokens())` —
  // gobble (redundant running head). Match Perl; preserving it errored on a
  // literal `&` in the running head. See 0709.4236 and aas_support_sty.rs.
  def_macro_noop("\\shortauthors{}")?;
  // \authors{author list} — alternative to \author; preserve as
  // author list note.
  DefMacro!("\\authors{}",
    "\\lx@add@frontmatter{ltx:note}[role=authors]{#1}");
  def_macro_noop("\\alignauthor")?;
  // \correspondingauthor{name/email} — common journal-class CS used
  // inside author lists (AAS / AGU / AMS / many journals). aas_support
  // routes it to ltx:contact[role=correspondent]. Provide the same in
  // OmniBus so unbound classes (e.g. ametsocV5) which use the CS
  // directly inside `\authors{...}` don't trip Error:undefined.
  // Witness 2110.11200 (ametsocV5 fallback to OmniBus).
  DefMacro!("\\correspondingauthor{}", "\\lx@add@contact[role=correspondent]{#1}");
  // \datastatement — ametsocV5.cls L992:
  // `\def\datastatement{\paragraph*{Data availability statement.}}`.
  // Used as a no-arg standalone heading marker. Witness 2203.02657.
  DefMacro!("\\datastatement", "\\paragraph*{Data availability statement.}");

  // Perl (PR #2767): email / speaker
  DefMacro!("\\email{}",     "\\lx@add@email{#1}");
  DefMacro!("\\emailaddr{}", "\\lx@add@email{#1}");
  // Perl PR #2767 has `\lx@add@email{#2}` here — `#2` is the trailing
  // OPTIONAL, so `\ead{user@example.org}` would emit an empty email and
  // drop the address. Upstream typo for `#1`; see KNOWN_PERL_ERRORS #29.
  DefMacro!("\\ead{}[]",     "\\lx@add@email{#1}");

  DefMacro!("\\emailname", "E-mail");
  DefMacro!("\\speaker{}", "\\lx@add@creator[role=speaker]{#1}");

  // Perl (PR #2767): affiliations, like from aas
  DefMacro!("\\altaffilmark Semiverbatim",
    "\\lx@request@frontmatter@annotation[altaffil]{#1}");
  DefMacro!("\\altaffiltext Semiverbatim {}",
    "\\lx@add@contact[role=altaffiliation,label={altaffil:#1}]{#2}");

  DefMacro!("\\address[]{}", "\\lx@add@address{#2}");
  Let!("\\affaddr", "\\address");
  DefMacro!("\\affil{}",       "\\lx@add@affiliation{#1}");
  DefMacro!("\\affiliation{}", "\\lx@add@affiliation{#1}");
  DefRegister!("\\affilskip" => Dimension::new(0));

  // Perl L104-123: misc name macros, mostly no-ops
  def_macro_identity("\\prefix{}")?;
  def_macro_identity("\\suffix{}")?;
  def_macro_identity("\\fnms{}")?;
  def_macro_identity("\\snm{}")?;
  def_macro_identity("\\inits{}")?;
  def_macro_identity("\\printaddresses{}")?;
  // \printead{email} — printed email address; preserve as contact.
  DefMacro!("\\printead{}",
    "\\lx@add@frontmatter{ltx:note}[role=email]{#1}");
  // Page numbers — author metadata; preserve as ltx:note.
  DefMacro!("\\firstpage{}",       "\\lx@add@frontmatter{ltx:note}[role=firstpage]{#1}");
  DefMacro!("\\lastpage{}",        "\\lx@add@frontmatter{ltx:note}[role=lastpage]{#1}");
  // \runauthor / \runtitle are running-header SHORT forms, layout-only and
  // redundant with \author/\title. Perl OmniBus.cls.ltxml L114-115 GOBBLES both
  // (`DefMacro('\runauthor{}', Tokens())`); preserving them digests the
  // running-head content and errors on author typos (stray `\` before a name →
  // undefined CS). Gobble to match Perl; \author/\title keep the real content.
  def_macro_noop("\\runauthor{}")?;
  def_macro_noop("\\runtitle{}")?;
  // \corref{label} — marker for corresponding author. Preserve as note.
  DefMacro!("\\corref{}",          "\\lx@add@frontmatter{ltx:note}[role=corref]{#1}");
  DefMacro!("\\listofauthors{}",   "\\lx@add@frontmatter{ltx:note}[role=listofauthors]{#1}");
  DefMacro!("\\indexauthor{}",     "\\lx@add@frontmatter{ltx:note}[role=indexauthor]{#1}");
  def_macro_noop("\\preface")?;
  def_macro_noop("\\thankstext")?;
  def_macro_noop("\\numberofauthors{}")?;
  // \equalcontrib / \equalcont are defined kernel-level in
  // latex_constructs.rs — needed for ALL classes (not only OmniBus
  // fallback) because aaai22.sty etc. ride on \documentclass{article}.
  // Springer Nature `sn-jnl.cls` style author-name and org-address parts
  // (cls L519-525, L599-606). The cls defines them as low-level `\def`s
  // wrapped in `\leavevmode\hbox{...}`, but our raw-class load path skips
  // class .cls files by default (INCLUDE_CLASSES=false unless rawclasses
  // option is passed), so the user's `\author{\fnm{First} \sur{Last}}` hits
  // undefined CS. Stub at OmniBus level as passthrough — for unknown
  // Springer-style classes that fall through to OmniBus, the author name
  // renders as plain text in <ltx:personname>. Doesn't affect papers
  // where the cls binding IS loaded (those override). Driver: 2403.18604,
  // 2110.04544, ~40 sn-jnl papers in canvas pool.
  def_macro_identity("\\fnm{}")?;      // first name
  DefMacro!("\\sur{}",   " #1");     // surname (cls inserts ~)
  def_macro_identity("\\spfx{}")?;      // surname prefix (e.g. "van")
  def_macro_identity("\\pfx{}")?;      // name prefix (e.g. "Dr.")
  def_macro_identity("\\sfx{}")?;      // name suffix
  def_macro_identity("\\tanm{}")?;      // title-as-name
  // `\dgr` (Springer sn-jnl "degree", 1-arg) CONFLICTS with the common
  // physics-paper convention `\newcommand{\dgr}{\dagger}` (0-arg). Defining
  // it eagerly here blocks the user's `\newcommand` (already-defined) so a
  // later `c_i^\dgr c_j` consumed the following `c` as `\dgr`'s argument →
  // `c_i^{c}_j` → spurious `Double subscript` (witness 1603.02507, ×23; Perl
  // rc=0 — Perl's OmniBus never defines `\dgr`). Defer to \AtBeginDocument
  // with \providecommand so a user preamble definition wins, while the
  // Springer `\author{… \dgr{…} …}` (expanded at \maketitle, after
  // begin-document) still gets the 1-arg fallback. `##1` doubles for the hook.
  RawTeX!(r"\AtBeginDocument{\providecommand{\dgr}[1]{##1}}");
  def_macro_identity("\\orgdiv{}")?;
  def_macro_identity("\\orgname{}")?;
  def_macro_identity("\\orgaddress{}")?;
  def_macro_identity("\\street{}")?;
  def_macro_identity("\\postcode{}")?;
  def_macro_identity("\\city{}")?;
  def_macro_identity("\\country{}")?;
  // `\state` is a TeX 4-token `\count` register inside many classes (article
  // declares it as `\newcount` for some configurations). We do NOT stub it
  // here — overlapping with the kernel register would break papers that
  // expect the integer. The few sn-jnl papers using `\state{...}` for
  // address components will keep that error; all the OTHER fields above
  // are non-conflicting. Same caution for `\affil` — already overloaded
  // by amsart and other classes; leaving it to specific class bindings.
  def_macro_noop("\\bibcommenthead")?;
  def_macro_noop("\\jyear[]")?;
  DefMacro!("\\resumen{}",         "\\lx@add@abstract{#1}");
  DefMacro!("\\ion{}{}",           "{#1 \\textsc{#2}}");
  Let!("\\fulladdresses", "\\address");
  DefMacro!("\\smonth{}", "\\month=#1\\relax");
  DefMacro!("\\syear{}",  "\\year=#1\\relax");

  // Perl (PR #2767): keyword macros
  DefMacro!("\\keywords{}", "\\lx@add@keywords{#1}");
  DefMacro!("\\kword{}",    "\\lx@add@keywords{#1}");
  DefMacro!("\\kwd[]{}",    "\\lx@add@keywords{#2, }");

  // Perl L133-156: {keyword}, {keywords} as environments, plus auto-variants
  // via `\keywords` that can be used as a section-like bare macro.
  // Push the digested body directly into the frontmatter map under
  // `ltx:classification[scheme=keywords]`, matching Perl's
  // after_digest_keywords (OmniBus.cls.ltxml:after_digest_keywords)
  // which does `push(@{ $$frontmatter{'ltx:classification'} }, [tag, attrs, @LaTeXML::LIST])`.
  //
  // History: the previous Rust binding called
  //   stomach::raw_tex("\\lx@add@frontmatter{ltx:classification}[scheme=keywords]{#body}")
  // which silently misused `#body` (a Constructor template placeholder
  // only valid inside a DefConstructor template string) inside a
  // raw_tex literal. The `#` was tokenized as PARAM and reached the
  // stomach — emitting `Error:misdefined:#` and dumping the literal
  // string `#body` as the classification element's text content.
  // Witness: ifacconf-class papers (2305.08080, 2305.09991 — 84 wp4
  // entries with `Error:misdefined:#` first-errors).
  DefEnvironment!("{keyword}", "",
    after_digest_body => sub[whatsit] {
      push_keyword_body_to_frontmatter(whatsit)
    });
  DefEnvironment!("{keywords}", "",
    after_digest_body => sub[whatsit] {
      push_keyword_body_to_frontmatter(whatsit)
    });
  // Perl L143: Let('\lx@begin@keywords', '\keywords'); — saved before overload
  Let!("\\lx@begin@keywords", "\\keywords");
  // Perl OmniBus.cls.ltxml L154. We differ from Perl's
  // `\begin{keywords}#1\end{keywords}` path because our `{keywords}` env
  // currently emits <ltx:classification> inline (a content-model error in
  // contexts like <ltx:abstract>). Routing directly through
  // `\@add@frontmatter` matches Perl's net effect — its after_digest_keywords
  // pushes the body into `frontmatter`{ltx:classification} — without the
  // inline detour that confuses the schema.
  // NOTE: unlike Perl L154 we do NOT append `\let\endkeyword\relax
  // \let\endkeywords\relax` here. Those `\let`s exist in Perl to neutralise the
  // ENV ending after its `\begin{keywords}#1\end{keywords}` one-shot. We route
  // through `\@add@frontmatter` (no env opened, see comment above), so the
  // `\let`s are vestigial — and harmful: a later BARE `\keywords text` opens a
  // classification whose auto-close hook is `\maybe@end@keywords`→`\endkeywords`,
  // and a persisted `\endkeywords=\relax` made that close a no-op, so the
  // classification stayed open and the following `\section` nested inside it
  // ("ltx:section isn't allowed in ltx:classification"). This bites whenever a
  // braced `\keywords{}` precedes a bare one — e.g. `\category{a}{b}{c}` expands
  // to `…\keywords{#4}` (empty #4) THEN the document's own `\keywords …`.
  // Witness 1601.07962 (sig-alternate, \category + bare \terms/\keywords).
  // Deliberate divergence from post-PR-2767 Perl (which uses
  // `\begin{keywords}#1\end{keywords}\let\endkeywords\relax`): the trailing
  // \lets persist and break a later bare `\keywords` (witness 1601.07962,
  // see the comment above). Keep the no-env direct call, expressed in the
  // new frontmatter API.
  DefMacro!("\\keywords@onearg{}",
    "\\lx@add@classification[scheme=keywords]{#1}");
  DefMacro!("\\maybe@end@keywords",
    "\\endkeywords\\let\\maybe@end@keywords\\relax");
  // Perl L145-153: `\keyword` / `\keywords` overloaded: with {...} arg, run
  // \keywords@onearg; otherwise hook a pending \endkeywords via the section-
  // start hook so `\keyword foo \section{bar}` auto-closes the keywords env.
  DefMacro!("\\keyword", sub[_args] {
    let next = read_token()?;
    if let Some(ref t) = next {
      unread(Tokens!(*t));
      if t.get_catcode() == Catcode::BEGIN {
        return Ok(Tokens!(T_CS!("\\keywords@onearg")));
      }
    }
    Ok(Tokens::new(vec![
      T_CS!("\\g@addto@macro"),
      T_CS!("\\@startsection@hook"),
      T_CS!("\\maybe@end@keywords"),
      T_CS!("\\lx@begin@keywords"),
    ]))
  });
  DefMacro!("\\keywords", sub[_args] {
    let next = read_token()?;
    if let Some(ref t) = next {
      unread(Tokens!(*t));
      if t.get_catcode() == Catcode::BEGIN {
        return Ok(Tokens!(T_CS!("\\keywords@onearg")));
      }
    }
    Ok(Tokens::new(vec![
      T_CS!("\\g@addto@macro"),
      T_CS!("\\@startsection@hook"),
      T_CS!("\\maybe@end@keywords"),
      T_CS!("\\lx@begin@keywords"),
    ]))
  });
  Let!("\\addto@keywords@list", "\\keyword");

  // Perl (PR #2767): classifications
  DefMacro!("\\classification{}", "\\lx@add@classification{#1}");
  DefMacro!("\\pacs{}",
    "\\lx@add@classification[scheme=pacs]{#1}", locked => true);
  // \doi — frontmatter in preamble, url-like in body.
  DefMacro!("\\doi{}",
    "\\if@in@preamble{\\lx@add@pubnote[role=doi]{#1}\
     \\else\\lx@doi{#1}\\fi");
  DefConstructor!("\\lx@doi{}",
    "<ltx:ref href='https://doi.org/#1'>#1</ltx:ref>",
    enter_horizontal => true);

  // Perl L167: \category (acm)
  DefMacro!("\\category{}{}{}[]",
    "\\lx@add@classification[scheme=category]{#1 #2 #3}\\keywords{#4}");

  // Perl L169-219: theorem env autoloads — if a common theorem env name is
  // used without being declared, load amsthm and define a standard set.
  let theorem_preload = concat!(
    "\\newtheorem{theorem}{Theorem}[section]",
    "\\newtheorem{conjecture}[theorem]{Conjecture}",
    "\\newtheorem{proposition}[theorem]{Proposition}",
    "\\newtheorem{proof}[theorem]{Proof}",
    "\\newtheorem{lemma}[theorem]{Lemma}",
    "\\newtheorem{corollary}[theorem]{Corollary}",
    "\\newtheorem{example}[theorem]{Example}",
    "\\newtheorem{exercise}[theorem]{Exercise}",
    "\\newtheorem{definition}[theorem]{Definition}",
    "\\newtheorem{problem}[theorem]{Problem}",
    "\\newtheorem{question}[theorem]{Question}",
    "\\newtheorem{remark}[theorem]{Remark}",
    "\\newtheorem{solution}[theorem]{Solution}",
    "\\newtheorem{step}[theorem]{Step}",
    "\\newtheorem{note}[theorem]{Note}",
    "\\newtheorem{thm}{Theorem}",
    "\\newtheorem{cor}[thm]{Corollary}",
    "\\newtheorem{lem}[thm]{Lemma}",
    "\\newtheorem{claim}[thm]{Claim}",
    "\\newtheorem{axiom}[thm]{Axiom}",
    "\\newtheorem{conj}[thm]{Conjecture}",
    "\\newtheorem{fact}[thm]{Fact}",
    "\\newtheorem{hypo}[thm]{Hypothesis}",
    "\\newtheorem{assum}[thm]{Assumption}",
    "\\newtheorem{prop}[thm]{Proposition}",
    "\\newtheorem{crit}[thm]{Criterion}",
    // Guard `\theoremstyle{definition}` with `\@ifundefined` so the
    // stub does NOT error when the document deliberately undefined
    // `\theoremstyle` (e.g. `\let\theoremstyle\@undefined` followed
    // by `\usepackage{amsthm}`-as-no-op-because-already-loaded). The
    // stub is a *fallback* for env auto-loading; if the document
    // chose to disable `\theoremstyle`, respect that choice instead
    // of resurrecting an undefined-error. Witness: arXiv:2603.11260,
    // 2603.11265 (ifacconf.cls -> theorem.sty -> amsthm pre-loaded,
    // then user `\let\theoremstyle\@undefined`).
    "\\@ifundefined{theoremstyle}{}{\\theoremstyle{definition}}",
    "\\newtheorem{defn}[thm]{Definition}",
    "\\newtheorem{exmp}[thm]{Example}",
    "\\newtheorem{rem}[thm]{Remark}",
    "\\newtheorem{prob}[thm]{Problem}",
    "\\newtheorem{prin}[thm]{Principle}",
    "\\newtheorem{alg}{Algorithm}",
  );
  // Only install autoload stubs for CSes that aren't already defined. If
  // amsthm was pre-loaded (e.g. via dep-scan of a local .cls that
  // `\RequirePackage{amsthm}`), its own `\theoremstyle`/`\newtheorem`
  // definitions are already in place. Overwriting with our stub would
  // create an infinite loop:
  //   stub invokes → require_package('amsthm') no-ops (already loaded)
  //   → re-emits `\theoremstyle` → stub invokes again …
  // Observed in arxiv 0906.1883 where birkmult.cls's dep-scan pre-loaded
  // amsthm, and the resulting 163M-iteration pin loop blew the arena
  // past `u32::MAX` offset (SymStr wraparound → garbled error text).
  for env in [
    "conjecture", "theorem", "corollary", "definition", "example", "exercise",
    "lemma", "note", "problem", "proof", "proposition", "question", "remark",
    "solution",
    "thm", "cor", "lem", "claim", "axiom", "conj", "fact", "hypo", "assum",
    "prop", "crit", "defn", "exmp", "rem", "prob", "prin", "alg",
  ] {
    let beginenv = s!("\\begin{{{env}}}");
    let cs = T_CS!(&beginenv);
    if IsDefined!(&cs) { continue; }
    let beginenv_clone = beginenv.clone();
    let preload = theorem_preload.to_string();
    def_macro(cs, None,
      ExpansionBody::Closure(Rc::new(move |_args| {
        require_package("amsthm", RequireOptions::default())?;
        let mut expanded = preload.clone();
        expanded.push_str(&beginenv_clone);
        Ok(mouth::tokenize_internal(&expanded))
      })), None)?;
  }
  // Perl L216-219: newtheorem aliases auto-load amsthm
  for alias in ["\\newproclaim", "\\newdef", "\\newremark"] {
    let cs = T_CS!(alias);
    if IsDefined!(&cs) { continue; }
    def_macro(cs, None,
      ExpansionBody::Closure(Rc::new(move |_args| {
        require_package("amsthm", RequireOptions::default())?;
        Ok(Tokens!(T_CS!("\\newtheorem")))
      })), None)?;
  }
  // Perl L220: \theoremstyle autoloads amsthm
  {
    let cs = T_CS!("\\theoremstyle");
    if !IsDefined!(&cs) {
      def_macro(cs, None,
        ExpansionBody::Closure(Rc::new(move |_args| {
          require_package("amsthm", RequireOptions::default())?;
          Ok(Tokens!(T_CS!("\\theoremstyle")))
        })), None)?;
    }
  }

  // Perl L222-223: abstract aliases
  Let!("\\abstracts", "\\abstract");
  Let!("\\abst",      "\\abstract");

  // Perl L226-235: acknowledgments
  DefConstructor!("\\acknowledgments", "<ltx:acknowledgements name='#name'>",
    properties => {
      Ok(stored_map!("name" => digest(T_CS!("\\acknowledgmentsname"))?))
    });
  // \endacknowledgments — tolerant close. A common pattern is
  //   \begin{acknowledgments} ... \bibliography{...} \end{acknowledgments}
  // where \bibliography opens <ltx:bibliography>; the auto_close on
  // <ltx:acknowledgements> below cascades shut at that point, so the
  // explicit \end{acknowledgments} would otherwise hit a malformed-close
  // error. Check current node first; only emit </ltx:acknowledgements>
  // when one is actually open. Driver: 2202.04803 R=1 → R=0; ~9 papers
  // in this cluster.
  DefConstructor!("\\endacknowledgments", sub[document, _whatsit, _props] {
    let cur = document.get_node().clone();
    let has_open = document.findnode("ancestor-or-self::ltx:acknowledgements", Some(&cur)).is_some();
    if has_open {
      document.close_element("ltx:acknowledgements")?;
    }
  });
  Tag!("ltx:acknowledgements", auto_close => true);
  DefMacro!("\\acknowledgmentsname", "Acknowledgements");
  Let!("\\acknowledgements",      "\\acknowledgments");
  Let!("\\endacknowledgements",   "\\endacknowledgments");
  Let!("\\theacknowledgments",    "\\acknowledgments");
  Let!("\\endtheacknowledgments", "\\endacknowledgments");

  // Perl (PR #2767): editorial metadata
  DefMacro!("\\editors{}",          "\\lx@add@editor{#1}");
  DefMacro!("\\received{}",         "\\lx@add@date[role=received]{#1}");
  DefMacro!("\\revised{}",          "\\lx@add@date[role=revised]{#1}");
  DefMacro!("\\accepted{}",         "\\lx@add@date[role=accepted]{#1}");
  DefMacro!("\\pubyear{}",          "\\lx@add@date[role=publication]{#1}");
  DefMacro!("\\copyrightyear{}",    "\\lx@add@copyrightyear{#1}");
  DefMacro!("\\communicated{}",     "\\lx@add@date[role=communicated]{#1}");
  DefMacro!("\\preprint{}",         "\\lx@add@pubnote[role=preprint]{#1}");
  DefMacro!("\\dedicated{}",        "\\lx@add@pubnote[role=dedication]{#1}");
  DefMacro!("\\presented{}",        "\\lx@add@date[role=presented]{#1}");
  DefMacro!("\\articletype{}",      "\\lx@add@pubnote[role=type]{#1}");
  DefMacro!("\\issue{}",            "\\lx@add@pubnote[role=issue]{#1}");
  DefMacro!("\\journal{}",          "\\lx@add@pubnote[role=journal]{#1}");
  DefMacro!("\\jname{}",            "\\lx@add@pubnote[role=journal]{#1}");
  DefMacro!("\\volume{}",           "\\lx@add@pubnote[role=volume]{#1}");
  DefMacro!("\\titlenote{}",        "\\lx@add@pubnote[role=note]{#1}");
  DefMacro!("\\terms{}",            "\\lx@add@keywords{#1}");
  DefMacro!("\\conferenceinfo{}{}", "\\lx@add@pubnote[role=conference]{#1 #2}");

  // Perl L257 gobbles to Tokens(); we surpass by rendering as
  // superscript (matches latex_constructs kernel-level treatment
  // and IEEE \IEEEauthorrefmark).
  DefMacro!("\\thanksref{}", "\\textsuperscript{#1}");

  // Perl L260-264: ACM variants
  Let!("\\CopyrightYear", "\\copyrightyear");
  DefRegister!("\\confinfo"     => Tokens!());
  DefRegister!("\\acmcopyr"     => Tokens!());
  DefRegister!("\\copyrightetc" => Tokens!());
  Let!("\\crdata", "\\acmcopyr");

  // Perl L266-280: \references constructor (works as env or bare macro)
  // Simplified: Perl's before_digest/after_digest setup for bibliography
  // hooks is not yet publicly exposed; the constructor below produces the
  // correct outer wrappers, relying on \bibitem's own auto_open/auto_close.
  DefConstructor!("\\references",
    "<ltx:bibliography xml:id='#id'><ltx:biblist>");
  DefConstructor!("\\endreferences", sub[document] {
    let _ = document.maybe_close_element("ltx:biblist");
    let _ = document.maybe_close_element("ltx:bibliography");
  });
  Let!("\\reference", "\\bibitem");

  // Perl L282-284
  def_macro_noop("\\comment{}")?;
  DefMacro!("\\etal",         "\\textit{et al.}");
  def_macro_noop("\\firstsection")?;

  // Perl L286-297: math/package autoloads — when a trigger CS is used and
  // not yet defined, require the specified package/class and re-trigger.
  // The Perl `DefAutoload` macro registers this semantic; we implement it
  // inline. Perl encodes the target via the `.sty.ltxml` / `.cls.ltxml`
  // suffix; we mirror that by carrying an explicit "sty" / "cls" kind
  // alongside the bare name.
  //
  // Why the kind matters: `\thechapter` autoloads `book.cls.ltxml` in Perl,
  // not `book.sty`. Routing through `require_package("book")` instead
  // finds the obsolete `book.sty` shim (TL's 2.09-compat file) which
  // immediately fires `\LoadClass{book}` from inside the body — past the
  // preamble — and errors with "\LoadClass can only appear in the preamble".
  // Witness: arXiv:2602.10407 (`\documentclass{saunders}` + `\chapter` in
  // an `\include`'d chapter; saunders.cls is unbound so OmniBus takes over,
  // and the chapter trigger hits `\thechapter`). Perl handles it cleanly
  // by autoloading the class binding instead of the obsolete sty.
  for (trigger, name, ext) in [
    // env triggers: `\begin{align}` etc. In Rust, we only dispatch on the
    // bare CS name of the trigger — works for control sequences like
    // `\multline`, `\numberwithin`, `\mathfrak`, `\mathbb`, `\deluxetable`,
    // `\curraddr`, `\subjclass`, `\thechapter`. For envs, autoload key is
    // `\begin{env}` which is a CS token.
    ("\\begin{align}",                "amsmath",      "sty"),
    ("\\begin{subequations}",         "amsmath",      "sty"),
    ("\\begin{split}",                "amsmath",      "sty"),
    ("\\multline",                    "amsmath",      "sty"),
    ("\\csname multline*\\endcsname", "amsmath",      "sty"),
    ("\\numberwithin",                "amsmath",      "sty"),
    ("\\mathfrak",                    "amsfonts",     "sty"),
    ("\\mathbb",                      "amsfonts",     "sty"),
    ("\\begin{deluxetable}",          "deluxetable",  "sty"),
    ("\\curraddr",                    "ams_support",  "sty"),
    ("\\subjclass",                   "ams_support",  "sty"),
    ("\\thechapter",                  "book",         "cls"),
  ] {
    let cs = T_CS!(trigger);
    if !IsDefined!(&cs) {
      let cs_clone = cs;
      let name_str = name.to_string();
      let trigger_str = trigger.to_string();
      let is_cls = ext == "cls";
      def_macro(cs, None,
        ExpansionBody::Closure(Rc::new(move |_args| {
          // Mirrors Perl's DefAutoload → ClearAutoLoad in Package.pm:
          // clear this autoload CS before loading, then re-emit the trigger as
          // tokenized text. Re-tokenizing is important for `\begin{env}` triggers
          // — amsmath defines `\split` (not `\begin{split}`), so the raw single-CS
          // token would look undefined after clearing. Tokenizing expands into
          // `\begin` + `{env}` which the standard `\begin{}` dispatcher resolves.
          assign_meaning(
            &cs_clone, Stored::None,
            Some(Scope::Global));
          if is_cls {
            load_class(
              &name_str, Vec::new(), Tokens::default())?;
          } else {
            require_package(&name_str, RequireOptions::default())?;
          }
          Ok(mouth::tokenize_internal(&trigger_str))
        })), None)?;
    }
  }

  // Perl L302-307: old-style Section/Subsection aliases
  DefMacro!("\\Section",       "\\@startsection{section}{1}{}{}{}{}", locked => true);
  DefMacro!("\\Subsection",    "\\@startsection{subsection}{2}{}{}{}{}", locked => true);
  DefMacro!("\\Subsubsection", "\\@startsection{subsubsection}{3}{}{}{}{}", locked => true);
  DefMacro!("\\Paragraph",     "\\@startsection{paragraph}{4}{}{}{}{}", locked => true);
  DefMacro!("\\Subparagraph",  "\\@startsection{subparagraph}{5}{}{}{}{}", locked => true);

  // Perl L310: author block env
  DefEnvironment!("{aug}", "#body");
});
