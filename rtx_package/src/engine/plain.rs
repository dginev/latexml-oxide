use crate::prelude::*;
LoadDefinitions!({


  // If folks start using plain TeX macros, and never load LaTeX.pool,
  // they might benefit from a ltx-plain.css?
  DefMacro!("\\beginsection Until:\\par", r"\@beginsection{{\bf #1}}");
  DefConstructor!("\\@beginsection {}",
    "<ltx:section><ltx:title>#1</ltx:title>");


  // POSSIBLY #1 is a name or reference number and  #2 is the theoremm TITLE
  //  If so, how do know when the theorem ends?
  DefMacro!(T_CS!("\\proclaim"), 
    parse_def_parameters(&T_CS!("\\proclaim"), Tokenize!("#1. #2\\par"))?,
    Some(r"\@proclaim{{\bf #1}}{{\sl #2}}".into()));
  DefConstructor!("\\@proclaim{}{}",
    "<ltx:theorem><ltx:title font='#titlefont' _force_font='true' >#title</ltx:title>#2",
    after_construct => sub[doc,_args] { doc.maybe_close_element("ltx:theorem")?; },
    properties     => sub[args] {
      if let Some(ref title) = args[0] {
        Ok(stored_map!("title" => title, "titlefont" => title.get_font()?)) 
      } else { Ok(SymHashMap::default()) }
    });


  Let!("\\empty", "\\lx@empty");
});