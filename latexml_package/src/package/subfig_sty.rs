use latexml_core::{binding::def::builder::OptionValue, document::Document};

use crate::{
  engine::latex_constructs::{after_float, before_untyped_float},
  prelude::*,
};

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: subfig.sty.ltxml — 118 lines
  // Subfigure/subtable support with counter management

  // subfig loads caption (subfig.sty:124-142: `\RequirePackage{caption}` by default, `{caption3}`
  // with `caption=false`; there is no caption3 binding, so that option also gets caption's, whose
  // `\caption*` and `\captionof` pdflatex would leave undefined — lenient), so `\captionsetup` and
  // `\clearcaptionsetup` are caption's (caption3.sty:244-280). First, so subfig's own definitions
  // below win. Perl's own `\captionsetup[]{}` stub (subfig.sty.ltxml:107; KPE #319) replaced
  // caption's working one when subfig loaded after caption.
  // The `caption` option (subfig.sty:107-128 `\sf@split`, `\sf@setbool`: `0`, `false`, `no`, `off` turn it off) picks
  // which: caption's kernel alone (`caption3`, which the caption binding stands in for and marks
  // `\lx@caption@kernel@only`, caption_sty.rs) leaves a class's own `\@makecaption` in place, and caption replaces it —
  // a load TeX makes even after floatrow's `caption3`, so the mark goes (jfm_cls.rs; 1805.00129, 1612.03374). Guards
  // `perfect_kernel_batch78a::{jfm_copy_caption_kept_with_subfig_without_caption,
  // jfm_copy_caption_yields_to_caption_loaders_after_floatrow}`.
  // (the last `caption=` option wins, as subfig.sty's `\@for` over `\sf@split` sets the switch once per option)
  let use_caption = lookup_vecdeque("opt@subfig.sty").is_none_or(|options| {
    options
      .iter()
      .filter_map(|option| {
        let option = option.to_string();
        option
          .split_once('=')
          .filter(|(key, _)| key.trim() == "caption")
          .map(|(_, value)| !matches!(value.trim(), "0" | "false" | "no" | "off"))
      })
      .next_back()
      .unwrap_or(true)
  });
  if use_caption {
    RequirePackage!("caption");
    RawTeX!(r"\global\let\lx@caption@kernel@only\@undefined");
  } else if !lookup_bool("caption.sty_loaded") {
    RequirePackage!("caption3");
  }
  // subfig's caption-family keys (subfig.sty:163-167, 271-282).
  for key in ["listofformat", "listofindent", "listofnumwidth", "farskip", "topadjust",
              "captionskip", "nearskip"] {
    DefKeyVal!("caption", key, "");
  }
  // subfig.sty:208-225 `\ProcessPackageOptions` (run at :297): the package options, less the
  // `caption` and `config` keys (`\sf@split`, :188-195), are `\captionsetup[subfloat]` settings.
  DeclareOption!(None, {
    // The key is read expanded, as subfig.sty:125 and :217 `\edef` the whole option list before `\sf@split` (with
    // `\protect` = `\relax` at package load): a robust command whose body is not yet defined is an undefined-command
    // Error under pdflatex too (`[labelformat=\rbs]{subfig}`, `\DeclareRobustCommand\rbs{\laterdefined}`: 2 errors)
    // — unlike caption's handler (caption_sty.rs `current_option_text`), which reads the option as written.
    let option = Expand!(T_CS!("\\CurrentOption")).to_string();
    let key = option.split('=').next().unwrap_or_default().trim();
    if key != "caption" && key != "config" {
      // The option's tokens unexpanded (`\sf@split` hands them to `\captionsetup`), as caption_sty.rs passes its own.
      Digest!(r"\def\lx@subfig@optionsetup#1{\captionsetup[subfloat]{#1}}\expandafter\lx@subfig@optionsetup\expandafter{\CurrentOption}")?;
    }
  });
  ProcessOptions!();

  // Perl L26-27: \refstepcounter@noreset passes noreset=1 to RefStepCounter,
  // which steps the counter but skips the usual subcounter reset. Rust
  // previously aliased it to plain \refstepcounter (which DOES reset
  // subcounters), so a \subfloat inside an uncaptioned float temporarily
  // stepping the parent counter would zero out the subcounter the next
  // \subfloat was trying to increment.
  DefPrimitive!("\\refstepcounter@noreset{}", sub[(cs)] {
    let cs_expanded = Expand!(cs).to_string();
    RefStepCounter!(&cs_expanded, true)?;
  });

  // Perl L30-62: \newsubfloat — installs subfloat machinery for a float type.
  //
  // For figure / table the machinery is pre-baked below (counters,
  // \thesub<name>, \fnum@sub<name>, \p@sub<name>, \lx@subfloat@<name>,
  // and the lx@subfloat@@<name> environment). Perl's RawTeX trailer
  // calls \newsubfloat{figure} and \newsubfloat{table} at end-of-load
  // guarded by \@ifundefined{c@subfigure}/\@ifundefined{c@subtable}, so
  // those calls are no-ops once our pre-bake has run — exact parity.
  //
  // For a caller-defined float type (subfig.sty:239-263 `\newsubfloat{<type>}`; 2411.16612's
  // `\newsubfloat[captionskip=10pt]{listing}` for minted's `listing`) the same machinery is made at run time
  // (`define_subfloat`, Perl's NewCounter + Let + DefMacroI + DefEnvironmentI). Was a no-op, so a `\subfloat` in
  // such a float found no `\lx@subfloat@<type>` and its sub-floats were lost.
  // As subfig.sty:244-266: only for a type with no `sub<type>` counter yet (else subfig's warning, and nothing: a
  // document's `\newsubfloat{figure}` keeps the pre-built figure machinery), listed in `\sf@counterlist`, its keys the
  // `[sub<type>]` caption settings (`\captionsetup[sub#2]{#1}`). Guards
  // `perfect_kernel_batch78s::{newsubfloat_makes_sub_floats_of_a_new_type, newsubfloat_keys_and_an_existing_type}`.
  DefMacro!("\\newsubfloat[]{}",
    r"\@ifundefined{c@sub#2}{\lx@subfig@newsubfloat{#2}\edef\sf@counterlist{\@ifundefined{sf@counterlist}{}{\sf@counterlist,}sub#2}\captionsetup[sub#2]{#1}}{\PackageWarning{subfig}{The sub#2\space type is already defined.}}");
  DefPrimitive!("\\lx@subfig@newsubfloat{}", sub[(name)] {
    define_subfloat(&name.to_string())?;
  });

  // \subfloat — Perl L69-79
  DefMacro!("\\subfloat",
    "\\ifx\\@captype\\@undefined\\expandafter\\@gobble\\else\\expandafter\\@firstofone\\fi{\\sf@subfloat}");
  // subfig.sty:348-349: without an optional the caption is `\@empty` and none is printed
  // (:391, :410), the counter stepped all the same — the phantom sub-float
  // (`\lx@subfloat@<type>@phantom`, a `\phantomcaption` for its caption). Perl printed an empty
  // caption. Guard `perfect_kernel_gemini::bare_subfloat_has_a_phantom_caption`.
  DefMacro!("\\sf@subfloat",
    "\\@ifnextchar[{\\csname lx@subfloat@\\@captype\\endcsname}{\\csname lx@subfloat@\\@captype @phantom\\endcsname}");
  // caption3.sty (TL :1376-1387; the older copies papers ship alike) replaces subfig v1.3's
  // `\sf@subfloat` at begin document with one built on raw subfig internals (`\sf@ifpositiontop`,
  // `\sf@@subfloat`) this native subfloat does not have: it comes back after all begin-document
  // code (2003.01262 ships caption3 v1.8h: "undefined \sf@ifpositiontop", TooManyErrors).
  RawTeX!(r"\let\lx@sf@subfloat\sf@subfloat
\AddToHook{begindocument/end}{\let\sf@subfloat\lx@sf@subfloat}");
  // subfig L73: \sidesubfloat — side-by-side subfloat variant. Real def
  // wraps \subfloat with a minipage and lineup arg. Stub as plain
  // \subfloat so the subfloat machinery still kicks in. Witness 2309.00194.
  DefMacro!("\\sidesubfloat", "\\subfloat");

  // \subref — Perl L82-84
  DefMacro!("\\subref",       "\\@ifstar\\sf@@subref\\sf@subref");
  DefMacro!("\\sf@subref{}",  "\\ref{sub@#1}");
  DefMacro!("\\sf@@subref{}", "\\pageref{sub@#1}");

  // Caption setup stubs — Perl L86-90
  def_macro_noop("\\DeclareCaptionListOfFormat{}{}")?;
  def_macro_noop("\\DeclareSubrefFormat{}{}")?;
  // subfig.sty:476-479 `\listsubcaptions` tests for a star; Perl's `''` (subfig.sty.ltxml:105)
  // left it as text (KPE #316).
  def_macro_noop("\\listsubcaptions OptionalMatch:*")?;
  DefConditional!("\\ifmaincaptiontop");
  DefConditional!("\\iflx@donecaption");

  // Counter setup — Perl L36 uses NewCounter with `idprefix => 'sf'` and
  // `idwithin => $name` so subfigure/subtable get xml:ids like `F1.sf2`,
  // `T3.sf1`. The prior Rust port routed through `\newcounter{subfigure}
  // [figure]` via RawTeX, which skipped LaTeXML's id machinery entirely,
  // leaving subfigures with bare numeric ids that collided across floats.
  // Call NewCounter directly with the Perl options; the `\@ifundefined`
  // guard is dropped because NewCounter is itself idempotent (Perl L36
  // reads as a fresh-or-overwrite, mirroring `\newcounter` semantics).
  // subfig.sty:298-299 makes a sub-float only where no package made the counter (`\@ifundefined{c@subfigure}
  // {\newsubfloat{figure}}{}`), and only those join `\sf@counterlist` (below).
  let made_by_subfig: Vec<&str> = ["subfigure", "subtable"]
    .into_iter()
    .filter(|sub| !has_meaning(&T_CS!(s!("\\c@{sub}"))))
    .collect();
  NewCounter!("subfigure", "figure", idprefix => "sf", idwithin => "figure");
  NewCounter!("subtable", "table", idprefix => "sf", idwithin => "table");
  NewCounter!("subfigure@save");
  NewCounter!("subtable@save");

  // Perl L37/Perl-tail RawTeX `\@ifundefined{c@subfigure}{\newsubfloat
  // {figure}}{}` (and the table variant) executes \newsubfloat which
  // among other things does `Let('\ext@sub' . $name, '\ext@' . $name)`.
  // Since the Rust `\newsubfloat` is a stub, the figure/table cases are
  // pre-baked here so `\caption` machinery resolving `\ext@subfigure`
  // (e.g. via `subfig` callers) finds the figure-extension list. Sandbox
  // 0911.3405 (subfig + eptcs) cluster: 119 papers undefined
  // `\ext@subfigure`.
  Let!("\\ext@subfigure", "\\ext@figure");
  Let!("\\ext@subtable", "\\ext@table");
  // subfig.sty:261-263: `\newsubfloat` lists the sub-float counters it made in `\sf@counterlist`, which caption's
  // `\DeclareCaptionSubType` names in its error for a counter subfig made (caption3.sty:1848-1856) and caption's
  // `\captionof` refuses (caption.sty:348; 1506.06268). subfigure.sty's `subfigure` (loaded first: 2508.00020) is
  // not subfig's: caption accepts `\captionof{subfigure}` there, a panel caption formats as its own, "(a): First"
  // as pdflatex. Guard `perfect_kernel_batch77s::captionof_subfigure_sty_is_a_panel`.
  for sub in made_by_subfig {
    RawTeX!(&s!(r"\edef\sf@counterlist{{\@ifundefined{{sf@counterlist}}{{}}{{\sf@counterlist,}}{sub}}}"));
  }

  // Subfigure display macros
  DefMacro!("\\thesubfigure", "\\alph{subfigure}");
  DefMacro!("\\thesubtable", "\\alph{subtable}");
  // A sub-caption's label is its label format applied to `\thesub<type>` (caption3.sty:734-737):
  // the `[sub<type>]` setting over the `[subfloat]` one — subfig's package options among them
  // (subfig.sty:208-225, :333-334) — over subfig's default `labelformat=parens` (:285-288). Was a
  // hard-coded `(\thesubfigure)`: `labelformat=simple` over a parenthesized `\thesubfigure` printed
  // "((a))" (guard `perfect_kernel_gemini::subfig_label_follows_the_caption_label_format`).
  DefMacro!("\\lx@subfig@fnum{}{}", sub[(subtype, number)] {
    let keys = [s!("CAPTION_{}_labelformat", subtype.to_string()), s!("CAPTION_subfloat_labelformat")];
    caption_sty::sub_label_tokens(&keys, "parens", number)
  });
  DefMacro!("\\fnum@subfigure", "\\lx@subfig@fnum{subfigure}{\\thesubfigure}");
  DefMacro!("\\fnum@subtable", "\\lx@subfig@fnum{subtable}{\\thesubtable}");
  DefMacro!("\\p@subfigure", "\\thefigure");
  DefMacro!("\\p@subtable", "\\thetable");

  // The sub-caption is the kernel's (`\lx@caption`), not the document's `\caption`: subfig.sty:384-411 builds it with
  // `\sf@subcaption` (caption3's `\caption@make`), so a document's `\def\caption` does not reach it (the 1808.10020
  // mechanism; KNOWN_PERL_ERRORS #1126). `[list][caption]` captions with the second (subfig.sty:350-353 `\sf@@subfloat`,
  // the list entry the first); Perl's `\caption{#1}` printed the list entry (subfig.sty.ltxml:47; KNOWN_PERL_ERRORS #1130,
  // OXIDIZED_DESIGN_DIVERGENCES #876). Guard
  // `perfect_kernel_batch78t::subfig_caption_is_not_the_documents_caption`.
  // \lx@subfloat@figure — Perl L45-60. Perl's afterDigest on the inner
  // environment copies the final subfigure counter into subfigure@save so
  // \ContinuedFloat can restore it. Rust embeds `\setcounter{subfigure@save}
  // {\value{subfigure}}` at the end of the expansion; runs at the same
  // moment (after caption digests, so sub counter is final).
  DefMacro!("\\lx@subfloat@figure[][]{}", sub[(list, caption, body)] {
    Ok(subfloat_tokens("\\lx@subfloat@figure@", list, caption, body))
  });
  DefMacro!("\\lx@subfloat@figure@{}{}",
    "\\iflx@donecaption\\else\\refstepcounter@noreset{\\@captype}\\fi\\begin{lx@subfloat@@figure}#2#1\\end{lx@subfloat@@figure}\\iflx@donecaption\\else\\addtocounter{\\@captype}{\\m@ne}\\fi\\setcounter{subfigure@save}{\\value{subfigure}}");
  DefMacro!("\\lx@subfloat@figure@phantom{}",
    "\\iflx@donecaption\\else\\refstepcounter@noreset{\\@captype}\\fi\\begin{lx@subfloat@@figure}#1\\phantomcaption\\end{lx@subfloat@@figure}\\iflx@donecaption\\else\\addtocounter{\\@captype}{\\m@ne}\\fi\\setcounter{subfigure@save}{\\value{subfigure}}");
  // Perl L58-60: beforeDigest=>{beforeFloat('subfigure')}, afterDigest=>
  // {afterFloat + SetCounter('subfigure@save', CounterValue('subfigure'))}.
  // beforeFloat sets \@captype='subfigure' so the nested \caption steps the
  // sub-counter and emits sub-id labels. afterFloat finalizes caption
  // properties on the whatsit. The subfigure@save counter sync is still
  // handled in the macro-body `\setcounter{subfigure@save}{\value{subfigure}}`.
  DefEnvironment!("{lx@subfloat@@figure}",
    "^ <ltx:figure xml:id='#id'>#tags#body</ltx:figure>",
    mode => "internal_vertical",
    before_digest => { before_untyped_float("subfigure")?; },
    after_digest  => sub[whatsit] { after_float(whatsit); }
  );

  // \lx@subfloat@table — Perl L45-60 (table variant)
  DefMacro!("\\lx@subfloat@table[][]{}", sub[(list, caption, body)] {
    Ok(subfloat_tokens("\\lx@subfloat@table@", list, caption, body))
  });
  DefMacro!("\\lx@subfloat@table@{}{}",
    "\\iflx@donecaption\\else\\refstepcounter@noreset{\\@captype}\\fi\\begin{lx@subfloat@@table}#2#1\\end{lx@subfloat@@table}\\iflx@donecaption\\else\\addtocounter{\\@captype}{\\m@ne}\\fi\\setcounter{subtable@save}{\\value{subtable}}");
  DefMacro!("\\lx@subfloat@table@phantom{}",
    "\\iflx@donecaption\\else\\refstepcounter@noreset{\\@captype}\\fi\\begin{lx@subfloat@@table}#1\\phantomcaption\\end{lx@subfloat@@table}\\iflx@donecaption\\else\\addtocounter{\\@captype}{\\m@ne}\\fi\\setcounter{subtable@save}{\\value{subtable}}");
  DefEnvironment!("{lx@subfloat@@table}",
    "^ <ltx:table xml:id='#id'>#tags#body</ltx:table>",
    mode => "internal_vertical",
    before_digest => { before_untyped_float("subtable")?; },
    after_digest  => sub[whatsit] { after_float(whatsit); }
  );

  // \ContinuedFloat — Perl L98-102
  // Perl decrements the parent counter AND restores the sub-counter from
  // sub<captype>@save. Prior Rust only decremented the parent, leaving
  // the sub-counter at whatever value the prior float ended on, so a
  // \ContinuedFloat followed by a \subfloat would keep counting from the
  // stale sub index instead of rewinding.
  RawTeX!(r"\def\lx@subfig@continue@restore#1{\setcounter{sub#1}{\value{sub#1@save}}}");
  // subfig.sty:581-590: step back, caption's continuation (`\caption@ContinuedFloat`, a no-op
  // without caption, :591), then the sub-counter from its save. Witness 2605.17685; guards
  // `perfect_kernel_batch56::{continuedfloat_suppresses_one_step_per_real_float,
  // continuedfloat_counts_only_the_main_floats_steps}`.
  RawTeX!(r"\providecommand*\caption@ContinuedFloat[1]{}");
  DefMacro!("\\ContinuedFloat",
    r"\addtocounter{\@captype}{\m@ne}\expandafter\caption@ContinuedFloat\expandafter{\@captype}\expandafter\lx@subfig@continue@restore\expandafter{\@captype}");
});

