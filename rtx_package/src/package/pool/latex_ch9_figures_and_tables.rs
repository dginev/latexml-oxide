use crate::package::*;
LoadDefinitions!(state, {

  //======================================================================
  // C.9.1 Figures and Tables
  //======================================================================

  // Note that, the number is associated with the caption.
  // (to allow multiple figures per figure environment?).
  // Whatever reason, that causes complications: We can only increment
  // counters with the caption, but then have to arrange for the counters,
  // refnums, ids, get passed on to the figure, table when needed.
  // AND, as soon as possible, since other items may base their id's on the id of the table!

  DefMacro!("\\figurename",  "Figure");
  DefMacro!("\\figuresname",  "Figures");    // Never used?
  DefMacro!("\\tablename",  "Table");
  DefMacro!("\\tablesname",  "Tables");

  // Let the fonts for float be the default for all floats, figures, tables, etc.
  DefMacro!("\\fnum@font@float",         "\\@empty");
  DefMacro!("\\format@title@font@float", "\\@empty");

  DefMacro!("\\fnum@font@figure",         "\\fnum@font@float");
  DefMacro!("\\fnum@font@table",          "\\fnum@font@float");
  DefMacro!("\\format@title@font@figure", "\\format@title@font@float");
  DefMacro!("\\format@title@font@table",  "\\format@title@font@float");

  // Could perhaps parameterize further with a separator?
  DefMacro!("\\format@title@figure{}", "\\lx@tag[][: ]{\\lx@fnum@@{figure}}#1");
  DefMacro!("\\format@title@table{}",  "\\lx@tag[][: ]{\\lx@fnum@@{table}}#1");

  DefMacro!("\\ext@figure", "lof");
  DefMacro!("\\ext@table",  "lot");

  DefConditional!("\\iflx@donecaption");
  DefMacro!("\\caption",
    "\\lx@donecaptiontrue\\@ifundefined{@captype}{\\@@generic@caption}{\\expandafter\\@caption\\expandafter{\\@captype}}");
  DefMacro!("\\@caption{}[]{}",
    "\\@ifnext\\label{\\@caption@postlabel{#1}{#2}{#3}}{\\@caption@{#1}{#2}{#3}}");
  // Check for trailing \label, move it into the caption
  DefMacro!("\\@caption@postlabel{}{}{} SkipMatch:\\label Semiverbatim",
    "\\@caption@{#1}{#2}{#3\\label{#4}}");
  DefMacro!("\\@caption@{}{}{}",
    "\\@@add@caption@counters\
      \\@@toccaption{\\lx@format@toctitle@@{#1}{\\ifx.#2.#3\\else#2\\fi}}\
      \\@@caption{\\lx@format@title@@{#1}{#3}}");

// Note that the counters only get incremented by \caption, NOT by \table, \figure, etc.
DefPrimitive!("\\@@add@caption@counters", sub[stomach, args, state] {
  let captype = stomach.digest(vec![T_CS!("\\@captype")], state)?.to_string();
  let props   = ref_step_counter(&captype, false, stomach, state)?;
  let inlist  = stomach.digest(vec![T_CS!(s!("\\ext@{}", captype))], state)?.to_string();
  state.assign_value(&s!("{}_tags", captype), props.get("tags"), Some(Scope::Global));
  state.assign_value(&s!("{}_id", captype), props.get("id"),   Some(Scope::Global));
  state.assign_value(&s!("{}_inlist", captype), inlist,      Some(Scope::Global));
});

DefConstructor!("\\@@generic@caption[]{}", "<ltx:text class='ltx_caption'>#2</ltx:text>",
  before_digest => before_digest!(stomach, state, {
    Error!("unexpected", "\\caption", stomach, state,
      "Use of \\caption outside any known float"); }));

// Note that even without \caption, we'd probably like to have xml:id.
Tag!("ltx:figure", after_close => tagsub!(document, node, state, { generate_id(document, node, "fig", state)?; }));
Tag!("ltx:table",  after_close => tagsub!(document, node, state, { generate_id(document, node, "tab", state)?; }));
Tag!("ltx:float",  after_close => tagsub!(document, node, state, { generate_id(document, node, "tab", state)?; }));

// # These may need to float up to where they're allowed,
// # or they may need to close <p> or similar.
// TODO: prefix both replacements with ^^ when we can compile them.
DefConstructor!("\\@@caption{}", "<ltx:caption>#1</ltx:caption>");
DefConstructor!("\\@@toccaption{}", "<ltx:toccaption>#1</ltx:toccaption>"
  // sizer => "0"
);

// TODO: implement optional argument {figure}[]
DefEnv!("{figure}",r###"
  <ltx:figure xml:id='#id' inlist='#inlist'>
    #tags
    #body
  </ltx:figure>
  "###,
  properties   => { map!("layout" => "vertical".into()) },
  before_digest => { DefMacro!("\\@captype", "figure"); },
  after_digest  => sub[stomach, tag, state] {
    rescue_caption_counters("figure", tag, stomach, state);
  }
);
// DefEnvironment('{figure*}[]',
//   "<ltx:figure xml:id='#id' inlist='#inlist' ?#1(placement='#1')>"
//     . "#tags"
//     . "#body"
//     . "</ltx:figure>",
//   properties   => { layout => 'vertical' },
//   beforeDigest => sub { DefMacroI('\@captype', undef, 'figure'); },
//   afterDigest  => sub { RescueCaptionCounters('figure', $_[1]); });
// DefEnvironment('{table}[]',
//   "<ltx:table xml:id='#id' inlist='#inlist' ?#1(placement='#1')>"
//     . "#tags"
//     . "#body"
//     . "</ltx:table>",
//   properties   => { layout => 'vertical' },
//   beforeDigest => sub { DefMacroI('\@captype', undef, 'table'); },
//   afterDigest  => sub { RescueCaptionCounters('table', $_[1]); });
// DefEnvironment('{table*}[]',
//   "<ltx:table xml:id='#id' inlist='#inlist' ?#1(placement='#1')>"
//     . "#tags"
//     . "#body"
//     . "</ltx:table>",
//   properties   => { layout => 'vertical' },
//   beforeDigest => sub { DefMacroI('\@captype', undef, 'table'); },
//   afterDigest  => sub { RescueCaptionCounters('table', $_[1]); });


});
