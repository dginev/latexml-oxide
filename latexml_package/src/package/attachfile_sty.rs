use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("keyval");
  RequirePackage!("ifpdf");
  RequirePackage!("calc");
  RequirePackage!("color");
  // Perl L24-27: \attachfilesetup accumulates global keyval options
  DefMacro!("\\lx@attachfile@options", None);
  DefPrimitive!("\\attachfilesetup {}", sub[(opts)] {
    let cs = T_CS!("\\lx@attachfile@options");
    AddToMacro!(cs, opts);
  });
  // Perl attachfile.sty.ltxml L29-32: forward to internal \\lx@… CSes
  // with the accumulated global options list prepended to the per-call
  // keyvals. Previous Rust stubs dropped to raw "#2" / "#3", which
  // silently lost both the icon rendering and the file URL.
  DefMacro!("\\noattachfile []",
    "\\lx@noattachfile{\\lx@attachfile@options,#1}");
  DefMacro!("\\notextattachfile []{}",
    "\\lx@notextattachfile{\\lx@attachfile@options,#1}{#2}");
  DefMacro!("\\attachfile []{}",
    "\\lx@attachfile{\\lx@attachfile@options,#1}{#2}");
  DefMacro!("\\textattachfile []{}{}",
    "\\lx@textattachfile{\\lx@attachfile@options,#1}{#2}{#3}");

  // Perl L51-66 defines the four \\lx@… constructors with icon+color
  // keyval processing. Port keeps the surrounding <ltx:text> wrapper
  // but emits a fixed pushpin icon — keyval `icon`/`color` extraction
  // requires the full RequiredKeyVals machinery and an attachfileicon
  // dispatch table not yet exposed here. The wrappers preserve the file
  // path as an ltx:ref href, still more faithful than the previous
  // silent-drop stubs.
  DefConstructor!(
    "\\lx@noattachfile RequiredKeyVals",
    "<ltx:text>\u{1F4CC}</ltx:text>",
    mode => "restricted_horizontal"
  );
  DefConstructor!(
    "\\lx@notextattachfile RequiredKeyVals {}",
    "<ltx:text>#2</ltx:text>",
    mode => "restricted_horizontal"
  );
  DefConstructor!(
    "\\lx@attachfile RequiredKeyVals {}",
    "<ltx:ref href='#2'>\u{1F4CC}</ltx:ref>",
    mode => "restricted_horizontal"
  );
  DefConstructor!(
    "\\lx@textattachfile RequiredKeyVals {}{}",
    "<ltx:ref href='#2'>#3</ltx:ref>",
    mode => "restricted_horizontal"
  );
});
