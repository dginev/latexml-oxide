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
    unpack_to_token!(args => counter);
    let gullet = stomach.get_gullet_mut();
    let counter = Expand!(counter, gullet);
    begin_itemize("list", Some(&counter.to_string()), !counter.is_empty(), state)?;
  });

  DefMacro!("\\list{}{}",
  "\\let\\@listctr\\@empty#2\\ifx\\@listctr\\@empty\\usecounter{}\\fi\\expandafter\\def\\csname fnum@\\@listctr\\endcsname{#1}\\lx@list");
  DefMacro!("\\endlist", "\\endlx@list");

  DefConstructor!("\\lx@list DigestedBody",
    "<ltx:itemize>#1</ltx:itemize>",
    before_digest => before_digest!(stomach, state, { stomach.bgroup(state); }));
  DefPrimitive!("\\endlx@list", sub[stomach, args, state] { stomach.egroup(state)?; });

  DefConstructor!("\\list@item OptionalUndigested",
    "<ltx:item xml:id='#id' itemsep='#itemsep'>#tags",
    properties => properties!(sub[stomach, args, state] {
      unpack!(args => tag);
      ref_step_item_counter(&tag.to_string(), stomach, state) })
  );

  // This isn't quite right, although it seems right for deep, internal uses with a single \item.
  // Perhaps we need to check trivlist's afterwards and if they are just a single item,
  // reduce it to an ltx:p ??
  // DefMacro!('\trivlist@item[]', '');
  // DefEnvironment!('{trivlist}',
  //   '<ltx:p>#body</ltx:p>',
  //   beforeDigest => sub { Let('\item', '\trivlist@item'); });

  DefEnvironment!("{trivlist}",
    "<ltx:itemize>#body</ltx:itemize>",
    properties      => properties!(stomach, args, state, { begin_itemize("trivlist", None, false, state) }),
    before_digest_end => before_digest!({ Digest!("\\par")?; })
  );

  DefMacro!("\\trivlist@item", "\\par\\trivlist@item@");
  DefConstructor!("\\trivlist@item@ OptionalUndigested",
    "<ltx:item xml:id='#id' itemsep='#itemsep'>\
      <ltx:tags><ltx:tag>#tag</ltx:tag></ltx:tags>",    // At least an empty tag! ?
    properties => properties!(stomach, args, state, { 
      // TODO: So, I hear you like boilerplate...
      // in Perl this was the super simple:
      // Digest(Expand($_[1]))
      if let Some(ref tag) = args[0] {
        let gullet = stomach.get_gullet_mut();
        let expanded = Expand!(tag, gullet);
        let digested = stomach.digest(expanded, state)?;
        Ok(map!("tag" => Stored::Digested(Box::new(digested))))
      } else {
        Ok(HashMap::new())
      }
    })
  );

  DefRegister!("\\topsep"             => Glue::new(0.0));
  DefRegister!("\\partopsep"          => Glue::new(0.0));
  DefRegister!("\\lx@default@itemsep" => Glue::new(0.0));
  DefRegister!("\\itemsep"            => Glue::new(0.0));
  DefRegister!("\\parsep"             => Glue::new(0.0));
  DefRegister!("\\@topsep"            => Glue::new(0.0));
  DefRegister!("\\@topsepadd"         => Glue::new(0.0));
  DefRegister!("\\@outerparskip"      => Glue::new(0.0));
  DefRegister!("\\leftmargin"         => Dimension::new(0.0));
  DefRegister!("\\rightmargin"        => Dimension::new(0.0));
  DefRegister!("\\listparindent"      => Dimension::new(0.0));
  DefRegister!("\\itemindent"         => Dimension::new(0.0));
  DefRegister!("\\labelwidth"         => Dimension::new(0.0));
  DefRegister!("\\labelsep"           => Dimension::new(0.0));
  DefRegister!("\\@totalleftmargin"   => Dimension::new(0.0));
  DefRegister!("\\leftmargini"        => Dimension::new(0.0));
  DefRegister!("\\leftmarginii"       => Dimension::new(0.0));
  DefRegister!("\\leftmarginiii"      => Dimension::new(0.0));
  DefRegister!("\\leftmarginiv"       => Dimension::new(0.0));
  DefRegister!("\\leftmarginv"        => Dimension::new(0.0));
  DefRegister!("\\leftmarginvi"       => Dimension::new(0.0));
  DefRegister!("\\@listdepth"         => Number::new(0.0));
  DefRegister!("\\@itempenalty"       => Number::new(0.0));
  DefRegister!("\\@beginparpenalty"   => Number::new(0.0));
  DefRegister!("\\@endparpenalty"     => Number::new(0.0));
  DefRegister!("\\labelwidthi"        => Dimension::new(0.0));
  DefRegister!("\\labelwidthii"       => Dimension::new(0.0));
  DefRegister!("\\labelwidthiii"      => Dimension::new(0.0));
  DefRegister!("\\labelwidthiv"       => Dimension::new(0.0));
  DefRegister!("\\labelwidthv"        => Dimension::new(0.0));
  DefRegister!("\\labelwidthvi"       => Dimension::new(0.0));

  DefRegister!("\\@itemdepth" => Number::new(0.0));

});
