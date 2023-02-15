use crate::common::dimension::Dimension;
use crate::common::numeric_ops::{NumericOps, UNITY};
use crate::common::store::Stored;
use crate::state::State;
use crate::{BoxOps, Digested, DigestedData, Result};
use lazy_static::lazy_static;
/// Note that this has evolved way beynond just "font",
/// but covers text properties (or even display properties) in general
/// including basic font information, color & background color
/// as well as encoding and language information.
///
/// NOTE: This is now in Common that it may evolve to be useful in Post processing...
use regex::Regex;
use std::borrow::Cow;
use std::cmp::max;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};

mod standard_metrics;
use standard_metrics::{MetricData, STDMETRICS};

pub type Fontmap = Vec<Option<char>>;

static DEFFAMILY: &str = "serif";
static DEFSERIES: &str = "medium";
static DEFSHAPE: &str = "upright";
static DEFCOLOR: &str = "black";
static DEFBACKGROUND: &str = "white";
static DEFOPACITY: &str = "1";
static DEFENCODING: &str = "OT1";
static DEFLANGUAGE: &str = "en";
static DEFSIZE: f32 = 10.0; // TODO: master consults state "NOMINAL_FONT_SIZE" before defaulting to 10

pub const TEXT_FONTS: [&str; 6] = ["cmr", "cmm", "cmsy", "cmex", "amsa", "amsb"];
pub const MATH_FONTS: [&str; 6] = ["cmm", "cmsy", "cmex", "amsa", "amsb", "cmr"];

// static FORCE_FAMILY : i8 = 0x1;
// static FORCE_SERIES : i8 = 0x2;
// static FORCE_SHAPE : i8  = 0x4;

