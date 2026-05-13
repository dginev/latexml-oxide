//! amsppt.sty — AMSTeX plain TeX compatibility
//! Perl: amsppt.sty.ltxml — 500 lines
//! Document class for AMSTeX-style plain TeX documents.
//! Provides frontmatter, theorem environments, bibliography.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // amsppt loads the AmSTeX pool — Perl L27.
  // The `\documentstyle{amsppt}` path in tex_job.rs already takes care
  // of LoadPool('AmSTeX'). The `\input amsppt.sty` direct-load path
  // does NOT — papers like arXiv:hep-th9312119 start with `\input
  // amsppt.sty` and hit undefined `\document`, `\newline`, `\flushpar`
  // etc. that the AmSTeX pool would have provided. Load the pool
  // explicitly here so both entry paths produce the same definition
  // set.
  if !lookup_bool("AmSTeX.pool_loaded") {
    let _ = input_definitions("AmSTeX", InputDefinitionOptions {
      extension: Some(Cow::Borrowed("pool")),
      ..InputDefinitionOptions::default()
    });
  }
  // AmSTeX pool is partially ported (~30%); residual undefined-CS
  // errors on amsppt papers (\text, \proof, \theorem env, \endmatrix,
  // \foldedtext, \eightbf, \AmSTeX, \DN@, \frills@, etc — see
  // LaTeXML/lib/LaTeXML/Engine/AmSTeX.pool.ltxml) trace to the
  // missing pool. Tested RequirePackage("amsmath") — covers \text but
  // exposes more AmS-TeX-specific CSes on math0111087 (54 undef →
  // worse-looking in absolute count). Proper fix is a real port of
  // AmSTeX.pool.ltxml; quick amsmath shim deferred.

  // Frontmatter — Perl L32-80. Original (pre-LaTeX) AMSPPT syntax uses
  // `\title Foo \endtitle` (tokens terminated by `\endtitle`, not a
  // `{…}` group). Prior Rust stubbed these as naked DefMacro expanding
  // to `\@add@frontmatter{ltx:X}`, which only works when the user writes
  // `\title{Foo}` (LaTeX-ish form). Switch to the `Until:\endX` delimiter
  // form the Perl port uses, with the `\endX` `Let`-ed to `\relax`.
  DefMacro!("\\makeheadline", "");
  DefMacro!("\\makefootline", "");

  // LaTeX2e typesetting commands users sometimes mix into AmS-TeX
  // sources (the AmSTeX.pool path doesn't load latex_constructs). Stub
  // as no-ops — vertical/horizontal spacing and font-size selection
  // have no XML meaning. Perl LaTeXML Fatals on these undefined CSes;
  // we emit a clean document by absorbing the argument (where one
  // exists) or treating the CS as a benign no-op.
  // Witnesses (stage-4 of 100k warning corpus):
  //   arXiv:funct-an9211012/13 — \\vspace{1\\jot} inside aligned equations
  //   arXiv:funct-an9312004    — \\scriptsize font-size
  DefMacro!("\\vspace OptionalMatch:* {}", None);
  DefMacro!("\\hspace OptionalMatch:* {}", None);
  // LaTeX2e font-size and font-family CSes (no-op in AMSTeX mode).
  for sz in ["\\tiny", "\\scriptsize", "\\footnotesize", "\\small",
             "\\normalsize", "\\large", "\\Large", "\\LARGE",
             "\\huge", "\\Huge",
             // Font-family selectors (NFSS; absent in AmSTeX)
             "\\normalfont", "\\rmfamily", "\\sffamily", "\\ttfamily",
             "\\mdseries", "\\bfseries", "\\upshape", "\\itshape",
             "\\slshape", "\\scshape"] {
    if !state::has_meaning(&T_CS!(sz)) {
      def_macro(T_CS!(sz), None, None, None)?;
    }
  }
  DefMacro!("\\title Until:\\endtitle", "\\@add@frontmatter{ltx:title}{#1}");
  Let!("\\endtitle", "\\relax");
  DefMacro!("\\author Until:\\endauthor",
    "\\@add@frontmatter{ltx:creator}[role=author]{\\@personname{#1}}");
  Let!("\\endauthor", "\\relax");

  // Affiliations and contacts — Perl L85-130
  DefConstructor!("\\@@@affil{}", "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  DefMacro!("\\affil Until:\\endaffil",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@affil{#1}}");
  Let!("\\endaffil", "\\relax");
  DefConstructor!("\\@@@address{}", "^ <ltx:contact role='address'>#1</ltx:contact>");
  DefMacro!("\\address Until:\\endaddress",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@address{#1}}");
  Let!("\\endaddress", "\\relax");
  DefConstructor!("\\@@@curraddr{}", "^ <ltx:contact role='current_address'>#1</ltx:contact>");
  DefMacro!("\\curraddr Until:\\endcurraddr",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@curraddr{#1}}");
  Let!("\\endcurraddr", "\\relax");
  DefConstructor!("\\@@@email{}", "^ <ltx:contact role='email'>#1</ltx:contact>");
  DefMacro!("\\email Until:\\endemail",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}}");
  Let!("\\endemail", "\\relax");
  DefConstructor!("\\@@@urladdr{}", "^ <ltx:contact role='url'>#1</ltx:contact>");
  DefMacro!("\\urladdr Until:\\endurladdr",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@urladdr{#1}}");
  Let!("\\endurladdr", "\\relax");

  // Perl amsppt.sty.ltxml L72-75: thanks/date/dedicatory/translator —
  // previously absent in Rust.
  DefMacro!("\\thanks Until:\\endthanks",
    "\\@add@frontmatter{ltx:note}[role=support]{#1}");
  Let!("\\endthanks", "\\relax");
  DefMacro!("\\date Until:\\enddate",
    "\\@add@frontmatter{ltx:date}[role=creation]{#1}");
  Let!("\\enddate", "\\relax");
  DefMacro!("\\dedicatory Until:\\enddedicatory",
    "\\@add@frontmatter{ltx:note}[role=dedicatory]{#1}");
  Let!("\\enddedicatory", "\\relax");
  DefMacro!("\\translator Until:\\endtranslator",
    "\\@add@frontmatter{ltx:creator}[role=translator]{\\@personname{#1}}");
  Let!("\\endtranslator", "\\relax");

  // Abstract and classification — Perl L76-79.
  DefMacro!("\\keywords Until:\\endkeywords",
    "\\@add@frontmatter{ltx:keywords}{#1}");
  Let!("\\endkeywords", "\\relax");
  DefMacro!("\\subjclass Until:\\endsubjclass",
    "\\@add@frontmatter{ltx:classification}[scheme=MSC]{#1}");
  Let!("\\endsubjclass", "\\relax");
  DefMacro!("\\abstract Until:\\endabstract",
    "\\@add@frontmatter{ltx:abstract}{#1}");
  Let!("\\endabstract", "\\relax");

  // Section structure — Perl L112-147. AmSTeX uses terminator-delimited
  // syntax (`\head Foo \endhead`) not balanced `\section{Foo}`.
  // We use DefConstructors matching Perl to avoid relying on LaTeX's
  // `\section*` which is undefined in pure AmSTeX contexts.
  NewCounter!("chapter", "document", idprefix => "C", nested => vec!["section"]);
  NewCounter!("section", "chapter", idprefix => "S", nested => vec!["subsection"]);
  NewCounter!("subsection", "section", idprefix => "SS", nested => vec!["subsubsection"]);
  NewCounter!("subsubsection", "subsection", idprefix => "SSS", nested => vec!["paragraph"]);

  DefConstructor!("\\head Until:\\endhead",
    "<ltx:section inlist='toc' xml:id='#id'><ltx:title>#1</ltx:title></ltx:section>",
    properties => sub[_args] { RefStepID!("section") });
  Let!("\\endhead", "\\relax");

  DefMacro!("\\heading Until:\\endheading", "\\head#1\\endhead");
  Let!("\\endheading", "\\relax");

  // Perl amsppt.sty.ltxml L133-141: \subheading dispatches on next token.
  DefMacro!("\\subheading", sub[_args] {
    let next = gullet::read_token()?;
    if let Some(t) = next {
      gullet::unread(Tokens!(t));
      if t.get_catcode() == Catcode::BEGIN {
        return Ok(Tokens!(T_CS!("\\subheading@onearg")));
      }
    }
    Ok(Tokens!(T_CS!("\\subheading@env")))
  }, locked => true);

  DefMacro!("\\subheading@onearg{}", "\\subhead#1\\endsubhead");
  DefMacro!("\\subheading@env Until:\\endsubheading", "\\subhead#1\\endsubhead");
  DefMacro!("\\endsubheading", "");

  DefConstructor!("\\specialhead Until:\\endspecialhead",
    "<ltx:chapter inlist='toc' xml:id='#id'><ltx:title>#1</ltx:title></ltx:chapter>",
    properties => sub[_args] { RefStepID!("chapter") });
  Let!("\\endspecialhead", "\\relax");

  DefConstructor!("\\subhead Until:\\endsubhead",
    "<ltx:subsection inlist='toc' xml:id='#id'><ltx:title>#1</ltx:title></ltx:subsection>",
    properties => sub[_args] { RefStepID!("subsection") });
  Let!("\\endsubhead", "\\relax");

  DefConstructor!("\\subsubhead Until:\\endsubsubhead",
    "<ltx:subsubsection inlist='toc' xml:id='#id'><ltx:title>#1</ltx:title></ltx:subsubsection>",
    properties => sub[_args] { RefStepID!("subsubsection") });
  Let!("\\endsubsubhead", "\\relax");

  // Theorem environments — Perl L170-243 use
  //   DefConstructor('\<kind> Undigested DigestUntil:\end<kind>', …)
  // The Rust port now follows that pattern (with title font/counter
  // glue intentionally simplified — see #44 amsthm-counter aliasing).
  // Earlier Rust versions routed through `\begin{theorem}` macros,
  // which leaked an internal_vertical mode frame at \enddocument
  // time when the body content closed paragraphs naturally. Reverting
  // to the Perl-faithful DefConstructor approach avoids that — the
  // body digests in surrounding mode without pushing a new frame.
  // Perl-faithful port of L170-243 — DefConstructor with `Undigested
  // DigestUntil:\end<kind>` so the body digests in surrounding mode
  // without pushing/popping a mode frame (which the previous
  // `\begin{theorem}` macro routing did, and which leaked at
  // \enddocument time when the body content closed paragraphs
  // naturally — see amsppt mode-leak repro `/tmp/min_amstex.tex`,
  // commit `aa3304142`'s residual). The Perl `properties => RefStepID`
  // and per-kind counter glue is omitted; ltx:theorem auto_close
  // (already registered above) handles the closing.
  DefConstructor!("\\proclaim Undigested DigestUntil:\\endproclaim",
    "<ltx:theorem class='ltx_theorem_proclaim'><ltx:title>#1</ltx:title>#2",
    after_construct => sub[doc,_a] { doc.maybe_close_element("ltx:theorem")?; });
  Let!("\\endproclaim", "\\relax");

  DefConstructor!("\\definition Undigested DigestUntil:\\enddefinition",
    "<ltx:theorem class='ltx_theorem_definition'><ltx:title>#1</ltx:title>#2",
    after_construct => sub[doc,_a] { doc.maybe_close_element("ltx:theorem")?; });
  Let!("\\enddefinition", "\\relax");

  DefConstructor!("\\remark Undigested DigestUntil:\\endremark",
    "<ltx:theorem class='ltx_theorem_remark'><ltx:title>#1</ltx:title>#2",
    after_construct => sub[doc,_a] { doc.maybe_close_element("ltx:theorem")?; });
  Let!("\\endremark", "\\relax");

  DefConstructor!("\\example Undigested DigestUntil:\\endexample",
    "<ltx:theorem class='ltx_theorem_example'><ltx:title>#1</ltx:title>#2",
    after_construct => sub[doc,_a] { doc.maybe_close_element("ltx:theorem")?; });
  Let!("\\endexample", "\\relax");

  DefConstructor!("\\demo Undigested DigestUntil:\\enddemo",
    "<ltx:theorem class='ltx_theorem_demonstration'><ltx:title>#1</ltx:title>#2",
    after_construct => sub[doc,_a] { doc.maybe_close_element("ltx:theorem")?; });
  Let!("\\enddemo", "\\relax");

  // Lists — Perl amsppt.sty.ltxml L245-259. Faithful Perl port replaces
  // the prior thin `\begin{enumerate}` wrapper, which left a mode-switch
  // frame on the stack at `\endroster` time and cascaded `\endgroup`
  // errors at every subsequent `\endref`/`\end` (math0104021 +
  // ~similar AmS-TeX papers using \roster). DigestUntil reads the body
  // in one shot, bounded=>true keeps the entire frame self-contained.
  NewCounter!("roster", "document", idprefix => "I");
  NewCounter!("rosteritem", "roster", idprefix => "i");
  DefMacro!("\\therosteritem{}", "\\rom{(#1)}");
  DefConstructor!("\\roster DigestUntil:\\endroster",
    "<ltx:enumerate>#body</ltx:enumerate>",
    bounded => true,
    properties => sub[_args] { RefStepID!("roster") },
    before_digest => { state::let_i(&T_CS!("\\item"), &T_CS!("\\roster@item"), Some(state::Scope::Local)); });
  DefConstructor!("\\roster@item",
    "<ltx:item xml:id='#id'>?#1(<ltx:tags><ltx:tag>#1</ltx:tag></ltx:tags>)",
    properties => sub[_args] { RefStepID!("rosteritem") });
  Let!("\\endroster", "\\relax");

  // Perl amsppt.sty.ltxml L261-263: \block — simple block-quote container.
  // Previously unported. DigestUntil parameter type landed in 27cc66b60
  // makes this a direct translation.
  DefConstructor!(
    "\\block DigestUntil:\\endblock",
    "<ltx:quote>#1</ltx:quote>"
  );
  Let!(T_CS!("\\endblock"), T_CS!("\\relax"));

  // Footnotes — Perl L305-350
  // Perl amsppt L276 is `DefConstructor('\footnote', <ltx:note role='footnote'>)`
  // — a direct constructor. Rust delegates to `\lx@note{footnote}` (a
  // helper in `latex_constructs.rs` that already carries the same
  // ltx:note wrapper + role attr). Intentional DefConstructor →
  // DefMacro kind divergence via delegation (WISDOM #44).
  // Perl amsppt.sty.ltxml L272-304 — footnote infrastructure. AmS-TeX flow
  // (via `\input amstex \documentstyle{amsppt}`) does NOT load
  // latex_constructs.rs's `\lx@note*` helpers, so naive delegation fails.
  // We mirror Perl L272-304 directly: NewCounter sets up `\c@footnote` +
  // default `\thefootnote`; `\footnote`/`\footnotemark`/`\footnotetext`
  // are self-contained DefConstructors. When latex_constructs IS loaded
  // (e.g. via `\usepackage{amsppt}` after a LaTeX `\documentclass`), these
  // override the locked latex_constructs versions with amsppt's own
  // definitions — same as Perl's behavior in that flow.
  NewCounter!("footnote");
  // Perl L273 redefines \thefootnote as a sub reading \c@footnote directly;
  // NewCounter already gives us the same behavior via `\arabic{footnote}`.
  // The closure form would only differ in tokenization corner cases not
  // exercised by amsppt.
  DefConstructor!("\\footnote[]{}",
    "<ltx:note role='footnote' mark='#mark' xml:id='#id'>#2</ltx:note>",
    mode => "internal_vertical",
    before_digest => { neutralize_font(); },
    properties => sub [args] {
      let mark: Stored = match args[0].as_ref() {
        Some(m) => m.clone().into(),
        None => {
          ref_step_counter("footnote", false)?;
          digest_text(Tokens!(T_CS!("\\thefootnote")))?.into()
        },
      };
      let mut props = SymHashMap::default();
      props.insert("mark", mark);
      Ok(props)
    });
  DefConstructor!("\\footnotemark[]",
    "<ltx:note role='footnotemark' mark='#mark' xml:id='#id'/>",
    mode => "text", enter_horizontal => true,
    properties => sub [args] {
      let mark: Stored = match args[0].as_ref() {
        Some(m) => m.clone().into(),
        None => {
          ref_step_counter("footnote", false)?;
          digest_text(Tokens!(T_CS!("\\thefootnote")))?.into()
        },
      };
      let mut props = SymHashMap::default();
      props.insert("mark", mark);
      Ok(props)
    });
  // Perl L298-304: \footnotetext does NOT step the counter; reads
  // current \thefootnote value directly.
  DefConstructor!("\\footnotetext[]{}",
    "<ltx:note role='footnotetext' mark='#mark' xml:id='#id'>#2</ltx:note>",
    mode => "internal_vertical",
    properties => sub [args] {
      let mark: Stored = match args[0].as_ref() {
        Some(m) => m.clone().into(),
        None => digest_text(Tokens!(T_CS!("\\thefootnote")))?.into(),
      };
      let mut props = SymHashMap::default();
      props.insert("mark", mark);
      Ok(props)
    });
  // Perl L306: same Tag-afterClose hook as latex_constructs.rs:3195
  Tag!("ltx:note", after_close => sub[doc, node] { relocate_footnote(doc, node)?; });

  // Bibliography — Perl L355-500
  // Complex Perl closure system for reference formatting
  // Perl L359, L456: amsppt extends ltx:biblist + ltx:bibblock with
  // autoOpen so a bibitem child without an explicit \begin{biblist}
  // wrapper still nests correctly. Core latex_constructs already sets
  // auto_close on biblist/bibblock; here we add auto_open on top to
  // reach amsppt's documented spec.
  Tag!("ltx:biblist",  auto_open => true, auto_close => true);
  Tag!("ltx:bibblock", auto_open => true, auto_close => true);

  // Perl amsppt.sty.ltxml L167-168, L358: auto_close on theorem/proof/
  // bibliography so AmSTeX's implicit structure (no explicit \end) still
  // closes on the next block-level open. AmSTeX documents rely on this
  // because the top-level CSes like \proclaim and \demo don't pair with
  // \end markers. Without these, a `\proclaim` followed by a top-level
  // paragraph would try to nest the paragraph inside the theorem.
  Tag!("ltx:theorem",      auto_close => true);
  Tag!("ltx:proof",        auto_close => true);
  Tag!("ltx:bibliography", auto_close => true);
  // Perl L306 also registers `Tag('ltx:note', afterClose => \&relocate
  // Footnote)` — the closure walks the node tree to re-parent stray
  // footnotes onto their originating paragraph. Deferred: requires the
  // full relocateFootnote infra. No amsppt test in the suite, so leaving
  // note-handling unported is acceptable for now.

  // Perl amsppt.sty.ltxml L457-458: token-valued \holdoverbox register
  // and 1-arg \holdover{#1} no-op. AmSTeX bib entries use \holdover{...}
  // to bounce a partial field to the next entry; the token register
  // accumulates the held tokens across the bib-block. Rust doesn't run
  // the accumulation (stubbed), but both CSes must still resolve so
  // bibliographies using \holdover don't hit undefined-CS.
  DefRegister!("\\holdoverbox" => Tokens!());
  DefMacro!("\\holdover{}", "");

  // Bibliography — full Perl-faithful port of amsppt.sty.ltxml L340-495.
  //
  // Architecture: AmS-TeX bibliography uses field-marker macros
  // (`\key`, `\by`, `\paper`, `\jour`, etc.) that each terminate the
  // previous field via `\@end@bibfield` and start the next via
  // `\@bibfield{<name>}`. `\@bibfield` is an `XUntil:\@end@bibfield`
  // macro that captures the trailing tokens into state under
  // `amsbibitem@<field>`. After all fields are read, `\endref` calls
  // `\@fill@bibitem` which reads the captured fields back from state
  // and emits formatted output via `\@bibitem@field` and `\@bibitem@tag`
  // constructors.
  //
  // The `\@auto@Refs` indirection lets bare `\ref … \endref` work
  // outside an explicit `\Refs … \endRefs` wrapper by auto-opening the
  // bibliography on first invocation, then redefining `\@auto@Refs`
  // empty so subsequent `\ref`s don't re-open.
  NewCounter!("@bibitem", "bibliography", idprefix => "bib");
  DefMacro!("\\the@bibitem", "\\number\\c@@bibitem");
  // Perl L338: id resolves to <docid>.bib if document has an id,
  // else just "bib".
  DefMacro!("\\the@lx@bibliography@ID",
    "\\ifx.\\thedocument@id.\\thedocument@id.bib\\else bib\\fi");

  // Perl L340-348: \@auto@Refs / \Refs / \endRefs.
  DefMacro!("\\@auto@Refs", "\\Refs");
  DefMacro!("\\Refs", "\\@Refs\\def\\@auto@Refs{}");
  DefMacro!("\\endRefs", "\\def\\@auto@Refs{\\Refs}\\end@Refs");
  DefConstructor!("\\@Refs",
    "<ltx:bibliography xml:id='#id'>\
       <ltx:title>References</ltx:title>\
       <ltx:biblist>",
    properties => sub[_args] {
      let id_str = digest_text(Tokens!(T_CS!("\\the@lx@bibliography@ID")))?
        .to_string();
      let mut props = SymHashMap::default();
      props.insert("id", Stored::String(arena::pin(&id_str)));
      Ok(props)
    });
  DefConstructor!("\\end@Refs", "</ltx:bibliography>");

  // Perl L375-379: `\@bibitem` / `\@bibblock` constructors. RefStepID
  // assigns `xml:id` from the @bibitem counter.
  DefConstructor!("\\@bibitem", "<ltx:bibitem xml:id='#id'>",
    properties => sub[_args] { ref_step_id("@bibitem") });
  DefConstructor!("\\@end@bibitem", "</ltx:bibitem>");
  DefConstructor!("\\@bibblock", "<ltx:bibblock>");
  DefConstructor!("\\@end@bibblock", "</ltx:bibblock>");

  // Perl L446-453: `\@bibfield{type} XUntil:\@end@bibfield` — captures
  // the trailing tokens into `amsbibitem@<field>` state, after trimming
  // leading/trailing T_SPACE.
  DefMacro!("\\@bibfield{} XUntil:\\@end@bibfield", sub[args] {
    let field = args[0].clone().into_tokens_result()?.to_string();
    let tokens = args[1].clone().into_tokens_result()?;
    let mut tk_vec: Vec<Token> = tokens.unlist();
    while tk_vec.first().map(|t| t.get_catcode() == Catcode::SPACE).unwrap_or(false) {
      tk_vec.remove(0);
    }
    while tk_vec.last().map(|t| t.get_catcode() == Catcode::SPACE).unwrap_or(false) {
      tk_vec.pop();
    }
    if !tk_vec.is_empty() {
      let key = format!("amsbibitem@{}", field);
      state::assign_value(&key, Stored::Tokens(Tokens::new(tk_vec)), None);
    }
    Ok(Tokens!())
  });
  Let!("\\@end@bibfield", "\\relax");

  // Perl L442-445: \@bibitem@field / \@bibitem@tag constructors.
  DefConstructor!("\\@bibitem@field{}{}",
    "<ltx:text class='#class' _noautoclose='1'>#2</ltx:text>",
    properties => sub [args] {
      let field_name = args[0].as_ref().map(|d| d.to_string()).unwrap_or_default();
      let class_str = format!("ltx_bib_{}", field_name);
      let mut props = SymHashMap::default();
      props.insert("class", Stored::String(arena::pin(&class_str)));
      Ok(props)
    });
  // Perl L445.
  DefConstructor!("\\@bibitem@tag{}",
    "^key='#1'<ltx:tags><ltx:tag role='refnum'>#1</ltx:tag></ltx:tags>");

  // Perl L381-424: `\@fill@bibitem` — emits the formatted bib entry by
  // reading captured `amsbibitem@*` fields from state and building
  // Invocation tokens for `\@bibitem@tag` and `\@bibitem@field` per the
  // case dispatch (book / paper-in-book / paper-in-journal / random).
  DefMacro!("\\@fill@bibitem", sub[_args] {
    let mut body: Vec<Token> = Vec::new();

    fn lookup_field(field: &str) -> Option<Tokens> {
      match state::lookup_value(&format!("amsbibitem@{}", field)) {
        Some(Stored::Tokens(t)) => Some(t),
        _ => None,
      }
    }

    // Append `\@bibitem@field{<field>}{<value>}` invocation.
    fn push_field(body: &mut Vec<Token>, field: &str, value: Tokens) {
      body.push(T_CS!("\\@bibitem@field"));
      body.push(T_BEGIN!());
      for ch in field.chars() { body.push(T_OTHER!(ch.to_string())); }
      body.push(T_END!());
      body.push(T_BEGIN!());
      body.extend(value.unlist());
      body.push(T_END!());
    }

    // Perl `ppunbox` — emit `[punct][pre]<field>[post]` if non-empty.
    fn pp(body: &mut Vec<Token>,
          punct: Option<&str>, pre: Option<Tokens>,
          field: &str, post: Option<Tokens>) {
      if let Some(value) = lookup_field(field) {
        if let Some(p) = punct {
          for ch in p.chars() { body.push(T_OTHER!(ch.to_string())); }
        }
        if let Some(pre_tk) = pre {
          body.extend(pre_tk.unlist());
        } else {
          body.push(T_SPACE!());
        }
        push_field(body, field, value);
        if let Some(post_tk) = post {
          body.extend(post_tk.unlist());
        }
      }
    }
    // Perl `commaunbox`: `ppunbox(',', '\space', <field>, undef)`.
    fn comma(body: &mut Vec<Token>, field: &str) {
      pp(body, Some(","), Some(Tokens!(T_CS!("\\space"))), field, None);
    }

    // Tag at start (skip if this is a `\moreref` continuation).
    let is_moreref = lookup_field("moreref").is_some();
    if !is_moreref {
      let tag_value = lookup_field("key")
        .or_else(|| lookup_field("refnum"))
        .unwrap_or_else(|| Tokens!(T_CS!("\\the@bibitem")));
      body.push(T_CS!("\\@bibitem@tag"));
      body.push(T_BEGIN!());
      body.extend(tag_value.unlist());
      body.push(T_END!());
    }
    // Authors / editors fallback (Perl L387-389).
    if lookup_field("authors").is_none() {
      if let Some(eds) = lookup_field("editors") {
        let mut combined: Vec<Token> = eds.unlist();
        // `\space(\edtext)` suffix.
        combined.push(T_CS!("\\space"));
        combined.push(T_OTHER!("("));
        combined.push(T_CS!("\\edtext"));
        combined.push(T_OTHER!(")"));
        state::assign_value("amsbibitem@authors",
          Stored::Tokens(Tokens::new(combined)), None);
      }
    }
    pp(&mut body, None, None, "authors", None);

    if lookup_field("book").is_some() {
      // Case 1: Book.
      comma(&mut body, "book");
      comma(&mut body, "bookinfo");
      pp(&mut body, None, Some(Tokens!(T_CS!("\\space"), T_OTHER!("("))),
         "proceedingsinfo", Some(Tokens!(T_OTHER!(")"))));
      pp(&mut body, Some(","), Some(Tokens!(T_CS!("\\space"),
        T_OTHER!("v"), T_OTHER!("o"), T_OTHER!("l"), T_OTHER!("."), T_OTHER!("~"))),
         "volume", None);
      pp(&mut body, None, Some(Tokens!(T_CS!("\\space"), T_OTHER!("("))),
         "editors",
         Some(Tokens!(T_OTHER!(","), T_CS!("\\space"), T_CS!("\\edtext"), T_OTHER!(")"))));
      comma(&mut body, "publisher");
      comma(&mut body, "publisheraddr");
      comma(&mut body, "year");
      pp(&mut body, Some(","),
         Some(Tokens!(T_CS!("\\space"), T_CS!("\\pagestext"), T_OTHER!("~"))),
         "pages", None);
    } else {
      // Case 2: Paper.
      comma(&mut body, "paper");
      comma(&mut body, "paperinfo");
      if lookup_field("inbook").is_some() {
        // Case 2a: Paper in book.
        comma(&mut body, "inbook");
        pp(&mut body, None, Some(Tokens!(T_CS!("\\space"), T_OTHER!("("))),
           "proceedingsinfo", Some(Tokens!(T_OTHER!(")"))));
        pp(&mut body, None, Some(Tokens!(T_CS!("\\space"), T_OTHER!("("))),
           "editors",
           Some(Tokens!(T_OTHER!(","), T_CS!("\\space"), T_CS!("\\edtext"), T_OTHER!(")"))));
        comma(&mut body, "bookinfo");
        pp(&mut body, Some(","),
           Some(Tokens!(T_CS!("\\space"), T_CS!("\\voltext"), T_OTHER!("~"))),
           "volume", None);
        comma(&mut body, "publisher");
        comma(&mut body, "publisheraddr");
        comma(&mut body, "year");
        pp(&mut body, Some(","),
           Some(Tokens!(T_CS!("\\space"), T_CS!("\\pagestext"), T_OTHER!("~"))),
           "pages", None);
      } else if lookup_field("random").is_none() {
        // Case 2b: Paper in journal.
        comma(&mut body, "journal");
        pp(&mut body, None, Some(Tokens!(T_CS!("\\space"))), "volume", None);
        pp(&mut body, None, Some(Tokens!(T_CS!("\\space"), T_OTHER!("("))),
           "year", Some(Tokens!(T_OTHER!(")"))));
        pp(&mut body, Some(","),
           Some(Tokens!(T_CS!("\\space"), T_CS!("\\issuetext"), T_OTHER!("~"))),
           "issue", None);
        comma(&mut body, "publisher");
        comma(&mut body, "publisheraddr");
        comma(&mut body, "pages");
      } else {
        // Case 2c: Random text leftover.
        comma(&mut body, "random");
      }
    }
    comma(&mut body, "finalinfo");
    pp(&mut body, None, Some(Tokens!(T_CS!("\\space"), T_OTHER!("("))),
       "note", Some(Tokens!(T_OTHER!(")"))));
    body.push(T_OTHER!("."));
    pp(&mut body, None, Some(Tokens!(T_CS!("\\space"), T_OTHER!("("))),
       "language", Some(Tokens!(T_OTHER!(")"))));
    comma(&mut body, "mathreview");

    // Clear all amsbibitem@* fields for next entry. Perl relies on
    // `\begingroup`/`\endgroup` to local-scope them; Rust's `assign_value`
    // with `None` scope follows the active group, so the same scoping
    // applies. Explicit clear isn't strictly needed but keeps state
    // tidy across `\moreref` chains.
    Ok(Tokens::new(body))
  });

  // Perl L367-373: `\ref` / `\endref` / `\moreref` orchestrate
  // capturing-then-emitting a bibitem.
  DefMacro!("\\ref",
    "\\@auto@Refs\\begingroup\\@bibitem\\@bibfield{random}");
  DefMacro!("\\endref",
    "\\@end@bibfield\\@fill@bibitem\\@end@bibitem\\endgroup");
  DefMacro!("\\moreref",
    "\\@end@bibfield\\@fill@bibitem\\@end@bibblock\\endgroup\
     \\begingroup\\@bibblock\\@bibfield{moreref}nonempty\\@end@bibfield\
     \\@bibfield{random}");

  // Perl L460-493: field-marker macros — each terminates the current
  // field and opens the next. The trailing `\it`/`\bf` for paper/book/
  // vol gets prepended to the captured tokens (XUntil reads through it).
  DefMacro!("\\key",       "\\@end@bibfield\\@bibfield{key}");
  DefMacro!("\\no",        "\\@end@bibfield\\@bibfield{refnum}");
  DefMacro!("\\by",        "\\@end@bibfield\\@bibfield{authors}");
  DefMacro!("\\bysame",    "\\by  ---");
  Let!("\\manyby", "\\by");
  DefMacro!("\\ed",        "\\@end@bibfield\\@bibfield{editors}");
  DefMacro!("\\eds",       "\\@end@bibfield\\@bibfield{editors}");
  DefMacro!("\\paper",     "\\@end@bibfield\\@bibfield{paper}\\it");
  DefMacro!("\\paperinfo", "\\@end@bibfield\\@bibfield{paperinfo}");
  DefMacro!("\\inbook",    "\\@end@bibfield\\@bibfield{inbook}");
  DefMacro!("\\book",      "\\@end@bibfield\\@bibfield{book}\\it");
  DefMacro!("\\bookinfo",  "\\@end@bibfield\\@bibfield{bookinfo}");
  DefMacro!("\\procinfo",  "\\@end@bibfield\\@bibfield{proceedingsinfo}");
  DefMacro!("\\finalinfo", "\\@end@bibfield\\@bibfield{finalinfo}");
  DefMacro!("\\jour",      "\\@end@bibfield\\@bibfield{journal}");
  DefMacro!("\\vol",       "\\@end@bibfield\\@bibfield{volume}\\bf");
  DefMacro!("\\voltext",   "vol.");
  DefMacro!("\\issue",     "\\@end@bibfield\\@bibfield{issue}");
  DefMacro!("\\issuetext", "no.");
  DefMacro!("\\yr",        "\\@end@bibfield\\@bibfield{year}");
  DefMacro!("\\page",      "\\@end@bibfield\\@bibfield{pages}");
  DefMacro!("\\pages",     "\\@end@bibfield\\@bibfield{pages}");
  DefMacro!("\\pagestext", "pp.");
  DefMacro!("\\lang",      "\\@end@bibfield\\@bibfield{language}");
  DefMacro!("\\publ",      "\\@end@bibfield\\@bibfield{publisher}");
  DefMacro!("\\publaddr",  "\\@end@bibfield\\@bibfield{publisheraddress}");
  DefMacro!("\\miscnote",  "\\@end@bibfield\\@bibfield{note}");
  DefMacro!("\\toappear",  "\\miscnote to appear");
  DefMacro!("\\MR",        "\\@end@bibfield\\@bibfield{mathreview}MR ");
  DefMacro!("\\AMSPPS",    "\\@end@bibfield\\@bibfield{ams-preprint}AMS-PPS ");
  DefMacro!("\\CMP",       "\\@end@bibfield\\@bibfield{CMP}CMP ");

  // Miscellaneous — Perl L480-500
  DefMacro!("\\nologo", "");
  DefMacro!("\\NoBlackBoxes", "");

  // AmSTeX pool compatibility stubs — Perl AmSTeX.pool.ltxml L75-114.
  // amsppt.sty (Perl) implicitly loads the AmSTeX pool, which provides
  // these as author-ignorable no-ops. Rust doesn't port AmSTeX pool
  // (~30% ported per L10 comment), so documents using bare amsppt risk
  // undefined-CS on these formatting controls. Adding as empty stubs
  // keeps documents compile without altering XML output.
  DefMacro!("\\NoPageNumbers", "");
  DefMacro!("\\BlackBoxes", "");
  DefMacro!("\\TagsAsMath", "");
  DefMacro!("\\TagsAsText", "");
  DefMacro!("\\TagsOnLeft", "");
  DefMacro!("\\TagsOnRight", "");
  DefMacro!("\\CenteredTagsOnSplits", "");
  DefMacro!("\\TopOrBottomTagsOnSplits", "");
  DefMacro!("\\LimitsOnInts", "");
  DefMacro!("\\NoLimitsOnInts", "");
  DefMacro!("\\LimitsOnNames", "");
  DefMacro!("\\NoLimitsOnNames", "");
  DefMacro!("\\LimitsOnSums", "");
  DefMacro!("\\NoLimitsOnSums", "");
  DefMacro!("\\UseAMSsymbols", "");
  DefMacro!("\\loadbold", "");
  DefMacro!("\\loadeufb", "");
  DefMacro!("\\loadeufm", "");
  DefMacro!("\\loadeurb", "");
  DefMacro!("\\loadeurm", "");
  DefMacro!("\\loadeusb", "");
  DefMacro!("\\loadeusm", "");
  DefMacro!("\\loadmathfont", "");
  DefMacro!("\\loadmsam", "");
  DefMacro!("\\loadmsbm", "");
  DefMacro!("\\boldnotloaded{}", "");
  DefMacro!("\\galleys", "");
  // Perl AmSTeX.pool L114: \flushpar = \par\noindent
  DefMacro!("\\flushpar", "\\par\\noindent");

  // Page-layout no-ops — Perl AmSTeX.pool L116-119.
  DefMacro!("\\pagewidth{Dimension}", "");
  DefMacro!("\\pageheight{Dimension}", "");
  DefMacro!("\\hcorrection{Dimension}", "");
  DefMacro!("\\vcorrection{Dimension}", "");

  // Perl L186: \tie = \unskip\nobreak\␣ (non-breaking space with
  // preceding skip-absorption).
  DefMacro!("\\tie", "\\unskip\\nobreak\\ ");

  // Perl L299-300: math superscript accents via manual ^{...}.
  // Siblings \spcheck/\sptilde are already in Rust plain_base.
  DefMacro!("\\spacute", "^{'}");
  DefMacro!("\\spgrave", "^{`}");

  // Perl AmSTeX.pool L133-134: frontmatter bracket markers.
  // Rust amsppt handles frontmatter via \title/\author/\abstract
  // directly, so the outer bracket is a no-op.
  DefMacro!("\\topmatter", "");
  DefMacro!("\\endtopmatter", "");

  // Perl L256-257: set-braces via \overbrace/\underbrace with the
  // "label" part from before `\to` as superscript/subscript.
  DefMacro!("\\oversetbrace Until:\\to {}",  "\\overbrace{#2}^{#1}");
  DefMacro!("\\undersetbrace Until:\\to {}", "\\underbrace{#2}^{#1}");

  // Perl L289-295: \thickfrac / \thickfracwithdelims. Perl peeks
  // for a following `\thickness` keyword to dispatch between the
  // `\@thickfrac` and `\frac` forms. Rust doesn't implement the
  // `\thickness`-peek dispatch, so route directly to \frac (the
  // no-thickness variant) — the most common case. Same for the
  // delims variant.
  DefMacro!("\\thickfrac", "\\frac");
  DefMacro!("\\thickfracwithdelims{}{}", "\\fracwithdelims{#1}{#2}");
  DefMacro!("\\@thickfrac Token Number {}{}", "\\genfrac{}{}{#2}{}{#3}{#4}");
  DefMacro!("\\@thickfracwithdelims {}{} Token Number {}{}",
    "\\genfrac{#1}{#2}{#4}{}{#5}{#6}");

  // Perl AmSTeX.pool L34: \AmSTeX — logo constructor; render as plain text.
  DefMacro!("\\AmSTeX", "AMSTeX");
  // Perl L175-184: page/line/math break hints — all empty (layout-only).
  DefMacro!("\\bigpagebreak", "");
  DefMacro!("\\allowlinebreak", "");
  DefMacro!("\\allowmathbreak", "");
  DefMacro!("\\allowdisplaybreak", "");
  DefMacro!("\\allowdisplaybreaks", "");
  // Perl L270-284: pass-through math-font wrappers. Perl uses
  // DefConstructor with `bounded => 1, requireMath => 1` to scope
  // the font change; Rust simplifies to the identity DefMacro since
  // the body already carries the math-mode context.
  DefMacro!("\\Cal{}", "#1");
  DefMacro!("\\italic{}", "#1");
  DefMacro!("\\boldkey{}", "#1");
  // Perl L395: \botaligned = \aligned[b] (bottom-vertically-aligned).
  DefMacro!("\\botaligned", "\\aligned[b]");

  // Perl L173-182: more layout-hint empty stubs.
  DefMacro!("\\smallpagebreak", "");
  DefMacro!("\\medpagebreak", "");
  DefMacro!("\\mathbreak", "");
  DefMacro!("\\nomathbreak", "");
  DefMacro!("\\nomultlinegap", "");
  DefMacro!("\\MultlineGap Dimension", "");

  // Perl L350-358: top/bot shave and smash — pass-through text wrappers
  // (Perl DefConstructor with enterHorizontal, flattened to DefMacro
  // identity since the wrapper is decorative).
  DefMacro!("\\botshave{}", "#1");
  DefMacro!("\\topshave{}", "#1");
  DefMacro!("\\topsmash{}", "#1");
  DefMacro!("\\botsmash{}", "#1");

  // Perl L354: \pretend Until:\haswidth {body} — body is #1 up through
  // \haswidth, then {width} follows. Drop the width spec, keep body.
  DefMacro!("\\pretend Until:\\haswidth {}", "#1");

  // Perl L303-308: spdddot / spddddot / spbar / spvec / spbreve —
  // math superscript accents (siblings of existing \spcheck/\sptilde/
  // \spacute/\spgrave). Complete the family.
  DefMacro!("\\spdddot", "^{...}");
  DefMacro!("\\spddddot", "^{....}");
  DefMacro!("\\spbar", "^{-}");
  DefMacro!("\\spvec", "^{\\rightarrow}");

  // Perl L348, L356, L393, L456-458: more empty stubs and aliases.
  DefMacro!("\\ResetBuffer", "");
  DefMacro!("\\snug", "");
  DefMacro!("\\printoptions", "");
  DefMacro!("\\showallocations", "");
  DefMacro!("\\syntax", "");
  // Perl L393: \topaligned = \aligned[t] (sibling of \botaligned).
  DefMacro!("\\topaligned", "\\aligned[t]");

  // Perl L164-166: \textfonti, \textfontii — plain-TeX font-switch
  // primitives, no LaTeXML-observable effect.
  DefMacro!("\\textfonti", "");
  DefMacro!("\\textfontii", "");

  // Perl L281-282: \slanted{#1} — math-font wrapper flattened to
  // identity (same rationale as \Cal/\italic/\boldkey).
  DefMacro!("\\slanted{}", "#1");

  // Perl L349: \shave{#1} → #1 (sibling of \botshave/\topshave).
  DefMacro!("\\shave{}", "#1");

  // (Perl amsppt.sty.ltxml L460-495 \@bibfield/\@end@bibfield/\key/\no/
  // \inbook/\procinfo/\MR/\AMSPPS/\CMP are now defined above in the
  // Perl-faithful bibliography port — no duplicate stubs needed here.)

  // Perl L169: \spreadlines {Dimension} — line-spacing dimension
  // consumer, no output (DefConstructor with empty emission).
  DefMacro!("\\spreadlines{}", "");
  // Perl L360: \spreadmatrixlines Dimension — same shape, Dimension
  // param.
  DefMacro!("\\spreadmatrixlines Dimension", "");


  DefMacro!("\\redefine", "\\def");
  DefMacro!("\\define", "\\def");

  // Identity / version metadata — Perl amsppt.sty.ltxml L29-35.
  DefMacro!("\\filename", "amsppt.sty");
  DefMacro!("\\fileversion", "2.1h");
  DefMacro!("\\filedate", "1997/02/02");
  DefMacro!("\\fileversiontest", "\\fileversion\\space(\\filedate)");
  DefMacro!("\\styname", "AMSPPT");
  DefMacro!("\\styversion", "\\fileversion");
  DefMacro!("\\plainend", "\\end");

  // Page-layout no-ops — Perl L40-52. Running-head tokens + page-contents
  // are TeX plain-format hooks with no LaTeXML analogue; swallow their
  // args.
  DefMacro!("\\leftheadline", "");
  DefMacro!("\\rightheadline", "");
  DefMacro!("\\leftheadtext{}", "");
  DefMacro!("\\rightheadtext{}", "");
  Let!("\\flheadline", "\\hfil");
  Let!("\\frheadline", "\\hfil");
  DefMacro!("\\headmark{}", "");
  DefMacro!("\\pagecontents", "");
  DefMacro!("\\cvolyear{}", "");
  DefMacro!("\\issueinfo{}{}{}{}", "");
  DefMacro!("\\NoRunningHeads", "");
  DefMacro!("\\Monograph", "");

  // Per-field "pre" hooks — Perl L90-95. No-ops; user can `\def` to override.
  Let!("\\pretitle", "\\relax");
  Let!("\\preauthor", "\\relax");
  Let!("\\preaffil", "\\relax");
  Let!("\\predate", "\\relax");
  Let!("\\preabstract", "\\relax");
  Let!("\\prepaper", "\\relax");

  // AMS Fonts + QED — Perl L320-330. `\rom` drops its arg into rm.
  // `\qed` emits `\ltx@qed`, which is isMath-aware (end-of-proof symbol).
  // `\tildechar` is the amsppt literal `~` in typewriter (bibliography
  // key separator).
  DefMacro!("\\rom{}", "{\\rm #1}");
  DefMacro!("\\PSAMSFonts", "");
  RawTeX!("\\newif\\ifPSAMSFonts\\PSAMSFontstrue");
  DefMacro!("\\qed", "\\ltx@qed");
  DefConstructor!(
    "\\ltx@qed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
    reversion => "\\qed"
  );
  // Perl L327: DefPrimitiveI('\tildechar', undef, "~", font => { family => 'typewriter' })
  // — emits a literal `~` other-token in typewriter family, immediately during
  // digestion, with no expansion. Prior Rust DefMacro!("\\tildechar",
  // "\\texttt{\\textasciitilde}") routed through textcomp's font dispatch,
  // changing both the CS class (macro vs primitive) and producing different
  // token structure (\texttt opens a bounded font scope; the literal `~` does
  // not). Restored to faithful primitive form.
  DefPrimitive!("\\tildechar", "~", font => { family => "typewriter" });
  DefMacro!("\\breakcheck", "");
  DefMacro!("\\usualspace", " ");
  // Perl L329: \normalparindent — zero-Dimension register. Without it,
  // `\the\normalparindent` fails on amsppt documents that probe it.
  DefRegister!("\\normalparindent" => Dimension::new(0));

  // References section — Perl L333, L361-365.
  DefMacro!("\\Refsname", "References");
  DefRegister!("\\refindentwd" => Dimension::new(0));
  DefMacro!("\\refstyle{}", "");
  DefMacro!("\\keyformat{}", "#1");
  DefMacro!("\\refbreaks", "");
  DefMacro!("\\defaultreftexts", "");

  // Perl L335: \cite for plain-AMSTeX documents.
  // amsppt is a plain-TeX style, so the latex cite machinery isn't
  // loaded; this DefConstructor provides the minimal cite→bibref
  // surface that \Refs/\bibitem pair against.
  DefConstructor!("\\cite Semiverbatim",
    "<ltx:cite>[<ltx:bibref show='refnum' bibrefs='#1'/>]</ltx:cite>");

  // Head-toks and head-skip registers — Perl L44-45, L151-158. Token
  // registers for running-head content; eight length/glue registers
  // controlling head spacing. All default to zero — amsppt uses them
  // to drive its plain-format page layout, which LaTeXML ignores but
  // user code may still `\the...` or `\setlength` them.
  DefRegister!("\\leftheadtoks"        => Tokens!());
  DefRegister!("\\rightheadtoks"       => Tokens!());
  DefRegister!("\\aboveheadskip"       => Glue::new(0));
  DefRegister!("\\belowheadskip"       => Dimension::new(0));
  DefRegister!("\\abovespecialheadskip" => Glue::new(0));
  DefRegister!("\\subheadskip"         => Glue::new(0));
  DefRegister!("\\subsubheadskip"      => Glue::new(0));
  DefRegister!("\\headlineheight"      => Dimension::new(0));
  DefRegister!("\\headlinespace"       => Dimension::new(0));
  DefRegister!("\\dropfoliodepth"      => Dimension::new(0));
  DefMacro!("\\widestnumber Token {}", "");
  DefMacro!("\\nofrillscheck{}", "");
  DefMacro!("\\toc Until:\\endtoc", "");
  Let!("\\endtoc", "\\relax");

  // Theorem-env skip registers and name/font overrides — Perl L178-232.
  // The full DefConstructor bodies for \proclaim / \definition etc. still
  // need NewCounter+title infrastructure to match, but the register /
  // macro stubs are safe to land now so users can `\def\proclaimfont{…}`.
  DefRegister!("\\preproclaimskip"     => Glue::new(0));
  DefRegister!("\\postproclaimskip"    => Glue::new(0));
  DefMacro!("\\proclaimfont", "\\it");
  DefRegister!("\\remarkskip"          => Glue::new(0));
  DefRegister!("\\postdemoskip"        => Glue::new(0));
  DefRegister!("\\predefinitionskip"   => Glue::new(0));
  DefRegister!("\\postdefinitionskip"  => Glue::new(0));
  DefMacro!("\\definitionfont", "\\rm");
  DefMacro!("\\definitionname", "Definition");
  DefMacro!("\\remarkfont", "\\rm");
  DefMacro!("\\remarkname", "Remark");
  DefMacro!("\\demonstrationname", "Demonstration");
  // Perl amsppt.sty.ltxml L226 references `\examplename` in `\example`'s
  // title-Digest call but never defines it (Perl's L190 comment notes the
  // gap). Define it here for parity with `\definitionname` / `\remarkname` /
  // `\demonstrationname` so the `\example {...}` constructor's title format
  // doesn't trigger an undefined-CS error.
  DefMacro!("\\examplename", "Example");

  // Perl L468: \edtext expands to "ed." — the editor-marker inserted in
  // bib entries after an `\editors{...}` field.
  DefMacro!("\\edtext", "ed.");

  // Layout dimens — Perl L265-270. (`\rosteritemwd` is registered with
  // \roster above as part of the faithful Perl L246 port.)
  DefRegister!("\\pagenumwd"           => Dimension::new(0));
  DefRegister!("\\indenti"             => Dimension::new(0));
  DefRegister!("\\indentii"            => Dimension::new(0));
  DefMacro!("\\linespacing Number", "");
  DefMacro!("\\endquotes", "");

  // Perl amsppt.sty.ltxml L497: \smc (smallcaps) — plain-TeX font
  // switch used in AMSTeX running heads and bib entries.
  DefPrimitive!("\\smc", None, font => { shape => "smallcaps" });
});
