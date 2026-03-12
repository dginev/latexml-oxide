use crate::prelude::*;
#[rustfmt::skip]
LoadDefinitions!({
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
    r"\@ifnext\label{\@caption@postlabel{#1}{#2}{#3}}{\@caption@{#1}{#2}{#3}}",
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

  DefMacro!(
    "\\@caption@@@{}{}{}",
    r"\@@add@caption@counters\@@toccaption{\lx@format@toctitle@@{#1}{\ifx.#2.#3\else#2\fi}}\@@caption{\lx@format@title@@{#1}{#3}}"
  );

  // Note that the counters only get incremented by \caption, NOT by \table, \figure, etc.
  DefPrimitive!("\\@@add@caption@counters", {
    let captype = stomach::digest(T_CS!("\\@captype"))?.to_string();
    let props   = ref_step_counter(&captype, false)?;
    let inlist  = stomach::digest(T_CS!(s!("\\ext@{}", captype)))?.to_string();
    state::assign_value(&s!("{}_tags", captype), props.get("tags"), Some(Scope::Global));
    state::assign_value(&s!("{}_id", captype), props.get("id"),   Some(Scope::Global));
    state::assign_value(&s!("{}_inlist", captype), inlist,      Some(Scope::Global));
  });

  DefConstructor!("\\@@generic@caption[]{}", "<ltx:text class='ltx_caption'>#2</ltx:text>",
  before_digest => {
    Error!("unexpected", "\\caption", "Use of \\caption outside any known float"); });

  // Note that even without \caption, we'd probably like to have xml:id.
  Tag!("ltx:figure", after_close => sub[document, node] { document.generate_id(node, "fig")?; });
  Tag!("ltx:table",  after_close => sub[document, node] { document.generate_id(node, "tab")?; });
  Tag!("ltx:float",  after_close => sub[document, node] { document.generate_id(node, "tab")?; });

  // # These may need to float up to where they're allowed,
  // # or they may need to close <p> or similar.
  // TODO: prefix both replacements with ^^ when we can compile them.
  DefConstructor!("\\@@caption{}", "<ltx:caption>#1</ltx:caption>",
    mode => "text");
  DefConstructor!(
    "\\@@toccaption{}",
    "<ltx:toccaption>#1</ltx:toccaption>", //sizer => 0
    mode => "text");

  // TODO: implement optional argument {figure}[]
  DefEnvironment!("{figure}",r###"
  <ltx:figure xml:id='#id' inlist='#inlist'>
    #tags
    #body
  </ltx:figure>
  "###,
    properties   => { stored_map!("layout" => "vertical") },
    before_digest => { DefMacro!("\\@captype", "figure"); },
    after_digest  => sub[tag] {
      rescue_caption_counters("figure", tag);
    },
    mode => "internal_vertical"
  );
  // Perl: latex_constructs.pool.ltxml line 3460
  DefEnvironment!("{figure*}[]",r###"
  <ltx:figure xml:id='#id' inlist='#inlist' ?#1(placement='#1')>
    #tags
    #body
  </ltx:figure>
  "###,
    properties   => { stored_map!("layout" => "vertical") },
    before_digest => { DefMacro!("\\@captype", "figure"); },
    after_digest  => sub[tag] {
      rescue_caption_counters("figure", tag);
    },
    mode => "internal_vertical"
  );
  DefEnvironment!("{table}[]",
    "<ltx:table xml:id='#id' inlist='#inlist' ?#1(placement='#1')>#tags#body</ltx:table>",
    // TODO:
    // properties   => { layout => 'vertical' },
    before_digest => { DefMacro!("\\@captype", "table"); },
    after_digest  => sub[whatsit] { rescue_caption_counters("table", whatsit); },
    mode => "internal_vertical");
  // Perl: latex_constructs.pool.ltxml line 3478
  DefEnvironment!("{table*}[]",
    "<ltx:table xml:id='#id' inlist='#inlist' ?#1(placement='#1')>#tags#body</ltx:table>",
    properties   => { stored_map!("layout" => "vertical") },
    before_digest => { DefMacro!("\\@captype", "table"); },
    after_digest  => sub[whatsit] { rescue_caption_counters("table", whatsit); },
    mode => "internal_vertical");

  DefPrimitive!("\\flushbottom",      None);
  DefPrimitive!("\\suppressfloats[]", None);

  NewCounter!("topnumber");
  DefMacro!("\\topfraction", "0.25");
  NewCounter!("bottomnumber");
  DefMacro!("\\bottomfraction", "0.25");
  NewCounter!("totalnumber");
  DefMacro!("\\textfraction", "0.25");
  DefMacro!("\\floatpagefraction", "0.25");
  NewCounter!("dbltopnumber");
  DefMacro!("\\dbltopfraction",       "0.7");
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
  DefRegister!("\\abovecaptionskip" => Glue::new(0));
  DefRegister!("\\belowcaptionskip" => Glue::new(0));
  Let!("\\topfigrule", "\\relax");
  Let!("\\botfigrule", "\\relax");
  Let!("\\dblfigrule", "\\relax");

  DefMacro!("\\figurename",  "Figure");
  DefMacro!("\\figuresname", "Figures");    // Never used?
  DefMacro!("\\tablename",   "Table");
  DefMacro!("\\tablesname",  "Tables");

  Let!("\\outer@nobreak", "\\@empty");
  DefMacro!("\\@dbflt{}",           "#1");
  DefMacro!("\\@xdblfloat{}[]",     "\\@xfloat{#1}[#2]");
  DefMacro!("\\@floatplacement",    "");
  DefMacro!("\\@dblfloatplacement", "");

});
