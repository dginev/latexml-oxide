use core::{Core};
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