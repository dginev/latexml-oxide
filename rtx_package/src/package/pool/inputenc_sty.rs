use crate::package::*;

//**********************************************************************
fn set_input_encoding(encoding: &str, stomach: &mut Stomach, state: &mut State) -> Result<()> {
  // Initially disable all odd & upper half-plane chars
  // TODO:
  // for code in ((0 .. 8), 0xB, (0xE .. 0x1E), (128 .. 255)) {
  //   let ch : char = code as char;
  //   AssignCatcode!(ch, Catcode::ACTIVE);
  //   LetI!(&T_ACTIVE!(ch), T_CS!("\\@inpenc@undefined")); 
  // }
  state.input_encoding = None; // Disable the state-level decoding, if any.

  // Then load TeX's input encoding definitions.
  input_definitions(encoding,
    InputDefinitionOptions { extension: Some("def"), ..InputDefinitionOptions::default() },
    stomach, state)?;
  // NOTE: INPUT_ENCODING is never actually used anywhere!
  // So, presumably either Perl is magically converting to utf8
  // or more likely, treating the bytes as (misinterpreted?) utf8?
  // In latter case, perhaps it doesn't matter as long as we end up with the same bytes in/out???
  state.assign_value("INPUT_ENCODING", encoding.to_string(), None);
  let encoding_tokenized = TokenizeInternal!(encoding);
  def_macro(T_CS!("\\inputencodingname"), None, encoding_tokenized, None, state);
  Ok(())
}

LoadDefinitions!(outer_stomach, state, {

  //**********************************************************************
  DefPrimitive!("\\DeclareInputMath {Number} {}", sub[stomach, args, state] {
    unpack_to_token!(args => code, expansion);
    let ch = code.to_number().value_of() as u8 as char;
    AssignCatcode!(ch, Catcode::ACTIVE);
    DefMacroI!(T_ACTIVE!(ch), None, expansion);
  });

  DefPrimitive!("\\DeclareInputText {Number} {}", sub[stomach, args, state] {
    unpack_to_token!(args => code, expansion);
    let ch = code.to_number().value_of() as u8 as char;
    AssignCatcode!(ch, Catcode::ACTIVE);
    DefMacroI!(T_ACTIVE!(ch), None, expansion); 
  });

  DefMacro!("\\IeC{}", "#1");

  DefMacro!("\\@inpenc@undefined", sub[gullet, args, state] {
    let message = s!("Keyboard character used is undefined in inputencoding {}", state.input_encoding.as_ref().unwrap());
    Error!("unexpected", "<char>", gullet, state, message);
  });

  DefPrimitive!("\\inputencoding{}", sub[stomach, args, state] {
    unpack_to_token!(args => encoding);
    let gullet = stomach.get_gullet_mut();
    set_input_encoding(&Expand!(encoding, gullet).to_string(), stomach, state)?;
  });

  DeclareOption!(None, sub[stomach, state] {
    let gullet = stomach.get_gullet_mut();
    set_input_encoding(&Expand!(T_CS!("\\CurrentOption"), gullet).to_string(), stomach, state)?; 
  });

  ProcessOptions!(outer_stomach);

  //**********************************************************************

});
