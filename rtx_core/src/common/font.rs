use regex::Regex;
use std::borrow::Cow;
use std::collections::HashMap;
/// Note that this has evolved way beynond just "font",
/// but covers text properties (or even display properties) in general
/// including basic font information, color & background color
/// as well as encoding and language information.
///
/// NOTE: This is now in Common that it may evolve to be useful in Post processing...
use std::fmt;

use crate::state::State;

pub type Fontmap = Vec<Option<char>>;

static DEFFAMILY: &'static str = "serif";
static DEFSERIES: &'static str = "medium";
static DEFSHAPE: &'static str = "upright";
static DEFCOLOR: &'static str = "black";
static DEFBACKGROUND: &'static str = "white";
static DEFOPACITY: &'static str = "1";
static DEFENCODING: &'static str = "OT1";
static DEFLANGUAGE: &'static str = "en";
static DEFSIZE: &'static str = "10"; // TODO: master consults state "NOMINAL_FONT_SIZE" before defaulting to 10

// static FORCE_FAMILY : i8 = 0x1;
// static FORCE_SERIES : i8 = 0x2;
// static FORCE_SHAPE : i8  = 0x4;

lazy_static! {
  static ref LATIN_LETTER_RE: Regex = Regex::new(r"^[\p{Latin}&&\pL]$").unwrap();
  static ref GREEK_LETTER_RE: Regex = Regex::new(r"^[\p{Greek}&&\pL]$").unwrap();
  static ref UPPER_LETTER_RE: Regex = Regex::new(r"^[\p{Lu}]$").unwrap();
  static ref DIGIT_LETTER_RE: Regex = Regex::new(r"^[\p{N}]$").unwrap();

  //======================================================================
  // Mappings from various forms of names or component names in TeX
  // Given a font, we'd like to map it to the "logical" names derived from LaTeX,
  // (w/ loss of fine grained control).
  // I'd like to use Karl Berry's font naming scheme
  // (See http://www.tug.org/fontname/html/)
  // but it seems to be a one-way mapping, and moreover, doesn't even fit CM fonts!
  // We'll assume a sloppier version:
  //   family + series + variant + size
  // NOTE: This probably doesn't really belong in here...

  static ref FONT_FAMILY : HashMap<&'static str, HashMap<&'static str, &'static str>> = raw_map!(
    "cmr"  => raw_map!("family" => "serif"),      "cmss"  => raw_map!("family" => "sansserif"),
    "cmtt" => raw_map!("family" => "typewriter"), "cmvtt" => raw_map!("family" => "typewriter"),
    "cmti" => raw_map!("family" => "typewriter", "shape" => "italic"),
    "cmfib" => raw_map!("family" => "serif"),      "cmfr"  => raw_map!("family" => "serif"),
    "cmdh"  => raw_map!("family" => "serif"),      "cm"    => raw_map!("family" => "serif"),
    "ptm"   => raw_map!("family" => "serif"),      "ppl"   => raw_map!("family" => "serif"),
    "pnc"   => raw_map!("family" => "serif"),      "pbk"   => raw_map!("family" => "serif"),
    "phv"   => raw_map!("family" => "sansserif"),  "pag"   => raw_map!("family" => "serif"),
    "pcr"   => raw_map!("family" => "typewriter"), "pzc"   => raw_map!("family" => "script"),
    "put"   => raw_map!("family" => "serif"),      "bch"   => raw_map!("family" => "serif"),
    "psy"   => raw_map!("family" => "symbol"),     "pzd"   => raw_map!("family" => "dingbats"),
    "ccr"   => raw_map!("family" => "serif"),      "ccy"   => raw_map!("family" => "symbol"),
    "cmbr"  => raw_map!("family" => "sansserif"),  "cmtl"  => raw_map!("family" => "typewriter"),
    "cmbrs" => raw_map!("family" => "symbol"),     "ul9"   => raw_map!("family" => "typewriter"),
    "txr"   => raw_map!("family" => "serif"),      "txss"  => raw_map!("family" => "sansserif"),
    "txtt"  => raw_map!("family" => "typewriter"), "txms"  => raw_map!("family" => "symbol"),
    "txsya" => raw_map!("family" => "symbol"),     "txsyb" => raw_map!("family" => "symbol"),
    "pxr"   => raw_map!("family" => "serif"),      "pxms"  => raw_map!("family" => "symbol"),
    "pxsya" => raw_map!("family" => "symbol"),     "pxsyb" => raw_map!("family" => "symbol"),
    "futs"  => raw_map!("family" => "serif"),
    "uaq"   => raw_map!("family" => "serif"),      "ugq"   => raw_map!("family" => "sansserif"),
    "eur"   => raw_map!("family" => "serif"),      "eus"   => raw_map!("family" => "script"),
    "euf"   => raw_map!("family" => "fraktur"),    "euex"  => raw_map!("family" => "symbol"),
    // The following are actually math fonts.
    "ms"    => raw_map!("family" => "symbol"),
    "ccm"   => raw_map!("family" => "serif", "shape" => "italic"),
    "cmm"   => raw_map!("family" => "italic", "encoding" => "OML"),
    "cmex"  => raw_map!("family" => "symbol", "encoding" => "OMX"),       // Not really symbol, but...
    "cmsy"  => raw_map!("family" => "symbol", "encoding" => "OMS"),
    "ccitt" => raw_map!("family" => "typewriter", "shape" => "italic"),
    "cmbrm" => raw_map!("family" => "sansserif", "shape" => "italic"),
    "futm"  => raw_map!("family" => "serif", "shape" => "italic"),
    "futmi" => raw_map!("family" => "serif", "shape" => "italic"),
    "txmi"  => raw_map!("family" => "serif", "shape" => "italic"),
    "pxmi"  => raw_map!("family" => "serif", "shape" => "italic"),
    "bbm"   => raw_map!("family" => "blackboard"),
    "bbold" => raw_map!("family" => "blackboard"),
    "bbmss" => raw_map!("family" => "blackboard"),
    // some ams fonts
    "cmmib" => raw_map!("family" => "italic", "series"   => "bold"),
    "cmbsy" => raw_map!("family" => "symbol", "series"   => "bold"),
    "msa"   => raw_map!("family" => "symbol", "encoding" => "AMSa"),
    "msb"   => raw_map!("family" => "symbol", "encoding" => "AMSb"),
    // Are these really the same?
    "msx" => raw_map!("family" => "symbol", "encoding" => "AMSa"),
    "msy" => raw_map!("family" => "symbol", "encoding" => "AMSb")
  );
  // Maps the "series code" to an abstract font series name
  static ref FONT_SERIES : HashMap<&'static str, HashMap<&'static str, &'static str>> = raw_map!(
    "" => raw_map!("series" => "medium"), "m" => raw_map!("series" => "medium"), "mc" => raw_map!("series" => "medium"),
    "b"  => raw_map!("series" => "bold"),   "bc"  => raw_map!("series" => "bold"),   "bx" => raw_map!("series" => "bold"),
    "sb" => raw_map!("series" => "bold"),   "sbc" => raw_map!("series" => "bold"),   "bm" => raw_map!("series" => "bold")
  );

  // Maps the "shape code" to an abstract font shape name.
  static ref FONT_SHAPE : HashMap<&'static str, HashMap<&'static str, &'static str>> = raw_map!(
    "" => raw_map!("shape" => "upright"), "n" => raw_map!("shape" => "upright"),
     "i" => raw_map!("shape" => "italic"), "it" => raw_map!("shape" => "italic"), "sl" => raw_map!("shape" => "slanted"),
     "sc" => raw_map!("shape" => "smallcaps"), "csc" => raw_map!("shape" => "smallcaps")
  );
}

