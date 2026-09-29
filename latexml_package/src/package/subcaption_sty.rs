use crate::{
  engine::latex_constructs::{after_float, before_float, before_float_ex},
  prelude::*,
};

/// Perl subcaption.sty.ltxml L66/76/86/96: `properties => sub { (width => $_[2]) }`.
/// Perl sets this on ALL four sub-float envs ({subfigure}, {subfigure*},
/// {subtable}, {subtable*}); the Rust-only {subcaptionblock}/{subcaptionblock*}
/// aliases inherit the same semantics. Records the sub-float's `{Dimension}`
/// argument (args[1]) as the box's `width` property, so
/// `arrange_panels_and_breaks` can compute the per-row layout from actual panel
/// widths (without it, a panel reports its natural content width — the full
/// float width — and every panel starts its own row; arXiv 2605.00347). Mirrors
/// Perl storing the digested Dimension object.
fn subcaption_width_props(args: &[Option<Digested>]) -> Result<SymHashMap<Stored>> {
  let mut props: SymHashMap<Stored> = SymHashMap::default();
  if let Some(w) = args
    .get(1)
    .and_then(|a| a.as_ref())
    .and_then(|a| Dimension::spec_to_f64(&a.to_string()).ok())
    // #6903: a non-positive Dimension (e.g. `\subcaptionbox`'s `{0pt}` default,
    // subcaption_sty L219) must NOT pin the panel width to 0 — that reads as a
    // zero-width box in `arrange_panels` and the panels never share a row. Skip
    // it so `panel_width` falls back to the panel's natural content width.
    .filter(|w| *w > 0.0)
  {
    props.insert("width", Stored::Dimension(Dimension::new_f64(w)));
  }
  Ok(props)
}

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: subcaption.sty.ltxml
  // Provides subfigure/subtable environments and \subcaption, \subfloat, \subcaptionbox, \subref

  RequirePackage!("caption");

  //======================================================================
  // Counters and formatting
  // subcaption.sty:214-222: the package options are `\captionsetup[sub]` settings, over subcaption's
  // own defaults for sub-captions (`labelformat=parens`, …).
  DeclareOption!(None, {
    Digest!("\\edef\\lx@subcaption@option{\\noexpand\\captionsetup[sub]{\\CurrentOption}}\\lx@subcaption@option")?;
  });
  ProcessOptions!();
  RawTeX!(r"\DeclareCaptionLabelFormat{subsimple}{#2}\DeclareCaptionLabelFormat{subparens}{(#2)}");
  // A sub-caption's label (`\fnum@sub<type>`) is its label format applied to `\thesub<type>`
  // (caption3.sty:734-737), the format being the `[sub<type>]` setting, else the `[sub]` one (the
  // package options among them), else subcaption's `parens` (subcaption.sty:218-222): "(a)" by
  // default, "a" with `labelformat=simple` — which authors pair with a parenthesized `\thesubfigure`
  // (2605.01394).
  DefMacro!("\\lx@subcaption@fnum{}{}", sub[(subtype, number)] {
    let keys = [s!("CAPTION_{}_labelformat", subtype.to_string()), s!("CAPTION_sub_labelformat")];
    caption_sty::sub_label_tokens(&keys, "parens", number)
  });
  // `\subcaption@DeclareType` (subcaption.sty:226-230) declares a sub-type only when its counter is
  // new: after subfigure.sty (whose `\thesubfigure` is `(\alph{subfigure})`, subfigure.sty:118)
  // the counter, its number and label stay subfigure's (2605.01846).
  if !has_meaning(&T_CS!("\\c@subfigure")) {
    NewCounter!("subfigure", "figure", idprefix => "sf", idwithin => "figure");
    // `\DeclareCaptionSubType` makes `\the<sub>` the bare letter and `\p@<sub>` the parent number
    // (caption3.sty:1803-1806); the parentheses are the label format's, so `\ref` prints "1a" and
    // the caption "(a)". Perl bakes them into the counter (subcaption.sty.ltxml:27-28), giving
    // "1(a)" (KNOWN_PERL_ERRORS #330; 2605.01361, 2605.01961, 2605.02222); subfig_sty.rs has the
    // same shape.
    DefMacro!("\\thesubfigure", "\\alph{subfigure}");
    DefMacro!("\\fnum@subfigure", "\\lx@subcaption@fnum{subfigure}{\\thesubfigure}");
    Let!("\\p@subfigure",   "\\thefigure");
  }
  if !has_meaning(&T_CS!("\\c@subtable")) {
    NewCounter!("subtable",  "table",  idprefix => "st", idwithin => "table");
    DefMacro!("\\thesubtable",  "\\alph{subtable}");
    DefMacro!("\\fnum@subtable",  "\\lx@subcaption@fnum{subtable}{\\thesubtable}");
    Let!("\\p@subtable",    "\\thetable");
  }
  Let!("\\ext@subfigure", "\\ext@figure");
  Let!("\\ext@subtable",  "\\ext@table");

  DefMacro!("\\fnum@font@float",         "\\small");
  DefMacro!("\\format@title@font@float", "\\small");

  DefMacro!("\\fnum@font@subfigure",         "\\fnum@font@figure");
  DefMacro!("\\fnum@font@subtable",          "\\fnum@font@table");
  DefMacro!("\\format@title@font@subfigure", "\\format@title@font@figure");
  DefMacro!("\\format@title@font@subtable",  "\\format@title@font@table");

  // Perl: \format@title@subfigure and \format@title@subtable use " " separator (not ": ")
  DefMacro!(
    "\\format@title@subfigure{}",
    "\\lx@tag[][ ]{\\lx@fnum@@{subfigure}}#1"
  );
  DefMacro!(
    "\\format@title@subtable{}",
    "\\lx@tag[][ ]{\\lx@fnum@@{subtable}}#1"
  );

  //======================================================================
  // \subcaption — Perl L47-56: if \@captype is defined, prepend "sub" (unless already
  // sub-prefixed) locally, then delegate to \caption.
  DefMacro!("\\subcaption OptionalMatch:* []{}", sub[(_star, opt, caption)] {
    let mut tokens = Vec::new();
    if has_meaning(&T_CS!("\\@captype")) {
      let ctype = do_expand(Tokens!(T_CS!("\\@captype")))?.to_string();
      let ctype = ctype.trim().to_string();
      if !ctype.is_empty() && !ctype.starts_with("sub") {
        // Local redefinition via \def\@captype{sub<ctype>} tokens.
        tokens.push(T_CS!("\\def"));
        tokens.push(T_CS!("\\@captype"));
        tokens.push(T_BEGIN!());
        tokens.extend(Explode!(s!("sub{}", ctype)));
        tokens.push(T_END!());
      }
    }
    tokens.push(T_CS!("\\caption"));
    if let Some(o) = opt {
      tokens.push(T_OTHER!("["));
      tokens.extend(o.unlist());
      tokens.push(T_OTHER!("]"));
    }
    tokens.push(T_BEGIN!());
    tokens.extend(caption.unlist());
    tokens.push(T_END!());
    Ok(Tokens::new(tokens))
  });

  //======================================================================
  // Subfigure environments
  // Perl: beforeFloat('subfigure', preincrement => 'figure') / afterFloat
  //
  // EMULATE LaTeX's `\newenvironment` "already defined" guard. subcaption and
  // the (unsupported) subfigure package are officially INCOMPATIBLE: both want
  // to own `\subfigure`/`\subtable`. subfigure.sty binds them as
  // `\subfigure[][]{}` / `\subtable[][]{}` MACROS (self-contained, one
  // mandatory body); subcaption binds them as `{subfigure}[]{Dimension}` /
  // `{subtable}[]{Dimension}` ENVIRONMENTS. In real LaTeX, subcaption declares
  // these via `\newenvironment{subfigure}`, which REFUSES to redefine an
  // already-defined `\subfigure` (it raises "Command \subfigure already defined"
  // and keeps subfigure.sty's macro); our `DefEnvironment!` is unconditional
  // (like `\def`, not `\newenvironment`), so it used to CLOBBER the macro.
  // Then a `\subfigure[]{\includegraphics{...}}` (subfigure.sty's macro syntax)
  // reparsed as `\begin{subfigure}` with `{\includegraphics{...}}` read as the
  // `{Dimension}` (→ "Missing number", treated as zero) and the environment
  // opened with no matching `\end{subfigure}` in the source, leaking an
  // internal_vertical group that swallowed the rest of the document (figures,
  // sections, bibliography). Vendored Perl clobbers the same way and *times out*
  // on the witness — upstream candidate (KNOWN_PERL_ERRORS #48). So mirror the
  // `\newenvironment` guard: only bind these two envs when the colliding command
  // is not already defined, and Warn about the package conflict when it is.
  // (`{subfigure*}`/`{subtable*}`/`{subcaptionblock}` names don't collide with
  // subfigure.sty and stay unconditional.) Witness 2507.21938.
  let subfigure_predefined = has_meaning(&T_CS!("\\subfigure"));
  let subtable_predefined = has_meaning(&T_CS!("\\subtable"));
  if subfigure_predefined {
    // subcaption.sty:229-230 notes it at Info: its sub-type loop declares no `{subfigure}` for a
    // counter another package made.
    Info!("unexpected", "subcaption",
      "the counter `subfigure' was already defined by subfigure.sty, so subcaption's \
       {subfigure} environment is not installed; subfigure.sty's \\subfigure macro is kept");
  } else {
    DefEnvironment!("{subfigure}[]{Dimension}",
      "^<ltx:figure xml:id='#id' inlist='#inlist' ?#1(placement='#1')>\
        #tags\
        #body\
      </ltx:figure>",
      mode => "internal_vertical",
      properties => sub[args] { subcaption_width_props(args) },
      before_digest => { before_float("subfigure", Some("figure")); },
      after_digest => sub[whatsit] { after_float(whatsit); }
    );
  }

  // Perl L77: `{subfigure*}` passes double => 1, widening \hsize to
  // \textwidth for two-column spans (vs \columnwidth).
  DefEnvironment!("{subfigure*}[]{Dimension}",
    "^<ltx:figure xml:id='#id' inlist='#inlist' ?#1(placement='#1')>\
      #tags\
      #body\
    </ltx:figure>",
    mode => "internal_vertical",
    properties => sub[args] { subcaption_width_props(args) },
    before_digest => { before_float_ex("subfigure", Some("figure"), true); },
    after_digest => sub[whatsit] { after_float(whatsit); }
  );

  // subcaption v1.3+ added `{subcaptionblock}` as a sibling of `{subfigure}`
  // — same signature and semantics, just a more-generic name. Witness
  // 2306.17516 + 2 stage-2 papers (`undefined:{subcaptionblock}`).
  DefEnvironment!("{subcaptionblock}[]{Dimension}",
    "^<ltx:figure xml:id='#id' inlist='#inlist' ?#1(placement='#1')>\
      #tags\
      #body\
    </ltx:figure>",
    mode => "internal_vertical",
    properties => sub[args] { subcaption_width_props(args) },
    before_digest => { before_float("subfigure", Some("figure")); },
    after_digest => sub[whatsit] { after_float(whatsit); }
  );
  DefEnvironment!("{subcaptionblock*}[]{Dimension}",
    "^<ltx:figure xml:id='#id' inlist='#inlist' ?#1(placement='#1')>\
      #tags\
      #body\
    </ltx:figure>",
    mode => "internal_vertical",
    properties => sub[args] { subcaption_width_props(args) },
    before_digest => { before_float_ex("subfigure", Some("figure"), true); },
    after_digest => sub[whatsit] { after_float(whatsit); }
  );

  // Same `\newenvironment` guard for `\subtable` (subfigure.sty's `\subtable`
  // macro), for the same reason as `\subfigure` above.
  if subtable_predefined {
    Info!("unexpected", "subcaption",
      "the counter `subtable' was already defined by subfigure.sty, so subcaption's \
       {subtable} environment is not installed; subfigure.sty's \\subtable macro is kept");
  } else {
    DefEnvironment!("{subtable}[]{Dimension}",
      "^<ltx:table xml:id='#id' inlist='#inlist' ?#1(placement='#1')>\
        #tags\
        #body\
      </ltx:table>",
      mode => "internal_vertical",
      properties => sub[args] { subcaption_width_props(args) },
      before_digest => { before_float("subtable", Some("table")); },
      after_digest => sub[whatsit] { after_float(whatsit); }
    );
  }

  // Perl L97: `{subtable*}` passes double => 1 (see {subfigure*} above).
  DefEnvironment!("{subtable*}[]{Dimension}",
    "^<ltx:table xml:id='#id' inlist='#inlist' ?#1(placement='#1')>\
      #tags\
      #body\
    </ltx:table>",
    mode => "internal_vertical",
    properties => sub[args] { subcaption_width_props(args) },
    before_digest => { before_float_ex("subtable", Some("table"), true); },
    after_digest => sub[whatsit] { after_float(whatsit); }
  );

  //======================================================================
  // \subfloat — a sub-float with its caption. subcaption.sty:278-291 reads `[list][caption]{body}`
  // through `\subcaptionbox`: a lone optional is the caption (`\subcaptionbox{#1}`, no list entry),
  // two are `\subcaptionbox[{#1}]{#2}`; the sub-float follows `\@captype`, so a `\subfloat` in a
  // table is a subtable. Perl's `\subfloat[][]{}` (subcaption.sty.ltxml:104, witness 2111.00007)
  // read a lone optional as the list entry, so `\subfloat[Caption]{…}` came out uncaptioned (KPE
  // #323; fixture svg_subfloat_2563). Without an optional subcaption sets a `\phantomcaption`
  // (:293-300), as here (`\lx@subcaption@subfloat@phantom`; Perl printed an empty caption).
  // `\columnwidth` stands in for the box's natural width (Perl L102-103).
  DefMacro!("\\subfloat",
    "\\kernel@ifnextchar[\\lx@subcaption@subfloat@list\\lx@subcaption@subfloat@phantom");
  DefMacro!("\\lx@subcaption@subfloat@list[]",
    "\\kernel@ifnextchar[{\\lx@subcaption@subfloat@caption{#1}}{\\lx@subcaption@subfloat@@{}{#1}}");
  DefMacro!("\\lx@subcaption@subfloat@caption{}[]", "\\lx@subcaption@subfloat@@{#1}{#2}");
  DefMacro!("\\lx@subcaption@subfloat@@{}{}{}", sub[(list, caption, body)] {
    subfloat_tokens(list, Some(caption), body)
  });
  // Without an optional, subcaption's `\subfloat` boxes the body with a `\phantomcaption`
  // (subcaption.sty:293-300): the sub-float is numbered and tagged, but prints no caption and
  // enters no list. `\subfloat[]{…}` (an empty caption) still prints its "(b)".
  DefMacro!("\\lx@subcaption@subfloat@phantom{}", sub[(body)] {
    subfloat_tokens(Tokens!(), None, body)
  });

  //======================================================================
  // \subcaptionbox — delegates to sub<captype> environment
  DefMacro!("\\subcaptionbox",
    "\\expandafter\\@@subcaptionbox\\expandafter{\\@captype}"
  );
  DefMacro!("\\@@subcaptionbox{} []{} Optional:0pt []{}",
    "\\begingroup\\csname sub#1\\endcsname{#4}\
     #6\
     \\caption{#3}\
     \\ifx.#2.\\else\\lx@subcaption@addinlist{#2}\\fi\
     \\csname endsub#1\\endcsname\\endgroup"
  );

  //======================================================================
  // Perl L116-117: \lx@subcaption@addinlist — `^ inlist='#1'` floats from the CURRENT node
  // (the subfigure) to the first one that can take `inlist` (Document.pm:1080-1092). Setting
  // it on the node's parent instead renamed the enclosing figure's list, so every figure
  // holding a `\subfloat[entry]{…}` or `\subcaptionbox[entry]{…}{…}` left the List of Figures.
  DefConstructor!("\\lx@subcaption@addinlist{}", "^ inlist='#1'");

  //======================================================================
  // \subref — delegates to \ref
  DefMacro!("\\subref OptionalMatch:* Semiverbatim", "\\ref{#2}");

  //======================================================================
  // \DeclareCaptionSubType — stub (should be in caption/caption3)
  def_macro_noop("\\DeclareCaptionSubType OptionalMatch:* [] {}")?;
});

