use crate::prelude::*;

/// KERNEL_CAPABILITIES K10 — the pdfTeX byte mouth, package-scoped.
///
/// pdfTeX is an 8-bit engine: the mouth yields one char token per BYTE
/// (tex.web §30-31, §341-356) against a 256-entry `\catcode` table, and
/// utf8.def:174-190 then makes the bytes 0x80..0xC1 active-and-invalid and
/// the lead bytes 0xC2..0xF4 active octet readers that reassemble the code
/// point. Our default mouth decodes UTF-8 into code points and `utf8_def.rs`
/// deliberately keeps them native (the bibarts fix; a global byte mouth would
/// split every accented Latin character — rejected, K10). The packages that
/// are BUILT on the byte model — kotexutf (kotexutf-core.tex:100-136 lead-byte
/// readers, kotexutf.sty josa control symbols over a lead byte), dhucs
/// (dhucs.sty:44 `\ifx 가가` byte probe), CJK (CJK.sty:896-968
/// `\CJK@makeActive` + `\CJK@XXX` byte arithmetic) — switch the mouth to bytes
/// for the rest of the run and install utf8.def's activation, so their raw
/// code runs as under pdfTeX; the packages' own funnels are then bound to
/// emit the reassembled code point (`kotexutf_sty.rs`, `cjk_sty.rs`).
/// No-op under the LuaTeX persona (a Unicode engine). Returns whether the
/// byte mouth is on.
pub fn enable_pdftex_byte_mouth() -> Result<bool> {
  if lookup_bool("LUATEX_PROFILE") {
    return Ok(false);
  }
  if lookup_bool("PDFTEX_BYTE_MOUTH") {
    return Ok(true);
  }
  AssignValue!("PDFTEX_BYTE_MOUTH" => true, Some(Scope::Global));
  state::set_input_encoding(Some("bytes".to_string()));
  install_utf8_byte_activation()?;
  Ok(true)
}

/// utf8.def:174-190 under the byte mouth: what the real `utf8.def` installs
/// and our `utf8_def.rs` binding suppresses for the native-code-point mouth.
/// Re-applied by `utf8_def.rs` whenever inputenc reloads utf8 while the byte
/// mouth is on (kotexutf.sty:34, CJKutf8.sty:26 both `\RequirePackage[utf8]
/// {inputenc}` after the switch).
pub fn install_utf8_byte_activation() -> Result<()> {
  // utf8.def:174-176: 0x80..0xC1 active and invalid on their own —
  // inputenc's undefined-character handler here (`\@inpenc@undefined`, the
  // same "keyboard character undefined" report as `\UTFviii@invalid@err`);
  // 0xF5..0xFF (utf8.def:190-192, invalid too) are left as they are.
  let undef_cs = T_CS!("\\@inpenc@undefined");
  for code in 0x80..=0xC1u8 {
    let ch = code as char;
    assign_catcode(ch, Catcode::ACTIVE, Some(Scope::Global));
    Let!(T_ACTIVE!(ch), undef_cs.clone(), Scope::Global);
  }
  // utf8.def:177-190: each lead byte is a PARAMETERLESS protected active
  // character expanding to its octet reader applied to itself
  // (`\protected\edef~{\noexpand\UTFviii@three@octets\noexpand~}`), so the
  // packages' re-wrapping idiom `\protected\edef~{\unexpanded\expandafter{~}}`
  // (cjkutf8-ko.sty:150-154) sees one clean level; the reader takes the lead
  // and its continuation bytes and yields the code point — here directly as
  // the character, the shape utf8.def's `\u8:…` names resolve to under our
  // native-code-point `\DeclareUnicodeCharacter`.
  DefMacro!("\\lx@utfviii@two@octets{}{}", sub[args] { Ok(utf8_octets_of(&args)) });
  DefMacro!("\\lx@utfviii@three@octets{}{}{}", sub[args] { Ok(utf8_octets_of(&args)) });
  DefMacro!("\\lx@utfviii@four@octets{}{}{}{}", sub[args] { Ok(utf8_octets_of(&args)) });
  for code in 0xC2..=0xF4u8 {
    let ch = code as char;
    assign_catcode(ch, Catcode::ACTIVE, Some(Scope::Global));
    let act = T_ACTIVE!(ch);
    let reader = if code <= 0xDF {
      "\\lx@utfviii@two@octets"
    } else if code <= 0xEF {
      "\\lx@utfviii@three@octets"
    } else {
      "\\lx@utfviii@four@octets"
    };
    let body = Tokens!(T_CS!(reader), act);
    DefMacro!(act, None, body, protected => true);
  }
  Ok(())
}

/// The byte value a byte-mouth macro argument carries: the first character
/// token (active or other) of the argument, when its code is a byte.
pub fn byte_of_arg(arg: &ArgWrap) -> Option<u8> {
  let code = match arg {
    ArgWrap::Token(t) => t.get_charcode(),
    ArgWrap::Tokens(ts) => ts
      .unlist_ref()
      .iter()
      .find(|t| t.code != Catcode::CS)
      .map_or(256, |t| t.get_charcode()),
    _ => 256,
  };
  u8::try_from(code).ok()
}

/// UTF-8 reassembly of a lead byte and its continuation-byte arguments: the
/// code point as one OTHER character token; an invalid sequence yields the
/// bytes back unchanged (utf8.def:113 `\UTFviii@invalid@err` reports; we keep
/// the text).
/// The reader's arguments are the lead byte and its continuation bytes.
pub fn utf8_octets_of(args: &[ArgWrap]) -> Tokens {
  match args
    .split_first()
    .and_then(|(lead, rest)| byte_of_arg(lead).map(|l| (l, rest)))
  {
    Some((lead, rest)) => utf8_octets(lead, rest),
    None => Tokens::new(Vec::new()),
  }
}

pub fn utf8_octets(lead: u8, args: &[ArgWrap]) -> Tokens {
  let mut bytes = vec![lead];
  bytes.extend(args.iter().filter_map(byte_of_arg));
  if let Ok(text) = str::from_utf8(&bytes)
    && let Some(ch) = text.chars().next()
    && bytes.len() == args.len() + 1
  {
    // utf8.def:55-90: a declared `\u8:<bytes>` (`\DeclareUnicodeCharacter`,
    // the `.dfu` files) wins over the plain character.
    let u8_name = T_CS!(s!("\\u8:{}", ch));
    if lookup_definition(&u8_name).ok().flatten().is_some() {
      return Tokens!(u8_name);
    }
    return Tokens!(Token {
      text: pin_char(ch),
      code: Catcode::OTHER,
      #[cfg(feature = "token-locators")]
      loc: 0,
    });
  }
  let mut out = vec![Token {
    text: pin_char(lead as char),
    code: Catcode::OTHER,
    #[cfg(feature = "token-locators")]
    loc: 0,
  }];
  for a in args {
    if let Some(ts) = a.clone().owned_tokens() {
      out.extend(ts.unlist());
    }
  }
  Tokens::new(out)
}

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
  // Disable the perl-level decoding, if any — unless the pdfTeX byte mouth is
  // on (K10): a package reloading `[utf8]{inputenc}` after the switch
  // (kotexutf.sty:34) keeps reading bytes, as pdfTeX would.
  if !lookup_bool("PDFTEX_BYTE_MOUTH") {
    state::set_input_encoding(None);
  }

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
  let encoding_tokenized = TokenizeInternal!(TeXString::assembled(encoding.to_string()));
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