/// Global auxiliary for font family lookup
pub fn lookup_font_family(code: &str) -> Option<&HashMap<&'static str, &'static str>> {
  FONT_FAMILY.get(code)
}

pub fn lookup_font_feries(code: &str) -> Option<&HashMap<&'static str, &'static str>> {
  FONT_SERIES.get(code)
}

pub fn lookup_font_fhape(code: &str) -> Option<&HashMap<&'static str, &'static str>> {
  FONT_SHAPE.get(code)
}

/// This struct is a little interesting, as we want to pass overrides that partially modify (via a
/// merge) the current font, in each definitional binding. To accommodate that with this struct,
/// every single field needs to be an Option, in order to unambiguously tell the "intend" of
/// override (Some) vs no intent (None).
#[derive(Clone, PartialEq)]
pub struct Font {
  pub family: Option<Cow<'static, str>>,
  pub series: Option<Cow<'static, str>>,
  pub shape: Option<Cow<'static, str>>,
  pub size: Option<Cow<'static, str>>,
  pub color: Option<Cow<'static, str>>,
  pub bg: Option<Cow<'static, str>>,
  pub opacity: Option<Cow<'static, str>>,
  pub encoding: Option<Cow<'static, str>>,
  pub language: Option<Cow<'static, str>>,
  pub mathstyle: Option<Cow<'static, str>>,
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
      family: Some(Cow::Borrowed(DEFFAMILY)),
      series: Some(Cow::Borrowed(DEFSERIES)),
      shape: Some(Cow::Borrowed(DEFSHAPE)),
      size: Some(Cow::Borrowed(DEFSIZE)),
      color: Some(Cow::Borrowed(DEFCOLOR)),
      bg: Some(Cow::Borrowed(DEFBACKGROUND)),
      opacity: Some(Cow::Borrowed(DEFOPACITY)),
      encoding: Some(Cow::Borrowed(DEFENCODING)),
      language: Some(Cow::Borrowed(DEFLANGUAGE)),
      mathstyle: None,
      forceseries: None,
      forcefamily: None,
      forceshape: None,
    }
  }
  pub fn math_default() -> Self {
    Font {
      family: Some(Cow::Borrowed("math")),
      series: Some(Cow::Borrowed(DEFSERIES)),
      shape: Some(Cow::Borrowed("italic")),
      size: Some(Cow::Borrowed(DEFSIZE)),
      color: Some(Cow::Borrowed(DEFCOLOR)),
      bg: Some(Cow::Borrowed(DEFBACKGROUND)),
      opacity: Some(Cow::Borrowed(DEFOPACITY)),
      encoding: None,
      language: Some(Cow::Borrowed(DEFLANGUAGE)),
      mathstyle: Some(Cow::Borrowed("text")),
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
  pub fn get_encoding(&self) -> Option<Cow<str>> { self.encoding.clone() }
  pub fn get_family(&self) -> Option<Cow<str>> { self.family.clone() }

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
      self.family.clone().unwrap_or_else(|| DEFFAMILY.into())
    } else {
      DEFFAMILY.into()
    };
    let defseries = if self.forceseries.unwrap_or(false) {
      self.series.clone().unwrap_or_else(|| DEFSERIES.into())
    } else {
      DEFSERIES.into()
    };
    let defshape = if self.forceshape.unwrap_or(false) {
      self.shape.clone().unwrap_or_else(|| DEFSERIES.into())
    } else {
      DEFSHAPE.into()
    };

    if LATIN_LETTER_RE.is_match(text) {
      // Latin Letter
      if new.shape.is_none() && new.family.is_none() {
        new.shape = Some("italic".into());
      }
    } else if GREEK_LETTER_RE.is_match(text) {
      // Single Greek character?
      if UPPER_LETTER_RE.is_match(text) {
        // Uppercase
        if new.family.is_none() || (new.family == Some(Cow::Borrowed("math"))) {
          new.family = Some(deffamily);
          if new.shape.is_some() && (new.shape != Some(DEFSHAPE.into())) {
            new.shape = Some(defshape); // if ANY shape, must be default
          }
        }
      } else {
        // Lowercase
        if new.family.is_none() || new.family != Some(DEFFAMILY.into()) {
          new.family = Some(deffamily);
        }
        if new.shape.is_none() || new.forceshape == Some(true) {
          // always ?
          new.shape = Some("italic".into());
        }
        if new.series.is_some() && (new.series != Some(DEFSERIES.into())) {
          new.series = Some(defseries);
        }
      }
    } else if DIGIT_LETTER_RE.is_match(text) {
      // Digit
      if new.family.is_none() || (new.family == Some(Cow::Borrowed("math"))) {
        new.family = Some(deffamily);
        new.shape = Some(defshape); // defaults, always.
      }
    } else {
      // Other Symbol
      new.family = Some(deffamily);
      new.shape = Some(defshape); // defaults, always.
      if new.series.is_some() && (new.series != Some(Cow::Borrowed(DEFSERIES))) {
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
    let family = match self.family {
      Some(ref fam) => {
        if *fam == Cow::Borrowed("math") {
          Some(Cow::Borrowed("serif"))
        } else {
          Some(fam.clone())
        }
      },
      None => None,
    };
    let other_family = match other.family {
      Some(ref fam) => {
        if fam == "math" {
          Some(Cow::Borrowed("serif"))
        } else {
          Some(fam.clone())
        }
      },
      None => None,
    };
    let mut diffs = vec![];
    let mut font_properties = Font::default();
    if is_diff(&family, &other_family) {
      diffs.push(family.clone().unwrap());
      font_properties.family = family;
    }
    if is_diff(&self.series, &other.series) {
      let series = self.series.clone().unwrap();
      diffs.push(series);
      font_properties.series = self.series.clone();
    }
    if is_diff(&self.shape, &other.shape) {
      let shape = self.shape.clone().unwrap();
      diffs.push(shape);
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

fn is_diff(x: &Option<Cow<str>>, y: &Option<Cow<str>>) -> bool {
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
        font.get_encoding().unwrap_or_default().into_owned()
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
          if let Some(fmap) = state.lookup_value(&s!("{}_{}_fontmap", encoding, family)) {
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
  } else if let Some(map) = map {
    match map.get(code as usize) {
      None => None,
      Some(c) => *c,
    }
  } else {
    None
  }
}

pub fn decode_string(
  string: &str,
  encoding_opt: Option<&str>,
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
        match font.get_encoding() {
          Some(s) => s,
          None => Cow::Borrowed(""),
        }
      } else {
        Cow::Borrowed("")
      }
    },
    Some(encoding) => Cow::Borrowed(encoding),
  };

  let mut map: Option<&Fontmap> = None;
  if !encoding.is_empty() {
    if let Some(encmap) = state.load_font_map(&encoding) {
      // OK got some map.
      map = Some(encmap);
      if let Some(ref font) = font {
        if let Some(family) = (*font).get_family() {
          if let Some(fmap) = state.lookup_value(&s!("{}_{}_fontmap", encoding, family)) {
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
    } else if let Some(map) = map {
      let code = c as u8;
      if let Some(mapc) = map.get(code as usize) {
        if let Some(mapc_val) = mapc {
          result_string.push(*mapc_val);
        }
      }
    }
  }
  result_string
}