lazy_static! {
  pub static ref FONT_TEXT_DEFAULT : Font = Font::text_default();
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

  static ref FONT_FAMILY : HashMap<&'static str, Font> = raw_map!(
    "cmr"  => fontmap!(family => "serif"),      "cmss"  => fontmap!(family => "sansserif"),
    "cmtt" => fontmap!(family => "typewriter"), "cmvtt" => fontmap!(family => "typewriter"),
    "cmti" => fontmap!(family => "typewriter", shape => "italic"),
    "cmfib" => fontmap!(family => "serif"),      "cmfr"  => fontmap!(family => "serif"),
    "cmdh"  => fontmap!(family => "serif"),      "cm"    => fontmap!(family => "serif"),
    "ptm"   => fontmap!(family => "serif"),      "ppl"   => fontmap!(family => "serif"),
    "pnc"   => fontmap!(family => "serif"),      "pbk"   => fontmap!(family => "serif"),
    "phv"   => fontmap!(family => "sansserif"),  "pag"   => fontmap!(family => "serif"),
    "pcr"   => fontmap!(family => "typewriter"), "pzc"   => fontmap!(family => "script"),
    "put"   => fontmap!(family => "serif"),      "bch"   => fontmap!(family => "serif"),
    "psy"   => fontmap!(family => "symbol"),     "pzd"   => fontmap!(family => "dingbats"),
    "ccr"   => fontmap!(family => "serif"),      "ccy"   => fontmap!(family => "symbol"),
    "cmbr"  => fontmap!(family => "sansserif"),  "cmtl"  => fontmap!(family => "typewriter"),
    "cmbrs" => fontmap!(family => "symbol"),     "ul9"   => fontmap!(family => "typewriter"),
    "txr"   => fontmap!(family => "serif"),      "txss"  => fontmap!(family => "sansserif"),
    "txtt"  => fontmap!(family => "typewriter"), "txms"  => fontmap!(family => "symbol"),
    "txsya" => fontmap!(family => "symbol"),     "txsyb" => fontmap!(family => "symbol"),
    "pxr"   => fontmap!(family => "serif"),      "pxms"  => fontmap!(family => "symbol"),
    "pxsya" => fontmap!(family => "symbol"),     "pxsyb" => fontmap!(family => "symbol"),
    "futs"  => fontmap!(family => "serif"),
    "uaq"   => fontmap!(family => "serif"),      "ugq"   => fontmap!(family => "sansserif"),
    "eur"   => fontmap!(family => "serif"),      "eus"   => fontmap!(family => "script"),
    "euf"   => fontmap!(family => "fraktur"),    "euex"  => fontmap!(family => "symbol"),
    // The following are actually math fonts.
    "ms"    => fontmap!(family => "symbol"),
    "ccm"   => fontmap!(family => "serif", shape => "italic"),
    "cmm"   => fontmap!(family => "italic", encoding => "OML"),
    "cmex"  => fontmap!(family => "symbol", encoding => "OMX"),       // Not really symbol, but...
    "cmsy"  => fontmap!(family => "symbol", encoding => "OMS"),
    "ccitt" => fontmap!(family => "typewriter", shape => "italic"),
    "cmbrm" => fontmap!(family => "sansserif", shape => "italic"),
    "futm"  => fontmap!(family => "serif", shape => "italic"),
    "futmi" => fontmap!(family => "serif", shape => "italic"),
    "txmi"  => fontmap!(family => "serif", shape => "italic"),
    "pxmi"  => fontmap!(family => "serif", shape => "italic"),
    "bbm"   => fontmap!(family => "blackboard"),
    "bbold" => fontmap!(family => "blackboard"),
    "bbmss" => fontmap!(family => "blackboard"),
    // some ams fonts
    "cmmib" => fontmap!(family => "italic", series   => "bold"),
    "cmbsy" => fontmap!(family => "symbol", series   => "bold"),
    "msa"   => fontmap!(family => "symbol", encoding => "AMSa"),
    "msb"   => fontmap!(family => "symbol", encoding => "AMSb"),
    // Are these really the same?
    "msx" => fontmap!(family => "symbol", encoding => "AMSa"),
    "msy" => fontmap!(family => "symbol", encoding => "AMSb")
  );
  /// Maps the "series code" to an abstract font series name
  static ref FONT_SERIES : HashMap<&'static str, Font> = raw_map!(
    "" => fontmap!(series => "medium"), "m" => fontmap!(series => "medium"), "mc" => fontmap!(series => "medium"),
    "b"  => fontmap!(series => "bold"),   "bc"  => fontmap!(series => "bold"),   "bx" => fontmap!(series => "bold"),
    "sb" => fontmap!(series => "bold"),   "sbc" => fontmap!(series => "bold"),   "bm" => fontmap!(series => "bold")
  );

  /// Maps the "shape code" to an abstract font shape name.
  static ref FONT_SHAPE : HashMap<&'static str, Font> = raw_map!(
    "" => fontmap!(shape => "upright"), "n" => fontmap!(shape => "upright"),
     "i" => fontmap!(shape => "italic"), "it" => fontmap!(shape => "italic"), "sl" => fontmap!(shape => "slanted"),
     "sc" => fontmap!(shape => "smallcaps"), "csc" => fontmap!(shape => "smallcaps")
  );

  /// Symbolic font sizes, relative to the NOMINAL_FONT_SIZE (often 10)
  /// extended logical font sizes, based on nominal document size of 10pts
  /// Possibly should simply use absolute font point sizes, as declared in class...
  static ref FONT_SIZE : HashMap<&'static str, f32> = raw_map!(
  "tiny"   => 0.5,   "SMALL" => 0.7, "Small" => 0.8,  "small" => 0.9,
  "normal" => 1.0,   "large" => 1.2, "Large" => 1.44, "LARGE" => 1.728,
  "huge"   => 2.074, "Huge"  => 2.488,
  "big"    => 1.2,   "Big"   => 1.6, "bigg" => 2.1, "Bigg" => 2.6);

  static ref SCRIPT_STYLE_MAP  : HashMap<&'static str, &'static str> = raw_map!(
    "display" => "script", "text" => "script",
    "script" => "scriptscript", "scriptscript" => "scriptscript");

  static ref FRAC_STYLE_MAP : HashMap<&'static str, &'static str> = raw_map!(
    "display" => "text", "text" => "script",
    "script" => "scriptscript", "scriptscript" => "scriptscript");

  static ref STYLE_SIZE  : HashMap<&'static str, usize> = raw_map!(
    "display" => 10, "text" => 10, "script" => 7, "scriptscript" => 5);

  static ref MATH_STYLE_SIZE : HashMap<&'static str, f32> = raw_map!(
    "display" => 1.0, "text" => 1.0, "script" => 0.7, "scriptscript" => 0.5);

  /// A special form of merge when copying/moving nodes to a new context,
  /// particularly math which become scripts or such.
  static ref MATH_STYLE_STEP : HashMap<&'static str, HashMap<&'static str, i32>> = raw_map!(
    "display" => raw_map!(
      "display" => 0, "text" => 1, "script" => 2, "scriptscript" => 3),
    "text"=> raw_map!("display" => -1, "text" => 0, "script" => 1, "scriptscript" => 2),
    "script"=> raw_map!("display" => -2, "text" => -1, "script" => 0, "scriptscript" => 1),
    "scriptscript" => raw_map!("display" => -3, "text" => -2, "script" => -1, "scriptscript" => 0));
  static ref STEP_MATH_STYLE : HashMap<&'static str, HashMap<i32, &'static str>> = raw_map!(
  "display" => raw_map!(-3 => "display", -2 => "display", -1 => "display",
    0 => "display", 1 => "text", 2 => "script", 3 => "scriptscript"),
  "text" => raw_map!(-3 => "display", -2 => "display", -1 => "display",
    0 => "text", 1 => "script", 2 => "scriptscript", 3 => "scriptscript"),
  "script" => raw_map!(-3 => "display", -2 => "display", -1 => "text",
    0 => "script", 1 => "scriptscript", 2 => "scriptscript", 3 => "scriptscript"),
  "scriptscript" => raw_map!(-3 => "display", -2 => "text", -1 => "script",
    0 => "scriptscript", 1 => "scriptscript", 2 => "scriptscript", 3 => "scriptscript"));
}

