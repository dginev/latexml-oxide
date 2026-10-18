use crate::prelude::*;

/// Whether the float in effect is an aastex `{longrotatetable}` itself, not a float nested in it: its frame
/// (`lx@lrt@frame`, set at its start by aas_support_sty.rs) is the current float's (`lx@float@frame`, each float's
/// own, `begin_float`).
pub(crate) fn in_longrotatetable_body() -> bool {
  match (lookup_value("lx@lrt@frame"), lookup_value("lx@float@frame")) {
    (Some(lrt), Some(float)) => {
      !matches!(lrt, Stored::None) && lrt.to_string() == float.to_string()
    },
    _ => false,
  }
}

/// What a table-note mark shows, the key that pairs a `\tablenotetext` with its `\tablenotemark`s (latex_constructs
/// `relocate_footnote` matches their `mark`s, ignoring spaces) and the mark the note prints: the characters and
/// symbols as set, in text or math, a command's shown arguments and a formula's body read for theirs (`$\dagger$`,
/// `^\dagger` and `\dag` are "†", `\rm{b}` and `\textrm{b}` are "b", `\hyperlink{tn5}{e}` is "e" and
/// `\raisebox{1pt}{g}` "g": a value, a key-value list or a semiverbatim name sets no character), its spaces collapsed
/// (`c, d` prints "c, d", as TeX sets it, and pairs with `c,d`). The one function serves both commands, so a mark
/// pairs by what it shows whatever its source. Math's `*` and `\ast` are TeX's one asterisk (plain.tex
/// `\mathcode`\*="2203`, `\mathchardef\ast="2203`), set here as U+2217: an asterisk, as a text `*` is.
fn shown_mark(digested: &Digested) -> String {
  /// Parameter types whose argument is a value, a name or a key-value list, which sets no character.
  const UNSHOWN: &[&str] = &[
    "Number",
    "NumExpr",
    "Dimension",
    "DimExpr",
    "Glue",
    "GlueExpr",
    "MuDimension",
    "MuGlue",
    "MuExpr",
    "Length",
    "Float",
    "TempboxaDimension",
    "GraphixDimension",
    "GraphixDimensions",
    "SetlengthDimension",
    "SetlengthGlue",
    "Keyword",
    "CSName",
    "CSNameQuiet",
    "Token",
    "XToken",
    "DefToken",
    "Undigested",
    "OptionalUndigested",
    "TeXFileName",
    "DirectoryList",
    "PackageOptions",
  ];
  fn shows(parameter: &Parameter) -> bool {
    let unshown = parameter.semiverbatim.is_some()
      || with(parameter.name, |name| {
        UNSHOWN.contains(&name) || name.contains("KeyVals")
      });
    !unshown
      && parameter
        .inner
        .as_ref()
        .is_none_or(|inner| inner.get_parameters().into_iter().any(shows))
  }
  fn shown(digested: &Digested, out: &mut String) {
    match digested.data() {
      DigestedData::List(list) => list.borrow().boxes.iter().for_each(|item| shown(item, out)),
      DigestedData::RegisterValue(_) | DigestedData::KeyVals(_) => {},
      DigestedData::Whatsit(whatsit) => {
        let whatsit = whatsit.borrow();
        let body = whatsit.get_body().ok().flatten();
        let args = whatsit.get_args();
        if args.iter().all(Option::is_none) && body.is_none() {
          out.push_str(&digested.to_attribute());
        } else {
          // One argument per valued parameter (`Parameters::read_arguments_and_digest`); without the parameters
          // every argument is read.
          let definition = whatsit.get_definition();
          let parameters: Vec<&Parameter> = definition
            .get_parameters()
            .map(|parameters| {
              parameters
                .get_parameters()
                .into_iter()
                .filter(|p| !p.novalue)
                .collect()
            })
            .unwrap_or_default();
          let aligned = parameters.len() == args.len();
          for (index, arg) in args.iter().enumerate() {
            if let Some(arg) = arg
              && (!aligned || shows(parameters[index]))
            {
              shown(arg, out);
            }
          }
          if let Some(body) = body {
            shown(&body, out);
          }
        }
      },
      _ => out.push_str(&digested.to_attribute()),
    }
  }
  let mut text = String::new();
  shown(digested, &mut text);
  text
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
    .chars()
    .map(|c| if c == '\u{2217}' { '*' } else { c })
    .collect()
}

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: deluxetable.sty.ltxml — deluxetable environment for AAS styles

  //======================================================================
  // 2.15.1 The deluxetable Environment
  DefMacro!("\\dummytable", "\\refstepcounter{table}");

  // Perl: DefMacroI('\deluxetable', '{}', '\set@deluxetable@template{#1}\def\@deluxetable@header{}\begin{table}');
  DefMacro!("\\deluxetable{}",
    "\\set@deluxetable@template{#1}\\def\\@deluxetable@header{}\\lx@deluxe@begin{table}");
  DefMacro!("\\enddeluxetable", "\\lx@deluxe@pending@caption\\spew@tblnotes\\end{table}");

  // The table opens with its placement (`\begin{deluxetable*}{lrr}[h]`, aastex701.cls `\xdeluxetable#1[#2]`;
  // 1511.09213) read by `\begin{table}`, then the stored caption is taken inside its group.
  DefMacro!("\\lx@deluxe@begin{}",
    r"\@ifnextchar[{\lx@deluxe@begin@opt{#1}}{\begin{#1}\lx@deluxe@stored@caption}");
  DefMacro!("\\lx@deluxe@begin@opt{}[]", r"\begin{#1}[#2]\lx@deluxe@stored@caption");
  // star variant — use runtime def_macro for star-containing names
  {
    let params = parse_parameters("{}", &T_CS!("\\deluxetable*"), true)?;
    let expansion = TokenizeInternal!(
      "\\set@deluxetable@template{#1}\\def\\@deluxetable@header{}\\lx@deluxe@begin{table}");
    def_macro(T_CS!("\\deluxetable*"), params, ExpansionBody::Tokens(expansion), None)?;
  }
  {
    let expansion = TokenizeInternal!("\\lx@deluxe@pending@caption\\spew@tblnotes\\end{table}");
    def_macro(T_CS!("\\enddeluxetable*"), None, ExpansionBody::Tokens(expansion), None)?;
  }

  // Perl: DefMacro('\set@deluxetable@template AlignmentTemplate', sub { AssignValue(...); }).
  // Rust uses DefPrimitive — same AssignValue side effect at stomach time.
  // WISDOM #44: DefMacro↔DefPrimitive is NOT universally equivalent; safe
  // here because `\set@deluxetable@template` is only emitted by the
  // `\deluxetable{...}` expansion and consumed immediately in document
  // body, never captured by `\edef`.
  // WISDOM #44 verified 2026-04-23: zero `\edef`/`\ifx`/`\expandafter`
  // uses of `\set@deluxetable@template` across LaTeXML/lib + ar5iv-bindings.
  DefPrimitive!("\\set@deluxetable@template AlignmentTemplate", sub[(template)] {
    AssignValue!("@deluxetable@template", template);
  });

  // (`\iflx@deluxetable@data`: the data is open, for an end that closes it itself — AASTeX 3's `{planotable}`,
  // aas_support_sty.rs)
  RawTeX!("\\newif\\iflx@deluxetable@data");
  // The caption first, where the class prints it (`\lx@deluxe@print@caption`, below).
  DefMacro!("\\startdata",
    "\\lx@deluxe@print@caption\\global\\lx@deluxetable@datatrue\
     \\bgroup\\@deluxetable@bindings\\@@deluxetabular\\lx@begin@alignment\\hline\\hline\\@deluxetable@header");
  DefMacro!("\\enddata",
    "\\global\\lx@deluxetable@datafalse\\\\\\hline\\lx@end@alignment\\@end@deluxetabular\\egroup");

  // Perl: DefPrimitive('\@deluxetable@bindings', sub { tabularBindings(LookupValue('@deluxetable@template')); });
  DefPrimitive!("\\@deluxetable@bindings", {
    let template_stored = lookup_value("@deluxetable@template");
    if let Some(Stored::Template(template_rc)) = template_stored {
      let template = (*template_rc).clone();
      tabular_bindings(template, SymHashMap::default(), HashMap::default())?;
    }
  });

  DefConstructor!("\\@@deluxetabular DigestedBody",
    "#1",
    reversion => "\\begin{tabular}[#1]{#2}#3\\end{tabular}",
    before_digest => { bgroup(); },
    mode => "restricted_horizontal"
  );
  DefPrimitive!("\\@end@deluxetabular", {
    egroup()?;
  });

  //======================================================================
  // 2.15.2 Preamble to the deluxetable

  DefRegister!("\\pt@width", Dimension!("6pt"));
  DefRegister!("\\pt@line", Dimension!("0pt"));
  DefRegister!("\\pt@column", Dimension!("0pt"));
  DefRegister!("\\pt@nlines", Dimension!("0pt"));
  DefRegister!("\\pt@ncol", Dimension!("0pt"));
  DefRegister!("\\pt@page", Dimension!("0pt"));

  def_macro_noop("\\tabletypesize{}")?;
  def_macro_noop("\\rotate")?;
  // \tabletail{text} — text shown at the bottom of every table page
  // (e.g. "Continued on next page"). HTML output is single-page so
  // we preserve the text as ltx:note role='tabletail' rather than
  // gobbling — author body, not config.
  DefMacro!("\\tabletail{}",
    "\\@add@frontmatter{ltx:note}[role=tabletail]{#1}");
  // aastex62.cls:1849-1858 `\def\tablewidth#1{\ifdim#1=\z@ \else …\fi}`: for `0pt`, the common use, it
  // expands to nothing, so `\tablewidth{0pt}` at the start of a `tabular` row leaves the row's `\hline` a
  // `\noalign`. Perl's `\pt@width=#1\relax` (deluxetable.sty.ltxml:70) is an assignment, which opened a cell and
  // made the `\hline` misplaced (1805.09353). Guard: `perfect_kernel_batch65::tablewidth_zero_is_expandable`.
  DefMacro!("\\tablewidth{}", r"\ifdim#1=\z@\else\pt@width=#1\relax\fi");
  def_macro_noop("\\tableheadfrac{}")?;
  DefMacro!("\\tablenum{}", "\\def\\thetable{#1}");

  def_macro_noop("\\tablecolumns{Number}")?;

  // `\tablecaption` is set where the class prints it. Every AAS class stores the caption (AASTeX 5.x aastex.cls:1280
  // `\gdef\pt@caption`; emulateapj.cls:1293-1294 `\gdef\@tablecaption`, 1010.4329's copy :1217-1218; aastex701.cls
  // :8417-8418, aastex631.cls alike) and prints the store at its table's print point: AASTeX 5's `\startdata` (:1507
  // `\caption{\pt@caption}`, every table, an empty store too), emulateapj's `\tablehead` and `\startdata` (:1325, :1333,
  // every table), AASTeX 6+'s `\startdata` (aastex701.cls:11391-11399, only a non-empty store; a long table —
  // `\startlongtable`, `{longrotatetable}` — always, :11568/:11575, its flag set until the long table ends, :11609).
  // Perl lets it be `\caption` (deluxetable.sty.ltxml:77): outside a float an Error (1010.4329, 1903.10014, 2204.03327),
  // in a float set where it stands (KNOWN_PERL_ERRORS #964). Here (`\lx@deluxe@tablecaption`):
  // - in a deluxetable, before its print point, it is this table's caption (`\lx@deluxe@own`), set at the print point
  //   (`\lx@deluxe@print@caption`, the class's place: before the tabular); given in the data (after the print point)
  //   it is kept for the table's end, unnumbered (the table has its number; OXIDIZED_DESIGN_DIVERGENCES #664);
  // - in a `{longtable}` it is the longtable's caption while the longtable has none — numbered, the longtable's own
  //   `\caption` (`\lx@longtable@caption`, tag and list entry): the table takes its number there, as the PDF counts it
  //   (repro class-bindings/longtable_tablecaption_is_numbered: `\ref` 1, the next table 2) — else stored, never
  //   printed there (1903.10014: the longtable's `\caption` stays; aastex.cls:1280/:1507, aastex701.cls:8418);
  // - directly in an aastex `{longrotatetable}` or outside a float it is stored; in another float it is that float's
  //   `\caption`, as Perl.
  // The emulateapj and AASTeX 5 store (`\iflx@deluxe@persistent`, the default) is the classes' own: global, never
  // emptied, so a later table without a caption of its own prints it again, as pdflatex (OXIDIZED_DESIGN_DIVERGENCES
  // #740). AASTeX 6+ (aastex_cls.rs, `\lx@deluxe@persistentfalse`) empty the store at every deluxetable's start
  // (aastex701.cls:11303-11304), so a caption given before the table is lost in the PDF; it is kept here (user ruling 2,
  // 2026-10-18; OXIDIZED_DESIGN_DIVERGENCES #739), scoped to its group, and — as the PDF prints no caption there — set
  // unnumbered (`\lx@deluxe@unnumbered`), so every later number and `\ref` is the PDF's (2204.03327's `tab:wasp43_res`
  // is 3); a long table prints its numbered caption in the PDF, empty or not, so there it is numbered (repro
  // class-bindings/longrotatetable_tablecaption_goes_to_its_table).
  // Guards: `perfect_kernel_batch73a::{aastex_tablecaption_before_deluxetable, tablecaption_inside_the_table_wins,
  // stored_tablecaption_scoped_to_its_group, longtable_keeps_its_printed_caption, emulateapj_stored_caption_repeats,
  // kept_tablecaption_is_unnumbered, longrotatetable_tablecaption_goes_to_its_table, kept_caption_label_names_the_table,
  // longtable_tablecaption_is_numbered, emulateapj_repeat_drops_the_label}`.
  RawTeX!(r"\newif\iflx@deluxe@persistent\lx@deluxe@persistenttrue\newif\iflx@deluxe@intable\newif\iflx@deluxe@printed
\newif\iflx@deluxe@long\newif\iflx@deluxe@emptyhead\gdef\lx@deluxe@caption{}\global\let\lx@deluxe@own\@empty\global\let\lx@deluxe@pending\@empty
\global\let\lx@deluxe@late\@empty\let\lx@deluxe@caption@orig\caption\global\let\lx@deluxe@shown\relax");
  DefMacro!("\\lx@deluxe@caption@store{}",
    r"\iflx@deluxe@persistent\gdef\lx@deluxe@caption{#1}\else\def\lx@deluxe@caption{#1}\fi");
  // A kept caption the PDF never prints takes no counter step and no list entry; holding a `\label`, it takes an id step
  // (Perl `RefStepID`, Package.pm; main-session call 2026-10-18): the table gets an id (`<type>_id`, set on the float by
  // `after_float`, Perl `RescueCaptionCounters`), so the `\label` names the table (Perl `floatToLabel` takes the nearest
  // ancestor with an id; pdflatex never runs it, "??"). A caption without one takes none, so a `\label` written after
  // the stored caption names what TeX's `\label` names, the last numbered unit (2204.03327: `tab:allesFitted` = 4.1.5,
  // its aux). A table whose own caption already gave it an id keeps it. Repros class-bindings/{aastex_tablecaption_
  // before_deluxetable, kept_caption_label_names_the_table}.
  DefPrimitive!("\\lx@deluxe@id@step{}", sub[(caption)] {
    let captype = do_expand(T_CS!("\\@captype"))?.to_string();
    let key = s!("{captype}_id");
    let label = T_CS!("\\label");
    if !captype.is_empty()
      && caption.unlist().contains(&label)
      && lookup_value(&key).is_none_or(|v| matches!(v, Stored::None))
    {
      let props = ref_step_id(&captype)?;
      assign_value(&key, props.get("id"), Some(Scope::Global));
    }
  });
  DefMacro!("\\lx@deluxe@unnumbered{}", r"\lx@deluxe@id@step{#1}\lx@setfloatcapt{\lx@@caption{#1}}");
  DefMacro!("\\lx@deluxe@tablecaption",
    r"\iflx@deluxe@intable\expandafter\lx@deluxe@caption@intable\else
\ifx\caption\lx@longtable@caption\expandafter\expandafter\expandafter\lx@deluxe@caption@inlongtable
\else\expandafter\expandafter\expandafter\lx@deluxe@caption@outside\fi\fi");
  // A caption given in the data is set on its table here (OXIDIZED_DESIGN_DIVERGENCES #664), where TeX only stores it:
  // the emulateapj/AASTeX 5 store keeps it for the next table, which prints it again, as pdflatex — without its
  // `\label` (`\lx@deluxe@shown`, `\lx@deluxe@caption@nolabel`), which names the table that shows it first.
  DefMacro!("\\lx@deluxe@caption@intable{}",
    r"\iflx@deluxe@persistent\gdef\lx@deluxe@caption{#1}\iflx@deluxe@printed\gdef\lx@deluxe@shown{#1}\fi\fi
\iflx@deluxe@printed\iflx@deluxe@emptyhead\global\lx@deluxe@emptyheadfalse
\g@addto@macro\lx@deluxe@late{\lx@deluxe@caption@orig{#1}}\else\g@addto@macro\lx@deluxe@late{\lx@deluxe@unnumbered{#1}}\fi
\else\gdef\lx@deluxe@own{#1}\fi");
  // A longtable has its caption once any `\caption` set one (longtable_sty.rs's three slots, emptied at its start).
  DefMacro!("\\lx@deluxe@caption@inlongtable{}", sub[(caption)] {
    let captioned = ["LONGTABLE_CAPTIONS", "LONGTABLE_HEAD_CAPTIONS", "LONGTABLE_FOOT_CAPTIONS"]
      .iter()
      .any(|key| lookup_value(key).is_some_and(|value| !matches!(value, Stored::None)));
    let cs = if captioned { "\\lx@deluxe@longtable@stored" } else { "\\lx@deluxe@longtable@caption" };
    Ok(Invocation!(T_CS!(cs), vec![Some(caption)]))
  });
  DefMacro!("\\lx@deluxe@longtable@stored{}", r"\iflx@deluxe@persistent\gdef\lx@deluxe@caption{#1}\fi");
  DefMacro!("\\lx@deluxe@longtable@caption{}",
    r"\iflx@deluxe@persistent\gdef\lx@deluxe@caption{#1}\fi\lx@longtable@caption{#1}");
  // Outside a deluxetable and a longtable: a float's own caption, unless the float is an aastex `{longrotatetable}`
  // (aas_support_sty.rs, its frame `lx@lrt@frame`), whose `\tablecaption` is stored for the table it holds.
  DefMacro!("\\lx@deluxe@caption@outside{}", sub[(caption)] {
    let in_float = has_meaning(&T_CS!("\\@captype")) && !in_longrotatetable_body();
    let cs = if in_float { "\\lx@deluxe@caption@infloat" } else { "\\lx@deluxe@caption@store" };
    Ok(Invocation!(T_CS!(cs), vec![Some(caption)]))
  });
  DefMacro!("\\lx@deluxe@caption@infloat{}", r"\iflx@deluxe@persistent\gdef\lx@deluxe@caption{#1}\fi\caption{#1}");
  // At the table's start, inside its group: AASTeX 6+ take the store as this table's pending caption and empty it;
  // a `\caption` the table sets marks its caption printed.
  DefMacro!("\\lx@deluxe@stored@caption",
    r"\lx@deluxe@intabletrue\global\lx@deluxe@printedfalse\global\lx@deluxe@emptyheadfalse\global\let\lx@deluxe@own\@empty\global\let\lx@deluxe@late\@empty
\iflx@deluxe@persistent\global\let\lx@deluxe@pending\@empty\else\global\let\lx@deluxe@pending\lx@deluxe@caption
\global\let\lx@deluxe@caption\@empty\fi
\ifx\caption\lx@deluxe@caption@mark\else\let\lx@deluxe@caption@orig\caption\let\caption\lx@deluxe@caption@mark\fi");
  // The marking `\caption`, wrapped once: a deluxetable begun inside another's group (an author's runaway argument
  // swallowing `\end{deluxetable}`: 0903.2259) finds `\caption` already wrapped, and re-wrapping it would make the
  // wrapper call itself (Fatal infinite expansion; repro captions-floats/deluxetable_inside_deluxetable).
  DefMacro!("\\lx@deluxe@caption@mark", r"\global\lx@deluxe@printedtrue\lx@deluxe@caption@orig");
  // The print point: the store (emulateapj, AASTeX 5: numbered, empty or not), or AASTeX 6+'s own caption (numbered),
  // the kept caption (unnumbered; numbered in a long table) or, in a long table, an empty numbered one. An empty store
  // is the table's number alone (`\iflx@deluxe@emptyhead`), set at the table's end — as the caption of the first
  // `\tablecaption` given in the data, if any (OXIDIZED_DESIGN_DIVERGENCES #664; 1104.2619), else empty.
  DefMacro!("\\lx@deluxe@print@caption",
    r"\iflx@deluxe@printed\else\global\lx@deluxe@printedtrue
\iflx@deluxe@persistent\let\lx@deluxe@next\lx@deluxe@caption@orig\let\lx@deluxe@arg\lx@deluxe@caption
\ifx\lx@deluxe@caption\lx@deluxe@shown\let\lx@deluxe@next\lx@deluxe@caption@nolabel\fi
\ifx\lx@deluxe@caption\@empty\global\lx@deluxe@emptyheadtrue\let\lx@deluxe@next\@gobble\fi
\else\ifx\lx@deluxe@own\@empty\let\lx@deluxe@arg\lx@deluxe@pending
\ifx\lx@deluxe@pending\@empty\iflx@deluxe@long\let\lx@deluxe@next\lx@deluxe@caption@orig\else\let\lx@deluxe@next\@gobble\fi
\else\iflx@deluxe@long\let\lx@deluxe@next\lx@deluxe@caption@orig\else\let\lx@deluxe@next\lx@deluxe@unnumbered\fi\fi
\else\let\lx@deluxe@next\lx@deluxe@caption@orig\let\lx@deluxe@arg\lx@deluxe@own\fi\fi
\global\let\lx@deluxe@own\@empty\global\let\lx@deluxe@pending\@empty
\expandafter\lx@deluxe@next\expandafter{\lx@deluxe@arg}\fi");
  DefMacro!("\\lx@deluxe@caption@nolabel{}",
    r"\let\lx@deluxe@label\label\let\label\@gobble\lx@deluxe@caption@orig{#1}\let\label\lx@deluxe@label");
  // The table's end: a table that reached no print point sets its own caption (numbered) or the kept one
  // (unnumbered); the captions given in the data follow; a long table's flag ends with it.
  DefMacro!("\\lx@deluxe@pending@caption",
    r"\iflx@deluxe@printed\else\global\lx@deluxe@printedtrue
\ifx\lx@deluxe@own\@empty\let\lx@deluxe@next\lx@deluxe@unnumbered\let\lx@deluxe@arg\lx@deluxe@pending
\else\let\lx@deluxe@next\lx@deluxe@caption@orig\let\lx@deluxe@arg\lx@deluxe@own\fi
\ifx\lx@deluxe@arg\@empty\let\lx@deluxe@next\@gobble\fi
\global\let\lx@deluxe@own\@empty\global\let\lx@deluxe@pending\@empty
\expandafter\lx@deluxe@next\expandafter{\lx@deluxe@arg}\fi
\lx@deluxe@late\global\let\lx@deluxe@late\@empty
\iflx@deluxe@emptyhead\global\lx@deluxe@emptyheadfalse\lx@deluxe@caption@orig{}\fi\global\lx@deluxe@longfalse");
  Let!("\\tablecaption", "\\lx@deluxe@tablecaption");

  DefMacro!("\\tablehead{}",
    "\\def\\@deluxetable@header{\\lx@alignment@begin@heading#1\\\\\\hline\\lx@alignment@end@heading}");
  DefMacro!("\\colhead{}", "\\multicolumn{1}{c}{#1}");
  DefMacro!("\\twocolhead{}", "\\multicolumn{2}{c}{\\hss #1 \\hss}");
  DefMacro!("\\nocolhead{}", "\\multicolumn{1}{h}{#1}");
  DefMacro!("\\dcolhead{}", "\\multicolumn{1}{c}{$\\relax#1$}");

  DefMacro!("\\nl", "\\\\[0pt]");
  DefMacro!("\\nextline", "\\\\[0pt]");
  DefMacro!("\\tablevspace{}", "\\noalign{\\vskip#1}");

  //======================================================================
  // 2.15.3 Content of deluxetable

  // aastex701.cls:12652 `\newcommand\tablebreak{\\[-11pt]\noalign{\break}}`: the row ends and the
  // page may break after it (the negative space hides the break's empty row). Its explicit `[…]`
  // keeps a next row led by `[` (`[Fe/H]`) from being read as the row spacing, as `\nl` above.
  // Perl's deluxetable.sty.ltxml:94 overrides its own `\\[0pt]` (L89) with nothing, so the next
  // row ran into this one: an extra `&` per cell (astro-ph0604363, 1811.07447).
  DefMacro!("\\tablebreak", "\\\\[0pt]");
  def_macro_noop("\\nodata")?;

  DefMacro!("\\cutinhead{}", "\\hline\\multicolumn{\\lx@alignment@ncolumns}{c}{#1}\\\\\\hline");
  DefMacro!("\\sidehead{}", "\\hline\\multicolumn{\\lx@alignment@ncolumns}{l}{#1}\\\\\\hline");

  DefMacro!("\\tableline", "\\hline");

  DefConstructor!("\\tablenotemark{}",
    "<ltx:note role='footnotemark' mark='#mark'></ltx:note>",
    mode => "restricted_horizontal",
    properties => sub[args] {
      let mark = args[0].as_ref().map(shown_mark).unwrap_or_default();
      Ok(stored_map!("mark" => Stored::String(pin(&mark))))
    });
  // The note's mark is its `mark`, what the mark shows (`shown_mark`), which pairs it with a `\tablenotemark` showing
  // the same. Perl writes both as the argument's text (deluxetable.sty.ltxml:103-108), so `\tablenotemark{$\dagger$}`
  // showed its TeX source and paired with no note (KNOWN_PERL_ERRORS #873). A class that sets the mark in math sets it
  // in a box (`\lx@deluxe@tablenote@mark`, AASTeX's `\hbox to5pt{$#1$}`, aas_support_sty.rs): `$a$` there is an empty
  // formula, the text "a" and another empty formula, as TeX sets it (1807.05231).
  DefMacro!("\\lx@deluxe@tablenote@mark{}", "\\hbox{$#1$}");
  DefConstructor!("\\lx@deluxe@tablenotetext{}{}",
    "<ltx:note role='footnotetext' mark='#mark'>#2</ltx:note>",
    mode => "internal_vertical", sizer => sub[_whatsit] { Ok(out_of_line_size()) },
    properties => sub[args] {
      let mark = args[0].as_ref().map(shown_mark).unwrap_or_default();
      Ok(stored_map!("mark" => Stored::String(pin(&mark))))
    },
    reversion => "\\tablenotetext{#1}{#2}");
  DefMacro!("\\tablenotetext{}{}", "\\lx@deluxe@tablenotetext{#1}{#2}");

  // Perl uses AddToMacro to accumulate into \tblnote@list
  // We use \g@addto@macro which does the same at the TeX level
  DefMacro!("\\tablerefs{}", "\\g@addto@macro\\tblnote@list{\\@tableref{#1}\\let\\email\\@@email}");
  DefMacro!("\\@tableref{}", "\\par\n \\vspace*{3ex}%\n {\\parbox{\\textwidth}{\\hskip1em\\rmfamily References. --- #1}\\par}");

  DefMacro!("\\tablecomments{}", "\\g@addto@macro\\tblnote@list{\\@tablecom{#1}}");
  DefMacro!("\\@tablecom{}", "\\par \n \\vspace*{3ex}% \n {\\parbox{\\textwidth}{\\hskip1em\\rmfamily Note. --- #1}\\par}");

  DefMacro!("\\spew@tblnotes",
    "\\@tablenotes{\\tblnote@list}\\global\\let\\tblnote@list\\@empty");
  DefMacro!("\\@tablenotes{}", "\\par \n \\vspace{4.5ex}\\footnoterule\\vspace{.5ex}%\n {\\footnotesize #1}");

  //======================================================================
  // Other esoterica

  DefMacro!("\\ulap{}", "#1");
  DefMacro!("\\dlap{}", "#1");

  // Perl deluxetable.sty.ltxml L144-151 `AtBeginDocument` block. The
  // deferred timing matters for `\pt@width\textwidth`: at binding-load
  // \textwidth hasn't been sized by article.cls yet, so copying it here
  // yields the default 0pt. The \@empty lets and \pt@headfrac def are
  // order-insensitive but keep them together with the width init so the
  // block matches Perl one-for-one.
  at_begin_document(TokenizeInternal!(
    r"\let\tblnote@list\@empty\let\pt@caption\@empty\let\pt@head\@empty\let\pt@tail\@empty\pt@width\textwidth\def\pt@headfrac{.1}"
  ))?;
});