/// `{list}{caption}{body}` in a `sub<type>` environment. The type is `\@captype` less a leading
/// `sub` (inside a `{subfigure}` it is `subfigure`), as Perl's `\subcaption` resolves it (L50-53);
/// `figure` — Perl's only choice — outside a float, or when no `sub<type>` environment exists (a
/// `\newfloat` type). No `caption` is subcaption's phantom one (subcaption.sty:293-300): a
/// `\phantomcaption` where the caption would be.
fn subfloat_tokens(list: Tokens, caption: Option<Tokens>, body: Tokens) -> Result<Tokens> {
  let mut ctype = String::from("figure");
  if has_meaning(&T_CS!("\\@captype")) {
    let captype = do_expand(Tokens!(T_CS!("\\@captype")))?.to_string();
    let captype = captype.trim();
    let base = captype.strip_prefix("sub").unwrap_or(captype);
    if !base.is_empty()
      && (is_defined(&format!("\\sub{base}")) || is_defined(&format!("\\begin{{sub{base}}}")))
    {
      ctype = base.to_string();
    }
  }
  let env = Tokens!(T_BEGIN!(), Explode!(s!("sub{}", ctype)), T_END!());
  let mut tokens = vec![T_CS!("\\begin")];
  tokens.extend(env.clone().unlist());
  tokens.extend([T_BEGIN!(), T_CS!("\\columnwidth"), T_END!()]);
  tokens.extend(body.unlist());
  match caption {
    Some(caption) => {
      tokens.extend([T_CS!("\\caption"), T_BEGIN!()]);
      tokens.extend(caption.unlist());
      tokens.push(T_END!());
    },
    None => tokens.push(T_CS!("\\phantomcaption")),
  }
  if !list.is_empty() {
    tokens.extend([T_CS!("\\lx@subcaption@addinlist"), T_BEGIN!()]);
    tokens.extend(list.unlist());
    tokens.push(T_END!());
  }
  tokens.push(T_CS!("\\end"));
  tokens.extend(env.unlist());
  Ok(Tokens::new(tokens))
}