/// Global auxiliary for font family lookup
pub fn lookup_font_family(code: &str) -> Option<&Font> { FONT_FAMILY.get(code) }

/// Global auxiliary for font series lookup
pub fn lookup_font_series(code: &str) -> Option<&Font> { FONT_SERIES.get(code) }

/// Global auxiliary for font shape lookup
pub fn lookup_font_shape(code: &str) -> Option<&Font> { FONT_SHAPE.get(code) }

/// ???
pub fn decode_fontname(name: &str, at: Option<f32>, scaled: Option<f32>) -> Option<Font> {
  // TODO!

  // if ($name =~ /^$FONTREGEXP$/o) {
  //   my %props;
  //   my ($fam, $ser, $shp, $size) = ($1, $2, $3, $4);
  //   if (my $ffam = lookupFontFamily($fam)) { map { $props{$_} = $$ffam{$_} } keys %$ffam; }
  //   if (my $fser = lookupFontSeries($ser)) { map { $props{$_} = $$fser{$_} } keys %$fser; }
  //   if (my $fsh  = lookupFontShape($shp))  { map { $props{$_} = $$fsh{$_} } keys %$fsh; }
  //   $size = 1 unless $size;    # Yes, also if 0, "" (from regexp)
  //   $size = $at if defined $at;
  //   $size *= $scaled if defined $scaled;
  //   $props{size} = $size;
  //   # Experimental Hack !?!?!?
  //   $props{encoding} = 'OT1' unless defined $props{encoding};
  //   $props{at}       = $at . "pt" if defined $at;
  //   return %props; }
  // else {
  //   return; }
  None
}

/// This struct is a little interesting, as we want to pass overrides that partially modify (via a
/// merge) the current font, in each definitional binding. To accommodate that with this struct,
/// every single field needs to be an Option, in order to unambiguously tell the "intend" of
/// override (Some) vs no intent (None).
#[derive(Clone, PartialEq, Default)]
pub struct Font {
  pub family: Option<Cow<'static, str>>,
  pub series: Option<Cow<'static, str>>,
  pub shape: Option<Cow<'static, str>>,
  pub size: Option<f32>,
  pub color: Option<Cow<'static, str>>,
  pub bg: Option<Cow<'static, str>>,
  pub opacity: Option<Cow<'static, str>>,
  pub encoding: Option<Cow<'static, str>>,
  pub language: Option<Cow<'static, str>>,
  pub mathstyle: Option<Cow<'static, str>>,
  pub mathstylestep: Option<Cow<'static, str>>,
  pub name: Option<Cow<'static, str>>,
  pub emph: Option<bool>,
  pub scripted: Option<bool>,
  // Note: forcefamily, forceseries, forceshape (& forcebold for compatibility)
  // are only useful for fonts in math; See the specialize method below.
  pub forceseries: Option<bool>,
  pub forcefamily: Option<bool>,
  pub forceshape: Option<bool>,
  pub scale: Option<f32>,
}

