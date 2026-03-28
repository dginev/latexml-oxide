import re

with open('latexml_core/src/common/mathchar.rs', 'r') as f:
    content = f.read()

# 1. Update MathCharProps to include reversion and font
content = re.sub(
    r'pub struct MathCharProps \{([^}]*?)\}',
    r'pub struct MathCharProps {\1  pub reversion: Option<crate::tokens::Tokens>,\n  pub font: Option<crate::common::font::Font>,\n}',
    content
)

# 2. Update mathchar_props_from_info and mathchar_props_from_unicode to initialize them
content = re.sub(
    r'mathstyle: None,\n\s*\}\n\}',
    r'mathstyle: None,\n    reversion: None,\n    font: None,\n  }\n}',
    content
)

# 3. Rewrite decode_math_char
old_fn = re.search(r'pub fn decode_math_char\(mut n: u16\) -> Result<MathCharProps> \{.*?\n\}', content, re.DOTALL)

new_fn = """pub fn decode_math_char(mut n: u16, reversion: Option<crate::tokens::Tokens>) -> Result<MathCharProps> {
  let class: u16 = n / (16 * 256);
  n %= 16 * 256;
  let mut fam: u16 = n / 256;
  
  let curfam_val: i32 = match state::lookup_register("\\\\fam", Vec::new()) {
    Ok(Some(crate::definition::register::RegisterValue::Number(curfam))) => curfam.0 as i32,
    _ => -1,
  };

  if class == 7 {
    if curfam_val >= 0 && curfam_val <= 15 {
      fam = curfam_val as u16;
    }
  }
  n %= 256;

  let curfont = state::lookup_font().unwrap();

  let mut use_current_font = false;
  let mut maybe_rev = curfam_val >= 0 && fam != 1;
  let mut fontdef_tok: Option<Token> = None;
  
  if class == 7 && curfam_val < 0 {
    let family = curfont.family.as_deref().unwrap_or("");
    if family != "math" {
      use_current_font = true;
      maybe_rev = true;
      fontdef_tok = Some(crate::T_CS!("\\\\font"));
    }
  }

  let mut downsize = 0;
  if fontdef_tok.is_none() {
    let style = curfont.get_mathstyle().map(|s| s.to_string()).unwrap_or_default();
    let style_str = if style == "script" || style == "scriptscript" || style == "text" { style.as_str() } else { "text" };
    if style_str == "text" {
      if let Some(Stored::Token(t)) = state::lookup_value(&crate::s!("textfont_{fam}")) { fontdef_tok = Some(t.clone()); }
    } else if style_str == "script" {
      if let Some(Stored::Token(t)) = state::lookup_value(&crate::s!("scriptfont_{fam}")) { fontdef_tok = Some(t.clone()); }
      else if let Some(Stored::Token(t)) = state::lookup_value(&crate::s!("textfont_{fam}")) { fontdef_tok = Some(t.clone()); downsize = 1; }
    } else if style_str == "scriptscript" {
      if let Some(Stored::Token(t)) = state::lookup_value(&crate::s!("scriptscriptfont_{fam}")) { fontdef_tok = Some(t.clone()); }
      else if let Some(Stored::Token(t)) = state::lookup_value(&crate::s!("scriptfont_{fam}")) { fontdef_tok = Some(t.clone()); downsize = 1; }
      else if let Some(Stored::Token(t)) = state::lookup_value(&crate::s!("textfont_{fam}")) { fontdef_tok = Some(t.clone()); downsize = 2; }
    }
  }
  
  let c = n as u8 as char;
  let class_role = MATH_CLASS_ROLE[class as usize];

  let mut f = (*curfont).clone();
  if let Some(ftok) = &fontdef_tok {
    state::with_font_info(ftok, |fontinfo| {
      if let Some(Stored::Font(ref info)) = fontinfo.unwrap_or(None) {
        f = f.merge((**info).clone());
      }
    });
  }

  if downsize > 0 {
    f = (*curfont).clone();
    f.scripted = Some(true);
  }

  let d = f.relative_to(&curfont);

  let glyph = if use_current_font {
    if let Some(ref data) = curfont.encoding {
      crate::common::font::decode(n as u8, Some(data.to_string()), false)
    } else {
      Some(c)
    }
  } else if let Some(ftok) = &fontdef_tok {
    state::with_font_info(ftok, |fontinfo| {
      let cinfo = if let Some(Stored::Font(ref info)) = fontinfo? {
        if let Some(ref data) = info.encoding {
          crate::common::font::decode(n as u8, Some(data.to_string()), false)
        } else {
          Some(c)
        }
      } else {
        None
      };
      Ok::<Option<char>, crate::common::error::Error>(cinfo)
    })?
  } else {
    Some(c)
  };

  let glyph_char = glyph.unwrap_or(c);
  let charinfo = unicode_math_properties(glyph_char);
  let mut props = charinfo.clone().unwrap_or_default();
  props.glyph = glyph;

  let mut role = charinfo.as_ref().and_then(|info| info.role.clone());
  if role.is_none() && !class_role.is_empty() {
    role = Some(class_role.to_string());
  }
  if role.is_some() && props.role.is_none() {
    props.role = role;
  }

  props.resolve_style_props();

  let mut final_reversion = reversion;
  if let Some(rev) = final_reversion.clone() {
    let mut wrap = maybe_rev && !d.is_empty();
    if state::lookup_value("LaTeX.pool.ltxml_loaded").is_some() {
      wrap = false;
    }
    if wrap {
      if let Some(ftok) = fontdef_tok {
        let mut new_rev = vec![crate::T_BEGIN!(), ftok];
        new_rev.extend(rev.unlist());
        new_rev.push(crate::T_END!());
        final_reversion = Some(crate::tokens::Tokens::new(new_rev));
      }
    }
    props.reversion = final_reversion;
  }

  props.font = Some(f);

  Ok(props)
}"""

