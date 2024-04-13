use crate::package::*;

#[rustfmt::skip]
LoadDefinitions!({
  //======================================================================
  // C.6.3 The list and trivlist environments.
  //======================================================================
  // Generic lists are given a way to format the item label, and presumably
  // a counter.

  DefConditional!("\\if@nmbrlist");
  DefMacro!("\\@listctr", "");
  DefPrimitive!("\\usecounter{}", sub[(counter)] {
    let counter = Expand!(counter).to_string();
    if counter.is_empty() {
      begin_itemize("list", None, BeginItemizeOptions::default())?;
    } else {
      begin_itemize("list", Some(&counter), BeginItemizeOptions {
        nolevel:true,
        ..BeginItemizeOptions::default() })?;
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
    before_digest => { bgroup(); });
  // Close the anonymous list if we're still within one.
  DefConstructor!("\\endlx@list", sub[document] {
    document.maybe_close_element("ltx:itemize")?; },
    before_digest => { egroup()?; });

  DefConstructor!("\\list@item OptionalUndigested",
    "<ltx:item xml:id='#id' itemsep='#itemsep'>#tags",
    properties => sub[args] {
      let undigested = args[0].as_ref().map(|d| d.raw_tokens()).unwrap_or_default();
      ref_step_item_counter(undigested) }
  );

  DefEnvironment!("{trivlist}",
    "<ltx:itemize>#body</ltx:itemize>",
    properties => {
      begin_itemize("trivlist", None, BeginItemizeOptions::default()) },
    before_digest_end => { Digest!("\\par")?; }
  );

  DefMacro!("\\trivlist@item", "\\par\\trivlist@item@");
  DefConstructor!("\\trivlist@item@ OptionalUndigested",
    "<ltx:item xml:id='#id' itemsep='#itemsep'><ltx:tags><ltx:tag>#tag</ltx:tag></ltx:tags>",
    // At least an empty tag! ?
    properties => sub[args] {
      if let Some(ref arg) = args[0] {
        if let DigestedData::Postponed(ref tag_tokens) = arg.data() {
          let tag_expanded = Expand!(tag_tokens.clone());
          let tag = stomach::digest(tag_expanded)?;
          Ok(stored_map!("tag" => tag))
        } else {
          Ok(SymHashMap::default())
        }
      } else {
          Ok(SymHashMap::default())
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
