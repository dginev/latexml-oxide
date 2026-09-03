//! `latex_constructs` section 9: C.9 Figures and Other Floating Bodies
//!
//! Split verbatim from the single `LoadDefinitions!` body (2026-09-03); the
//! definitions run in the original order from `super::load_definitions`.

use super::*;

#[rustfmt::skip]
pub(crate) fn load() -> Result<()> {
  // ======================================================================
  // C.9 Figures and Other Floating Bodies
  // ======================================================================

  //======================================================================
  // C.9.1 Figures and Tables
  //======================================================================

  // Note that, the number is associated with the caption.
  // (to allow multiple figures per figure environment?).
  // Whatever reason, that causes complications: We can only increment
  // counters with the caption, but then have to arrange for the counters,
  // refnums, ids, get passed on to the figure, table when needed.
  // AND, as soon as possible, since other items may base their id's on the id of the table!

  DefMacro!("\\figurename", "Figure");
  DefMacro!("\\figuresname", "Figures"); // Never used?
  DefMacro!("\\tablename", "Table");
  DefMacro!("\\tablesname", "Tables");

  // Let the fonts for float be the default for all floats, figures, tables, etc.
  DefMacro!("\\fnum@font@float", "\\@empty");
  DefMacro!("\\format@title@font@float", "\\@empty");

  DefMacro!("\\fnum@font@figure", "\\fnum@font@float");
  DefMacro!("\\fnum@font@table", "\\fnum@font@float");
  DefMacro!("\\format@title@font@figure", "\\format@title@font@float");
  DefMacro!("\\format@title@font@table", "\\format@title@font@float");

  // Could perhaps parameterize further with a separator?
  DefMacro!(
    "\\format@title@figure{}",
    "\\lx@tag[][: ]{\\lx@fnum@@{figure}}#1"
  );
  DefMacro!(
    "\\format@title@table{}",
    "\\lx@tag[][: ]{\\lx@fnum@@{table}}#1"
  );

  DefMacro!("\\ext@figure", "lof");
  DefMacro!("\\ext@table", "lot");

  DefConditional!("\\iflx@donecaption");
  DefMacro!(
    "\\caption",
    r"\lx@donecaptiontrue\@ifundefined{@captype}{\@@generic@caption}{\expandafter\@caption\expandafter{\@captype}}"
  );
  // First, check for trailing \label, move it into the caption as a standard position
  // NOTE: If one day we want to unlock \@caption, make sure to test against arXiv:cond-mat/0001395
  // for a passing build.
  DefMacro!(
    "\\@caption{}[]{}",
    r"\@ifnextchar\label{\@caption@postlabel{#1}{#2}{#3}}{\@caption@{#1}{#2}{#3}}",
    locked=>true
  );
  // Check for trailing \label, move it into the caption
  DefMacro!(
    r"\@caption@postlabel{}{}{} SkipMatch:\label Semiverbatim",
    r"\@caption@{#1}{#2}{#3\label{#4}}"
  );
  DefMacro!(
    r"\@caption@{}{}{}",
    r"\@hack@caption@{#1}{#2}{}#3\label\endcaption"
  );
  DefMacro!(
    r"\@hack@caption@{}{}{} Until:\label Until:\endcaption",
    r"\ifx.#5.\@caption@@@{#1}{#2}{#3#4}\else\@@@hack@caption@{#1}{#2}{#3#4}#5\endcaption\fi"
  );
  DefMacro!(
    r"\@@@hack@caption@{}{}{} Semiverbatim Until:\label Until:\endcaption",
    r"\lx@note@caption@label{#4}\@hack@caption@{#1}{#2}{#3\label{#4}#5}\label#6\endcaption"
  );

  DefPrimitive!("\\lx@note@caption@label{}", sub[(label)] {
    let label = label.to_string();
    maybe_note_label(&label); });

  // OXIDIZED_DESIGN #182 (PLANS P16 ii): a `\caption` whose `\@captype` is set
  // but which sits in a box that is NOT a float — tufte-common.def:1110-1133
  // `marginfigure` (`\marginpar{\usebox{…}}` around a minipage), raw tocbasic
  // `\captionaboveof{table}` at top level — has no ancestor that can hold an
  // `ltx:caption`, so the float+tag form errored once per caption
  // (`<ltx:caption> isn't allowed in <ltx:block>` + the `ltx:toccaption`
  // sibling: pgfornament ornaments 40+40, memman 46+46, xltabular). Degrade to
  // the inline `\@@generic@caption` shape (an `ltx:text class="ltx_caption"`,
  // no counter tag, no toc entry) — what Perl's own no-`\@captype` path emits.
  // Guard: `perfect_kernel_batch54::caption_without_a_float_ancestor_degrades_to_text`.
  DefMacro!(
    "\\@caption@@@{}{}{}",
    r"\@@add@caption@counters\@@toccaption{\lx@format@toctitle@@{#1}{\ifx.#2.#3\else#2\fi}}\@@caption{\lx@format@title@@{#1}{#3}}"
  );

  // Note that the counters only get incremented by \caption, NOT by \table, \figure, etc.
  // Perl: latex_constructs.pool.ltxml L3250-3258
  // Checks PREINCREMENTED_ first (set by beforeFloat with preincrement option).
  DefPrimitive!("\\@@add@caption@counters", {
    // Perl: $type = ToString(Digest(T_CS('\@captype')))
    // Rust port had used `stomach::digest`, but stomach::digest's
    // `read_x_token` loop in vmode (figure-environment body is vmode)
    // can leak a trailing `\par` token into the result when the captype
    // expansion completes — the digester continues reading beyond the
    // expansion and picks up an environment-emitted `\par`. Use
    // `do_expand` instead: it expands the macro one level (`\@captype`
    // → "figure" letter tokens) and stops, mirroring Perl's `ToString`
    // of the captype's body without invoking stomach digestion.
    // Witness: math0010095 BoxedEPS+figure+caption produced
    // `\thefigure\par` undefined errors when captype was "figure\par".
    let captype = do_expand(T_CS!("\\@captype"))?.to_string();
    let prekey = s!("PREINCREMENTED_{captype}");
    let props = match remove_value(&prekey) {
      Some(Stored::HashStored(pre)) => pre,
      _ => ref_step_counter(&captype, false)?,
    };
    let inlist = digest(T_CS!(s!("\\ext@{}", captype)))?.to_string();
    assign_value(
      &s!("{}_tags", captype),
      props.get("tags"),
      Some(Scope::Global),
    );
    assign_value(&s!("{}_id", captype), props.get("id"), Some(Scope::Global));
    assign_value(&s!("{}_inlist", captype), inlist, Some(Scope::Global));
  });

  DefConstructor!("\\@@generic@caption[]{}", "<ltx:text class='ltx_caption'>#2</ltx:text>",
  before_digest => {
    Error!("unexpected", "\\caption", "Use of \\caption outside any known float"); });

  // Note that even without \caption, we'd probably like to have xml:id.
  // Perl: BuildPanelsAndID + collapseFloat (afterClose hooks)
  Tag!("ltx:figure", after_close => sub[document, node, whatsit] {
    document.generate_id(node, "fig")?;
    arrange_panels(document, node, float_width_of(whatsit))?;
    collapse_float(document, node)?;
  });
  Tag!("ltx:table",  after_close => sub[document, node, whatsit] {
    document.generate_id(node, "tab")?;
    arrange_panels(document, node, float_width_of(whatsit))?;
    collapse_float(document, node)?;
  });
  Tag!("ltx:float",  after_close => sub[document, node, whatsit] {
    document.generate_id(node, "tab")?;
    arrange_panels(document, node, float_width_of(whatsit))?;
    collapse_float(document, node)?;
  });

  // # These may need to float up to where they're allowed,
  // # or they may need to close <p> or similar.
  // Perl: latex_constructs.pool.ltxml L3423-3427
  // ^^ prefix means "float up" in LaTeXML's document model
  // OXIDIZED_DESIGN #182 (PLANS P16 ii): a `\caption` whose `\@captype` is set
  // but which sits in a box that is NOT a float — tufte-common.def:1110-1133
  // `marginfigure` (`\marginpar{\usebox{…}}` around a minipage), raw tocbasic
  // `\captionaboveof{table}` at top level — has no ancestor that can hold an
  // `ltx:caption`, so the float form errored once per caption
  // (`<ltx:caption> isn't allowed in <ltx:block>`, plus its `ltx:toccaption`
  // sibling and the `ltx:tag`: pgfornament ornaments 40+40, memman 46+46).
  // Degrade to the inline shape Perl's own no-`\@captype` path emits — an
  // `ltx:text class="ltx_caption"` holding the title, minus the counter tag
  // (which no inline element may carry) — and drop the toc entry. Guard:
  // `perfect_kernel_batch54::caption_without_a_float_ancestor_degrades_to_text`.
  DefConstructor!("\\@@caption{}", sub[document, args] {
    let body = args[0].clone();
    if caption_can_float(document, "ltx:caption") {
      // `^^`: float up, closing what can be closed on the way.
      let save = document.float_to_element("ltx:caption", true)?;
      document.open_element("ltx:caption", None, None)?;
      if let Some(ref body) = body {
        document.absorb(body, None)?;
      }
      document.close_element("ltx:caption")?;
      if let Some(save) = save {
        document.set_node(&save);
      }
    } else {
      let node = document.open_element(
        "ltx:text",
        Some(string_map!("class" => "ltx_caption")),
        None,
      )?;
      if let Some(ref body) = body {
        absorb_without_tags(document, body)?;
      }
      document.maybe_close_node(&node)?;
    }
  }, mode => "text");
  DefConstructor!("\\@@toccaption{}", sub[document, args] {
    if caption_can_float(document, "ltx:toccaption") {
      let body = args[0].clone();
      let save = document.float_to_element("ltx:toccaption", true)?;
      document.open_element("ltx:toccaption", None, None)?;
      if let Some(ref body) = body {
        document.absorb(body, None)?;
      }
      document.close_element("ltx:toccaption")?;
      if let Some(save) = save {
        document.set_node(&save);
      }
    }
  }, mode => "text");

  // Perl: latex_constructs.pool.ltxml L3450-3458
  // Uses beforeFloat('figure') / afterFloat — sets LAST_FLOATTYPE, rescues counters.
  DefEnvironment!("{figure}[]",
    "<ltx:figure xml:id='#id' inlist='#inlist' ?#1(placement='#1')>\
    #tags\
    #body\
    </ltx:figure>",
    properties   => { stored_map!("layout" => "vertical") },
    before_digest => { before_float("figure", None); },
    after_digest  => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical"
  );
  // Perl: latex_constructs.pool.ltxml line 3460
  DefEnvironment!("{figure*}[]",
    "<ltx:figure xml:id='#id' inlist='#inlist' ?#1(placement='#1')>\
    #tags\
    #body\
    </ltx:figure>",
    properties   => { stored_map!("layout" => "vertical") },
    before_digest => { before_float_ex("figure", None, true); }, // double=true for *
    after_digest  => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical"
  );
  // Perl: latex_constructs.pool.ltxml L3469-3477
  DefEnvironment!("{table}[]",
    "<ltx:table xml:id='#id' inlist='#inlist' ?#1(placement='#1')>#tags#body</ltx:table>",
    properties   => { stored_map!("layout" => "vertical") },
    before_digest => { before_float("table", None); },
    after_digest  => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical");
  // Perl: latex_constructs.pool.ltxml line 3478
  DefEnvironment!("{table*}[]",
    "<ltx:table xml:id='#id' inlist='#inlist' ?#1(placement='#1')>#tags#body</ltx:table>",
    properties   => { stored_map!("layout" => "vertical") },
    before_digest => { before_float_ex("table", None, true); }, // double=true for *
    after_digest  => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical");

  // Perl: latex_constructs.pool.ltxml L3199-3212 — internal @float/@dblfloat
  // Used by raw TeX packages (e.g., nips_2017.sty) via \@float{type}[placement]
  // Since the float type arg isn't known at compile time, we use a properties
  // closure to call beforeFloat dynamically.
  DefEnvironment!("{@float}{}[]",
    "<ltx:float xml:id='#id' inlist='#inlist' ?#2(placement='#2') class='ltx_float_#1'>\
    #tags#body\
    </ltx:float>",
    properties => sub[args] {
      let float_type = args.first().and_then(|a| a.as_ref())
        .map(|d| d.to_string()).unwrap_or_default();
      before_float(&float_type, None);
      Ok(stored_map!("layout" => "vertical"))
    },
    after_digest  => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical");
  DefEnvironment!("{@dblfloat}{}[]",
    "<ltx:float xml:id='#id' inlist='#inlist' ?#2(placement='#2') class='ltx_float_#1'>\
    #tags#body\
    </ltx:float>",
    properties => sub[args] {
      let float_type = args.first().and_then(|a| a.as_ref())
        .map(|d| d.to_string()).unwrap_or_default();
      before_float_ex(&float_type, None, true);
      Ok(stored_map!("layout" => "vertical"))
    },
    after_digest  => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical");

  def_primitive_noop("\\flushbottom")?;
  def_primitive_noop("\\suppressfloats[]")?;

  NewCounter!("topnumber");
  DefMacro!("\\topfraction", "0.25");
  NewCounter!("bottomnumber");
  DefMacro!("\\bottomfraction", "0.25");
  NewCounter!("totalnumber");
  DefMacro!("\\textfraction", "0.25");
  DefMacro!("\\floatpagefraction", "0.25");
  NewCounter!("dbltopnumber");
  DefMacro!("\\dbltopfraction", "0.7");
  DefMacro!("\\dblfloatpagefraction", "0.25");
  DefRegister!("\\floatsep"         => Glue!("12.0pt plus 2.0pt minus 2.0pt"));
  DefRegister!("\\textfloatsep"     => Glue!("20.0pt plus 2.0pt minus 4.0pt"));
  DefRegister!("\\intextsep"        => Glue!("12.0pt plus 2.0pt minus 2.0pt"));
  DefRegister!("\\dblfloatsep"      => Glue!("12.0pt plus 2.0pt minus 2.0pt"));
  DefRegister!("\\dbltextfloatsep"  => Glue!("20.0pt plus 2.0pt minus 4.0pt"));
  DefRegister!("\\@maxsep"          => Dimension::new(0));
  DefRegister!("\\@dblmaxsep"       => Dimension::new(0));
  DefRegister!("\\@fptop"           => Glue::new(0));
  DefRegister!("\\@fpsep"           => Glue::new(0));
  DefRegister!("\\@fpbot"           => Glue::new(0));
  DefRegister!("\\@dblfptop"        => Glue::new(0));
  DefRegister!("\\@dblfpsep"        => Glue::new(0));
  DefRegister!("\\@dblfpbot"        => Glue::new(0));
  // Perl LaTeX.pool.ltxml L3648-3649 defines these in the BASE (not only in
  // article.cls.ltxml), so they are available under ANY document class. The
  // prior Rust comment ("not in Perl engine") was mistaken — it saw only the
  // article.cls copy. A paper on a custom class that does NOT load article
  // (e.g. `\documentclass{style/vldb}`, witness 1703.00080) then hit
  // `undefined:\abovecaptionskip` on `\setlength{\abovecaptionskip}{…}`.
  // Define them here too (Glue 0, exactly Perl), as a base fallback that
  // article/book/ams_support still override with their own values.
  DefRegister!("\\abovecaptionskip" => Glue::new(0));
  DefRegister!("\\belowcaptionskip" => Glue::new(0));
  Let!("\\topfigrule", "\\relax");
  Let!("\\botfigrule", "\\relax");
  Let!("\\dblfigrule", "\\relax");

  // \figurename / \figuresname / \tablename / \tablesname already defined
  // earlier in this file (figure/table caption block); avoid identical
  // re-definitions here.

  Let!("\\outer@nobreak", "\\@empty");
  def_macro_identity("\\@dbflt{}")?;
  DefMacro!("\\@xdblfloat{}[]", "\\@xfloat{#1}[#2]");
  def_macro_noop("\\@floatplacement")?;
  def_macro_noop("\\@dblfloatplacement")?;

  DefConditional!("\\if@reversemargin");
  Let!("\\reversemarginpar", "\\@reversemargintrue");
  Let!("\\normalmarginpar", "\\@reversemarginfalse");
  // Perl: latex_constructs.pool.ltxml lines 3543-3546
  // `bounded => true` scopes font/catcode changes inside the margin note to the
  // note itself. This is an INTENTIONAL surpass-Perl divergence (OXIDIZED_DESIGN
  // #39, KNOWN_PERL_ERRORS): upstream Perl LaTeXML's `\marginpar` is NOT bounded,
  // so a `\marginpar{\Large …}` leaks the size switch into the body text after
  // it (real pdflatex scopes it to the margin box — the leak is a LaTeXML bug,
  // shared by both engines). Mirrors `\mbox`'s `bounded => true`. Witness:
  // mhchem.tex `\marginpar{\Large !}` made the entire manual render at 144%.
  DefConstructor!("\\marginpar[]{}", r###"?#1(<ltx:note role='margin' class='ltx_marginpar_left'><ltx:inline-logical-block>#1</ltx:inline-logical-block></ltx:note>?#2(<ltx:note role='margin' class='ltx_marginpar_right'><ltx:inline-logical-block>#2</ltx:inline-logical-block></ltx:note>))(<ltx:note role='margin' class='ltx_marginpar'><ltx:inline-logical-block>#2</ltx:inline-logical-block></ltx:note>)"###,
    bounded => true);

  DefRegister!("\\marginparpush", Dimension::new(0));

  Ok(())
}
