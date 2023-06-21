use crate::package::*;

//**********************************************************************
fn set_input_encoding(encoding: &str) -> Result<()> {
  // Initially disable all odd & upper half-plane chars
  // TODO:
  // for code in ((0 .. 8), 0xB, (0xE .. 0x1E), (128 .. 255)) {
  //   let ch : char = code as char;
  //   AssignCatcode!(ch, Catcode::ACTIVE);
  //   Let!(&T_ACTIVE!(ch), T_CS!("\\@inpenc@undefined"));
  // }
  state::input_encoding = None; // Disable the state::level decoding, if any.

  // Then load TeX's input encoding definitions.
  input_definitions(
    encoding,
    InputDefinitionOptions {
      extension: Some("def".into()),
      ..InputDefinitionOptions::default()
    },
    stomach,
  )?;
  // NOTE: INPUT_ENCODING is never actually used anywhere!
  // So, presumably either Perl is magically converting to utf8
  // or more likely, treating the bytes as (misinterpreted?) utf8?
  // In latter case, perhaps it doesn't matter as long as we end up with the same bytes in/out???
  state_mut!().assign_value("INPUT_ENCODING", encoding.to_string(), None);
  let encoding_tokenized = TokenizeInternal!(encoding);
  def_macro(
    T_CS!("\\inputencodingname"),
    None,
    encoding_tokenized,
    None,
  )
}

LoadDefinitions!(outer_stomach, {
  //**********************************************************************
  DefPrimitive!("\\DeclareInputMath {Number} {}", sub[ (code,expansion)] {
    let ch = code.value_of() as u8 as char;
    AssignCatcode!(ch, Catcode::ACTIVE);
    DefMacro!(T_ACTIVE!(ch), None, expansion);
  });

  DefPrimitive!("\\DeclareInputText {Number} {}", sub[ (code, expansion)] {
    let ch = code.value_of() as u8 as char;
    AssignCatcode!(ch, Catcode::ACTIVE);
    DefMacro!(T_ACTIVE!(ch), None, expansion);
  });

  DefMacro!("\\IeC{}", "#1");

  DefMacro!("\\@inpenc@undefined", sub[ ()] {
    let message = s!("Keyboard character used is undefined in inputencoding {}",
      state::input_encoding.as_ref().unwrap());
    Error!("unexpected", "<char>", gullet,  message);
  });

  DefPrimitive!("\\inputencoding{}", sub[ (encoding)] {
    let gullet = gullet_mut!();
    set_input_encoding(&Expand!(encoding).to_string())?;
  });

  DeclareOption!(None, sub[stomach] {
    let gullet = gullet_mut!();
    set_input_encoding(&Expand!(T_CS!("\\CurrentOption")).to_string())?;
  });

  ProcessOptions!(outer_stomach);

  //**********************************************************************
});
