use crate::package::*;

LoadDefinitions!(state, {
  //======================================================================
  // TeX Book, Appendix B. p. 357

  DefPrimitive!("\\hrulefill", None);
  DefPrimitive!("\\dotfill", None);
  DefPrimitive!("\\rightarrowfill", None);
  DefPrimitive!("\\leftarrowfill", None);
  DefPrimitive!("\\upbracefill", None);
  DefPrimitive!("\\downbracefil", None);

  Let!("\\bye", "\\end");

  Let!("\\sp", T_SUPER!());
  Let!("\\sb", T_SUB!());

  DefPrimitive!("\\lx@thinmuskip", sub[stomach,(),state] {
    Tbox::new(arena::pin_static("\u{2009}"), None, None, Tokens!(T_CS!("\\,")),
      stored_map!("name"  => "thinspace", "isSpace" => true,
      "width" => state.lookup_value("\\thinmuskip")), state)
  });
  DefPrimitive!("\\lx@thinspace", sub[stomach,(),state] {
    Tbox::new(arena::pin_static("\u{2009}"), None, None, Tokens!(T_CS!("\\,")),
      stored_map!("name" => "thinspace", "width" => Dimension::from_str("0.16667em",state)?,
       "isSpace" => true), state)
  });
  DefMacro!(
    "\\,",
    r"\ifmmode\lx@thinmuskip\else\lx@thinspace\fi",
    protected => true
  );

  DefMacro!(
    "\\!",
    "\\ifmmode\\@math@negthinmuskip\\else\\@text@negthinmuskip\\fi"
  );

  DefPrimitive!("\\!", sub[stomach,(),state] {
    Tbox::new(arena::pin_static("\u{200B}"), None, None, Tokens!(T_CS!("\\!")),  // zero width space
      stored_map!("name"  => "negthinspace", "isSpace" => true,
      "width" => state.lookup_dimension("\\thinmuskip").unwrap().negate()), state)
  });

  DefPrimitive!("\\>", sub[stomach,(),state] {
    Tbox::new(arena::pin_static("\u{2005}"), None, None, Tokens!(T_CS!("\\>")),
      stored_map!("name"  => "medspace", "isSpace" => true,
      "width" => state.lookup_value("\\medmuskip")), state)
  });
  DefPrimitive!("\\;", sub[stomach, (), state] {
    Tbox::new(arena::pin_static("\u{2004}"), None, None, Tokens!(T_CS!("\\;")),
      stored_map!("name"  => "thickspace", "isSpace" => true,
      "width" => state.lookup_value("\\thickmuskip")), state)
  });

  Let!("\\:", "\\>");

  DefPrimitive!("\\ ", sub[stomach,(),state] {
    Tbox::new(arena::pin_static("\u{00A0}"), None, None, Tokens!(T_CS!("\\ ")),
      stored_map!("name" => "space", "isSpace" => true,
      "width" => Dimension::from_str("0.5em", state)?), state)
  });

  DefPrimitive!("\\\t", sub[stomach,(),state] {
    Tbox::new(arena::pin_static("\u{00A0}"), None, None, Tokens!(T_CS!("\\\t")),
      stored_map!("isSpace" => true, "width" => Dimension::from_str("1em",state)?), state)
  });

  DefPrimitive!("\\/", sub[stomach,(),state] {
    Tbox::new(arena::pin_static(""), None, None, Tokens!(T_CS!("\\/")),
      stored_map!("isSpace" => true, "name" => "italiccorr", "width" => Dimension::default()),state)
  });

});