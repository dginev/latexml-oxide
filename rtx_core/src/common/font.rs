use regex::Regex;
use std::collections::HashMap;
/// Note that this has evolved way beynond just "font",
/// but covers text properties (or even display properties) in general
/// including basic font information, color & background color
/// as well as encoding and language information.
///
/// NOTE: This is now in Common that it may evolve to be useful in Post processing...
use std::fmt;

use state::State;

pub type Fontmap = Vec<Option<char>>;

static DEFFAMILY: &'static str = "serif";
static DEFSERIES: &'static str = "medium";
static DEFSHAPE: &'static str = "upright";
static DEFCOLOR: &'static str = "black";
static DEFBACKGROUND: &'static str = "white";
static DEFOPACITY: &'static str = "1";
static DEFENCODING: &'static str = "OT1";
static DEFLANGUAGE: &'static str = "en";
static DEFSIZE: i8 = 10; // TODO: master consults state "NOMINAL_FONT_SIZE" before defaulting to 10

// static FORCE_FAMILY : i8 = 0x1;
// static FORCE_SERIES : i8 = 0x2;
// static FORCE_SHAPE : i8  = 0x4;

lazy_static! {
  static ref LATIN_LETTER_RE: Regex = Regex::new(r"^[\p{Latin}&&\pL]$").unwrap();
  static ref GREEK_LETTER_RE: Regex = Regex::new(r"^[\p{Greek}&&\pL]$").unwrap();
  static ref UPPER_LETTER_RE: Regex = Regex::new(r"^[\p{Lu}]$").unwrap();
  static ref DIGIT_LETTER_RE: Regex = Regex::new(r"^[\p{N}]$").unwrap();
}

/// This struct is a little interesting, as we want to pass overrides that partially modify (via a
/// merge) the current font, in each definitional binding. To accommodate that with this struct,
/// every single field needs to be an Option, in order to unambiguously tell the "intend" of
/// override (Some) vs no intent (None).
#[derive(Clone, PartialEq)]
pub struct Font {
  pub family: Option<String>,
  pub series: Option<String>,
  pub shape: Option<String>,
  pub size: Option<String>,
  pub color: Option<String>,
  pub bg: Option<String>,
  pub opacity: Option<String>,
  pub encoding: Option<String>,
  pub language: Option<String>,
  pub mathstyle: Option<String>,
  pub forceseries: Option<bool>,
  pub forcefamily: Option<bool>,
  pub forceshape: Option<bool>,
}

// Note: forcefamily, forceseries, forceshape (& forcebold for compatibility)
// are only useful for fonts in math; See the specialize method below.
impl Default for Font {
  fn default() -> Self {
    Font {
      family: None,
      series: None,
      shape: None,
      size: None,
      color: None,
      bg: None,
      opacity: None,
      encoding: None,
      language: None,
      mathstyle: None,
      forceseries: None,
      forcefamily: None,
      forceshape: None,
    }
  }
}

impl fmt::Debug for Font {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.to_string()) }
}

impl Font {
  pub fn text_default() -> Self {
    Font {
      family: Some(DEFFAMILY.to_string()),
      series: Some(DEFSERIES.to_string()),
      shape: Some(DEFSHAPE.to_string()),
      size: Some(DEFSIZE.to_string()),
      color: Some(DEFCOLOR.to_string()),
      bg: Some(DEFBACKGROUND.to_string()),
      opacity: Some(DEFOPACITY.to_string()),
      encoding: Some(DEFENCODING.to_string()),
      language: Some(DEFLANGUAGE.to_string()),
      mathstyle: None,
      forceseries: None,
      forcefamily: None,
      forceshape: None,
    }
  }
  pub fn math_default() -> Self {
    Font {
      family: Some(s!("math")),
      series: Some(DEFSERIES.to_string()),
      shape: Some(s!("italic")),
      size: Some(DEFSIZE.to_string()),
      color: Some(DEFCOLOR.to_string()),
      bg: Some(DEFBACKGROUND.to_string()),
      opacity: Some(DEFOPACITY.to_string()),
      encoding: None,
      language: Some(DEFLANGUAGE.to_string()),
      mathstyle: Some(s!("text")),
      forceseries: None,
      forcefamily: None,
      forceshape: None,
    }
  }
  pub fn to_string(&self) -> String {
    let mut parts = Vec::new();
    if let Some(ref family) = self.family {
      parts.push(s!("family: {:?}", family))
    }
    if let Some(ref series) = self.series {
      parts.push(s!("series: {:?}", series))
    }
    if let Some(ref shape) = self.shape {
      parts.push(s!("shape: {:?}", shape))
    }
    if let Some(ref size) = self.size {
      parts.push(s!("size: {:?}", size))
    }
    if let Some(ref color) = self.color {
      parts.push(s!("color: {:?}", color))
    }
    if let Some(ref bg) = self.bg {
      parts.push(s!("bg: {:?}", bg))
    }
    if let Some(ref opacity) = self.opacity {
      parts.push(s!("opacity: {:?}", opacity))
    }
    if let Some(ref encoding) = self.encoding {
      parts.push(s!("encoding: {:?}", encoding))
    }
    if let Some(ref language) = self.language {
      parts.push(s!("language: {:?}", language))
    }
    if let Some(ref mathstyle) = self.mathstyle {
      parts.push(s!("mathstyle: {:?}", mathstyle))
    }
    if let Some(ref forceseries) = self.forceseries {
      parts.push(s!("forceseries: {:?}", forceseries))
    }
    if let Some(ref forcefamily) = self.forcefamily {
      parts.push(s!("forcefamily: {:?}", forcefamily))
    }
    if let Some(ref forceshape) = self.forceshape {
      parts.push(s!("forceshape: {:?}", forceshape))
    }
    s!("Font[{}]", parts.join(", "))
  }