impl Hash for Font {
  // We need to implement hash since we have to tell Rust how to hash `f32` values
  // for now I have decided to go for a precision of 4 digits after the decimal point,
  // so multiplying by 1000
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.family.hash(state);
    self.series.hash(state);
    self.shape.hash(state);
    self.size.map(|size| (size * 1000.0) as i32).hash(state);
    self.color.hash(state);
    self.bg.hash(state);
    self.opacity.hash(state);
    self.encoding.hash(state);
    self.language.hash(state);
    self.mathstyle.hash(state);
    self.mathstylestep.hash(state);
    self.name.hash(state);
    self.emph.hash(state);
    self.scripted.hash(state);
    self.forceseries.hash(state);
    self.forcefamily.hash(state);
    self.forceshape.hash(state);
    self.scale.map(|scale| (scale * 1000.0) as i32).hash(state);
  }
}
impl Eq for Font {}
// display is used often for attributes in binding replacements,
// as in font="#font"
impl fmt::Display for Font {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.family.as_ref().unwrap_or(&Cow::Borrowed(""))) }
}
// elide Font debugging until we get to implementing them faithfully
// impl fmt::Debug for Font {
//   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//     write!(f, "[font]")
//   }
// }
impl fmt::Debug for Font {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    if let Some(ref scale) = self.scale {
      parts.push(s!("scale: {:?}", scale))
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
    if let Some(ref mathstylestep) = self.mathstyle {
      parts.push(s!("mathstylestep: {:?}", mathstylestep))
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
    if let Some(ref scripted) = self.scripted {
      parts.push(s!("scripted: {:?}", scripted))
    }
    write!(f, "Font[{}]", parts.join(", "))
  }
}

impl Font {
  pub fn text_default() -> Self {
    Font {
      family: Some(Cow::Borrowed(DEFFAMILY)),
      series: Some(Cow::Borrowed(DEFSERIES)),
      shape: Some(Cow::Borrowed(DEFSHAPE)),
      size: Some(DEFSIZE),
      color: Some(Cow::Borrowed(DEFCOLOR)),
      bg: Some(Cow::Borrowed(DEFBACKGROUND)),
      opacity: Some(Cow::Borrowed(DEFOPACITY)),
      encoding: Some(Cow::Borrowed(DEFENCODING)),
      language: Some(Cow::Borrowed(DEFLANGUAGE)),
      mathstyle: None,
      mathstylestep: None,
      emph: None,
      name: None,
      scripted: None,
      forceseries: None,
      forcefamily: None,
      forceshape: None,
      scale: None,
    }
  }
  pub fn math_default() -> Self {
    Font {
      family: Some(Cow::Borrowed("math")),
      series: Some(Cow::Borrowed(DEFSERIES)),
      shape: Some(Cow::Borrowed("italic")),
      size: Some(DEFSIZE),
      color: Some(Cow::Borrowed(DEFCOLOR)),
      bg: Some(Cow::Borrowed(DEFBACKGROUND)),
      opacity: Some(Cow::Borrowed(DEFOPACITY)),
      encoding: None,
      language: Some(Cow::Borrowed(DEFLANGUAGE)),
      mathstyle: Some(Cow::Borrowed("text")),
      mathstylestep: None,
      emph: None,
      name: None,
      scripted: None,
      forceseries: None,
      forcefamily: None,
      forceshape: None,
      scale: None,
    }
  }

  pub fn to_hashable(&self) -> u64 {
    let mut hasher = DefaultHasher::new();
    Hash::hash(self, &mut hasher);
    hasher.finish()
  }

  pub fn math_bearing(&self, thisbox: &Digested, prevbox: &Digested) -> f32 {
    // my $r0      = $prevbox->getProperty('role') || 'ID';
    // my $r1      = $box->getProperty('role')     || 'ID';
    // my $t0      = $mathatomtype{$r0}            || 0;
    // my $t1      = $mathatomtype{$r1}            || 0;
    // my $bearing = $$mathbearings[$t0][$t1];
    // my $style   = $self->getMathstyle || 'text';
    // if (!$bearing || (($bearing < 0) && ($style ne 'display') && ($style ne 'text'))) {
    //   return 0; }
    // return $STATE->lookupDefinition($$mathbearingreg[abs($bearing)])->valueOf->spValue; }
    0.0
  }

  pub fn is_sticky(&self) -> bool {
    if let Some(ref family) = self.family {
      family == "serif" || family == "sansserif" || family == "typewriter"
    } else {
      false
    }
  }

