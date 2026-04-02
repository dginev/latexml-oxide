//! algorithm2e.sty — Algorithm typesetting package
//! Perl: algorithm2e.sty.ltxml — complex package with custom line management
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("float");

  // Load the raw algorithm2e.sty for all its internal definitions
  InputDefinitions!("algorithm2e", extension => Some("sty".into()), noltxml => true);

  Let!("\\@mathsemicolon", "\\;");
  // Counter setup
  NewCounter!("algorithm", "");
  DefMacro!("\\fnum@algorithm", "\\algorithmcfname\\nobreakspace\\thealgorithm");
  DefMacro!("\\fnum@font@algorithm", "\\bf");
  DefMacro!("\\ext@algorithm", "loa");

  // {algorithm} environment
  DefEnvironment!("{algorithm}[]",
    "<ltx:float xml:id='#id' class='ltx_algorithm'>#tags<ltx:listing class='ltx_lst_numbers_left'><ltx:listingline>#body</ltx:listingline></ltx:listing></ltx:float>",
    mode => "internal_vertical",
    before_digest => {
      use crate::engine::latex_ch9_figures_and_tables::before_float;
      let _ = before_float("algorithm", None);
      Let!("\\par", "\\lx@algo@par");
      Let!("\\\\", "\\lx@algo@par");
      DefMacro!("\\;", "\\ifmmode\\@mathsemicolon\\else\\@endalgoln\\fi");
    },
    after_digest => sub[whatsit] {
      use crate::engine::latex_ch9_figures_and_tables::after_float;
      after_float(whatsit);
    }
  );
  // {algorithm*} and {algorithm2e} — same as {algorithm}
  Let!("\\algorithm*", "\\algorithm");
  Let!("\\endalgorithm*", "\\endalgorithm");
  Let!("\\algorithm2e", "\\algorithm");
  Let!("\\endalgorithm2e", "\\endalgorithm");

  DefMacro!("\\lx@algo@parbox[]{}{}", "#3");
  DefMacro!("\\lx@algo@strut SkipMatch:\\par", "");
  DefMacro!("\\@marker{}", "");

  // Par management — prevents double line breaks
  DefMacro!("\\lx@algo@par",
    "\\lx@algo@endline\\lx@algo@startline");
  DefMacro!("\\lx@algo@parx",
    "\\lx@algo@endline\\lx@algo@startline");
  DefMacro!("\\lx@algo@parb",
    "\\lx@algo@endline\\lx@algo@startline");

  // Block and group macros
  DefMacro!("\\algocf@group{}", "#1");
  DefMacro!("\\algocf@@@block{}{}", "#1 #2\\lx@algo@parb");
  DefMacro!("\\algocf@Vline{}", "\\lx@algo@endline\\lx@algo@startline\\lx@algo@advline #1\\lx@algo@pop@indentation");
  DefMacro!("\\algocf@Vsline{}", "\\lx@algo@endline\\lx@algo@startline\\lx@algo@advline #1\\lx@algo@pop@indentation");
  DefMacro!("\\algocf@Noline{}", "\\lx@algo@endline\\lx@algo@startline\\lx@algo@advlevel #1");

  // Semicolon handling
  DefMacro!("\\algocf@endline", sub[_args] {
    if state::lookup_bool("algorithm_dont_print_semicolon") {
      Ok(Tokens!())
    } else {
      Ok(Tokens::new(vec![T_OTHER!(";")]))
    }
  }, locked => true);
  DefMacro!("\\@endalgoln", "\\@endalgocfline");
  DefMacro!("\\@endalgocfline", "\\algocf@endline\\lx@algo@par");
  DefMacro!("\\PrintSemicolon", sub[_args] {
    state::assign_value("algorithm_dont_print_semicolon", false, Some(Scope::Global));
    Ok(Tokens!())
  }, locked => true);
  DefMacro!("\\DontPrintSemicolon", sub[_args] {
    state::assign_value("algorithm_dont_print_semicolon", true, Some(Scope::Global));
    Ok(Tokens!())
  }, locked => true);

  // Indentation management
  DefMacro!("\\lx@algo@advlevel", "\\lx@algo@push@indentation{\\lx@algo@indent}");
  DefMacro!("\\lx@algo@advline", "\\lx@algo@push@indentation{\\lx@algo@indentline}");
  DefMacro!("\\lx@algo@indent", "\\hskip\\skiprule\\hskip\\skiptext");
  DefMacro!("\\lx@algo@indentline", "\\hskip\\skiprule\\lx@algo@rule\\hskip\\skiptext");
  DefConstructor!("\\lx@algo@rule", "<ltx:rule width='1px' height='100%'/>");

  // Line start/end constructors
  DefConstructor!("\\lx@algo@@startline", "<ltx:listingline xml:id='#id'>");
  DefConstructor!("\\lx@algo@@endline", "</ltx:listingline>");
  DefMacro!("\\lx@algo@startline", "\\lx@algo@@startline");
  DefMacro!("\\lx@algo@endline", "\\lx@algo@@endline");

  // Indentation prepending
  DefConstructor!("\\lx@prepend@indentation@{}", "");

  DefMacro!("\\lx@strippar{}", "#1");
});
