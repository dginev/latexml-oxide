use core::{Core};
use core::token::*;
use core::mouth::Mouth;
// use common::{Error};

pub fn input_content(core : &mut Core, request : String) -> Result<(),()> {
  match find_file(request, false) { // TODO: type => $options{type}, noltxml => 1
    Some(path) => Ok(load_tex_content(core, path)),
    None => Err(())
      // TODO:
      // Error("missing_file", request, state.get_stomach().get_gullet(),
      // "Can't find TeX file "+request, maybeReportSearchPaths(state)))
  }
}

pub fn load_tex_content(core: &mut Core, path : String) {
  let mut mouth = Mouth{notes: true, ..Mouth::default()};
  mouth.open(&path, &mut core.state);
  // TODO: 
  // If there is a file-specific declaration file (name.latexml), load it first!
  // let file = path;
  // file =~ s/\.tex//;
  // if (my $conf = !pathname_is_literaldata($pathname)
  //   && pathname_find("$file.latexml", paths => LookupValue('SEARCHPATHS'))) {
  //   loadLTXML($conf, $conf); }

  // TODO: Caching
  // content => LookupValue($pathname . '_contents')

  // Open a mouth for that TeX content
  let gullet = core.stomach.get_gullet();
  gullet.open_mouth(mouth, true);

}

pub fn find_file(request : String, forbid_ltxml : bool) -> Option<String> {
  // TODO: Actually find it!
  Some(request)

}

pub fn coerce_cs(t : String) -> Token {
  T_CS(t)
}

/// Macros and pool come at the end, so that they load seamlessly
use core::definition::Expandable;
#[macro_export]
macro_rules! DefMacroI(
    ($cs:expr, $paramlist:expr, $expansion:expr, $state:expr) => (
      {
        use $crate::core::definition::Expandable;
        use $crate::core::package;
        $state.install_definition(Expandable { cs: package::coerce_cs( $cs ), paramlist: $paramlist, expansion: $expansion, ..Expandable::default()}, &None);
      }
    )
  );


// macro_rules! DefMacroI(
//     ($cs:expr, $paramlist:expr, $expansion:expr, $state:expr) => (
//       {//, $options:tt
//       // Optimization: Defer till macro actually used
//       // if !$cs.is_empty() { // && $options{mathactive}
//         // $state.assign_mathcode($cs, 0x8000, $options{scope}); }
//       $state.install_definition(Expandable{ cs: coerce_cs( $cs ), paramlist: $paramlist, expansion: $expansion});//, %options), $options{scope});
//       // if $options{locked} {
//       //   $state.assign_value(ToString($cs)+":locked", true, "global")
//       // }
//       }
//     )
//   );

pub mod pool;