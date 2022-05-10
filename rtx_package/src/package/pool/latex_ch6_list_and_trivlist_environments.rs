use crate::package::*;
LoadDefinitions!(state, {
  //======================================================================
  // C.6.3 The list and trivlist environments.
  //======================================================================
  // Generic lists are given a way to format the item label, and presumably
  // a counter.

  DefConditional!("\\if@nmbrlist");
  DefMacro!("\\@listctr", "");
  DefPrimitive!("\\usecounter{}", sub[stomach, args, state] {
    unpack!(args => counter);
    let gullet = stomach.get_gullet_mut();
    let counter = Expand!(counter, gullet, state).to_string();
    if counter.is_empty() {
      begin_itemize("list", None, BeginItemizeOptions::default(), stomach, state)?;
    } else {
      begin_itemize("list", Some(&counter), BeginItemizeOptions {
        nolevel:true,
        ..BeginItemizeOptions::default() },
        stomach, state)?;
    }
  });

  DefMacro!(
    r"\list{}{}",
    r"\let\@listctr\@empty#2\ifx\@listctr\@empty\usecounter{}\fi\expandafter\def\csname fnum@\@listctr\endcsname{#1}\lx@list"
  );
  DefMacro!("\\endlist", "\\endlx@list");

  // Start an anonymous list (often misused)
  DefConstructor!("\\lx@list",
    "<ltx:itemize>",
    before_digest => sub[stomach, state] { stomach.bgroup(state); });
  // Close the anonymous list if we're still within one.
  DefConstructor!("\\endlx@list", sub[document, state] {
    document.maybe_close_element("ltx:itemize", state)?; },
    before_digest => sub[stomach,state] { stomach.egroup(state)?; });

  DefConstructor!("\\list@item OptionalUndigested",
    "<ltx:item xml:id='#id' itemsep='#itemsep'>#tags",
    properties => sub[stomach, args, state] {
      let undigested = args[0].as_ref().map(|d| d.raw_tokens());
      ref_step_item_counter(undigested, stomach, state) }
  );

  DefEnvironment!("{trivlist}",
    "<ltx:itemize>#body</ltx:itemize>",
    properties => sub[stomach, args, state] { begin_itemize("trivlist", None, BeginItemizeOptions::default(), stomach, state) },
    before_digest_end => { Digest!("\\par")?; }
  );

  DefMacro!("\\trivlist@item", "\\par\\trivlist@item@");
  DefConstructor!("\\trivlist@item@ OptionalUndigested",
    "<ltx:item xml:id='#id' itemsep='#itemsep'>\
      <ltx:tags><ltx:tag>#tag</ltx:tag></ltx:tags>",    // At least an empty tag! ?
    properties => sub[stomach, args, state] {
      if let Some(Digested::Postponed(ref tag_tokens)) = args[0] {
        let mut gullet = stomach.get_gullet_mut();
        let tag_expanded = Expand!(tag_tokens, gullet, state);
        let tag = stomach.digest(tag_expanded, state)?;
        Ok(stored_map!("tag" => tag))
      } else {
        Ok(HashMap::new())
      }
    }
  );

  DefRegister!("\\topsep"             => Glue::new(0));
  DefRegister!("\\partopsep"          => Glue::new(0));
  DefRegister!("\\lx@default@itemsep" => Glue::new(0));
  DefRegister!("\\itemsep"            => Glue::new(0));
  DefRegister!("\\parsep"             => Glue::new(0));
  DefRegister!("\\@topsep"            => Glue::new(0));
  DefRegister!("\\@topsepadd"         => Glue::new(0));
  DefRegister!("\\@outerparskip"      => Glue::new(0));
  DefRegister!("\\leftmargin"         => Dimension::new(0));
  DefRegister!("\\rightmargin"        => Dimension::new(0));
  DefRegister!("\\listparindent"      => Dimension::new(0));
  DefRegister!("\\itemindent"         => Dimension::new(0));
  DefRegister!("\\labelwidth"         => Dimension::new(0));
  DefRegister!("\\labelsep"           => Dimension::new(0));
  DefRegister!("\\@totalleftmargin"   => Dimension::new(0));
  DefRegister!("\\leftmargini"        => Dimension::new(0));
  DefRegister!("\\leftmarginii"       => Dimension::new(0));
  DefRegister!("\\leftmarginiii"      => Dimension::new(0));
  DefRegister!("\\leftmarginiv"       => Dimension::new(0));
  DefRegister!("\\leftmarginv"        => Dimension::new(0));
  DefRegister!("\\leftmarginvi"       => Dimension::new(0));
  DefRegister!("\\@listdepth"         => Number::new(0));
  DefRegister!("\\@itempenalty"       => Number::new(0));
  DefRegister!("\\@beginparpenalty"   => Number::new(0));
  DefRegister!("\\@endparpenalty"     => Number::new(0));
  DefRegister!("\\labelwidthi"        => Dimension::new(0));
  DefRegister!("\\labelwidthii"       => Dimension::new(0));
  DefRegister!("\\labelwidthiii"      => Dimension::new(0));
  DefRegister!("\\labelwidthiv"       => Dimension::new(0));
  DefRegister!("\\labelwidthv"        => Dimension::new(0));
  DefRegister!("\\labelwidthvi"       => Dimension::new(0));

  DefRegister!("\\@itemdepth" => Number::new(0));
});
