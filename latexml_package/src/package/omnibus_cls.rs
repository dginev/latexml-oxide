//! OmniBus.cls — fallback class for documents with unknown document classes.
//! Port of LaTeXML/Package/OmniBus.cls.ltxml (312 lines).
//!
//! Defines common frontmatter commands, theorem environments, natbib autoloads,
//! and various compatibility macros encountered in real-world arxiv submissions.
use crate::prelude::*;

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
      def_macro(cs, None,
        latexml_core::definition::ExpansionBody::Closure(Rc::new(move |_args| {
          require_package("natbib", RequireOptions::default())?;
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

  // Perl L62-63
  DefMacro!("\\shorttitle{}", "\\@add@frontmatter{ltx:toctitle}{#1}");
  DefMacro!("\\subtitle{}",   "\\@add@frontmatter{ltx:subtitle}{#1}");

  // Perl L65-76: ignored/running title/author variants
  DefMacro!("\\shortauthor{}", "");
  DefRegister!("\\titlerunning",  Tokens!());
  DefRegister!("\\authorrunning", Tokens!());
  Let!("\\runningauthor", "\\authorrunning");
  Let!("\\runauthor",     "\\authorrunning");
  DefMacro!("\\runningtitle{}", None);
  Let!("\\runninghead", "\\runningtitle");
  DefMacro!("\\shortauthors{}", None);
  DefMacro!("\\authors{}",      None);
  DefMacro!("\\alignauthor",    None);

  // Perl L78-83: email / speaker
  DefConstructor!("\\@@@email{}{}", "^ <ltx:contact role='#2'>#1</ltx:contact>");
  DefMacro!("\\email{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}{email}}");
  Let!("\\emailaddr", "\\email");
  DefMacro!("\\ead{}[]",   "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}{#2}}");
  DefMacro!("\\emailname", "E-mail");
  DefMacro!("\\speaker{}", "\\@add@frontmatter{ltx:creator}[role=speaker]{\\@personname{#1}}");

  // Perl L86-102: affiliations
  DefConstructor!("\\@@@affiliation{}", "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  DefMacro!("\\affil{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@affiliation{#1}}");
  DefMacro!("\\altaffilmark{}", sub[(marks)] {
    let parts = split_tokens(marks, vec![T_OTHER!(",")]);
    let mut out = Vec::new();
    for part in parts {
      out.push(T_CS!("\\@altaffilmark"));
      out.push(T_BEGIN!());
      out.extend(part.unlist());
      out.push(T_END!());
    }
    out
  });
  DefConstructor!("\\@altaffilmark{}",
    "?#1(<ltx:note role='affiliationmark' mark='#1'/> )()");
  Let!("\\affilnum", "\\@altaffilmark");
  DefConstructor!("\\altaffiltext{}{}",
    "?#2(<ltx:note role='affiliationtext' mark='#1'>#2</ltx:note>)()");

  DefConstructor!("\\@@@address{}", "^ <ltx:contact role='address'>#1</ltx:contact>");
  DefMacro!("\\address[]{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@address{#2}}");
  Let!("\\affaddr", "\\address");
  DefMacro!("\\affiliation{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@affiliation{#1}}");
  DefRegister!("\\affilskip" => Dimension::new(0));

  // Perl L104-123: misc name macros, mostly no-ops
  DefMacro!("\\prefix{}",          "#1");
  DefMacro!("\\suffix{}",          "#1");
  DefMacro!("\\fnms{}",            "#1");
  DefMacro!("\\snm{}",             "#1");
  DefMacro!("\\inits{}",           "#1");
  DefMacro!("\\printaddresses{}",  "#1");
  DefMacro!("\\printead{}",        None);
  DefMacro!("\\firstpage{}",       None);
  DefMacro!("\\lastpage{}",        None);
  DefMacro!("\\runauthor{}",       None);
  DefMacro!("\\runtitle{}",        None);
  DefMacro!("\\corref{}",          None);
  DefMacro!("\\listofauthors{}",   None);
  DefMacro!("\\indexauthor{}",     None);
  DefMacro!("\\preface",           None);
  DefMacro!("\\thankstext",        None);
  DefMacro!("\\numberofauthors{}", None);
  // Conference-template "equal contribution" markers. AAAI's aaai22.sty
  // and similar define \equalcontrib only inside \@maketitle (locally
  // scoped), so user code that calls it inside \author{} (before
  // \maketitle) hits an undefined-CS error. neurips_*.sty and others
  // also use this name. Pre-define to no-op at top level — the local
  // \@maketitle redefinition will override at \maketitle time.
  // Driver: 2103.05277, 2111.06599, 2006.08767 — 3 papers in the
  // canvas-failing pool.
  DefMacro!("\\equalcontrib",      None);
  DefMacro!("\\equalcont",         None);
  DefMacro!("\\resumen{}",         "\\@add@frontmatter{ltx:abstract}{#1}");
  DefMacro!("\\ion{}{}",           "{#1 \\textsc{#2}}");
  Let!("\\fulladdresses", "\\address");
  Let!("\\smonth",        "\\month");
  Let!("\\syear",         "\\year");

  // Perl L128-131: keyword macros
  DefMacro!("\\keywords{}", "\\@add@frontmatter{ltx:keywords}{#1}");
  DefMacro!("\\kword{}",    "\\@add@frontmatter{ltx:keywords}{#1}");
  DefMacro!("\\kwd[]{}",    "\\@add@frontmatter{ltx:keywords}{#2, }");

  // Perl L133-156: {keyword}, {keywords} as environments, plus auto-variants
  // via `\keywords` that can be used as a section-like bare macro.
  DefEnvironment!("{keyword}",
    "<ltx:classification scheme='keywords'>#body</ltx:classification>");
  DefEnvironment!("{keywords}",
    "<ltx:classification scheme='keywords'>#body</ltx:classification>");
  // Perl L143: Let('\lx@begin@keywords', '\keywords'); — saved before overload
  Let!("\\lx@begin@keywords", "\\keywords");
  // Perl OmniBus.cls.ltxml L154. We differ from Perl's
  // `\begin{keywords}#1\end{keywords}` path because our `{keywords}` env
  // currently emits <ltx:classification> inline (a content-model error in
  // contexts like <ltx:abstract>). Routing directly through
  // `\@add@frontmatter` matches Perl's net effect — its after_digest_keywords
  // pushes the body into `frontmatter`{ltx:classification} — without the
  // inline detour that confuses the schema.
  DefMacro!("\\keywords@onearg{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}\
     \\let\\endkeyword\\relax\\let\\endkeywords\\relax");
  DefMacro!("\\maybe@end@keywords",
    "\\endkeywords\\let\\maybe@end@keywords\\relax");
  // Perl L145-153: `\keyword` / `\keywords` overloaded: with {...} arg, run
  // \keywords@onearg; otherwise hook a pending \endkeywords via the section-
  // start hook so `\keyword foo \section{bar}` auto-closes the keywords env.
  DefMacro!("\\keyword", sub[_args] {
    let next = gullet::read_token()?;
    if let Some(ref t) = next {
      gullet::unread(Tokens!(*t));
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
    let next = gullet::read_token()?;
    if let Some(ref t) = next {
      gullet::unread(Tokens!(*t));
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

  // Perl L158-164: classifications
  DefMacro!("\\classification{}", "\\@add@frontmatter{ltx:classification}{#1}");
  DefMacro!("\\pacs{}",
    "\\@add@frontmatter{ltx:classification}[scheme=pacs]{#1}", locked => true);
  // \doi — frontmatter in preamble, url-like in body. Perl L161-163.
  DefMacro!("\\doi{}",
    "\\if@in@preamble{\\@add@frontmatter{ltx:classification}[scheme=doi]{#1}\
     \\else\\lx@doi{#1}\\fi");
  DefConstructor!("\\lx@doi{}",
    "<ltx:ref href='https://doi.org/#1'>#1</ltx:ref>",
    enter_horizontal => true);

  // Perl L167: \category (acm)
  DefMacro!("\\category{}{}{}[]",
    "\\@add@frontmatter{ltx:classification}[scheme=category]{#1 #2 #3}\\keywords{#4}");

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
    "\\theoremstyle{definition}",
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
      latexml_core::definition::ExpansionBody::Closure(Rc::new(move |_args| {
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
      latexml_core::definition::ExpansionBody::Closure(Rc::new(move |_args| {
        require_package("amsthm", RequireOptions::default())?;
        Ok(Tokens!(T_CS!("\\newtheorem")))
      })), None)?;
  }
  // Perl L220: \theoremstyle autoloads amsthm
  {
    let cs = T_CS!("\\theoremstyle");
    if !IsDefined!(&cs) {
      def_macro(cs, None,
        latexml_core::definition::ExpansionBody::Closure(Rc::new(move |_args| {
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
      Ok(stored_map!("name" => stomach::digest(T_CS!("\\acknowledgmentsname"))?))
    });
  DefConstructor!("\\endacknowledgments", "</ltx:acknowledgements>");
  Tag!("ltx:acknowledgements", auto_close => true);
  DefMacro!("\\acknowledgmentsname", "Acknowledgements");
  Let!("\\acknowledgements",      "\\acknowledgments");
  Let!("\\endacknowledgements",   "\\endacknowledgments");
  Let!("\\theacknowledgments",    "\\acknowledgments");
  Let!("\\endtheacknowledgments", "\\endacknowledgments");

  // Perl L237-254: editorial metadata
  DefMacro!("\\editors{}",          "\\@add@frontmatter{ltx:note}[role=editors]{#1}");
  DefMacro!("\\received{}",         "\\@add@frontmatter{ltx:date}[role=received]{#1}");
  DefMacro!("\\revised{}",          "\\@add@frontmatter{ltx:date}[role=revised]{#1}");
  DefMacro!("\\accepted{}",         "\\@add@frontmatter{ltx:date}[role=accepted]{#1}");
  DefMacro!("\\pubyear{}",          "\\@add@frontmatter{ltx:date}[role=publication]{#1}");
  DefMacro!("\\copyrightyear{}",    "\\@add@frontmatter{ltx:date}[role=copyright]{#1}");
  DefMacro!("\\preprint{}",         "\\@add@frontmatter{ltx:note}[role=preprint]{#1}");
  DefMacro!("\\communicated{}",     "\\@add@frontmatter{ltx:date}[role=communicated]{#1}");
  DefMacro!("\\dedicated{}",        "\\@add@frontmatter{ltx:note}[role=dedicated]{#1}");
  DefMacro!("\\presented{}",        "\\@add@frontmatter{ltx:date}[role=presented]{#1}");
  DefMacro!("\\articletype{}",      "\\@add@frontmatter{ltx:note}[role=articletype]{#1}");
  DefMacro!("\\issue{}",            "\\@add@frontmatter{ltx:note}[role=issue]{#1}");
  DefMacro!("\\journal{}",          "\\@add@frontmatter{ltx:note}[role=journal]{#1}");
  DefMacro!("\\jname{}",            "\\@add@frontmatter{ltx:note}[role=journal]{#1}");
  DefMacro!("\\volume{}",           "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
  DefMacro!("\\titlenote{}",        "\\@add@frontmatter{ltx:note}[role=titlenote]{#1}");
  DefMacro!("\\terms{}",            "\\@add@frontmatter{ltx:note}[role=terms]{#1}");
  DefMacro!("\\conferenceinfo{}{}", "\\@add@frontmatter{ltx:note}[role=conference]{#1 #2}");

  // Perl L257
  DefMacro!("\\thanksref{}", None);

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
  DefMacro!("\\comment{}",    None);
  DefMacro!("\\etal",         "\\textit{et al.}");
  DefMacro!("\\firstsection", None);

  // Perl L286-297: math/package autoloads — when a trigger CS is used and
  // not yet defined, require the specified package and re-trigger. The Perl
  // `DefAutoload` macro registers this semantic; we implement it inline.
  for (trigger, pkg) in [
    // env triggers: `\begin{align}` etc. In Rust, we only dispatch on the
    // bare CS name of the trigger — works for control sequences like
    // `\multline`, `\numberwithin`, `\mathfrak`, `\mathbb`, `\deluxetable`,
    // `\curraddr`, `\subjclass`, `\thechapter`. For envs, autoload key is
    // `\begin{env}` which is a CS token.
    ("\\begin{align}",         "amsmath"),
    ("\\begin{subequations}",  "amsmath"),
    ("\\begin{split}",         "amsmath"),
    ("\\multline",             "amsmath"),
    ("\\csname multline*\\endcsname", "amsmath"),
    ("\\numberwithin",         "amsmath"),
    ("\\mathfrak",             "amsfonts"),
    ("\\mathbb",               "amsfonts"),
    ("\\begin{deluxetable}",   "deluxetable"),
    ("\\curraddr",             "ams_support"),
    ("\\subjclass",            "ams_support"),
    ("\\thechapter",           "book"),
  ] {
    let cs = T_CS!(trigger);
    if !IsDefined!(&cs) {
      let cs_clone = cs;
      let pkg_str = pkg.to_string();
      let trigger_str = trigger.to_string();
      def_macro(cs, None,
        latexml_core::definition::ExpansionBody::Closure(Rc::new(move |_args| {
          // Mirrors Perl's DefAutoload → ClearAutoLoad in Package.pm:
          // clear this autoload CS before loading, then re-emit the trigger as
          // tokenized text. Re-tokenizing is important for `\begin{env}` triggers
          // — amsmath defines `\split` (not `\begin{split}`), so the raw single-CS
          // token would look undefined after clearing. Tokenizing expands into
          // `\begin` + `{env}` which the standard `\begin{}` dispatcher resolves.
          latexml_core::state::assign_meaning(
            &cs_clone, latexml_core::common::store::Stored::None,
            Some(Scope::Global));
          require_package(&pkg_str, RequireOptions::default())?;
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