  // Accessors
  pub fn get_family(&self) -> Option<&Cow<str>> { self.family.as_ref() }
  pub fn get_series(&self) -> Option<&Cow<str>> { self.series.as_ref() }
  pub fn get_shape(&self) -> Option<&Cow<str>> { self.shape.as_ref() }
  pub fn get_size(&self) -> Option<f32> { self.size }
  pub fn get_color(&self) -> Option<&Cow<str>> { self.color.as_ref() }
  pub fn get_background(&self) -> Option<&Cow<str>> { self.bg.as_ref() }
  pub fn get_opacity(&self) -> Option<&Cow<str>> { self.opacity.as_ref() }
  pub fn get_encoding(&self) -> Option<&Cow<str>> { self.encoding.as_ref() }
  pub fn get_language(&self) -> Option<&Cow<str>> { self.language.as_ref() }
  pub fn get_mathstyle(&self) -> Option<&Cow<str>> { self.mathstyle.as_ref() }

  // NOTE: In math, NORMALLY, setting any one of
  //    family, series or shape
  // will, usually, automatically reset the others to thier defaults!
  // You must arrange this in the calls....
  pub fn merge(&self, other: Font) -> Self {
    let mut newfont = self.clone();
    // first set direct overrides.
    if let Some(value) = other.family {
      newfont.family = Some(value);
    }
    if let Some(value) = other.series {
      newfont.series = Some(value);
    }
    if let Some(value) = other.shape {
      newfont.shape = Some(value);
    }
    if other.emph == Some(true) {
      newfont.shape = if newfont.shape.unwrap_or(Cow::Borrowed("")) == "italic" {
        Some(Cow::Borrowed("upright"))
      } else {
        Some(Cow::Borrowed("italic"))
      };
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
    let mut has_mathstyle = false;
    if let Some(value) = other.mathstyle {
      has_mathstyle = true;
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

    // Now perform any dynamic adjustment directives
    if let Some(scale) = other.scale {
      if let Some(size) = newfont.size {
        newfont.size = Some(scale * size);
      }
    }
    // Set the mathstyle, and also the size from the mathstyle
    // But we may need to scale that size against the existing or requested size.
    let style_scale = if let Some(size) = newfont.size {
      let key = self.mathstyle.as_ref().unwrap_or(&Cow::Borrowed("display"));
      // the explicit &str typecast is currently needed for rust to
      // figure out how to use the Cow<str> in the HashMap lookup.
      let str_key: &str = key;
      size / *STYLE_SIZE.get(str_key).unwrap() as f32
    } else {
      1.0
    };
    // Explicitly requested size, use it; else
    if other.size.is_none() {
      if has_mathstyle {
        // otherwise set the size from mathstyle
        let str_mathstyle: &str = newfont.mathstyle.as_ref().unwrap();
        newfont.size = Some(style_scale * *STYLE_SIZE.get(str_mathstyle).unwrap() as f32);
      } else if Some(true) == other.scripted {
        // Or adjust both the mathstyle & size for scripts
        let str_stylekey: &str = self.mathstyle.as_ref().unwrap_or(&Cow::Borrowed("display"));
        newfont.mathstyle = SCRIPT_STYLE_MAP.get(str_stylekey).map(|c| Cow::Borrowed(*c));
        let str_mathstylekey: &str = newfont.mathstyle.as_ref().unwrap_or(&Cow::Borrowed("display"));
        newfont.size = Some(style_scale * *STYLE_SIZE.get(str_mathstylekey).unwrap() as f32);
      }
    }

    // TODO:
    // elsif ($options{fraction}) {     # Or adjust both for fractions
    //   $mathstyle = $fracstylemap{ $mathstyle            || 'display' };
    //   $size      = $style_scale * $stylesize{ $mathstyle || 'display' }; }

    // if ($options{emph}) {
    //   $shape = ($shape eq 'italic' ? 'upright' : 'italic');
    //   $flags |= $FLAG_EMPH; }
    // $flags &= ~$FLAG_EMPH if $mathstyle;    # Disable emph in math
    //   newfont
    // }
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
        if new.family.is_none() || (new.family.as_ref().unwrap() == "math") {
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
      if new.family.is_none() || (new.family.as_ref().unwrap() == "math") {
        new.family = Some(deffamily);
        new.shape = Some(defshape); // defaults, always.
      }
    } else {
      // Other Symbol
      new.family = Some(deffamily);
      new.shape = Some(defshape); // defaults, always.
      if new.series.is_some() && (new.series.as_ref().unwrap() != DEFSERIES) {
        new.series = Some(defseries);
      } // defaults, always.
    }
    new
  }

  pub fn distance(&self, other: &Font) -> i8 {
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
        if fam == "math" {
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

    if is_diff_f32(&self.size, &other.size) {
      result.insert(
        "fontsize".to_string(),
        (
          relative_font_size(self.size.unwrap(), other.size.unwrap()),
          Font {
            size: self.size,
            ..Font::default()
          },
        ),
      );
    }
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

  pub fn purestyle_changes(&self, other: &Font) -> Font {
    let mathstyle = self.get_mathstyle();
    let othermathstyle = other.get_mathstyle();
    let othercolor = other.get_color();
    let mut changes = Font {
      scale: Some(other.get_size().unwrap() / self.get_size().unwrap()),
      bg: other.bg.clone(),
      opacity: other.opacity.clone(), // should multiply or replace?
      ..Font::default()
    };
    if is_diff(&othercolor.cloned(), &Some(Cow::Borrowed(DEFCOLOR))) {
      changes.color = other.color.clone();
    }

    // TODO:
    // if mathstyle && othermathstyle {
    //   changes.mathstylestep = mathstylestep.get(mathstyle).unwrap().get(othermathstyle).unwrap();
    // }
    changes
  }

  pub fn get_metric(&self, c_opt: Option<char>) -> &MetricData {
    if let Some(c) = c_opt {
      let cstr = c.to_string();
      let fonts = match self.get_family() {
        Some(fname) if fname == "math" => MATH_FONTS,
        _ => TEXT_FONTS,
      };
      for name in fonts {
        if let Some(m) = STDMETRICS.get(name) {
          if m.sizes.contains_key(cstr.as_str()) {
            return m;
          }
        }
      }
    }
    STDMETRICS.get("cmr").unwrap()
  }

  pub fn get_em_width(&self) -> i32 {
    let size = self.get_size().unwrap_or(DEFSIZE);
    // Could (should) look for metric w/appropriate slant, weight, etc
    let m = STDMETRICS.get("cmr").unwrap();
    (size * m.emwidth).trunc() as i32
  }
  pub fn get_ex_height(&self) -> i32 {
    let size = self.get_size().unwrap_or(DEFSIZE);
    // Could (should) look for metric w/appropriate slant, weight, etc
    let m = STDMETRICS.get("cmr").unwrap();
    (size * m.exheight).trunc() as i32
  }
  pub fn get_mu_width(&self) -> i32 {
    let size = self.get_size().unwrap_or(DEFSIZE);
    // Could (should) look for metric w/appropriate slant, weight, etc
    let m = STDMETRICS.get("cmm").unwrap();
    (size * m.emwidth / 18.0).trunc() as i32
  }

  pub fn compute_string_size(&self, text: &str, options: HashMap<String, Stored>, state: &State) -> (Dimension, Dimension, Dimension) {
    if text.is_empty() || self.get_family().map(|fam| fam == "nullfont").unwrap_or(false) {
      return (Dimension::default(), Dimension::default(), Dimension::default());
    }
    let size = self.get_size().unwrap_or(DEFSIZE);
    let ismath = self.get_family().map(|fam| fam == "math").unwrap_or(false);
    let (mut w, mut h, mut d) = (0, 0, 0);
    for char in text.chars() {
      let metric = self.get_metric(Some(char));
      let entry_opt = metric.sizes.get(char.to_string().as_str());
      // let entry_opt  = metric.sizes.get(char);
      let (cw, ch, cd, ci) = if let Some(entry) = entry_opt {
        *entry
      } else {
        (0.75 * UNITY as f32, 0.7 * UNITY as f32, 0.2 * UNITY as f32, 0.0)
      };
      w += (cw * size).trunc() as i32;
      // if (my $kern = $chars[0] && $$metric{kerns}{ $char . $chars[0] }) {
      //   $w += int($size * $kern); }
      // if ($ismath && $ci) {
      //   $w += int($size * $ci); }
      h = max(h, (ch * size).trunc() as i32);
      d = max(d, (cd * size).trunc() as i32);
    }
    // The 1 is so that any actual glyph appears to be non-empty.
    // This is presumably only necessary to deal with the flawed emptiness heiristics in Alignment?
    if w == 0 {
      w = 1;
    }
    (Dimension::new(w), Dimension::new(h), Dimension::new(d))
  }

  // Here's where I avoid trying to emulate Knuth's line-breaking...
  // Mostly for List & Whatsit: compute the size of a list of boxes.
  // Options _SHOULD_ include:
  //   width:  if given, pretend to simulate line breaking to that width
  //   height,depth : ?
  //   vattach : top, bottom, center, baseline (...?) affects how the height & depth are
  //      allocated when there are multiple lines.
  //   layout : horizontal or vertical !!!
  // Boxes that arent a Core Box, List, Whatsit or a string are IGNORED
  //
  // The big problem with width is to have it propogate down from where
  // it may have been specified to the actual nested box that will get wrapped!
  // Try to mask this (temporarily) by unlisting, and (pretending to ) breaking up too wide items
  //
  // Another issue; SVG needs (sometimes) real sizes, even if the programmer
  // set some dimensions to 0 (eg.)   We may need to distinguish & store
  // requested vs real sizes?
  pub fn compute_boxes_size(
    &self,
    boxes: &[Digested],
    options: HashMap<String, Stored>,
    state: &mut State,
  ) -> Result<(Dimension, Dimension, Dimension)> {
    let fillwidth = match options.get("width") {
      Some(Stored::Int(fw)) => Some(*fw),
      None => match state.lookup_definition(&T_CS!("\\textwidth")) {
        Some(def) => def.value_of(Vec::new(), state).map(|x| x.value_of()),
        None => None,
      },
      _ => None,
    };
    let maxwidth = fillwidth.unwrap_or_default();
    //   # baselineskip, lineskip ??
    let baseline = state
      .lookup_definition(&T_CS!("\\baselineskip"))
      .expect("baseline skip should aways be defined")
      .value_of(Vec::new(), state)
      .expect("\\baselineskip should always have a value.")
      .value_of();
    let lineskip = state
      .lookup_definition(&T_CS!("\\lineskip"))
      .expect("lineskip should always be defined")
      .value_of(Vec::new(), state)
      .expect("\\lineskip should always have a value.")
      .value_of();
    let mut lines: Vec<(Dimension, Dimension, Dimension)> = Vec::new();
    let (mut wd, mut ht, mut dp) = (0.0, 0, 0);
    let (minwd, minht, mindp) = (0.0, 0.0, 0.0);
    let vattach = match options.get("vattach") {
      Some(Stored::String(vattach)) => vattach,
      _ => "baseline",
    };
    // Flatten top-level Lists (orrr pass-thru `fillwidth` ???)
    let filtered_boxes = boxes
      .iter()
      .flat_map(|thisbox| thisbox.unlist())
      .filter(|thisbox| !thisbox.has_property("isEmpty"));

    let mut prevbox_opt: Option<Digested> = None;
    for mut thisbox in filtered_boxes {
      // Should any `options` be inherited by the contained boxes?
      let (w, h, d) = thisbox.get_size(None, state)?;

      // DG: TODO: We'll have to figure out how to rearrange this logic,
      //           now that every emitted result of get_size is a Dimension.
      //           likely the sizing case moves elsewhere?
      // wd += if w._unit() == "mu" { w.sp_value() } else { w.value_of() };
      wd += w.value_of() as f32;

      //     if ((ref $h) && $h->can('_unit')) {
      //       $ht = max($ht, ($h->_unit eq 'mu' ? $h->spValue : $h->valueOf)); }
      ht = max(ht, h.value_of());

      //     if ((ref $d) && $d->can('_unit')) {
      //       $dp = max($dp, ($d->_unit eq 'mu' ? $d->spValue : $d->valueOf)); }
      dp = max(dp, d.value_of());

      // Kern HACK for lists of individual Box's
      if let Some(prevbox) = prevbox_opt {
        if matches!(prevbox.data(), DigestedData::TBox(_)) && matches!(thisbox.data(), DigestedData::TBox(_)) {
          let prevchar = prevbox.get_string(state)?.chars().last();
          let curchar = thisbox.get_string(state)?.chars().next();
          let metric = self.get_metric(curchar);
          if let Some(family) = self.get_family() {
            if family == "math" {
              wd += self.math_bearing(&thisbox, &prevbox);
            }
          }
          if let Some(prevc) = prevchar {
            if let Some(curc) = curchar {
              let kern_key = String::from(prevc) + &String::from(curc);
              if let Some(kern) = metric.kerns.get(kern_key.as_str()) {
                let size = self.get_size().unwrap_or(DEFSIZE);
                wd += size * kern;
              }
            }
          }
        }
      }
      //     my $newline = (($options{layout} || '') eq 'vertical')        # EVERY box is a row?
      //       || ((ref $box) && $box->getProperty('isBreak'))             # || $box is a linebreak
      //       || ((defined $maxwidth) && ($wd >= $maxwidth));             # or we've reached the requested width
      //     if ($newline) {
      //       if (@boxes) {
      //         if ($baseline > $ht + $dp) {
      //           $dp = $baseline - $ht; }
      //         else {
      //           $dp += $lineskip; } }
      //       push(@lines, [$wd, $ht, $dp]); $wd = $ht = $dp = 0; }
      prevbox_opt = Some(thisbox);
    }

    //   if ($wd || $ht || $dp) {    # be sure to get last line
    //     push(@lines, [$wd, $ht, $dp]); }
    //   # Deal with multiple lines
    //   my $nlines = scalar(@lines);
    //   if ($nlines == 0) {
    //     $wd = $ht = $dp = 0; }
    //   else {
    //     $wd = max(map { $$_[0] } @lines);
    //     $ht = sum(map { $$_[1] } @lines);
    //     $dp = sum(map { $$_[2] } @lines);
    //     if ($vattach eq 'top') {    # Top of box is aligned with top(?) of current text
    //       my ($w, $h, $d) = $font->getNominalSize;
    //       $h  = $h->valueOf;
    //       $dp = $ht + $dp - $h; $ht = $h; }
    //     elsif ($vattach eq 'bottom') {    # Bottom of box is aligned with bottom (?) of current text
    //       $ht = $ht + $dp; $dp = 0; }
    //     elsif ($vattach eq 'middle') {
    //       my ($w, $h, $d) = $font->getNominalSize;
    //       $h = $h->valueOf;
    //       my $c = ($ht + $dp) / 2;
    //       $ht = $c + $h / 2; $dp = $c - $h / 2; }
    //     else {                            # default is baseline (of the 1st line)
    //       my $h = $lines[0][1];
    //       $dp = $ht + $dp - $h; $ht = $h; } }

    Ok((Dimension::new_f32(wd), Dimension::new(ht), Dimension::new(dp)))
  }
}

fn is_diff(x: &Option<Cow<str>>, y: &Option<Cow<str>>) -> bool { x.is_some() && (y.is_none() || (x != y)) }

fn is_diff_f32(x: &Option<f32>, y: &Option<f32>) -> bool { x.is_some() && (y.is_none() || (x != y)) }

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
pub fn decode(code: u8, encoding_opt: Option<String>, implicit: bool, state: &State) -> Option<char> {
  let mut font = None;
  let encoding = match encoding_opt {
    None => {
      font = state.lookup_font();
      if let Some(ref font) = font {
        match font.get_encoding() {
          None => String::new(),
          Some(encoding) => encoding.clone().into_owned(),
        }
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

pub fn decode_string(string: &str, encoding_opt: Option<&str>, implicit: bool, state: &mut State) -> String {
  if string.is_empty() {
    return String::new();
  }
  let mut font = None;
  let encoding = match encoding_opt {
    None => {
      font = state.lookup_font();
      if let Some(ref font) = font {
        font.get_encoding().unwrap_or(&Cow::Borrowed(""))
      } else {
        ""
      }
    },
    Some(encoding) => encoding,
  };

  let mut map: Option<&Fontmap> = None;
  if !encoding.is_empty() {
    if let Some(encmap) = state.load_font_map(encoding) {
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
        let code = c as u16; // u16, so that Unicode chars get cast correctly
        if code < 128 {
          if let Some(Some(mapc_val)) = map.get(code as usize) {
            result_string.push(*mapc_val);
          }
        } else {
          result_string.push(c);
        }
      } else {
        result_string.push(c)
      }
    } else if let Some(map) = map {
      let code = c as u8;
      if let Some(Some(mapc_val)) = map.get(code as usize) {
        result_string.push(*mapc_val);
      }
    }
  }
  result_string
}

/// Convert stanard font size names, such as `tiny`, `Huge`, etc to f32
pub fn rationalize_font_size(size: &str) -> f32 {
  if let Some(symbolic) = FONT_SIZE.get(size) {
    *symbolic * DEFSIZE
  } else {
    DEFSIZE
  }
}

/// convert size to percent
pub fn relative_font_size(newsize: f32, oldsize: f32) -> String { s!("{}%", (0.5 + 100.0 * newsize / oldsize).floor()) }