  /// Getter for encoding field
  pub fn get_encoding(&self) -> Option<String> { self.encoding.clone() }
  pub fn get_family(&self) -> Option<String> { self.family.clone() }

  // NOTE: In math, NORMALLY, setting any one of
  //    family, series or shape
  // will, usually, automatically reset the others to thier defaults!
  // You must arrange this in the calls....
  pub fn merge(&self, other: Font) -> Self {
    let mut newfont = self.clone();
    if let Some(value) = other.family {
      newfont.family = Some(value);
    }
    if let Some(value) = other.series {
      newfont.series = Some(value);
    }
    if let Some(value) = other.shape {
      newfont.shape = Some(value);
    }
    if let Some(value) = other.size {
      newfont.size = Some(value);
    }
    if let Some(value) = other.color {
      newfont.color = Some(value);
    }
    if let Some(value) = other.bg {
      newfont.bg = Some(value);
    }
    if let Some(value) = other.opacity {
      newfont.opacity = Some(value);
    }
    if let Some(value) = other.encoding {
      newfont.encoding = Some(value);
    }
    if let Some(value) = other.language {
      newfont.language = Some(value);
    }
    if let Some(value) = other.mathstyle {
      newfont.mathstyle = Some(value);
    }
    if let Some(value) = other.forceseries {
      newfont.forceseries = Some(value);
    }
    if let Some(value) = other.forcefamily {
      newfont.forcefamily = Some(value);
    }
    if let Some(value) = other.forceshape {
      newfont.forceshape = Some(value);
    }
    newfont
  }

  /// Instanciate the font for a particular class of symbols.
  /// NOTE: This works in `normal' latex, but probably needs some tunability.
  /// Depending on the fonts being used, the allowable combinations may be different.
  /// Getting the font right is important, since the author probably
  /// thinks of the identity of the symbols according to what they SEE in the printed
  /// document.  Even though the markup might seem to indicate something else...
  ///
  /// Use Unicode properties to determine font merging.
  pub fn specialize(&self, text: &str) -> Self {
    let mut new = self.clone();
    if text.is_empty() {
      return new;
    } // ?
    let deffamily = if self.forcefamily.unwrap_or(false) {
      self.family.clone().unwrap_or(DEFFAMILY.to_string())
    } else {
      DEFFAMILY.to_string()
    };
    let defseries = if self.forceseries.unwrap_or(false) {
      self.series.clone().unwrap_or(DEFSERIES.to_string())
    } else {
      DEFSERIES.to_string()
    };
    let defshape = if self.forceshape.unwrap_or(false) {
      self.shape.clone().unwrap_or(DEFSERIES.to_string())
    } else {
      DEFSHAPE.to_string()
    };

    if LATIN_LETTER_RE.is_match(text) {
      // Latin Letter
      if new.shape.is_none() && new.family.is_none() {
        new.shape = Some(s!("italic"));
      }
    } else if GREEK_LETTER_RE.is_match(text) {
      // Single Greek character?
      if UPPER_LETTER_RE.is_match(text) {
        // Uppercase
        if new.family.is_none() || (new.family == Some(s!("math"))) {
          new.family = Some(deffamily);
          if new.shape.is_some() && (new.shape != Some(DEFSHAPE.to_string())) {
            new.shape = Some(defshape); // if ANY shape, must be default
          }
        }
      } else {
        // Lowercase
        if new.family.is_none() || new.family != Some(DEFFAMILY.to_string()) {
          new.family = Some(deffamily);
        }
        if new.shape.is_none() || new.forceshape == Some(true) {
          // always ?
          new.shape = Some(s!("italic"));
        }
        if new.series.is_some() && (new.series != Some(DEFSERIES.to_string())) {
          new.series = Some(defseries);
        }
      }
    } else if DIGIT_LETTER_RE.is_match(text) {
      // Digit
      if new.family.is_none() || (new.family == Some(s!("math"))) {
        new.family = Some(deffamily);
        new.shape = Some(defshape); // defaults, always.
      }
    } else {
      // Other Symbol
      new.family = Some(deffamily);
      new.shape = Some(defshape); // defaults, always.
      if new.series.is_some() && (new.series != Some(DEFSERIES.to_string())) {
        new.series = Some(defseries);
      } // defaults, always.
    }
    new
  }