/// subfig.sty:239-263 `\newsubfloat{<type>}` (Perl subfig.sty.ltxml:30-62): the `sub<type>` counter (ids `sf<n>` within
/// the float), its save counter, `\ext@sub<type>`, `\thesub<type>`, `\fnum@sub<type>` (the label format, as the
/// pre-baked figure and table ones), `\p@sub<type>`, `\lx@subfloat@<type>` and the `lx@subfloat@@<type>` environment: an
/// `ltx:figure`, `ltx:table` or `ltx:float` as Perl picks by the name.
fn define_subfloat(name: &str) -> Result<()> {
  use latexml_core::binding::def::{builder::EnvironmentBuilder, replacement};
  let sub = s!("sub{name}");
  NewCounter!(&sub, name, idprefix => "sf", idwithin => name);
  NewCounter!(&s!("{sub}@save"));
  Let!(T_CS!(s!("\\ext@{sub}")), T_CS!(s!("\\ext@{name}")));
  let define = |cs: String, params: &str, body: String| -> Result<()> {
    let cs = T_CS!(cs);
    let params = if params.is_empty() {
      None
    } else {
      parse_parameters(params, &cs, true)?
    };
    def_macro(
      cs,
      params,
      mouth::tokenize_internal(TeXString::assembled(body)),
      None,
    )?;
    Ok(())
  };
  define(s!("\\the{sub}"), "", s!("\\alph{{{sub}}}"))?;
  define(
    s!("\\fnum@{sub}"),
    "",
    s!("\\lx@subfig@fnum{{{sub}}}{{\\the{sub}}}"),
  )?;
  define(s!("\\p@{sub}"), "", s!("\\the{name}"))?;
  let step = r"\iflx@donecaption\else\refstepcounter@noreset{\@captype}\fi";
  let unstep = s!(
    r"\iflx@donecaption\else\addtocounter{{\@captype}}{{\m@ne}}\fi\setcounter{{{sub}@save}}{{\value{{{sub}}}}}"
  );
  // The kernel caption with the arguments as read, as the pre-built figure and table ones (`subfloat_tokens`;
  // 2411.16612's `\newsubfloat` types, OXIDIZED_DESIGN_DIVERGENCES #876).
  let builder = s!("\\lx@subfloat@{name}@");
  let cs = T_CS!(s!("\\lx@subfloat@{name}"));
  def_macro(
    cs,
    parse_parameters("[][]{}", &cs, true)?,
    Some(ExpansionBody::Closure(Rc::new(
      move |args: Vec<ArgWrap>| {
        let mut args = args.into_iter().map(ArgWrap::owned_tokens);
        let (list, caption) = (args.next().flatten(), args.next().flatten());
        let body = args.next().flatten().unwrap_or_default();
        Ok(subfloat_tokens(&builder, list, caption, body))
      },
    ))),
    None,
  )?;
  define(
    s!("\\lx@subfloat@{name}@"),
    "{}{}",
    s!(r"{step}\begin{{lx@subfloat@@{name}}}#2#1\end{{lx@subfloat@@{name}}}{unstep}"),
  )?;
  define(
    s!("\\lx@subfloat@{name}@phantom"),
    "{}",
    s!(r"{step}\begin{{lx@subfloat@@{name}}}#1\phantomcaption\end{{lx@subfloat@@{name}}}{unstep}"),
  )?;
  let element = if name.contains("figure") {
    "ltx:figure"
  } else if name.contains("table") {
    "ltx:table"
  } else {
    "ltx:float"
  };
  let ops = Rc::new(replacement::parse_replacement(&s!(
    "^ <{element} xml:id='#id'>#tags#body</{element}>"
  ))?);
  let sub_for_before = sub.clone();
  EnvironmentBuilder::new(&s!("{{lx@subfloat@@{name}}}"))?
    .replacement(Rc::new(move |document: &mut Document, args, props| {
      replacement::apply_ops(&ops, document, args, props)
    }))
    .set_option("mode", OptionValue::Str("internal_vertical".into()))?
    .before_digest(Rc::new(move || {
      before_untyped_float(&sub_for_before)?;
      Ok(Vec::new())
    }))
    .after_digest(Rc::new(|whatsit: &mut Whatsit| {
      after_float(whatsit);
      Ok(Vec::new())
    }))
    .install()
}

/// `<builder>{<caption>}{<body>}` for subfig's `\subfloat[list][caption]{body}` as read (subfig.sty:350-353): the
/// kernel caption `\lx@caption[list]{caption}` with both, `\lx@caption{list}` with one (`\sf@subfloat` sends only a
/// `\subfloat` with an optional here).
fn subfloat_tokens(
  builder: &str,
  list: Option<Tokens>,
  caption: Option<Tokens>,
  body: Tokens,
) -> Tokens {
  let mut tokens = vec![T_CS!(builder), T_BEGIN!(), T_CS!("\\lx@caption")];
  let list = list.unwrap_or_default();
  match caption {
    Some(caption) => {
      tokens.push(T_OTHER!("["));
      tokens.extend(list.unlist());
      tokens.extend([T_OTHER!("]"), T_BEGIN!()]);
      tokens.extend(caption.unlist());
    },
    None => {
      tokens.push(T_BEGIN!());
      tokens.extend(list.unlist());
    },
  }
  tokens.extend([T_END!(), T_END!(), T_BEGIN!()]);
  tokens.extend(body.unlist());
  tokens.push(T_END!());
  Tokens::new(tokens)
}
