use crate::{engine::latex_constructs::*, prelude::*};

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: thmtools.sty.ltxml

  // thmtools.sty:26-41 passes its case options to thm-kv, whose `\thmt@modifycase` (thm-kv.sty:30-42, `uppercase`
  // by default) prefixes the environment name when `\declaretheorem` gives no `name=` (thm-kv.sty:359
  // `\thmt@setthmname{\thmt@modifycase #1}`): `\MakeUppercase` takes only the first token, so `{defn}` is
  // headed "Defn". This binding stands in for thm-kv's `\declaretheorem`, so it declares the options here.
  RawTeX!(r"\gdef\thmt@modifycase{\protect\MakeUppercase}");
  DeclareOption!("lowercase", r"\gdef\thmt@modifycase{\protect\MakeLowercase}");
  DeclareOption!("uppercase", r"\gdef\thmt@modifycase{\protect\MakeUppercase}");
  DeclareOption!("anycase", r"\gdef\thmt@modifycase{}");
  DeclareOption!("debug", None);
  ProcessOptions!();

  // Dependencies: raw TeX sub-packages (thm-kv.sty, thm-restate.sty) expect
  // kvsetkeys and keyval to be loaded. The raw thmtools.sty loads these via
  // thm-kv.sty, but our binding replaces raw thmtools.sty so the chain breaks.
  RequirePackage!("keyval");
  RequirePackage!("kvsetkeys");
  // Real thmtools.sty L47-49 does `\RequirePackage{thm-patch, thm-kv,
  // thm-restate}`, so `\usepackage{thmtools}` alone provides the
  // `restatable` environment via thm-restate.sty. Mirror that dependency
  // so we don't have to (and must NOT) define `restatable` ourselves —
  // see the note below where the native DefEnvironment was removed.
  RequirePackage!("thm-restate");
  // thm-patch.sty (the first of those three), which this binding stands in for: its hook commands
  // `\addtotheorem{pre,post}{head,foot}hook[<env>]{<code>}` (thm-patch.sty:164-176) gather code into
  // `\thmt@<env>@<point>hook`, which its wrapper of each theorem runs around the head and the end
  // (thm-patch.sty:33-62, 130-142). The raw file re-wraps `\newtheorem` and every theorem with parseargs, over the native
  // theorems here, so the native theorem runs the hooks gathered here (latex_constructs `theorem_hooks`): the head
  // hooks after its head, inside it, the foot hooks before its end code. Papers write `\addtotheorempostheadhook[thm]{\crefalias{thmlisti}{thm}}`
  // so an `enumerate` item in a theorem is referenced as the theorem's (49 amsart and 20 article papers in run 353 with
  // the command undefined, Perl's binding lacks it too, KNOWN_PERL_ERRORS #695; witnesses 1710.06939, 1712.09887,
  // 1801.10129). The `generic` hooks (no `[<env>]`) run in every theorem; thm-patch.sty:146-149's generic pre-head hook
  // steps a dummy counter for its own references, left out here.
  RawTeX!(r"\providecommand\g@prependto@macro[2]{\begingroup\toks@\expandafter{#1}\toks\tw@{#2}%
\xdef#1{\the\toks\tw@\the\toks@}\endgroup}
\def\thmt@generic@preheadhook{}\def\thmt@generic@postheadhook{}
\def\thmt@generic@prefoothook{}\def\thmt@generic@postfoothook{}
\def\thmt@hookenv#1{\@ifundefined{thmt@#1@preheadhook}{%
\global\@namedef{thmt@#1@preheadhook}{}\global\@namedef{thmt@#1@postheadhook}{}%
\global\@namedef{thmt@#1@prefoothook}{}\global\@namedef{thmt@#1@postfoothook}{}}{}}
\newcommand\addtotheorempreheadhook[1][generic]{\thmt@hookenv{#1}%
\expandafter\g@addto@macro\csname thmt@#1@preheadhook\endcsname}
\newcommand\addtotheorempostheadhook[1][generic]{\thmt@hookenv{#1}%
\expandafter\g@addto@macro\csname thmt@#1@postheadhook\endcsname}
\newcommand\addtotheoremprefoothook[1][generic]{\thmt@hookenv{#1}%
\expandafter\g@prependto@macro\csname thmt@#1@prefoothook\endcsname}
\newcommand\addtotheorempostfoothook[1][generic]{\thmt@hookenv{#1}%
\expandafter\g@prependto@macro\csname thmt@#1@postfoothook\endcsname}");
  // thm-patch.sty:178-217: when amsthm's `proof` exists at this point (`\ifx\proof\endproof\else`), thm-patch wraps
  // it too, running only the `[proof]` hooks (its generic-hook calls there are commented out): the head hooks around
  // the head, the pre-foot hook before the end code, the post-foot hook after it. The binding's `proof` (amsthm_sty.rs
  // `\@proof`, `\end@proof`) runs them under this flag: the head hooks after its title, inside it; the pre-foot hook
  // before the QED mark; the post-foot hook after the proof closes. A proof defined after thmtools is not wrapped.
  if has_meaning(&T_CS!("\\proof")) {
    RawTeX!(r"\def\thmt@proof@preheadhook{}\def\thmt@proof@postheadhook{}
\def\thmt@proof@prefoothook{}\def\thmt@proof@postfoothook{}");
    AssignValue!("thmt@proof@hooks" => true, Some(Scope::Global));
  }
  // This binding is thm-patch: a later `\usepackage{thm-patch}` (or thm-listof's, thm-autoref's) must not read the raw
  // file over it, whose `\newcommand\addtotheorem…hook` collide with the ones above and whose parseargs wrappers
  // replace the native theorems.
  AssignValue!("thm-patch.sty_loaded" => true, Some(Scope::Global));

  // Internal registers and macros needed by thm-restate.sty and thm-kv.sty
  // which load as raw TeX and expect these from thmtools internals.
  DefRegister!("\\thmt@toks" => RegisterValue::Tokens(Tokens!()));
  DefMacro!("\\thmt@thmuse@families", "thm@track@keys");
  def_macro_noop("\\thmt@mkignoringkeyhandler{}")?;
  def_macro_noop("\\thmt@thmuse@iskvtrue")?;
  // thm-kv.sty:434-437: a theorem's optional argument may be thmtools' keys — `label=`, `name=`, `continues=` — which
  // the native theorem reads (latex_constructs `thmtools_use_keys`); `\thmcontinues` (:503-509) is the note
  // `continues=` gives, as the package provides it.
  AssignValue!("thmtools_thmuse_keys" => true, Some(Scope::Global));
  RawTeX!(r"\providecommand\thmcontinues[1]{\ifcsname hyperref\endcsname\hyperref[#1]{continuing}\else
continuing\fi\ from p.\,\pageref{#1}}");

  // Set savable theorem parameters
  set_savable_theorem_parameters(vec![
    "\\thm@bodyfont", "\\thm@headfont", "\\thm@notefont",
    "\\thm@bodyfont", "\\thm@headpunct",
    "\\thm@styling", "\\thm@headstyling",
  ]);

  // \declaretheorem [keyvals] {name, ...} [keyvals]: thm-kv.sty:333-348 reads a trailing optional argument too and
  // declares each name of the comma list (`\@for`, spaces stripped) with the options `#1,#3`. Perl
  // (thmtools.sty.ltxml) takes one name and no trailing options, so `\declaretheorem{corollary, lemma,
  // proposition}[sibling=theorem]` declared one environment "corollary, lemma, proposition" and left `{lemma}`
  // undefined (witness 2608.16854; KNOWN_PERL_ERRORS #1061). Guard
  // `perfect_kernel_batch78k::thmtools_declaretheorem_declares_each_listed_name`.
  DefMacro!("\\declaretheorem[]{}",
    "\\@ifnextchar[{\\lx@thmt@declaretheorem@i{#1}{#2}}{\\lx@thmt@declaretheorem@i{#1}{#2}[]}");
  DefMacro!("\\lx@thmt@declaretheorem@i{}{}[]", sub[(first, names, last)] {
    let mut options = first.unwrap_or_default().unlist();
    let last = last.unwrap_or_default().unlist();
    if !options.is_empty() && !last.is_empty() {
      options.push(T_OTHER!(","));
    }
    options.extend(last);
    let comma = T_OTHER!(",");
    let mut out = Vec::new();
    for name in names.unlist().split(|t| *t == comma) {
      let name: Vec<Token> = name.iter().skip_while(|t| t.get_catcode() == Catcode::SPACE).cloned().collect();
      let end = name.iter().rposition(|t| t.get_catcode() != Catcode::SPACE).map_or(0, |i| i + 1);
      if end == 0 {
        continue;
      }
      out.push(T_CS!("\\lx@thmt@declaretheorem"));
      out.push(T_OTHER!("["));
      out.extend(options.iter().cloned());
      out.push(T_OTHER!("]"));
      out.push(T_BEGIN!());
      out.extend(name[..end].iter().cloned());
      out.push(T_END!());
    }
    Ok(Tokens::new(out))
  });
  // Perl: DefPrimitive('\declaretheorem OptionalKeyVals {}', sub { ... }), one name.
  DefPrimitive!("\\lx@thmt@declaretheorem OptionalKeyVals {}", sub[(kv, thmset)] {
    let name = thmset.to_string();

    // Activate any requested style
    let style_str = kv.as_ref()
      .and_then(|k| k.get_value("style"))
      .map(|v| v.to_string())
      .unwrap_or_default();
    if !style_str.is_empty() {
      use_theorem_style(&style_str);
    } else {
      use_theorem_style("plain");
    }

    // Apply thmtools_style: save headfont/headpunct/notefont/bodyfont/headformat
    {
      let mut saved: Vec<(String, Stored)> = Vec::new();
      if let Some(headfont) = kv.as_ref().and_then(|k| k.get_value("headfont")) {
        saved.push(("\\thm@headfont".into(), Stored::Tokens(headfont.revert().unwrap_or_default())));
      }
      if let Some(headpunct) = kv.as_ref().and_then(|k| k.get_value("headpunct")) {
        saved.push(("\\thm@headpunct".into(), Stored::Tokens(headpunct.revert().unwrap_or_default())));
      }
      if let Some(notefont) = kv.as_ref().and_then(|k| k.get_value("notefont")) {
        saved.push(("\\thm@notefont".into(), Stored::Tokens(notefont.revert().unwrap_or_default())));
      }
      if let Some(bodyfont) = kv.as_ref().and_then(|k| k.get_value("bodyfont")) {
        saved.push(("\\thm@bodyfont".into(), Stored::Tokens(bodyfont.revert().unwrap_or_default())));
      }
      if let Some(headformat) = kv.as_ref().and_then(|k| k.get_value("headformat")) {
        let swap = headformat.eq_text("swapnumber");
        saved.push(("thm@swap".into(), Stored::Bool(swap)));
      }
      if !saved.is_empty() {
        save_theorem_style(&name, saved);
        use_theorem_style(&name);
      }
    }

    // Read title/name/heading for theorem type; without one the name is the environment's, case-modified
    // (thm-kv.sty:359 `\thmt@setthmname{\thmt@modifycase #1}`; Perl thmtools.sty.ltxml:80-82 `\MakeUppercase`
    // alone). Witness 1505.04294 (`\declaretheorem[style=mystyle,…]{definition}`: 90 theorems headed "1.1", "1.2"
    // where the PDF prints "Definition 1.1"). Guard `perfect_kernel_batch78k::thmtools_unnamed_theorem_*`.
    let type_tokens: Option<Tokens> = kv.as_ref()
      .and_then(|k| {
        k.get_value("title")
          .or_else(|| k.get_value("name"))
          .or_else(|| k.get_value("heading"))
      })
      .map(|v| v.revert().unwrap_or_default())
      .or_else(|| {
        let mut name = vec![T_CS!("\\thmt@modifycase")];
        name.extend(thmset.clone().unlist());
        Some(Tokens::new(name))
      });

    // Read the numbered keys in their order, the last one deciding: `numbered=no` and the value-less `unnumbered` and
    // `starred` (thm-kv.sty:241-242, `\thmt@isnumberedfalse`) declare an unnumbered theorem, `numbered=yes` a numbered
    // one (:248-262). Perl reads `numbered=no` alone (thmtools.sty.ltxml:84-85), so `[style=myproof, unnumbered]{proof}`
    // was headed "Proof 1" … "Proof 161" where pdflatex prints "Proof" (witnesses 2311.15711, 1706.08429;
    // KNOWN_PERL_ERRORS #1061).
    // Guard `perfect_kernel_batch78k::thmtools_unnumbered_and_starred_declare_unnumbered_theorems`.
    let mut unnumbered = false;
    if let Some(k) = kv.as_ref() {
      for (key, value) in k.get_pairs() {
        match key.as_str() {
          "unnumbered" | "starred" => unnumbered = true,
          "numbered" => {
            let value = value.to_string();
            unnumbered = value.trim() == "no";
          },
          _ => {},
        }
      }
    }
    let flag = if unnumbered { Some(Tokens!()) } else { None };

    // Read sibling/numberlike/sharenumber
    let other: Option<Tokens> = kv.as_ref()
      .and_then(|k| {
        k.get_value("sibling")
          .or_else(|| k.get_value("numberlike"))
          .or_else(|| k.get_value("sharenumber"))
      })
      .map(|v| v.revert().unwrap_or_default());

    // Read parent/numberwithin/within
    let within: Option<Tokens> = kv.as_ref()
      .and_then(|k| {
        k.get_value("parent")
          .or_else(|| k.get_value("numberwithin"))
          .or_else(|| k.get_value("within"))
      })
      .map(|v| v.revert().unwrap_or_default());

    // Handle refname / Refname
    if let Some(refname) = kv.as_ref().and_then(|k| k.get_value("refname")) {
      let refname_str = refname.to_string();
      let parts: Vec<&str> = refname_str.splitn(2, ',').collect();
      let singular = parts.first().map(|s| s.trim()).unwrap_or("");
      let plural = parts.get(1).map(|s| s.trim()).unwrap_or(singular);
      def_macro(T_CS!(s!("\\{name}autorefname")), None,
        Tokens::new(ExplodeText!(singular)), None)?;
      def_macro(T_CS!(s!("\\cref@{name}@name")), None,
        Tokens::new(ExplodeText!(singular)), None)?;
      def_macro(T_CS!(s!("\\cref@{name}@name@plural")), None,
        Tokens::new(ExplodeText!(plural)), None)?;
    }
    if let Some(refname) = kv.as_ref().and_then(|k| k.get_value("Refname")) {
      let refname_str = refname.to_string();
      let parts: Vec<&str> = refname_str.splitn(2, ',').collect();
      let singular = parts.first().map(|s| s.trim()).unwrap_or("");
      let plural = parts.get(1).map(|s| s.trim()).unwrap_or(singular);
      def_macro(T_CS!(s!("\\Cref@{name}@name")), None,
        Tokens::new(ExplodeText!(singular)), None)?;
      def_macro(T_CS!(s!("\\Cref@{name}@name@plural")), None,
        Tokens::new(ExplodeText!(plural)), None)?;
    }

    // thmtools' `bodyfont=` keyval is applied via \thm@bodyfont before this
    // call, so no per-theorem override is threaded here.
    define_new_theorem(flag, thmset, other, type_tokens, within, None)?;
  });

  // \declaretheoremstyle [keyvals] {name}
  // Perl: DefPrimitive('\declaretheoremstyle OptionalKeyVals {}', sub { ... })
  DefPrimitive!("\\declaretheoremstyle OptionalKeyVals {}", sub[(kv, name)] {
    let name_str = name.to_string();
    let mut saved: Vec<(String, Stored)> = Vec::new();

    if let Some(headfont) = kv.as_ref().and_then(|k| k.get_value("headfont")) {
      saved.push(("\\thm@headfont".into(), Stored::Tokens(headfont.revert().unwrap_or_default())));
    }
    if let Some(headpunct) = kv.as_ref().and_then(|k| k.get_value("headpunct")) {
      saved.push(("\\thm@headpunct".into(), Stored::Tokens(headpunct.revert().unwrap_or_default())));
    }
    if let Some(notefont) = kv.as_ref().and_then(|k| k.get_value("notefont")) {
      saved.push(("\\thm@notefont".into(), Stored::Tokens(notefont.revert().unwrap_or_default())));
    }
    if let Some(bodyfont) = kv.as_ref().and_then(|k| k.get_value("bodyfont")) {
      saved.push(("\\thm@bodyfont".into(), Stored::Tokens(bodyfont.revert().unwrap_or_default())));
    }
    if let Some(headformat) = kv.as_ref().and_then(|k| k.get_value("headformat")) {
      let swap = headformat.eq_text("swapnumber");
      saved.push(("thm@swap".into(), Stored::Bool(swap)));
    }

    save_theorem_style(&name_str, saved);
  });

  // NOTE: `restatable`/`restatable*` are intentionally NOT defined here.
  // Perl's thmtools.sty.ltxml does not define them either — the env comes
  // solely from thm-restate.sty (`\newenvironment{restatable}[3][]{...
  // \label{restatable:#3}\expandafter\gdef\csname #3\endcsname{...}}`),
  // which we RequirePackage above. A previous native DefEnvironment here
  // both diverged from Perl AND blocked thm-restate's definition: LaTeX's
  // `\newenvironment{restatable}` refuses to redefine an already-defined
  // env, so when a document loaded thmtools then thm-restate the buggy
  // native version stayed active. That version digested the store-name
  // argument (3rd arg) in text mode, so a name containing `_` (e.g.
  // `two_var_indp`) raised `Error:unexpected:_ Script _ can only appear
  // in math mode` once per use. Witness 2007.12335 (thmtools+thm-restate,
  // `\begin{restatable}{theorem}{two_var_indp}`: Rust-only errors → clean,
  // Perl is clean). The clean thm-restate `\newenvironment` builds
  // `\csname #3\endcsname` (catcode-agnostic) and never typesets `#3`.

  // \listtheoremname
  DefMacro!("\\listtheoremname", "List of Theorems");

  // \listoftheorems
  // Perl: DefConstructor('\listoftheorems OptionalKeyVals', ...)
  // Simplified: the Perl version reads ignoreall/show keyvals for filtering
  DefConstructor!("\\listoftheorems[]",
    "<ltx:TOC lists='#lists' scope='global'><ltx:title>#name</ltx:title></ltx:TOC>",
    properties => sub[_args] {
      let title = Stored::from(digest_in_group(T_CS!("\\listtheoremname"))?);
      let lists = String::from("thm");
      Ok(stored_map!("name" => title, "lists" => lists))
    });
});
