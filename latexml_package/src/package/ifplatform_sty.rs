use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl ifplatform.sty.ltxml L36-39 reads `$^O` at binding-load time and
  // branches on Win32/linux/darwin/cygwin. Rust equivalent uses
  // `std::env::consts::OS` ("windows"/"linux"/"macos"/…). Previous Rust
  // hardcoded Linux for every platform — on macOS builds `\ifmacosx` was
  // false and `\platformname` came out as `Linux`, contrary to Perl.
  let os = std::env::consts::OS;
  let is_windows = os == "windows";
  let is_linux   = os == "linux";
  let is_macos   = os == "macos";
  let is_cygwin  = false; // Rust has no dedicated cygwin target; matches
                          // Perl's fall-through for uncommon OSes.

  DefMacro!("\\windowsname",    "Windows");
  DefMacro!("\\notwindowsname", "*NIX");
  DefMacro!("\\linuxname",      "Linux");
  DefMacro!("\\macosxname",     "Mac\\,OS\\,X");
  DefMacro!("\\cygwinname",     "Cygwin");

  // Perl L32: '\ifshellescape' hardcoded true — LaTeXML runs the macro
  // expansion so shell-escape semantics don't apply.
  DefConditional!("\\ifshellescape", { true });

  // Perl L44-57: `\platformname` Let-aliased to whichever of
  // \linuxname/\windowsname/\macosxname/\cygwinname matches $^O, falling
  // back to `\unknownplatform` which carries the POSIX uname. Runtime
  // string body → use the def_macro / let_alias function forms instead
  // of the macro (the macro shape only takes literals).
  let (platform_cs, unknown_name): (&str, String) = if is_windows {
    ("\\windowsname", "[Unknown]".into())
  } else if is_linux {
    ("\\linuxname", "[Unknown]".into())
  } else if is_macos {
    ("\\macosxname", "[Unknown]".into())
  } else if is_cygwin {
    ("\\cygwinname", "[Unknown]".into())
  } else {
    ("\\unknownplatform", os.to_string())
  };
  def_macro(T_CS!("\\unknownplatform"), None,
    Tokenize!(&unknown_name), None)?;
  state::let_i(&T_CS!("\\platformname"), &T_CS!(platform_cs), None);

  // Perl L63-66: one conditional per OS, returning the same-named sub.
  DefConditional!("\\ifwindows", { is_windows });
  DefConditional!("\\iflinux",   { is_linux });
  DefConditional!("\\ifmacosx",  { is_macos });
  DefConditional!("\\ifcygwin",  { is_cygwin });
});