if old_fn:
    content = content.replace(old_fn.group(0), new_fn)

# 4. Update decode_math_char_for_stomach
content = re.sub(
    r'pub fn decode_math_char_for_stomach\(\n\s*mathcode: u16,\n\s*meaning: Token,\n\) -> Result<Option<Digested>> \{.*?Ok\(Some\(Digested::from\(Tbox::new\(\n\s*glyph_sym,\n\s*font,\n\s*None,\n\s*crate::Tokens!\(meaning\),\n\s*properties,\n\s*\)\)\)\)\n\}',
    r'''pub fn decode_math_char_for_stomach(
  mathcode: u16,
  meaning: Token,
) -> Result<Option<Digested>> {
  let props = decode_math_char(mathcode, Some(crate::Tokens!(meaning.clone())))?;

  let glyph = match props.glyph {
    Some(g) => g,
    None => return Ok(None),
  };

  let mut properties = SymHashMap::default();
  properties.insert("mode", Stored::String(*arena::MATH_SYM));
  if let Some(ref role) = props.role {
    properties.insert("role", Stored::String(arena::pin(role)));
  }
  if let Some(ref m) = props.meaning {
    properties.insert("meaning", Stored::String(arena::pin(m)));
  }
  if let Some(ref name) = props.name {
    properties.insert("name", Stored::String(arena::pin(name)));
  }
  if let Some(ref stretchy) = props.stretchy {
    properties.insert("stretchy", Stored::String(arena::pin(stretchy)));
  }

  let glyph_sym = arena::pin(glyph.to_string());

  let font = state::lookup_font().map(|f| {
    Rc::new(arena::with(glyph_sym, |s| f.specialize(s)))
  });
  Ok(Some(Digested::from(Tbox::new(
    glyph_sym,
    font,
    None,
    props.reversion.unwrap_or(crate::Tokens!(meaning)),
    properties,
  ))))
}''',
    content,
    flags=re.DOTALL
)

with open('latexml_core/src/common/mathchar.rs', 'w') as f:
    f.write(content)

