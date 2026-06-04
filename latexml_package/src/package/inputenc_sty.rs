use crate::prelude::*;

//**********************************************************************
fn set_input_encoding(encoding: &str) -> Result<()> {
  // Initially disable all odd & upper half-plane chars
  let undef_cs = T_CS!("\\@inpenc@undefined");
  for code in (0..=8u8)
    .chain(std::iter::once(0x0Bu8))
    .chain(0x0E..=0x1Eu8)
    .chain(128..=255u8)
  {
    let ch = code as char;
    AssignCatcode!(ch, Catcode::ACTIVE);
    Let!(T_ACTIVE!(ch), undef_cs);
  }
  state::set_input_encoding(None); // Disable the perl-level decoding, if any.

  // Then load TeX's input encoding definitions.
  // Then load TeX's input encoding definitions.
  input_definitions(encoding, InputDefinitionOptions {
    extension: Some("def".into()),
    reloadable: true,
    ..InputDefinitionOptions::default()
  })?;
  // NOTE: INPUT_ENCODING is never actually used anywhere!
  // So, presumably either Perl is magically converting to utf8
  // or more likely, treating the bytes as (misinterpreted?) utf8?
  // In latter case, perhaps it doesn't matter as long as we end up with the same bytes in/out???
  assign_value("INPUT_ENCODING", encoding.to_string(), None);
  let encoding_tokenized = TokenizeInternal!(encoding);
  def_macro(T_CS!("\\inputencodingname"), None, encoding_tokenized, None)
}

LoadDefinitions!({
  //**********************************************************************
  DefPrimitive!("\\DeclareInputMath {Number} {}", sub[(code,expansion)] {
    let ch = code.value_of() as u8 as char;
    AssignCatcode!(ch, Catcode::ACTIVE);
    DefMacro!(T_ACTIVE!(ch), None, expansion);
  });

  DefPrimitive!("\\DeclareInputText {Number} {}", sub[(code, expansion)] {
    let ch = code.value_of() as u8 as char;
    AssignCatcode!(ch, Catcode::ACTIVE);
    DefMacro!(T_ACTIVE!(ch), None, expansion);
  });

  DefMacro!("\\IeC{}", "#1");

  DefMacro!("\\@inpenc@undefined", {
    let enc = lookup_string("INPUT_ENCODING");
    let message = s!(
      "Keyboard character used is undefined in inputencoding {}",
      enc
    );
    Error!("unexpected", "<char>", message);
  });

  // `\@inpenc@test` is inputenc.sty's one-shot initialization guard.
  // The raw source (inputenc.sty L79) defines it inline as
  //   `\gdef\@inpenc@test{\global\let\@inpenc@test\relax}`
  // — i.e. self-defining and self-disabling — but our binding
  // short-circuits the raw-load before reaching that point, so
  // downstream code (e.g. utf8.def L195, the encoding .def files,
  // and `\DeclareInputMath`) hits `Error:undefined`. Mirror the
  // upstream's effective behavior: no-op (post-init state).
  // Witness: 15 papers in R-stages affected (~1 paper per stage).
  DefMacro!("\\@inpenc@test", None);

  DefPrimitive!("\\inputencoding{}", sub[(encoding)] {
    set_input_encoding(&Expand!(encoding).to_string())?;
  });

  DeclareOption!(None, {
    set_input_encoding(&Expand!(T_CS!("\\CurrentOption")).to_string())?;
  });

  ProcessOptions!();

  //**********************************************************************
});