  pub fn to_attribute(&self) -> String {
    let mut serialized = String::new();
    if let Some(ref value) = self.family {
      serialized = serialized + " " + value;
    }
    if let Some(ref value) = self.series {
      if value != "medium" {
        // TODO: this is a Hack for alltt.tex, ensure this is generalized
        serialized = serialized + " " + value;
      }
    }
    serialized.trim().to_string()
  }

  pub fn distance(&self, other_opt: Option<&Font>) -> i8 {
    if let Some(other) = other_opt {
      let mut distance = 0;
      if self.family != other.family {
        distance += 1;
      }
      if self.series != other.series {
        distance += 1;
      }
      if self.shape != other.shape {
        distance += 1;
      }
      if self.size != other.size {
        distance += 1;
      }
      if self.color != other.color {
        distance += 1;
      }
      if self.bg != other.bg {
        distance += 1;
      }
      if self.opacity != other.opacity {
        distance += 1;
      }
      if self.encoding != other.encoding {
        distance += 1;
      }
      if self.language != other.language {
        distance += 1;
      }
      if self.mathstyle != other.mathstyle {
        distance += 1;
      }
      if self.forceseries != other.forceseries {
        distance += 1;
      }
      if self.forcefamily != other.forcefamily {
        distance += 1;
      }
      if self.forceshape != other.forceshape {
        distance += 1;
      }
      distance
    } else {
      0
    }
  }

  /// This method compares 2 fonts, returning the differences between them.
  /// Noting that the font-related attributes in the schema distill the
  /// font properties into fewer attributes (font,fontsize,color,background,opacity),
  /// the return value encodes both the attribute changes that would be needed to effect
  /// the font change, along with the font properties that differed
  /// Namely, the result is a hash keyed on the attribute name and whose value is a FontDiff
  ///    value      => "new_attribute_value"
  ///    properties => { %fontproperties }
  /// or (String, Font)
  pub fn relative_to(&self, other: &Font) -> HashMap<String, (String, Font)> {
    let family = match &self.family {
      &Some(ref fam) => if fam == "math" {
        Some(s!("serif"))
      } else {
        Some(fam.to_string())
      },
      &None => None,
    };
    let other_family = match &other.family {
      &Some(ref fam) => if fam == "math" {
        Some(s!("serif"))
      } else {
        Some(fam.to_string())
      },
      &None => None,
    };
    let mut diffs = vec![];
    let mut font_properties = Font::default();
    if is_diff(&family, &other_family) {
      diffs.push(family.clone().unwrap());
      font_properties.family = family;
    }
    if is_diff(&self.series, &other.series) {
      let series = self.series.clone().unwrap();
      diffs.push(series.clone());
      font_properties.series = self.series.clone();
    }
    if is_diff(&self.shape, &other.shape) {
      let shape = self.shape.clone().unwrap();
      diffs.push(shape.clone());
      font_properties.shape = self.shape.clone();
    }

    let mut result = HashMap::new();

    if !diffs.is_empty() {
      let font_value = diffs.join(" ");
      result.insert(s!("font"), (font_value, font_properties));
    }

    // (is_diff($siz, $osiz)
    // ////      ? (fontsize => { value => $siz, properties => { size => $siz } })
    //   ? (fontsize => { value => relativeFontSize($siz, $osiz), properties => { size => $siz } })
    //   : ()),
    // (is_diff($col, $ocol)
    //   ? (color => { value => $col, properties => { color => $col } })
    //   : ()),
    // (is_diff($bkg, $obkg)
    //   ? (backgroundcolor => { value => $bkg, properties => { background => $bkg } })
    //   : ()),
    // (is_diff($opa, $oopa)
    //   ? (opacity => { value => $opa, properties => { opacity => $opa } })
    //   : ()),
    // (is_diff($lang, $olang)
    //   ? ('xml:lang' => { value => $lang, properties => { language => $lang } })
    //   : ()),

    //// Contemplate this: We do NOT want mathstyle showing up (automatically) in the attributes
    //// So, we presumably want to ignore differences in mathstyle
    //// They shouldn't (by themselves) affect the display?
    ////    (is_diff($mstyle, $omstyle)
    ////      ? ('mathstyle' => { value => $mstyle, properties => { mathstyle => $mstyle } })
    ////      : ()),
    result
  }
}

