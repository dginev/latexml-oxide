//! `latex_constructs` section 5: C.5 Classes, Packages and Page Styles
//!
//! Split verbatim from the single `LoadDefinitions!` body (2026-09-03); the
//! definitions run in the original order from `super::load_definitions`.

use super::*;

#[rustfmt::skip]
pub(crate) fn load() -> Result<()> {
  // ======================================================================
  // C.5 Classes, Packages and Page Styles
  // ======================================================================

  // ======================================================================
  // C.5.2 Packages
  // ======================================================================
  // We'll prefer to load package.pm, but will try package.sty or
  // package.tex (the latter being unlikely to work, but....)
  // See Stomach.pm for details
  // Ignorable packages ??
  // pre-defined packages??

  DefMacro!("\\@clsextension", "cls");
  DefMacro!("\\@pkgextension", "sty");
  Let!("\\@currext", "\\@empty");
  Let!("\\@currname", "\\@empty");
  Let!("\\@classoptionslist", "\\relax");
  Let!("\\@raw@classoptionslist", "\\relax");
  // `\@declaredoptions` is the comma list of the options the loading
  // class/package has `\DeclareOption`ed (latex.ltx L18536 `\xdef
  // \@declaredoptions{\@declaredoptions,#1}`, reset per file at L18890). The
  // native `\DeclareOption`/`\ProcessOptions` keep it in the `@declaredoptions`
  // State list (Package.pm L2405/L2511), so the macro renders that list —
  // one source of truth. Perl binds it EMPTY (pool L784); a raw option
  // processor that iterates it itself — scrbase.sty L323/L365 `\@for
  // \CurrentOption:=\@declaredoptions\do{\let\ds@…\relax}` — then never
  // retired the class's `\ds@<opt>` handlers, so the next KOMA member's
  // `\FamilyProcessOptions` re-ran the class's deprecated-option `\ds@`
  // (`tablecaptionabove` → `\KOMAExecuteOptions{captions=tableheading}` under
  // `\@currname`=typearea: "Member `.typearea.sty' … cannot handle option",
  // witness l2tabu/l2tabuen; pdflatex warns only). Guard
  // `perfect_kernel_batch53::declaredoptions_lists_declared_options`.
  DefMacro!("\\@declaredoptions", sub[_args] {
    let declared: Vec<String> = lookup_vecdeque("@declaredoptions")
      .unwrap_or_default()
      .iter()
      .flat_map(|item| match item {
        Stored::String(s) => vec![with(*s, |s| s.to_string())],
        Stored::Strings(ss) => ss.iter().map(|s| with(*s, |s| s.to_string())).collect(),
        _ => Vec::new(),
      })
      .collect();
    Ok(Tokens::new(Explode!(declared.join(","))))
  });
  def_macro_noop("\\@curroptions")?;
  def_macro_noop("\\@unusedoptionlist")?;

  DefConstructor!("\\usepackage OptionalSemiverbatim ExpandedSemiverbatim []",
                  "<?latexml package='#2' ?#1(options='#1')?>",
    before_digest => { only_preamble("\\usepackage") },
    after_digest => sub[whatsit] {
      let options: Option<&Digested> = whatsit.get_arg(1);
      let packages: Option<&Digested> = whatsit.get_arg(2);
      // Perl latex_constructs.pool.ltxml L795: `$pkg =~ s/\s+//g;` —
      // strip ALL whitespace (including internal) from each package name
      // so author typos like `\usepackage{graphic x}` resolve to graphicx.
      let package_list: Vec<String> = match packages {
        Some(value) => OPTS_REGEX.split(&value.to_string())
          .map(|s| s.chars().filter(|c| !c.is_whitespace()).collect::<String>())
          .filter(|s| !s.is_empty() && !s.starts_with('%')).collect(),
        None => Vec::new(),
      };
      // `untex` (reversion) keeps a braced value's braces — see `\documentclass`.
      let options_list = match options {
        Some(opts) => split_trim_options(&opts.untex()?),
        None => Vec::new(),
      };
      for package in package_list {
        // Record that THIS source-level \usepackage actually executed (the
        // dep-scan's executed-set gate, content.rs maybe_require_dependencies,
        // reads `<pkg>.usepackage_executed` to distinguish a top-level require
        // from one inside a false `\if…\fi` the raw-load skipped). Set only
        // here in the constructor — NOT in require_package — so the dep-scan's
        // own programmatic require_package loads don't self-populate the set.
        assign_value(&s!("{package}.usepackage_executed"), true, Some(Scope::Global));
        require_package(&package, RequireOptions {
          options: options_list.clone(),
          ..RequireOptions::default()
        })?
      }
      Ok(Vec::new())
    }
  );

  DefConstructor!("\\RequirePackage OptionalSemiverbatim ExpandedSemiverbatim []",
  "<?latexml package='#2' ?#1(options='#1')?>",
  before_digest =>  { only_preamble("\\RequirePackage") },
  after_digest => sub[whatsit] {
    let options: Option<&Digested> = whatsit.get_arg(1);
    let packages: Option<&Digested> = whatsit.get_arg(2);
    // Perl latex_constructs.pool.ltxml: `\RequirePackage` mirrors
    // `\usepackage`, with the same `$pkg =~ s/\s+//g;` whitespace strip.
    let package_list: Vec<String> = match packages {
      Some(value) => OPTS_REGEX.split(&value.to_string())
        .map(|s| s.chars().filter(|c| !c.is_whitespace()).collect::<String>())
        .filter(|s| !s.is_empty() && !s.starts_with('%')).collect(),
      None => Vec::new(),
    };
    let options_list: Vec<String> = match options {
      Some(opts) => split_trim_options(&opts.untex()?),
      None => Vec::new(),
    };
    for package in package_list {
      // See \usepackage above — record the executed source-level require for
      // the dep-scan's executed-set gate.
      assign_value(&s!("{package}.usepackage_executed"), true, Some(Scope::Global));
      require_package(&package, RequireOptions {
        options: options_list.clone(),
        ..RequireOptions::default()
      })?;
    }
    Ok(Vec::new())
  });

  DefConstructor!("\\LoadClass OptionalSemiverbatim ExpandedSemiverbatim []",
    "<?latexml class='#2' ?#1(options='#1')?>",
    before_digest => { only_preamble("\\LoadClass") }
    after_digest => sub[whatsit] {
      let options_arg: Option<&Digested> = whatsit.get_arg(1);
      let class_arg: Option<&Digested> = whatsit.get_arg(2);
      let class = class_arg.map(|c| c.to_string().replace(' ', "")).unwrap_or_default();
      let options: Vec<String> = match options_arg {
        Some(opts) => split_trim_options(&opts.to_string()),
        None => Vec::new(),
      };
      load_class(&class, options, Tokens!())?;
    }
  );

  // Related internal macros for package definition
  // Internals used in Packages
  def_macro_noop("\\NeedsTeXFormat{}[]")?;

  DefPrimitive!("\\ProvidesClass{}[]", sub[(class, version_opt)] {
    let ver_cs = T_CS!(s!("\\ver@{class}.cls"));
    let version = version_opt.unwrap_or_default();
    DefMacro!(ver_cs, None, version, scope => Some(Scope::Global));
  });

  // Note that these, like LaTeX, define macros like \var@mypkg.sty to give the version info.
  DefMacro!("\\ProvidesPackage{}[]", sub[(package, version_opt)] {
    let ver_cs = T_CS!(s!("\\ver@{package}.sty"));
    let version = version_opt.unwrap_or_default();
    DefMacro!(ver_cs, None, version, scope => Some(Scope::Global));
  });

  DefMacro!("\\ProvidesFile{}[]", sub[(file, version_opt)] {
    let ver_cs = T_CS!(s!("\\ver@{file}"));
    let version = version_opt.unwrap_or_default();
    DefMacro!(ver_cs, None, version, scope => Some(Scope::Global));
  });

  // anything useful?
  //\DeclareRelease{v4.46}{2020-03-19}{glossaries-2020-03-19.sty}
  def_macro_noop("\\DeclareRelease{}{}{}")?;
  //\DeclareCurrentRelease{v4.49}{2021-11-01}
  def_macro_noop("\\DeclareCurrentRelease{}{}")?;
  // `\IncludeInRelease{date}{cs}{descr}…body…\EndIncludeInRelease`
  // (LaTeX kernel `latexrelease.sty`). The kernel decides at run-time
  // whether `date` matches the current release; if yes, body runs; if
  // no, body is skipped. Packages like koma-script emit pairs of
  // blocks — one dated for the modern release, one with `0000/00/00`
  // as the always-fallback — wrapping `\newcommand*{\FOO}…`
  // definitions. With our prior `None`-body stub the entire body was
  // dropped on the floor, leaving `\FOO` undefined (witnesses:
  // arXiv:2506.12162 / .15311 — `\FamilyProcessOptions` undefined
  // cascade from scrbase.sty L611). Emit the captured body so the
  // first matching block defines its CSes. The second block in a
  // pair would then redefine via `\newcommand` and erroneously fire
  // "already defined" — but our `\newcommand` ignores redefinitions
  // with an `Info:ignore` (etoolbox_sty.rs:31), so the cascade lands
  // correctly with the most-recent definition still in effect.
  DefMacro!("\\IncludeInRelease{}{}{} Until:\\EndIncludeInRelease",
  sub[(_date, _cs, _descr, body)] {
    Ok(body)
  });
  DefMacro!("\\NewModuleRelease{}{}{} Until:\\EndModuleRelease",
  sub[(_date, _cs, _descr, body)] {
    Ok(body)
  });

  DefPrimitive!("\\DeclareOption{}{}", sub[(option, code)] {
    let option_str = option.to_string();
    if option_str == "*" {
      DeclareOption!(None, code);
    } else {
      DeclareOption!(option_str, code);
    }
    Ok(Vec::new())
  });

  // Perl: latex_constructs.pool.ltxml lines 868-878
  //
  // Options are pushed ONE PER ELEMENT via `pass_options` (Perl
  // Package.pm:2435 `PushValue('opt@…', map { ToString($_) } @options)`
  // spreads the list). Pushing the whole `Vec<String>` stored it as a single
  // nested `Stored::Strings`, which the `\opt@<file>` rebuild
  // (content.rs, `Stored::String` singulars) skipped — so every option routed
  // through these primitives read back EMPTY. Witness: brandeis-problemset
  // example.tex (the class forwards `\CurrentOption` to its own .sty, then
  // `\LoadClass[12pt]` clobbers `\@classoptionslist`, leaving `\opt@` as the
  // only channel; 87-error math-in-title storm).
  //
  // Split brace-aware on the REVERSION (`untex`, braces kept), as
  // `\documentclass`/`\usepackage` do above: the kernel's `\@pass@ptions`
  // (latex.ltx L18509-18526) stores the argument tokens, so
  // `\PassOptionsToPackage{paper={a4},x}{p}` is two options, not three, and
  // `\ProcessKeyOptions` reads them back with their braces intact.
  DefPrimitive!("\\PassOptionsToPackage{}{}", sub[(options, name)] {
    let name_str = Expand!(name).to_string().replace(' ', "");
    let opts = split_trim_options(&Expand!(options).untex());
    pass_options(&name_str, "sty", opts)?;
  });

  DefPrimitive!("\\PassOptionsToClass{}{}", sub[(options, name)] {
    let name_str = Expand!(name).to_string().replace(' ', "");
    let opts = split_trim_options(&Expand!(options).untex());
    pass_options(&name_str, "cls", opts)?;
  });

  // Perl `latex_constructs.pool.ltxml`:
  //   DefConstructor('\RequirePackageWithOptions Semiverbatim []',
  //     "<?latexml package='#1'?>",
  //     beforeDigest => onlyPreamble,
  //     afterDigest => sub {
  //       my $package = ToString($whatsit->getArg(1));
  //       $package =~ s/\s+//g;
  //       RequirePackage($package, withoptions => 1);
  //       return; });
  //
  // Rust used to NO-OP the `\RequirePackage` step (afterDigest was a
  // commented-out reference). That left `\citep`/`\citet` undefined
  // when a paper used a wrapper package that did
  // `\RequirePackageWithOptions{natbib}` (e.g. usbib.sty in
  // arXiv:2512.13468). Port the call to `require_package_with_options`,
  // which already handles the option-list lookup from
  // `opt@<currname>.<currext>`.
  DefConstructor!("\\RequirePackageWithOptions Semiverbatim []",
  "<?latexml package='#1'?>",
  before_digest => { only_preamble("\\RequirePackage") }
  after_digest => sub[whatsit] {
    let pkg_arg: Option<&Digested> = whatsit.get_arg(1);
    let pkg = pkg_arg.map(|c| c.to_string().replace(' ', "")).unwrap_or_default();
    require_package_with_options(&pkg)?;
  }
  );

  // Perl `latex_constructs.pool.ltxml`:
  //   DefConstructor('\LoadClassWithOptions Semiverbatim []',
  //     "<?latexml class='#1'?>",
  //     beforeDigest => onlyPreamble,
  //     afterDigest  => sub {
  //       my $class = ToString($whatsit->getArg(1));
  //       $class =~ s/\s+//g;
  //       LoadClass($class, withoptions => 1);
  //       $stomach->leaveHorizontal_internal;
  //       return; });
  //
  // Rust used to NO-OP the `\LoadClass` step — just emitted the PI.
  // That left `\abovecaptionskip`, `\belowcaptionskip`,
  // `\thesubsection`, `\Large`/`\small`/`\footnotesize`, etc. all
  // undefined when a paper used a custom class that did
  // `\LoadClassWithOptions{article}` (e.g. applemlr.cls in
  // arXiv:2512.10685). Port the `LoadClass` call.
  //
  // We pass empty options here (Perl's `withoptions => 1` would
  // inherit calling-class options from `\@classoptionslist`, but
  // for the no-options-on-\documentclass cases that dominate the
  // arxmliv warning corpus this is a no-op anyway).
  DefConstructor!("\\LoadClassWithOptions Semiverbatim []", "<?latexml class='#1'?>",
    before_digest => { only_preamble("\\LoadClassWithOptions") }
    after_digest => sub[whatsit] {
      let class_arg: Option<&Digested> = whatsit.get_arg(1);
      let class = class_arg.map(|c| c.to_string().replace(' ', "")).unwrap_or_default();
      load_class(&class, Vec::new(), Tokens!())?;
    }
  );
  // Perl: latex_constructs.pool.ltxml L900-903
  DefPrimitive!("\\@onefilewithoptions {} [][] {}", sub[(name, option1, _option2, ext)] {
    let name_str = Expand!(name).to_string();
    let ext_str = Expand!(ext).to_string();
    let opts_str = match option1 {
      Some(o) => Expand!(o).to_string(),
      None => String::new(),
    };
    let options: Vec<String> = opts_str.split(',')
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
      .collect();
    let _ = input_definitions(&name_str, NewDefault!(InputDefinitionOptions,
      extension => Some(Cow::Owned(ext_str)),
      handleoptions => true,
      options => options
    ));
  });

  def_macro_noop("\\CurrentOption")?;

  // Perl: latex_constructs.pool.ltxml lines 907-919
  DefPrimitive!("\\OptionNotUsed", {
    let option = Expand!(T_CS!("\\CurrentOption")).to_string();
    if !option.is_empty() {
      let ext = Expand!(T_CS!("\\@currext")).to_string();
      if ext == "cls" {
        push_value("@unusedoptionlist", option)?;
      }
    }
  });
  DefPrimitive!("\\@unknownoptionerror", {
    let option = Expand!(T_CS!("\\CurrentOption")).to_string();
    let name = Expand!(T_CS!("\\@currname")).to_string();
    Info!(
      "unexpected",
      &option,
      &s!("Unknown option '{}' for {}", option, name)
    );
  });

  DefPrimitive!("\\ExecuteOptions{}", sub[(options)] {
    let expanded = do_expand(options)?.to_string();
    let opts: Vec<&str> = expanded.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    execute_options(&opts)?;
    Ok(Vec::new())
  });

  DefPrimitive!("\\ProcessOptions OptionalMatch:*", sub[(star)] {
    // Perl: ProcessOptions(($star ? (inorder => 1) : ()));
    let inorder = star.is_some();
    process_options(inorder, &[])?;
    Ok(Vec::new())
  });
  DefMacro!("\\@options", "\\ProcessOptions*");

  Let!("\\@enddocumenthook", "\\@empty");
  DefMacro!("\\AtEndOfPackage{}", sub [(code)] {
    let name = Expand!(T_CS!("\\@currname")).to_string();
    let ttype = Expand!(T_CS!("\\@currext")).to_string();
    let hookcs = T_CS!(s!("\\{name}.{ttype}-h@@k"));
    AddToMacro!(hookcs, code);
  });

  DefMacro!("\\@ifpackageloaded", r"\@ifl@aded\@pkgextension");
  DefMacro!("\\@ifclassloaded", r"\@ifl@aded\@clsextension");
  // \ltx@ifpackageloaded / \ltx@ifclassloaded — LaTeXML-internal aliases
  // for the file-loaded predicates; live in `latex_constructs_rust_only.rs`.
  // Latex.ltx L15252-15256: LaTeX3-style aliases for the file-load
  // tracking commands. The `\If*LoadedTF/AtLeastTF` family is a
  // modern-LaTeX addition not in Perl LaTeXML; it lives in
  // `latex_constructs_rust_only.rs` (which loads last after this file)
  // — so we don't redefine the aliases here.
  DefMacro!("\\@ifl@aded{}{}", sub[(ext, name)] {
  let path = s!("{}.{}", Expand!(name), Expand!(ext));
  // Per OXIDIZED_DESIGN #23: a package is "loaded" when EITHER the
  // binding (`_loaded`) OR the raw .sty/.cls (`_raw_loaded`) is in
  // place. User-level `\@ifpackageloaded{X}` doesn't care which path.
  // Mirrors Perl `\@ifpackageloaded` checking `<X.sty>_loaded` (which
  // is set by both Perl `loadLTXML` and `loadTeXDefinitions`).
  if lookup_bool(&s!("{path}_loaded")) || lookup_bool(&s!("{path}_raw_loaded")) {
    T_CS!("\\@firstoftwo")
  } else {
    T_CS!("\\@secondoftwo")
  }});

  DefMacro!("\\@ifpackagewith", r"\@if@ptions\@pkgextension");
  DefMacro!("\\@ifclasswith", r"\@if@ptions\@clsextension");
  // Perl: latex_constructs.pool.ltxml lines 952-958
  DefMacro!("\\@if@ptions{}{}{}", sub[(ext, name, option)] {
    let option_str = Expand!(option).to_string();
    let key = s!("opt@{}.{}", Expand!(name), Expand!(ext));
    let found = with_value(&key, |val_opt| {
      if let Some(Stored::VecDequeStored(values)) = val_opt {
        values.iter().any(|v| v.to_string() == option_str)
      } else {
        false
      }
    });
    if found {
      T_CS!("\\@firstoftwo")
    } else {
      T_CS!("\\@secondoftwo")
    }
  });
  DefMacro!(
    "\\@ptionlist {}",
    r"\@ifundefined{opt@#1}\@empty{\csname opt@#1\endcsname}"
  );

  // latex.ltx:1832 `\long\def\g@addto@macro#1#2{\begingroup\toks@\expandafter
  // {#1#2}\xdef#1{\the\toks@}\endgroup}`: the append happens at DIGESTION
  // (the `\xdef`), not at expansion. Perl :968 (and the former port) made it
  // an expandable macro with a side effect, so a `\g@addto@macro` sitting
  // right after an `\ifnum` operand — `\ifnum\numspell@group@digit@i>0
  // \numspell@{ hundred}\fi` with `\numspell@#1` = `\g@addto@macro
  // \thenumspell{#1}` (numspell-english.sty:79-105) — was EXECUTED by the
  // number scan's one-token look-ahead (tex.web §444) even in a false
  // branch: every group of "12000" spelled ("hundred and -twotwelve thousand,
  // nought", then `\StrChar` on the leading space → `\GenericError`;
  // numspell 12 errors, KPE #170). Trigger: `\def\out{}\ifnum0>0
  // \g@addto@macro\out{WRONG}\fi[\out]`. Guard:
  // `perfect_kernel_batch54::g_addto_macro_appends_at_digestion`.
  RawTeX!(
    r"\long\def\g@addto@macro#1#2{\begingroup\toks@\expandafter{#1#2}\xdef#1{\the\toks@}\endgroup}"
  );
  DefMacro!("\\addto@hook DefToken {}", "#1\\expandafter{\\the#1#2}");

  // Alas, we're not tracking versions, so we'll assume it's "later" & cross fingers....
  DefMacro!("\\@ifpackagelater{}{}{}{}", "#3");
  DefMacro!("\\@ifclasslater{}{}{}{}", "#3");
  Let!("\\AtEndOfClass", "\\AtEndOfPackage");

  def_macro_noop("\\AtBeginDvi {}")?;

  TeX!(
    r###"
  \def\@ifl@t@r#1#2{%
    \ifnum\expandafter\@parse@version@#1//00\@nil<%
          \expandafter\@parse@version@#2//00\@nil
      \expandafter\@secondoftwo
    \else
      \expandafter\@firstoftwo
    \fi}
  \def\@parse@version@#1{\@parse@version0#1}
  \def\@parse@version#1/#2/#3#4#5\@nil{%
  \@parse@version@dash#1-#2-#3#4\@nil
  }
  \def\@parse@version@dash#1-#2-#3#4#5\@nil{%
    \if\relax#2\relax\else#1\fi#2#3#4 }"###
  );

  //======================================================================
  // Somewhat related I/O stuff
  // latex.ltx:228-281, the UNIX/DOS-style parser (`\@currdir` = `./`), taken
  // verbatim. Perl (pool:980) instead fully expands the argument, runs
  // `pathname_split` and re-tokenizes the pieces with `ExplodeText` — which
  // re-LETTERS them, so a caller that `\@onelevel@sanitize`s its argument
  // first (currfile.sty:78-85, then `\ifx\@tempa\currfilename` against
  // another sanitized string; import.sty; docstrip) sees a catcode mismatch
  // that real LaTeX never has. The raw macro keeps the argument's own tokens
  // in `\filename@area/base/ext`, exactly like latex.ltx. Perfect-kernel P50;
  // guard `perfect_kernel_batch50::filename_parse_keeps_argument_catcodes`.
  TeX!(
    r###"
  \def\filename@parse#1{%
    \let\filename@area\@empty
    \expandafter\filename@path#1/\\}
  \def\filename@path#1/#2\\{%
    \ifx\\#2\\%
       \def\reserved@a{\filename@simple#1.\\}%
    \else
       \edef\filename@area{\filename@area#1/}%
       \def\reserved@a{\filename@path#2\\}%
    \fi
    \reserved@a}
  \def\filename@simple#1.#2\\{%
    \ifx\\#2\\%
      \let\filename@ext\relax
      \edef\filename@base{#1}%
    \else
      \filename@dots{#1}#2\\%
    \fi}
  \def\filename@dots#1#2.#3\\{%
    \ifx\\#3\\%
      \def\filename@ext{#2}%
      \edef\filename@base{#1}%
    \else
      \filename@dots{#1.#2}#3\\%
    \fi}
  \def\filename@dot#1.\\{#1}"###
  );

  // latex.ltx initializes \@filelist to \@gobble, which eats the leading comma
  // from the first \@addtofilelist call. We replicate this by using \@gobble.
  DefMacro!("\\@filelist", "\\@gobble");
  DefMacro!("\\@addtofilelist{}", sub[(arg)] {
    let expansion = Expand!(Tokens!(T_CS!("\\@filelist"), T_OTHER!(","), arg.unlist()));
    DefMacro!("\\@filelist",None,expansion);
    Vec::new()
  });

  // Float-list bookkeeping stubs — Perl: latex_constructs.pool.ltxml L1015-1028.
  // \@toplist/\@botlist/etc. are ignored (LaTeXML doesn't track float-page
  // placement), but they need to be defined so latex.ltx-driven code paths
  // that consult them (e.g. via `\@cons`) don't error on `\@empty` lookups.
  def_macro_identity("\\@topnewpage{}")?;
  DefMacro!(
    "\\@next{}{}{}{}",
    r"\ifx#2\@empty #4\else\expandafter\@xnext #2\@@#1#2#3\fi"
  );
  TeX!(r"\def\@xnext \@elt #1#2\@@#3#4{\def#3{#1}\gdef#4{#2}}");
  Let!("\\@elt", "\\relax");
  def_macro_noop("\\@freelist")?;
  // `\@currbox` is NOT a list: real LaTeX `\let`s it to an unexpandable box
  // register (`\@next\@currbox\@freelist` in `\@xfloat`, latex.ltx:17443; the
  // freelist boxes come from `\newbox`, :424/442) and code uses it as one —
  // `\setbox\@currbox`, `\ht\@currbox`, `\string\@currbox`. Perl's empty
  // macro (latex_constructs.pool.ltxml:1025) makes dpfloat.sty:83-85
  // `\@namedef{LP:\expandafter\string\@currbox}` expand `\@currbox` to
  // NOTHING, so `\string` eats the `\endcsname` and the `\csname` scan runs
  // to the end of the document (memman 995/1001 errors, KNOWN_PERL_ERRORS
  // #115). A box register is what the kernel has. Guard:
  // `perfect_kernel_batch52::currbox_is_a_box_register`.
  TeX!(r"\newbox\@currbox");
  def_macro_noop("\\@toplist")?;
  def_macro_noop("\\@botlist")?;
  def_macro_noop("\\@midlist")?;
  def_macro_noop("\\@currlist")?;
  def_macro_noop("\\@deferlist")?;
  def_macro_noop("\\@dbltoplist")?;
  def_macro_noop("\\@dbldeferlist")?;
  def_macro_noop("\\@startcolumn")?;

  // Perl: latex_constructs.pool.ltxml L5510 — `Let('\@begindocumenthook', '\@empty');`
  // \@begindocumenthook is fired by `\document` (latex.ltx); we install it as
  // an alias of \@empty so that path is a no-op in the no-expl3-hooks case.
  // (When expl3-code.tex *is* loaded, our `\document` constructor dispatches
  // through `\hook_use:n{begindocument}` instead — see line 2820+ comment.)
  Let!("\\@begindocumenthook", "\\@empty");

  // Perl L5511: \@preamblecmds collects \@onlypreamble entries; empty by default
  def_macro_noop("\\@preamblecmds")?;

  // Perl L5536-5539: q-tokens used by \@notdefinable error formatting and by
  // various pattern-quoting expansion paths (e.g. \GenericWarning padding).
  DefMacro!("\\@qend", None, "end");
  DefMacro!("\\@qrelax", None, "relax");
  DefMacro!("\\@spaces", None, "\\space\\space\\space\\space");
  // Perl: `Let('\@sptoken', T_SPACE)` — alias for the literal SPACE token
  // (catcode 10), NOT the `\space` macro. Used by makecell.sty's `\ifx
  // \@sptoken\TeXr@temp` next-token check, which requires Token-level
  // (not macro-level) equivalence to a real space.
  Let!("\\@sptoken", T_SPACE!());

  //======================================================================
  // C.5.3 Page Styles
  //======================================================================
  // Ignored
  // Perl 74181415 (#2442): page counter starts at 1, not 0.
  NewCounter!("page");
  SetCounter!("page" => Number::new(1));
  DefMacro!("\\@mkboth", "\\@gobbletwo");
  DefMacro!(
    "\\ps@empty",
    "\\let\\@mkboth\\@gobbletwo\\let\\@oddhead\\@empty\\let\\@oddfoot\\@empty\
     \\let\\@evenhead\\@empty\\let\\@evenfoot\\@empty"
  );
  DefMacro!(
    "\\ps@plain",
    "\\let\\@mkboth\\@gobbletwo\
     \\let\\@oddhead\\@empty\\def\\@oddfoot{\\reset@font\\hfil\\thepage\
     \\hfil}\\let\\@evenhead\\@empty\\let\\@evenfoot\\@oddfoot"
  );
  Let!("\\@leftmark", "\\@firstoftwo");
  Let!("\\@rightmark", "\\@secondoftwo");

  // Expandable no-op MACROS, not primitives (Perl latex_constructs.pool.ltxml
  // L997-998 uses DefPrimitive). latex.ltx L18297-18300 defines `\pagestyle`
  // as a plain `\def`, and scrlayer.sty L2183-2196 patches it with the
  // triple-`\expandafter` freeze (`\renewcommand*\pagestyle[1]{\expandafter
  // \reserved@a\pagestyle{#1}…}`) that inlines the OLD body at definition
  // time. A non-expandable primitive cannot be inlined, so the literal
  // `\pagestyle{#1}` survives in the new body and `\AtBeginDocument
  // {\pagestyle{test}}` (scrlayer.sty L2198-2213) recurses forever
  // (Fatal:Timeout:PushbackLimit / Recursion; Perl 0.8.8 hangs on the same
  // 13-line freeze repro). The page style stays ignored either way — only
  // gullet-expandability is restored. Witness DEMO-TUDaPhD/TUDaThesis,
  // neoschool, bfh-ci (raw scrlayer-scrpage). KNOWN_PERL_ERRORS #121; guard
  // `perfect_kernel_batch53::pagestyle_expandafter_freeze_terminates`.
  def_macro_noop("\\pagestyle{}")?;
  def_macro_noop("\\thispagestyle{}")?;
  def_primitive_noop("\\markright{}")?;
  def_primitive_noop("\\markboth{}{}")?;
  def_primitive_noop("\\leftmark")?;
  def_primitive_noop("\\rightmark")?;
  def_primitive_noop("\\pagenumbering{}")?;
  // Perl: DefMacro('\twocolumn[]', '\ifx.#1.\else\par\noindent#1\fi\par').
  //
  // SURPASS-PERL (OXIDIZED_DESIGN): the optional argument is a one-column-
  // spanning header; real LaTeX typesets it in a box (`\@topnewpage`), so any
  // font/alignment declaration inside it (`\centering`, `\Large`, …) is scoped
  // to the header and does NOT leak into the body. Perl's simplified macro
  // splices `#1` unscoped, so a `\Large` in the header runs on into everything
  // after — e.g. cvpr's `\maketitlesupplementary` does
  // `\twocolumn[\centering\Large … Supplementary Material …]`, and the whole
  // Supplementary section renders oversized. (Perl escapes this only because it
  // has no cvpr binding and never runs `\maketitlesupplementary` at all.) Wrap
  // `#1` in a group with its own `\par`, matching the box scope real LaTeX
  // gives it. Witness html_feedback#6638 (arXiv:2511.14625v1).
  DefMacro!(
    "\\twocolumn[]",
    "\\ifx.#1.\\else\\par{\\noindent#1\\par}\\fi\\par"
  );
  // Perl: DefMacro('\onecolumn', '\par');
  DefMacro!("\\onecolumn", "\\par");
  DefMacro!("\\@onecolumna", "", locked => true);
  DefMacro!("\\@twocolumna", "", locked => true);

  // Style parameters from Fig. C.3, p.182
  DefRegister!("\\paperheight"     => Dimension!("11in"));
  DefRegister!("\\paperwidth"      => Dimension!("8.5in"));
  DefRegister!("\\textheight"      => Dimension!("550pt"));
  DefRegister!("\\textwidth"       => Dimension!("345pt"));
  DefRegister!("\\topmargin"       => Dimension::new(0));
  DefRegister!("\\headheight"      => Dimension::new(0));
  DefRegister!("\\headsep"         => Dimension::new(0));
  DefRegister!("\\footskip"        => Dimension::new(0));
  DefRegister!("\\footheight"      => Dimension::new(0));
  DefRegister!("\\evensidemargin"  => Dimension::new(0));
  DefRegister!("\\oddsidemargin"   => Dimension::new(0));
  DefRegister!("\\marginparwidth"  => Dimension::new(0));
  DefRegister!("\\marginparsep"    => Dimension::new(0));
  DefRegister!("\\columnwidth"     => Dimension!("6in"));
  DefRegister!("\\linewidth"       => Dimension!("6in"));
  DefRegister!("\\baselinestretch" => Dimension::new(0));
  // \columnsep / \columnseprule / \mathindent registers live in
  // `latex_constructs_rust_only.rs` section 8 (Perl
  // `latex_base.pool.ltxml` L309-311).

  // \@ifl@t@r and the \@parse@version chain are defined earlier in this
  // file (above L4181); the previous duplicate TeX!() block here was dead
  // code overriding identical bodies.

  //======================================================================
  // C.5.4 The Title Page and Abstract
  //======================================================================
  // See frontmatter support in TeX.ltxml

  Let!("\\@title", "\\@empty");
  Let!("\\@shorttitle", "\\@empty");
  // Perl (PR #2767): '\gdef\@shorttitle{#1}\gdef\@title{#2}
  //   \ifx.#1.\else\lx@add@toctitle{#1}\fi\lx@add@title{#2}'.
  // Rust-only: also \gdef the non-@ \shorttitle, which user styles reference
  // (e.g. arxiv.sty \hypersetup{pdftitle={\shorttitle}}; see \shortauthor note below).
  // Single logical line (here and below): a `\` + newline in a raw string is
  // a literal backslash + end-of-line = a spurious CONTROL-SPACE `\ ` in the
  // body (see the \maketitle note above; driver 1708.07027).
  DefMacro!("\\title[]{}",
    // The frontmatter copy is taken from the STORED macro (`\expandafter…
    // {\@title}`), not the raw `#2`: a `\def` inside the argument writes
    // `##` (latex.ltx:17214 `\gdef\@title{#1}` halves it once), and an
    // argument position does not halve, so the raw copy digested `\def\$##1…
    // {##2}` and a literal `#` reached the stomach — the RCS-keyword idiom
    // `\date{\def\$##1: ##2 ##3${##2}\$Revision: 3.1 $}` (ulineno.tex:16, 2
    // errors; Perl pool:1066 shares the raw copy). KNOWN_PERL_ERRORS #145.
    // Guard: `perfect_kernel_batch54::frontmatter_copies_the_halved_macro`.
    r"\gdef\@shorttitle{#1}\gdef\shorttitle{#1}\gdef\@title{#2}\ifx.#1.\else\lx@add@toctitle{#1}\fi\expandafter\lx@add@title\expandafter{\@title}",
    locked => true);
  DefMacro!("\\@date", "\\@empty");
  DefMacro!(
    "\\date{}",
    r"\def\@date{#1}\expandafter\lx@add@date@halved\expandafter{\@date}"
  );
  DefMacro!(
    "\\lx@add@date@halved{}",
    r"\lx@add@date[role=creation,name={\@ifundefined{datename}{}{\datename}}]{#1}"
  );
  // Conference-template "equal contribution" markers used inside \author{...}
  // by AAAI's aaai22.sty, NeurIPS templates, Springer Nature sn-jnl,
  // ACM acmart, etc. The class binding typically defines them locally
  // inside \@maketitle (scoped), which means user code that references
  // them in \author{} (BEFORE \maketitle expands) hits an undefined-CS
  // error. Pre-define at the kernel level as no-op markers — the local
  // \@maketitle redefinition still applies at \maketitle time, so styled
  // output keeps the footnote markers intact when the class supports them.
  // Driver: 2103.05277, 2111.06599, 2006.08767. Previously stubbed in
  // omnibus_cls.rs (commit 3f40bf8211) but OmniBus only loads as a
  // class-binding fallback for unknown documentclasses; papers using
  // \documentclass{article} with \usepackage{aaai22} don't trigger it.
  def_macro_noop("\\equalcontrib")?;
  def_macro_noop("\\equalcont")?;
  // NOTE: a `\person@thanks` constructor used to live here (a port of a since-
  // removed Perl construct). Current Perl handles an author's `\thanks` via
  // \lx@personname's beforeDigest (Let \thanks → \lx@add@thanks; see
  // base_utilities.rs), which routes to \lx@annotate@frontmatter@now and
  // applies the `\lx@contact@thanks@name` default ("Thanks: "). The stale
  // constructor produced a bare <contact role=thanks> with no name and is
  // removed.

  // `\thanksref{key}` — common in author-block / affiliation styles
  // (revtex, ifacconf, elsart, etc.). Each class typically defines a
  // version that emits a footnote-style mark. Round-34 surpass-Perl:
  // render as superscript so the marker reaches the author block
  // (Perl OmniBus L257 also gobbles to Tokens()). Witnesses:
  // arXiv:2507.06392 / .09311 (ifacconf.cls — no Rust binding).
  DefMacro!("\\thanksref{}", "\\textsuperscript{#1}");

  // NOTE: the `\cprime` / `\Cprime` / `\cdprime` / `\Cdprime` Cyrillic
  // transliteration family used to live here, which kept a non-Perl definition
  // in a file that mirrors `latex_constructs.pool.ltxml` byte-for-byte. They are
  // not LaTeX kernel commands at all — they belong to `mathscinet.sty` (AMS, in
  // the amsrefs bundle), now bound at
  // `latexml_package/src/package/mathscinet_sty.rs`, which all three witnesses
  // (arXiv:2508.13753 / .20226 / 2509.07628) actually load. An always-on stub
  // for `.bib`-borne use in a document loading no package was tried in
  // `latex_constructs_rust_only.rs` and retracted 2026-07-27 — see the
  // retraction comment there for why it is no longer needed.

  // `\polhk{char}` — Polish hook (ogonek) accent. Its real home is
  // `mathscinet.sty` L111-113 (NOT tipa.sty, as this comment said before the
  // source was read), where every encoding branch defines it as the kernel
  // accent `\k` — and `mathscinet_sty.rs` binds it that way. This identity stub
  // stays as the fallback for a document that uses `\polhk{a}` without loading
  // the package (bibliographies do), so the bare char still shows.
  // Witnesses: 2 papers in Stage-15 v3.
  def_macro_identity("\\polhk{}")?;
  // \@personname (now \lx@personname) and the ltx:personname sanitize Tag
  // moved to base_utilities.rs (Perl PR #2767: Base_Utility.pool.ltxml).

  DefConstructor!("\\and", " and ");

  DefMacro!("\\lx@author@sep", "\\qquad");
  DefMacro!("\\lx@author@conj", "\\qquad");

  DefMacro!("\\@author", "\\@empty");
  DefMacro!("\\@shortauthor", "\\@empty");
  // \shortauthor / \shorttitle: many journal classes (mnras, arxiv,
  // ICML, NeurIPS templates) reference \shortauthor in
  // \hypersetup{pdfauthor=...} and similar before \author has run.
  // Without an initial empty definition, the reference errors out.
  // (Rust-only; Perl PR #2767 only pre-defines the @-forms. Driver:
  // 2406.14142 arxiv.sty L64 `\hypersetup{pdfauthor={\shortauthor}}`
  // before \author fires.)
  DefMacro!("\\shortauthor", "\\@empty");
  DefMacro!("\\shorttitle", "\\@empty");
  // Perl (PR #2767): '\def\@shortauthor{#1}\def\@author{#2}\lx@add@authors{#2}'.
  // Rust-only: also \gdef the non-@ \shortauthor for user-style references
  // (see note above; our \author is locked so renewcommand can't add it).
  DefMacro!("\\author[]{}",
    r"\def\@shortauthor{#1}\gdef\shortauthor{#1}\def\@author{#2}\expandafter\lx@add@authors\expandafter{\@author}",
    locked => true);

  DefPrimitive!("\\lx@authors@oneline", {
    if lookup_mapping("DOCUMENT_CLASSES", "ltx_authors_multiline").is_none() {
      AssignMapping!("DOCUMENT_CLASSES", "ltx_authors_1line" => true);
    }
  });
  DefPrimitive!("\\lx@authors@multiline", {
    if lookup_mapping("DOCUMENT_CLASSES", "ltx_authors_1line").is_none() {
      AssignMapping!("DOCUMENT_CLASSES", "ltx_authors_multiline" => true);
    }
  });
  Let!("\\ltx@authors@oneline", "\\lx@authors@oneline");
  Let!("\\ltx@authors@multiline", "\\lx@authors@multiline");

  DefMacro!(
    "\\@add@conversion@date",
    "\\lx@add@date[role=conversion]{\\today}"
  );

  // Perl: latex_constructs.pool.ltxml L1128-1129
  // In case \@maketitle defines \And/\AND — we can't emulate that, so map them to \and
  // for and_split to properly separate authors.
  Let!("\\And", "\\and");
  Let!("\\AND", "\\and");

  // SURPASS-PERL (OXIDIZED_DESIGN #124, KNOWN_PERL_ERRORS #90): recover content
  // a document injects into the title via `\g@addto@macro\@maketitle{…}`.
  // LaTeXML redefines `\maketitle` to deposit its own captured frontmatter and
  // then discards `\@maketitle` wholesale (`\global\let\@maketitle\relax`, with
  // the source comment "we can't yet emulate that"), so a teaser figure appended
  // to `\@maketitle` — and its `\label` — were silently dropped by BOTH engines.
  //
  // Two pieces: (1) predefine `\@maketitle` EMPTY so `\g@addto@macro` appends
  // cleanly (LaTeXML never reimplements the title *layout*, so `\@maketitle` is
  // otherwise undefined and appending to it warns "not expandable" and leaves a
  // self-reference); (2) `\lx@deposit@maketitle` deposits its accumulated content
  // in a title-neutralized group (nulling `\@title`/`\@author`/`\@date`/`\@thanks`
  // so the rare class that stuffs real title-layout into `\@maketitle` does not
  // double-print the title; the common case holds only the injected content).
  // The `\ifx…\@empty` guard makes this a no-op for the vast majority of papers.
  // Witness arXiv:2506.23854 (html_feedback#4281).
  DefMacro!("\\@maketitle", "");
  DefMacro!(
    "\\lx@deposit@maketitle",
    r"\ifx\@maketitle\@empty\else{\let\@title\@empty\let\@author\@empty\let\@date\@empty\let\@thanks\@empty\let\and\relax\@maketitle}\fi"
  );

  // Doesn't produce anything (we're already inserting frontmatter),
  // But, it does make the various frontmatter macros into no-ops.
  // Locked: raw TeX packages (e.g., nips_2017.sty) may \renewcommand{\maketitle}, but
  // LaTeXML's frontmatter handling must take precedence. Perl achieves this by having
  // the compiled binding override raw TeX; we use `locked` to prevent raw overwrite.
  // NOTE: this body MUST be on one logical line with NO `\` line-continuations.
  // In a Rust raw string `\` + newline is a literal backslash followed by an
  // end-of-line, which the LaTeXML tokenizer reads as a CONTROL-SPACE `\ ` in
  // the macro body. That is normally harmless (it digests as a space), but a
  // document that (mis)redefines control-space — e.g. 1708.07027's
  // `\def\<eol>case#1#2{…}`, where the line break after `\def\` makes it
  // `\def\ case#1#2{…}` and `\ ` becomes a 2-arg macro — then has `\maketitle`'s
  // body invoke the corrupted `\ `, derailing the whole frontmatter into a
  // `\@maketitle`-undefined / XMApp-in-empty cascade. Perl builds this body by
  // string concatenation (no `\`+eol), so its `\maketitle` has no control-space
  // and is robust. Keep it single-line here to match.
  //
  // `\lx@deposit@maketitle` runs after `\lx@frontmatterhere` (so injected content
  // lands right after the title) and before `\global\let\@maketitle\relax` (while
  // `\@maketitle` still holds its content).
  DefMacro!(
    "\\maketitle",
    r"\lx@frontmatterhere\let\lx@frontmatter@fallback\relax\@startsection@hook\lx@deposit@maketitle\global\let\thanks\relax\global\let\maketitle\relax\global\let\@maketitle\relax\global\let\@thanks\@empty\global\let\@author\@empty\global\let\@date\@empty\global\let\@title\@empty\global\let\title\relax\global\let\author\relax\global\let\date\relax\global\let\and\relax",
    locked => true
  );
  // In case \maketitle isn't used in the document, let's check for it.
  AddToMacro!("\\@startsection@hook", "\\lx@frontmatter@fallback");
  // in cases such as titlepage, the document end is the last fallback.
  let _ = push_value(
    "@at@end@document",
    Tokens!(T_CS!("\\lx@frontmatter@fallback")),
  );

  DefMacro!("\\@thanks", "\\@empty");
  // Perl (PR #2767): `\thanks[]{}` → '\def\@thanks{#2}\lx@add@pubnote{#2}' —
  // optional arg for OmniBus use (thrown away). #2 is the required body.
  //
  // SURPASS-PERL (Cluster A, docs/SYNC_STATUS.md L201-208): `[opt]` is
  // identifier-shape (a label tag, e.g. `\thanks[funding-1]{…}`). Switch
  // to `OptionalSemiverbatim` so a literal `_` in the label doesn't bleed
  // through as `T_SUB` and trip the script-handler text-mode error.
  // Perl uses default catcodes for `[opt]` and SHARES the failure mode.
  DefMacro!(
    "\\thanks OptionalSemiverbatim {}",
    r"\def\@thanks{#2}\lx@add@pubnote{#2}"
  );
  // ijmart.cls:180-182's `\thanks` accumulates `\thankses`, which its
  // `\@maketitle` (:246) lays out; ours emits a pubnote instead, so the
  // accumulator stays at its class-initial empty value for the layout run
  // under `\lx@deposit@maketitle` (OXIDIZED_DESIGN #124; ijmart doc).
  DefMacro!("\\thankses", "");

  // Abstract SHOULD have been so simple, but seems to be a magnet for abuse & confusion.
  // Standard LaTeX classes expect it after \maketitle, and deposit it where found.
  // But many others expect it declared before \maketitle & include it within!
  // Moreover, while it's generally defined as an environment,
  // some users get away with writing \abstract{text} or even \abstract text ... \section?

  // If called as environment, it SHOULD close as environment, as well.
  DefMacro!(T_CS!("\\begin{abstract}"), None, "\\lx@begin@abstract");
  DefMacro!(T_CS!("\\end{abstract}"), None, "\\lx@end@abstract");
  // If called directly, maybe as
  //   \abstract{text}
  // OR \abstract text \endabstract
  // OR even \abstract text... \somethingelse  (section? \par ?)
  DefMacro!("\\abstract", {
    if if_next(T_BEGIN!())? {
      // `\abstract{…}` is NOT a macro call in LaTeX: `\abstract` is the
      // environment's begin code and `{…}` a plain group the body is read
      // through incrementally, so a `\makeatletter` inside it takes effect
      // before the `\patch@level` that follows (char-list-alphabeta.tex:88-103,
      // char-list). Taking the group as a pre-tokenized `{}` argument
      // (`\lx@add@abstract{#1}`) split it into `\patch`+`@level` — Perl shares
      // that (PLANS P74). Keep the group a group: open the abstract, re-emit
      // the `{`, and let the group's end close the abstract via `\aftergroup`.
      // Guard: `perfect_kernel_batch54::braced_abstract_reads_its_body_incrementally`.
      read_token()?;
      Tokens!(
        T_CS!("\\lx@begin@abstract"), T_BEGIN!(), T_CS!("\\aftergroup"), T_CS!("\\lx@end@abstract"))
    } else {
      // When \abstract is used without braces (e.g. \abstract ... \section{...}),
      // add \maybe@end@abstract to \@startsection@hook so the abstract closes
      // when the next sectioning command starts.
      Tokens!(
        T_CS!("\\g@addto@macro"), T_CS!("\\@startsection@hook"), T_CS!("\\maybe@end@abstract"),
        T_CS!("\\lx@begin@abstract"))
    }
  },
  locked => true);

  DefMacro!(
    "\\endabstract",
    "\\lx@end@abstract\\let\\maybe@end@abstract\\relax"
  );
  DefMacro!("\\maybe@end@abstract", "\\lx@end@abstract");
  DefMacro!(
    "\\lx@abstract@name",
    "\\format@title@abstract{\\abstractname}"
  ); // Redefine
  DefMacro!("\\abstractname", "Abstract");
  // Perl: `\format@title@abstract{}` -> `#1` (identity). SURPASS (html_feedback#6870,
  // OXIDIZED_DESIGN_DIVERGENCES #121): this hook is the designated place to format the
  // abstract heading, whose extracted `name=` is a plain-text label. When users write
  // `\renewcommand{\abstractname}{\centering {\large Abstract}}`, digesting the name
  // leaks `\centering`'s constructor reversion as literal text (`\centeringAbstract`);
  // Perl leaks it identically. Neutralize alignment declarations inside a group during
  // name extraction, mirroring the `titlepage` `Let('\centering','\relax')` precedent.
  // Font-size/series primitives (`\large`, `\bfseries`) already produce no text leak.
  DefMacro!(
    "\\format@title@abstract{}",
    "{\\let\\centering\\relax\\let\\raggedright\\relax\\let\\raggedleft\\relax#1}"
  );

  // Hmm, titlepage is likely to be hairy, low-level markup,
  // without even title, author, etc, specified as such!
  // Hmm, should this even redefine author, title, etc so that they
  // are simply output?
  // This is horrible hackery; What we really need, I think, is the
  // ability to bind some sort of "Do <this> when we create a text box"...
  // ON Second Thought...
  // For the time being, ignore titlepage!
  // Maybe we could do some of this if there is no title/author
  // otherwise defined? Ugh!

  //DefEnvironment('{titlepage}','');
  // Or perhaps it's better just to ignore the markers?
  //DefMacro('\titlepage','');
  //DefMacro('\endtitlepage','');

  // Or perhaps not....
  // There's a title and other stuff in here, but how could we guess?
  // Well, there's likely to be a sequence of <p><text font="xx" fontsize="yy">...</text></p>
  // Presumably the earlier, larger one is title, rest are authors/affiliations...
  // Particularly, if they start with a pseudo superscript or other "marker", they're probably
  // affil! For now, we just give an info message
  DefEnvironment!("{titlepage}", "<ltx:titlepage>#body",
    before_digest => {
      Let!("\\centering", "\\relax");
      assign_value("frontmatter_deferred", true, Some(Scope::Global));
      AddToMacro!("\\maketitle", "\\unwind@titlepage");
      // In titlepage, abstract is simpler: direct body. The
      // surrounding titlepage is internal_vertical, but if we leave
      // this redefinition without an explicit mode, paragraph entry
      // inside the abstract body pushes a horizontal frame, after
      // which BOUND_MODE no longer ends with "vertical" — and the
      // `$$math$$` display-math check in `TeX_Math.pool.ltxml:65`
      // (Rust mirror at tex_math.rs:447) silently fails to recognize
      // the second `$`, cascading to `_/^` "can only appear in math
      // mode" errors. Driver: hep-th0009013 (abstract inside
      // titlepage with `$$math$$` after preceding paragraph text).
      // The standard `\abstract` env at L4408 already sets
      // `mode => "internal_vertical"` for the same reason. Match it.
      DefEnvironment!("{abstract}", "<ltx:abstract>#body</ltx:abstract>",
        mode => "internal_vertical");
      // Perl (PR #2767): Titlepage env should redefine \abstract correctly.
      DefConstructor!("\\abstract{}", "<ltx:abstract>#1</ltx:abstract>");
    },
    before_digest_end => {
      digest(Tokens!(T_CS!("\\maybe@end@titlepage")))?
    },
    after_construct => sub[doc, _whatsit] {
      insert_frontmatter(doc)?;
    },
    // NOT locked: report/book define `{titlepage}` with `\newenvironment`, so
    // a class may legitimately `\def\titlepage{…}` as a plain vertical macro
    // (uwthesis.cls:610, used as `{… \titlepage }` at uwthesis.tex:95-102).
    // Perl latex_constructs.pool:1183 locks it, so the class `\def` was
    // refused and the bare `\titlepage` opened the internal_vertical
    // environment frame that the `}` then met ("Attempt to close a group
    // that switched to mode internal_vertical"; KPE #172). A document that
    // does not redefine it still gets this binding. Guard:
    // `perfect_kernel_batch54::titlepage_environment_is_overridable`.
    mode => "internal_vertical"
  );

  Tag!("ltx:titlepage", auto_close => true);

  // `\maybe@end@title` is a Rust-only addition (not in Perl); defined in
  // `latex_constructs_rust_only.rs` (which loads last, after this file).

  DefConstructor!("\\maybe@end@titlepage", sub[document,_args,_props] {
    document.maybe_close_element("ltx:titlepage")?;
  });
  DefConstructor!("\\unwind@titlepage", sub[document,_args,_props] {
    if let Some(titlepage) = document.maybe_close_element("ltx:titlepage")? {
      document.unwrap_nodes(titlepage)?;
    }
  });

  def_macro_noop("\\sectionmark{}")?;
  def_macro_noop("\\subsectionmark{}")?;
  def_macro_noop("\\subsubsectionmark{}")?;
  def_macro_noop("\\paragraphmark{}")?;
  def_macro_noop("\\subparagraphmark{}")?;
  def_macro_noop("\\@oddfoot")?;
  def_macro_noop("\\@oddhed")?;
  def_macro_noop("\\@evenfoot")?;
  def_macro_noop("\\@evenfoot")?;

  Ok(())
}
