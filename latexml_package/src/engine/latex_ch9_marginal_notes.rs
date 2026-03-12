use crate::prelude::*;

//======================================================================
// C.9.2 Marginal Notes
//======================================================================
LoadDefinitions!({
  DefConditional!("\\if@reversemargin");
  Let!("\\reversemarginpar", "\\@reversemargintrue");
  Let!("\\normalmarginpar", "\\@reversemarginfalse");
  // TODO: Can we find an ergonomic rust way to write multi-line XML strings for constructors?
  //       maybe have a "proper" template capability, as most Rust web frameworks do?
  //       consider the view! macro of Leptos for example:
  //       https://docs.rs/leptos/latest/leptos/macro.view.html
  //
  // DefConstructor!("\\marginpar[]{}", r###"\
  // ?#1(<ltx:note role='margin'
  // class='ltx_marginpar_left'><ltx:inline-logical-block>#1</ltx:inline-logical-block></ltx:note>\
  // ?#2(<ltx:note role='margin'
  // class='ltx_marginpar_right'><ltx:inline-logical-block>#2</ltx:inline-logical-block></ltx:
  // note>))\ (<ltx:note role='margin'
  // class='ltx_marginpar'><ltx:inline-logical-block>#2</ltx:inline-logical-block></ltx:note>)"###);

  DefRegister!("\\marginparpush", Dimension::new(0));

  //**********************************************************************
  // C.10 Lining It Up in Columns
  //**********************************************************************

  //======================================================================
  // C.10.1 The tabbing Environment
  //======================================================================
  DefRegister!("\\tabbingsep" => Dimension::new(0));

  DefMacro!(
    "\\tabbing",
    r"\par\@tabbing@bindings\@@tabbing\lx@begin@alignment"
  );
  DefMacro!("\\endtabbing", r"\lx@end@alignment\@end@tabbing\par");
  DefPrimitive!("\\@end@tabbing", { stomach::egroup() });
  DefConstructor!("\\@@tabbing SkipSpaces DigestedBody",
    "#1",
    reversion    => r"\begin{tabbing}#1\end{tabbing}",
    before_digest => { stomach::bgroup(); },
    mode         => "internal_vertical");

  DefMacro!("\\@tabbing@tabset", r"\@tabbing@tabset@marker&");
  DefMacro!("\\@tabbing@nexttab", r"\@tabbing@nexttab@marker&");
  DefMacro!(
    "\\@tabbing@newline OptionalMatch:* [Dimension]",
    r"\@tabbing@newline@marker\cr"
  );
  DefMacro!(
    "\\@tabbing@kill",
    r"\@tabbing@kill@marker\cr\@tabbing@start@tabs"
  );

  DefConstructor!("\\@tabbing@tabset@marker", "", reversion => r"\=",
    properties => { Ok(stored_map!("alignmentSkippable" => true))});
  DefConstructor!("\\@tabbing@nexttab@marker", "", reversion => r"\>",
    properties => { Ok(stored_map!("alignmentSkippable" => true)) });
  DefConstructor!("\\@tabbing@newline@marker", "", reversion => Tokens!(T_CS!("\\\\"), T_CR!()));
  DefConstructor!("\\@tabbing@kill@marker", "", reversion => "\\kill",
    after_digest => sub[_args] {
      if let Some(Some(env)) = lookup_alignment().as_ref().map(Digested::alignment_cell) {
        env.borrow_mut().remove_row();
      }
      Ok(Vec::new())
    },
    properties  => { Ok(stored_map!("alignmentSkippable" => true )) });

  AssignValue!("tabbing_start_tabs" => Tokens!());
  DefMacro!("\\@tabbing@start@tabs", {
    lookup_tokens("tabbing_start_tabs")
      .unwrap_or_default()
      .unlist()
  });
  DefPrimitive!("\\@tabbing@increment", {
    let tabs = lookup_tokens("tabbing_start_tabs")
      .unwrap_or_default()
      .unlist();
    assign_value(
      "tabbing_start_tabs",
      Tokens!(tabs, T_CS!("\\>")),
      Some(Scope::Global),
    );
  });
  DefPrimitive!("\\@tabbing@decrement", {
    let starts = lookup_tokens("tabbing_start_tabs")
      .unwrap_or_default()
      .unlist();
    let tabs = starts[1];
    assign_value("tabbing_start_tabs", Tokens!(tabs), Some(Scope::Global));
  });

  // NOTE: \< is NOT currently handled!!!
  // Ugh!! The way we're setting the initial tabs, we can't really handle this!
  DefPrimitive!("\\@tabbing@untab", None);

  // NOTE: \' and \` are NOT currently handled
  DefPrimitive!("\\@tabbing@flushright", None);
  DefPrimitive!("\\@tabbing@hfil", None);
  // NOTE: \pushtabs and \poptabs are NOT currently handled.
  DefPrimitive!("\\@tabbing@pushtabs", None);
  DefPrimitive!("\\@tabbing@poptabs", None);

  DefMacro!("\\@tabbing@accent{}", sub[(arg)] { T_CS!(s!("\\@tabbing@{}", arg)) });

  DefMacro!("\\pushtabs", "");
  DefMacro!("\\poptabs", "");
  DefMacro!("\\kill", "");

  DefPrimitive!("\\@tabbing@bindings", {
    tabbing_bindings();
  });
  // NOTE: Do it!!

  // Internals of tabbing, as an experiment (e.g. files using program.sty raw as in cs/0003026)
  DefMacro!(
    "\\@startfield",
    r"\global\setbox\@curfield\hbox\bgroup\color@begingroup"
  );
  DefMacro!("\\@stopfield", r"\color@endgroup\egroup");
  DefMacro!(
    "\\@contfield",
    r"\global\setbox\@curfield\hbox\bgroup\color@begingroup\unhbox\@curfield"
  );
  DefMacro!(
    "\\@addfield",
    r"\global\setbox\@curline\hbox{\unhbox\@curline\unhbox\@curfield}"
  );
});