fn is_diff(x: &Option<String>, y: &Option<String>) -> bool {
  x.is_some() && (y.is_none() || (x != y))
}

/// Decode a codepoint using the fontmap for a given font and/or fontencoding.
/// If `encoding` not provided, then lookup according to the current font's
/// encoding; the font family may also be used to choose the fontmap (think tt fonts!).
/// When `implicit` is false, we are "explicitly" asking for a decoding, such as
/// with \char, \mathchar, \symbol, DeclareTextSymbol and such cases.
/// In such cases, only codepoints specifically within the map are covered; the rest are undef.
/// If `implicit` is true, we'll decode token content that has made it to the stomach:
/// We're going to assume that SOME sort of handling of input encoding is taking place,
/// so that if anything above 128 comes in, it must already be Unicode!.
/// The lower half plane still needs to go through decoding, though, to deal
/// with TeX's rearrangement of ASCII...
pub fn decode(
  code: u8,
  encoding_opt: Option<String>,
  implicit: bool,
  state: &mut State,
) -> Option<char>
{
  let mut font = None;
  let encoding = match encoding_opt {
    None => {
      font = state.lookup_font();
      if let Some(ref font) = font {
        font.get_encoding().unwrap_or(String::new())
      } else {
        String::new()
      }
    },
    Some(encoding) => encoding,
  };

  let mut map: Option<&Fontmap> = None;
  if !encoding.is_empty() {
    if let Some(encmap) = state.load_font_map(&encoding) {
      // OK got some map.
      map = Some(encmap);
      if let Some(font) = font {
        if let Some(family) = (*font).get_family() {
          if let Some(fmap) = state.lookup_value(&format!("{}_{}_fontmap", encoding, family)) {
            map = fmap.into(); // Use the family specific map, if any.
          }
        }
      }
    }
  }

  if implicit {
    if let Some(map) = map {
      if code < 128 {
        match map.get(code as usize) {
          None => None,
          Some(c) => *c,
        }
      } else {
        Some(code.into())
      }
    } else {
      Some(code.into())
    }
  } else {
    if let Some(map) = map {
      match map.get(code as usize) {
        None => None,
        Some(c) => *c,
      }
    } else {
      None
    }
  }
}

pub fn decode_string(
  string: String,
  encoding_opt: Option<String>,
  implicit: bool,
  state: &mut State,
) -> String
{
  if string.is_empty() {
    return String::new();
  }
  let mut font = None;
  let encoding = match encoding_opt {
    None => {
      font = state.lookup_font();
      if let Some(ref font) = font {
        font.get_encoding().unwrap_or(String::new())
      } else {
        String::new()
      }
    },
    Some(encoding) => encoding,
  };

  let mut map: Option<&Fontmap> = None;
  if !encoding.is_empty() {
    if let Some(encmap) = state.load_font_map(&encoding) {
      // OK got some map.
      map = Some(encmap);
      if let Some(font) = font {
        if let Some(family) = (*font).get_family() {
          if let Some(fmap) = state.lookup_value(&format!("{}_{}_fontmap", encoding, family)) {
            map = fmap.into(); // Use the family specific map, if any.
          }
        }
      }
    }
  }

  let mut result_string: String = String::new();
  for c in string.chars() {
    if implicit {
      if let Some(map) = map {
        let code = c as u8;
        if code < 128 {
          if let Some(mapc) = map.get(code as usize) {
            if let Some(mapc_val) = mapc {
              result_string.push(*mapc_val);
            }
          }
        } else {
          result_string.push(c);
        }
      } else {
        result_string.push(c)
      }
    } else {
      if let Some(map) = map {
        let code = c as u8;
        if let Some(mapc) = map.get(code as usize) {
          if let Some(mapc_val) = mapc {
            result_string.push(*mapc_val);
          }
        }
      }
    }
  }
  result_string
}