// Should there be some rowsep/colsep set here?
// Note that {tabbign} really shouldn't be handled by a tabular AT ALL....
// Should be recording accumulated widths and wrapping in ltx:text, with specified widths.
fn tabbing_bindings() {
  // TODO:
  // state::assign_alignment(LaTeXML::Core::Alignment->new(
  //     template => LaTeXML::Core::Alignment::Template->new(
  //       repeated => [{ before => Tokens(T_CS('\lx@text@intercol')),
  //           after => Tokens(T_CS('\hfil'), T_CS('\lx@text@intercol')) }]),
  //     openContainer  => sub { $_[0]->openElement('ltx:tabular', @_[1 .. $#_]); },
  //     closeContainer => sub { $_[0]->closeElement('ltx:tabular'); },
  //     openRow        => sub { $_[0]->openElement('ltx:tr', @_[1 .. $#_]); },
  //     closeRow       => sub { $_[0]->closeElement('ltx:tr'); },
  //     openColumn     => sub { $_[0]->openElement('ltx:td', @_[1 .. $#_]); },
  //     closeColumn    => sub { $_[0]->closeElement('ltx:td'); },
  //     properties     => { attributes => { 'class' => 'ltx_tabbing' } }));

  state::let_i(&T_CS!("\\="), &T_CS!("\\@tabbing@tabset"), None);
  state::let_i(&T_CS!("\\>"), &T_CS!("\\@tabbing@nexttab"), None);
  state::let_i(&T_CS!("\\\\"), &T_CS!("\\@tabbing@newline"), None);
  state::let_i(&T_CS!("\\kill"), &T_CS!("\\@tabbing@kill"), None);
  state::let_i(&T_CS!("\\+"), &T_CS!("\\@tabbing@increment"), None);
  state::let_i(&T_CS!("\\-"), &T_CS!("\\@tabbing@decrement"), None);
  state::let_i(&T_CS!("\\<"), &T_CS!("\\@tabbing@untab"), None);
  state::let_i(&T_CS!("\\@tabbing@'"), &T_CS!("\\'"), None);
  state::let_i(&T_CS!("\\@tabbing@`"), &T_CS!("\\`"), None);
  state::let_i(&T_CS!("\\a"), &T_CS!("\\@tabbing@accent"), None);
  state::let_i(&T_CS!("\\'"), &T_CS!("\\@tabbing@flushright"), None);
  state::let_i(&T_CS!("\\`"), &T_CS!("\\@tabbing@hfil"), None);
  state::let_i(&T_CS!("\\pushtabs"), &T_CS!("\\@tabbing@pushtabs"), None);
  state::let_i(&T_CS!("\\poptabs"), &T_CS!("\\@tabbing@poptabs"), None);
}
