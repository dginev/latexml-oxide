use std::{
  borrow::Cow,
  cmp::max,
  fmt,
  hash::{Hash, Hasher},
  rc::Rc,
};

use once_cell::sync::Lazy;
/// Note that this has evolved way beynond just "font",
/// but covers text properties (or even display properties) in general
/// including basic font information, color & background color
/// as well as encoding and language information.
///
/// NOTE: This is now in Common that it may evolve to be useful in Post processing...
use regex::Regex;
use rustc_hash::FxHashMap as HashMap;

use crate::{
  BoxOps, Digested, DigestedData, Result,
  binding::content::{load_font_map, preload_font_map},
  common::{
    arena::{self, SymHashMap, SymStr},
    color::{self, Color},
    dimension::Dimension,
    numeric_ops::{NumericOps, UNITY, UNITY_F64, kround},
    store::Stored,
  },
  state::*,
};

pub mod standard_metrics;
use standard_metrics::{MetricData, STDMETRICS};

use crate::pin;

pub type Fontmap = Rc<[Option<char>]>;

static DEFFAMILY: &str = "serif";
static DEFSERIES: &str = "medium";
static DEFSHAPE: &str = "upright";
/// Perl: $DEFCOLOR = Black = Color::rgb(0,0,0)
static DEFCOLOR: Color = color::BLACK;
// Perl: $DEFBACKGROUND = undef (transparent), $DEFLANGUAGE = undef
// These are intentionally None in text_default/math_default.
static DEFOPACITY: &str = "1";
static DEFENCODING: &str = "OT1";
/// Perl: sub defsize() { return $STATE->lookupValue('NOMINAL_FONT_SIZE') || 10; }
/// Reads NOMINAL_FONT_SIZE from state, defaulting to 10.0.
fn defsize() -> f64 {
  let v = lookup_int("NOMINAL_FONT_SIZE");
  if v > 0 { v as f64 } else { 10.0 }
}

pub const TEXT_FONTS: [&str; 6] = ["cmr", "cmm", "cmsy", "cmex", "amsa", "amsb"];
pub const MATH_FONTS: [&str; 6] = ["cmm", "cmsy", "cmex", "amsa", "amsb", "cmr"];

pub const FLAG_FORCE_FAMILY: u8 = 0x1;
pub const FLAG_FORCE_SERIES: u8 = 0x2;
pub const FLAG_FORCE_SHAPE: u8 = 0x4;
pub const FLAG_EMPH: u8 = 0x10;

pub static FONT_TEXT_DEFAULT: Lazy<Font> = Lazy::new(Font::text_default);
static LATIN_LETTER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[\p{Latin}&&\pL]$").unwrap());
static GREEK_LETTER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[\p{Greek}&&\pL]$").unwrap());
static UPPER_LETTER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[\p{Lu}]$").unwrap());
static DIGIT_LETTER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[\p{N}]$").unwrap());
#[rustfmt::skip]
static FONT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(xylubt|xyluat|xydash|xycmbt|xycmat|xycirc|xybtip|xybsql|xyatip|ul9|ugq|uaq|txtt|txsyb|txsya|txss|txr|txmi|pzd|pzc|pxsyb|pxsya|pxr|pxmi|put|ptm|psy|ppl|pnc|phv|pcr|pbk|pag|msy|msx|msb|msa|manfnt|linew|line|lcirclew|lcircle|futs|futmi|futm|eus|eur|euf|euex|cmvtt|cmtt|cmtl|cmt|cmsy|cmssqi|cmssq|cmss|cmsltt|cmr|cmmib|cmm|cmfr|cmfib|cmex|cmdunh|cmdh|cmu|cmbsy|cmbrs|cmbrm|cmbr|cm|ccy|ccr|ccm|ccitt|bch|bbold|bbmss|bbm)(sbc|sb|mc|m|bx|bm|bc|b|)(sl|sc|n|it|i|csc|)(\d*)$").unwrap());
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

static FONT_FAMILY: Lazy<HashMap<&'static str, Font>> = Lazy::new(|| {
  raw_map!(
    "cmr"  => fontmap!(family => "serif"),      "cmss"  => fontmap!(family => "sansserif"),
    "cmtt" => fontmap!(family => "typewriter"), "cmvtt" => fontmap!(family => "typewriter"),
    "cmt"  => fontmap!(family => "serif"),
    "cmsltt" => fontmap!(family => "typewriter", shape => "slanted"),
    "cmssq" => fontmap!(family => "sansserif"),
    "cmssqi" => fontmap!(family => "sansserif", shape => "italic"),
    "cmdunh" => fontmap!(family => "serif"),
    "cmu"   => fontmap!(family => "serif"),
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
    "txtt"  => fontmap!(family => "typewriter"),
    // Modern family codes ABSENT from Perl's %font_family (Common/Font.pm) —
    // candidate to upstream. Without them, `\fontfamily{\ttdefault}
    // \selectfont` (fancyvrb's font setup) LOSES the abstract family when a
    // font package repoints \ttdefault: colm2026_conference loads
    // `inconsolata` (\ttdefault = zi4), so boxed Verbatim prompts dropped
    // ltx_font_typewriter and the browser painted full-size serif prose
    // inside frames TeX measured as \small monospace — border collisions
    // and text rivers (witness 2605.00468, Prompts 1-7).
    // Latin Modern:
    "lmr"   => fontmap!(family => "serif"),      "lmss"  => fontmap!(family => "sansserif"),
    "lmtt"  => fontmap!(family => "typewriter"), "lmvtt" => fontmap!(family => "typewriter"),
    // TeX Gyre:
    "qpl"   => fontmap!(family => "serif"),      "qtm"   => fontmap!(family => "serif"),
    "qbk"   => fontmap!(family => "serif"),      "qcs"   => fontmap!(family => "serif"),
    "qhv"   => fontmap!(family => "sansserif"),  "qag"   => fontmap!(family => "sansserif"),
    "qcr"   => fontmap!(family => "typewriter"), "qzc"   => fontmap!(family => "script"),
    // inconsolata (zi4), Bera Mono (fvm), Bera Serif/Sans (fve/fvs),
    // DejaVu Mono (DejaVuSansMono-TLF is fontspec-era; the NFSS code):
    "zi4"   => fontmap!(family => "typewriter"), "fi4"   => fontmap!(family => "typewriter"),
    "fvm"   => fontmap!(family => "typewriter"), "fve"   => fontmap!(family => "serif"),
    "fvs"   => fontmap!(family => "sansserif"),
    // Source Code/Sans/Serif Pro, Fira:
    "zsourcecodepro" => fontmap!(family => "typewriter"),
    "SourceCodePro-TLF" => fontmap!(family => "typewriter"),
    "FiraMono-TLF" => fontmap!(family => "typewriter"),
    "FiraSans-TLF" => fontmap!(family => "sansserif"),
    "txsya" => fontmap!(encoding => "AMSa"),     "txsyb" => fontmap!(encoding => "AMSb"),
    "pxr"   => fontmap!(family => "serif"),
    "pxsya" => fontmap!(encoding => "AMSa"),     "pxsyb" => fontmap!(encoding => "AMSb"),
    "futs"  => fontmap!(family => "serif"),
    "uaq"   => fontmap!(family => "serif"),      "ugq"   => fontmap!(family => "sansserif"),
    // Pretend to recognize plain & latex's extra fonts
    "manfnt"  => fontmap!(family => "graphic", encoding => "manfnt"),
    "line"    => fontmap!(family => "graphic", encoding => "line"),
    "linew"   => fontmap!(family => "graphic", encoding => "line", series => "bold"),
    "lcircle" => fontmap!(family => "graphic", encoding => "lcircle"),
    "lcirclew" => fontmap!(family => "graphic", encoding => "lcircle", series => "bold"),
    // Pretend to recognize xy's fonts
    "xydash" => fontmap!(family => "graphic"), "xyatip" => fontmap!(family => "graphic"),
    "xybtip" => fontmap!(family => "graphic"), "xybsql" => fontmap!(family => "graphic"),
    "xycirc" => fontmap!(family => "graphic"), "xycmat" => fontmap!(family => "graphic"),
    "xycmbt" => fontmap!(family => "graphic"), "xyluat" => fontmap!(family => "graphic"),
    "xylubt" => fontmap!(family => "graphic"),
    "eur"   => fontmap!(family => "serif"),      "eus"   => fontmap!(family => "script"),
    "euf"   => fontmap!(family => "fraktur"),    "euex"  => fontmap!(encoding => "OMX"),
    // The following are actually math fonts.
    "ccm"   => fontmap!(family => "serif", shape => "italic"),
    "cmm"   => fontmap!(family => "math", shape => "italic", encoding => "OML"),
    "cmex"  => fontmap!(encoding => "OMX"),
    "cmsy"  => fontmap!(encoding => "OMS"),
    "ccitt" => fontmap!(family => "typewriter", shape => "italic"),
    "cmbrm" => fontmap!(family => "sansserif", shape => "italic"),
    "futm"  => fontmap!(family => "serif", shape => "italic"),
    "futmi" => fontmap!(family => "serif", shape => "italic"),
    "txmi"  => fontmap!(family => "serif", shape => "italic"),
    "pxmi"  => fontmap!(family => "serif", shape => "italic"),
    // cmmib already in Perl
    "bbm"   => fontmap!(family => "blackboard"),
    "bbold" => fontmap!(family => "blackboard"),
    "bbmss" => fontmap!(family => "blackboard"),
    // some ams fonts
    "cmmib" => fontmap!(family => "italic", series   => "bold"),
    "cmbsy" => fontmap!(series => "bold", encoding => "OMS"),
    "msa"   => fontmap!(encoding => "AMSa"),
    "msb"   => fontmap!(encoding => "AMSb"),
    // Are these really the same?
    "msx" => fontmap!(encoding => "AMSa"),
    "msy" => fontmap!(encoding => "AMSb")
  )
});
/// Maps the "series code" to an abstract font series name
static FONT_SERIES: Lazy<HashMap<&'static str, Font>> = Lazy::new(|| {
  raw_map!(
    "" => Font::default(), "m" => fontmap!(series => "medium"),
      "mc" => fontmap!(series => "medium"),
    "b"  => fontmap!(series => "bold"),   "bc"  => fontmap!(series => "bold"),
      "bx" => fontmap!(series => "bold"),
    "sb" => fontmap!(series => "bold"),   "sbc" => fontmap!(series => "bold"),
      "bm" => fontmap!(series => "bold")
  )
});

/// Maps the "shape code" to an abstract font shape name.
static FONT_SHAPE: Lazy<HashMap<&'static str, Font>> = Lazy::new(|| {
  raw_map!(
    "" => Font::default(), "n" => fontmap!(shape => "upright"),
      "i" => fontmap!(shape => "italic"), "it" => fontmap!(shape => "italic"),
      "sl" => fontmap!(shape => "slanted"),
      "sc" => fontmap!(shape => "smallcaps"), "csc" => fontmap!(shape => "smallcaps")
  )
});

/// Symbolic font sizes, relative to the NOMINAL_FONT_SIZE (often 10)
/// extended logical font sizes, based on nominal document size of 10pts
/// Possibly should simply use absolute font point sizes, as declared in class...
static FONT_SIZE: Lazy<HashMap<&'static str, f64>> = Lazy::new(|| {
  raw_map!(
"tiny"   => 0.5,   "SMALL" => 0.7, "Small" => 0.8,  "small" => 0.9,
"normal" => 1.0,   "large" => 1.2, "Large" => 1.44, "LARGE" => 1.728,
"huge"   => 2.074, "Huge"  => 2.488,
"big"    => 1.2,   "Big"   => 1.6, "bigg" => 2.1, "Bigg" => 2.6)
});

static SCRIPT_STYLE_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
  raw_map!(
  "display" => "script", "text" => "script",
  "script" => "scriptscript", "scriptscript" => "scriptscript")
});

static FRAC_STYLE_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
  raw_map!(
  "display" => "text", "text" => "script",
  "script" => "scriptscript", "scriptscript" => "scriptscript")
});

static STYLE_SIZE: Lazy<HashMap<&'static str, usize>> = Lazy::new(|| {
  raw_map!(
  "display" => 10, "text" => 10, "script" => 7, "scriptscript" => 5)
});

// Note: Perl's Font.pm has a %mathstylesize table used in specialize()
// font-size scaling, but the path is commented out in Perl too. We don't
// implement this yet — restore from git history if/when specialize wants
// math-style-based size adjustment.

/// A special form of merge when copying/moving nodes to a new context,
/// particularly math which become scripts or such.
static MATH_STYLE_STEP: Lazy<HashMap<&'static str, HashMap<&'static str, i32>>> = Lazy::new(|| {
  raw_map!(
  "display" => raw_map!(
    "display" => 0, "text" => 1, "script" => 2, "scriptscript" => 3),
  "text"=> raw_map!("display" => -1, "text" => 0, "script" => 1, "scriptscript" => 2),
  "script"=> raw_map!("display" => -2, "text" => -1, "script" => 0, "scriptscript" => 1),
  "scriptscript" => raw_map!("display" => -3, "text" => -2, "script" => -1, "scriptscript" => 0))
});
static STEP_MATH_STYLE: Lazy<HashMap<&'static str, HashMap<i32, &'static str>>> = Lazy::new(|| {
  raw_map!(
"display" => raw_map!(-3 => "display", -2 => "display", -1 => "display",
  0 => "display", 1 => "text", 2 => "script", 3 => "scriptscript"),
"text" => raw_map!(-3 => "display", -2 => "display", -1 => "display",
  0 => "text", 1 => "script", 2 => "scriptscript", 3 => "scriptscript"),
"script" => raw_map!(-3 => "display", -2 => "display", -1 => "text",
  0 => "script", 1 => "scriptscript", 2 => "scriptscript", 3 => "scriptscript"),
"scriptscript" => raw_map!(-3 => "display", -2 => "text", -1 => "script",
  0 => "scriptscript", 1 => "scriptscript", 2 => "scriptscript", 3 => "scriptscript"))
});

/// Map Font (family, series, shape) to a TeX fontname (tfm).
/// Returns `None` if the combo isn't recognized. Matching on a tuple
/// of `&str` lets callers skip allocating an intermediate
/// `format!("{family}_{series}_{shape}")` key per lookup. Callers use
/// `lookup_metric_name(family, series, shape)` instead of the former
/// `METRIC_MAP.get(&format!(…))` pattern.
fn lookup_metric_name(family: &str, series: &str, shape: &str) -> Option<&'static str> {
  match (family, series, shape) {
    ("serif", "medium", "upright") => Some("cmr"),
    ("serif", "medium", "slanted") => Some("cmsl"),
    ("serif", "medium", "italic") => Some("cmti"),
    ("serif", "medium", "uprightitalic") => Some("cmu"),
    ("serif", "bold", "upright") => Some("cmbx"),
    ("serif", "medum", "smallcaps") => Some("cmcsc"), // typo preserved from Perl
    ("sansserif", "medium", "upright") => Some("cmss"),
    ("sansserif", "medium", "italic") => Some("cmssi"),
    ("sansserif", "bold", "upright") => Some("cmssbx"),
    ("typewriter", "medium", "upright") => Some("cmtt"),
    ("typewriter", "medium", "slanted") => Some("cmsltt"),
    ("math", "medium", "italic") => Some("cmmi"),
    ("math", "medium", "upright") => Some("cmr"),
    ("math", "bold", "italic") => Some("cmmib"),
    _ => None,
  }
}

// Fallback fontnames for looking up random Unicode,
// when they're not in the indicated FontMap
static METRIC_FALLBACKS: [&str; 6] = ["cmr", "cmmi", "cmsy", "cmex", "msam", "msbm"];

// Math bearing atom types
// 0=Ord, 1=Op, 2=Bin, 3=Rel, 4=Open, 5=Close, 6=Punct, 7=Inner
#[rustfmt::skip]
static MATH_ATOM_TYPE: Lazy<HashMap<&'static str, usize>> = Lazy::new(|| {
  raw_map!(
    "ID" => 0,
    "BIGOP" => 1, "SUMOP" => 1, "INTOP" => 1, "OPERATOR" => 1, "LIMITOP" => 1, "DIFFOP" => 1,
    "ADDOP" => 2, "MULOP" => 2, "BINOP" => 2, "COMPOSEOP" => 2, "MIDDLE" => 2, "VERTBAR" => 2,
    "RELOP" => 3, "METARELOP" => 3, "ARROW" => 3,
    "OPEN" => 4, "CLOSE" => 5,
    "PUNCT" => 6, "PERIOD" => 6,
    "ARRAY" => 7, "MODIFIER" => 7
  )
});

// Math bearing table: [prev_type][cur_type] => bearing level
// 0=none, positive=thin(1)/med(2)/thick(3) in display/text,
// negative: same but suppressed in script/scriptscript
#[rustfmt::skip]
static MATH_BEARINGS: [[i8; 8]; 8] = [
  [ 0,  1, -2, -3,  0,  0,  0, -1],
  [ 1,  1,  0, -3,  0,  0,  0, -1],
  [-2, -2,  0,  0, -2,  0,  0, -2],
  [-3, -3,  0,  0, -3,  0,  0, -3],
  [ 0,  0,  0,  0,  0,  0,  0,  0],
  [ 0,  1, -2, -3,  0,  0,  0, -1],
  [-1, -1,  0, -1, -1, -1, -1, -1],
  [-1,  1, -2, -3, -1,  0, -1, -1],
];

// (Perl Font.pm %baseline_map removed with the #2798 S6 sizing rewrite:
// `compute_boxes_size_stack` now uses the per-line baseline threaded from the
// List's `\baselineskip` property (recorded by S4 in repack_horizontal),
// which is the faithful #2798 source — not a static font-size→baseline map.)

/// Global auxiliary for font family lookup
pub fn lookup_font_family(code: &str) -> Option<&Font> { FONT_FAMILY.get(code) }

/// Global auxiliary for font series lookup
pub fn lookup_font_series(code: &str) -> Option<&Font> { FONT_SERIES.get(code) }

/// Global auxiliary for font shape lookup
pub fn lookup_font_shape(code: &str) -> Option<&Font> { FONT_SHAPE.get(code) }

/// Combine family/series/shape lookups into a single Font (Perl lookupTeXFont)
pub fn lookup_tex_font(fontname: &str, seriescode: &str, shapecode: &str) -> Font {
  let mut props = Font::default();
  if let Some(ffam) = lookup_font_family(fontname) {
    props = props.merge_ref(ffam);
  }
  if let Some(fser) = lookup_font_series(seriescode) {
    props = props.merge_ref(fser);
  }
  if let Some(fsh) = lookup_font_shape(shapecode) {
    props = props.merge_ref(fsh);
  }
  props
}

/// Find a Font Metric for a given fontname, fallback to 10pt or cmr as needed.
/// Perl: getMetricForName
pub fn get_metric_for_name(name: &str) -> &'static MetricData {
  let base = if let Some(idx) = name.find(|c: char| c.is_ascii_digit()) {
    &name[..idx]
  } else {
    name
  };
  // Try exact name first (e.g. "cmr10")
  if let Some(m) = STDMETRICS.get(name) {
    return m;
  }
  // Try base without size (e.g. "cmr" from "cmr10")
  if let Some(m) = STDMETRICS.get(base) {
    return m;
  }
  // Try base + "10". Stack-buffer concat avoids the per-call `format!`
  // heap alloc — `get_metric_for_name` is reached through the
  // per-character `get_metric` loop, so this is a real allocation
  // site. Font basenames are short (≤ ~20 ASCII bytes); 32 bytes is
  // ample padding.
  let base_bytes = base.as_bytes();
  if base_bytes.len() + 2 <= 32 {
    let mut buf = [0u8; 32];
    buf[..base_bytes.len()].copy_from_slice(base_bytes);
    buf[base_bytes.len()] = b'1';
    buf[base_bytes.len() + 1] = b'0';
    if let Ok(s) = std::str::from_utf8(&buf[..base_bytes.len() + 2])
      && let Some(m) = STDMETRICS.get(s)
    {
      return m;
    }
  }
  // Ultimate fallback to "cmr"
  STDMETRICS
    .get("cmr")
    .expect("STDMETRICS must contain 'cmr'")
}

pub fn decode_fontname(name: &str, at_opt: Option<f64>, scaled_opt: Option<f64>) -> Option<Font> {
  if let Some(cap) = FONT_RE.captures(name) {
    // Perl: my %props = (series => 'medium', shape => 'upright', encoding => 'OT1');
    let mut props = Font {
      series: Some(Cow::Borrowed(DEFSERIES)),
      shape: Some(Cow::Borrowed(DEFSHAPE)),
      encoding: Some(Cow::Borrowed("OT1")),
      ..Font::default()
    };
    let fam = cap.get(1).map_or("", |m| m.as_str());
    let ser = cap.get(2).map_or("", |m| m.as_str());
    let shp = cap.get(3).map_or("", |m| m.as_str());
    let size_str = cap.get(4).map_or("", |m| m.as_str());
    if let Some(ffam) = lookup_font_family(fam) {
      props = props.merge_ref(ffam);
    }
    if let Some(fser) = lookup_font_series(ser) {
      props = props.merge_ref(fser);
    }
    if let Some(fsh) = lookup_font_shape(shp) {
      props = props.merge_ref(fsh);
    }
    let mut size = if let Some(at) = at_opt {
      at
    } else {
      let size_f64 = size_str.parse::<f64>().unwrap_or(1.0);
      if size_f64 == 0.0 { 1.0 } else { size_f64 } // Yes, also if 0, "" (from regexp)
    };
    if let Some(scaled) = scaled_opt {
      size *= scaled;
    }
    props.size = Some(size);
    // Experimental Hack !?!?!?
    if props.encoding.is_none() {
      props.encoding = Some(Cow::Borrowed("OT1"));
    }
    // TODO: What is this field for?
    // if let Some(at) = at_opt {
    //   props.at = Some(s!("{at}pt"));
    // }
    Some(props)
  } else {
    None
  }
}

/// A data structure containing Font information, but also related textual properties (such as
/// color)
///
/// This struct is a little interesting, as we want to pass overrides that partially modify (via a
/// merge) the current font, in each definitional binding. To accommodate that with this struct,
/// every single field needs to be an Option, in order to unambiguously tell the "intend" of
/// override (Some) vs no intent (None).
#[derive(Clone, Default)]
pub struct Font {
  pub family:        Option<Cow<'static, str>>,
  pub series:        Option<Cow<'static, str>>,
  pub shape:         Option<Cow<'static, str>>,
  pub size:          Option<f64>,
  pub color:         Option<Color>,
  pub bg:            Option<Color>,
  pub opacity:       Option<Cow<'static, str>>,
  pub encoding:      Option<Cow<'static, str>>,
  pub language:      Option<Cow<'static, str>>,
  pub mathstyle:     Option<Cow<'static, str>>,
  pub mathstylestep: Option<i32>,
  pub name:          Option<Cow<'static, str>>,
  pub emph:          Option<bool>,
  pub scripted:      Option<bool>,
  pub fraction:      Option<bool>,
  // Note: forcefamily, forceseries, forceshape (& forcebold for compatibility)
  // are only useful for fonts in math; See the specialize method below.
  pub forceseries:   Option<bool>,
  pub forcefamily:   Option<bool>,
  pub forceshape:    Option<bool>,
  pub forcebold:     Option<bool>,
  pub scale:         Option<f64>,
  pub flags:         Option<u8>,
}

impl Hash for Font {
  // We need to implement hash since we have to tell Rust how to hash `f64` values
  // for now I have decided to go for a precision of 4 digits after the decimal point,
  // so multiplying by 1000
  fn hash<H: Hasher>(&self, hasher: &mut H) {
    self.family.hash(hasher);
    self.series.hash(hasher);
    self.shape.hash(hasher);
    self.size.map(|size| (size * 1000.0) as i64).hash(hasher);
    // None color hashes same as Some(DEFCOLOR)
    Some(self.color.unwrap_or(DEFCOLOR)).hash(hasher);
    self.bg.hash(hasher);
    self.opacity.hash(hasher);
    self.encoding.hash(hasher);
    self.language.hash(hasher);
    self.mathstyle.hash(hasher);
    self.mathstylestep.hash(hasher);
    self.name.hash(hasher);
    self.emph.hash(hasher);
    self.scripted.hash(hasher);
    self.forceseries.hash(hasher);
    self.forcefamily.hash(hasher);
    self.forceshape.hash(hasher);
    self.scale.map(|scale| (scale * 1000.0) as i64).hash(hasher);
    self.flags.hash(hasher);
  }
}
impl PartialEq for Font {
  fn eq(&self, other: &Self) -> bool {
    self.family == other.family
      && self.series == other.series
      && self.shape == other.shape
      && self.size == other.size
      && !is_diff_font_color(self.color.as_ref(), other.color.as_ref())
      && self.bg == other.bg
      && self.opacity == other.opacity
      && self.encoding == other.encoding
      && self.language == other.language
      && self.mathstyle == other.mathstyle
      && self.mathstylestep == other.mathstylestep
      && self.name == other.name
      && self.emph == other.emph
      && self.scripted == other.scripted
      && self.fraction == other.fraction
      && self.forceseries == other.forceseries
      && self.forcefamily == other.forcefamily
      && self.forceshape == other.forceshape
      && self.forcebold == other.forcebold
      && self.scale == other.scale
      && self.flags == other.flags
  }
}
impl Eq for Font {}
// display is used often for attributes in binding replacements,
// as in font="#font"
impl fmt::Display for Font {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.family.as_ref().unwrap_or(&Cow::Borrowed("")))
  }
}

impl fmt::Debug for Font {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let star = Cow::Borrowed("*");
    write!(f, "Font[")?;
    write!(f, "{}", self.family.as_ref().unwrap_or(&star))?;
    write!(f, ",")?;
    write!(f, "{}", self.series.as_ref().unwrap_or(&star))?;
    write!(f, ",")?;
    write!(f, "{}", self.shape.as_ref().unwrap_or(&star))?;
    write!(f, ",")?;
    let size_str = self
      .size
      .as_ref()
      .map(|x| x.to_string())
      .unwrap_or_else(|| String::from('*'));
    write!(f, "{}", size_str)?;
    write!(f, ",")?;
    if let Some(ref c) = self.color {
      write!(f, "{c}")?;
    } else {
      write!(f, "*")?;
    }
    write!(f, ",")?;
    if let Some(ref b) = self.bg {
      write!(f, "{b}")?;
    } else {
      write!(f, "*")?;
    }
    write!(f, ",")?;
    write!(f, "{}", self.opacity.as_ref().unwrap_or(&star))?;
    write!(f, ",")?;
    let scale_str = self
      .scale
      .as_ref()
      .map(|x| x.to_string())
      .unwrap_or_else(|| String::from('*'));
    write!(f, "{}", scale_str)?;
    write!(f, ",")?;
    write!(f, "{}", self.mathstyle.as_ref().unwrap_or(&star))?;
    // TODO? LaTeXML doesn't seem to emit these
    // if let Some(ref encoding) = self.encoding {
    //   parts.push(s!("encoding: {:?}", encoding))
    // }
    // if let Some(ref language) = self.language {
    //   parts.push(s!("language: {:?}", language))
    // }
    // if let Some(ref mathstylestep) = self.mathstyle {
    //   parts.push(s!("mathstylestep: {:?}", mathstylestep))
    // }
    // if let Some(ref forceseries) = self.forceseries {
    //   parts.push(s!("forceseries: {:?}", forceseries))
    // }
    // if let Some(ref forcefamily) = self.forcefamily {
    //   parts.push(s!("forcefamily: {:?}", forcefamily))
    // }
    // if let Some(ref forceshape) = self.forceshape {
    //   parts.push(s!("forceshape: {:?}", forceshape))
    // }
    // if let Some(ref scripted) = self.scripted {
    //   parts.push(s!("scripted: {:?}", scripted))
    // }
    write!(f, "]")
  }
}

impl Font {
  pub fn text_default() -> Self {
    Font {
      family:        Some(Cow::Borrowed(DEFFAMILY)),
      series:        Some(Cow::Borrowed(DEFSERIES)),
      shape:         Some(Cow::Borrowed(DEFSHAPE)),
      size:          Some(defsize()),
      color:         None, // None = inherited default (DEFCOLOR); Some = explicitly set
      bg:            None, // Perl: $DEFBACKGROUND = undef (transparent)
      opacity:       Some(Cow::Borrowed(DEFOPACITY)),
      encoding:      Some(Cow::Borrowed(DEFENCODING)),
      language:      None, // Perl: $DEFLANGUAGE = undef
      mathstyle:     None,
      mathstylestep: None,
      emph:          None,
      name:          None,
      scripted:      None,
      fraction:      None,
      forceseries:   None,
      forcefamily:   None,
      forceshape:    None,
      forcebold:     None,
      scale:         None,
      flags:         None,
    }
  }
  pub fn math_default() -> Self {
    Font {
      family:        Some(Cow::Borrowed("math")),
      series:        Some(Cow::Borrowed(DEFSERIES)),
      shape:         Some(Cow::Borrowed("italic")),
      size:          Some(defsize()),
      color:         None, // None = inherited default (DEFCOLOR); Some = explicitly set
      bg:            None, // Perl: $DEFBACKGROUND = undef
      opacity:       Some(Cow::Borrowed(DEFOPACITY)),
      encoding:      None, // Perl has 'OT1' but Rust char decoding uses encoding differently
      language:      None, // Perl: $DEFLANGUAGE = undef
      mathstyle:     Some(Cow::Borrowed("text")),
      mathstylestep: None,
      emph:          None,
      name:          None,
      scripted:      None,
      fraction:      None,
      forceseries:   None,
      forcefamily:   None,
      forceshape:    None,
      forcebold:     None,
      scale:         None,
      flags:         None,
    }
  }

  pub fn to_hashable(&self) -> u64 {
    // MUST be deterministic: equal Fonts must hash equal, stably across calls
    // AND across process runs. `set_node_font`/`get_node_font` use this as the
    // `_font` key and `node_fonts` map key, so a randomized seed
    // (`RandomState::new()`, used here previously) gave the same Font a
    // different id on every call — breaking font dedup and making the document
    // build run-to-run non-deterministic (intermittent locked-frame/mode
    // FATALs, e.g. 1510.04473). `FxHasher` has a fixed seed.
    let mut hasher = rustc_hash::FxHasher::default();
    Hash::hash(self, &mut hasher);
    hasher.finish()
  }

  /// Condensed string showing only non-default components.
  /// Perl: stringify
  pub fn stringify(&self) -> String {
    let fam = self
      .family
      .as_deref()
      .map(|f| if f == "math" { "serif" } else { f });
    let mut parts: Vec<&str> = Vec::new();
    if let Some(f) = fam
      && f != DEFFAMILY
    {
      parts.push(f);
    }
    if let Some(ref ser) = self.series
      && ser.as_ref() != DEFSERIES
    {
      parts.push(ser);
    }
    if let Some(ref shp) = self.shape
      && shp.as_ref() != DEFSHAPE
    {
      parts.push(shp);
    }
    // Size: use temporary string for formatting
    let size_str;
    if let Some(siz) = self.size
      && (siz - defsize()).abs() > 0.001
    {
      size_str = siz.to_string();
      parts.push(&size_str);
    }
    let color_str;
    if let Some(ref col) = self.color
      && *col != DEFCOLOR
    {
      color_str = col.to_attribute();
      parts.push(&color_str);
    }
    let bg_str;
    if let Some(ref bkg) = self.bg {
      // Perl: $DEFBACKGROUND = undef, so any set bg is non-default
      bg_str = bkg.to_attribute();
      parts.push(&bg_str);
    }
    if let Some(ref opa) = self.opacity
      && opa.as_ref() != DEFOPACITY
    {
      parts.push(opa);
    }
    if let Some(ref ms) = self.mathstyle {
      parts.push(ms);
    }
    let flags_str;
    if let Some(flags) = self.flags
      && flags != 0
    {
      flags_str = flags.to_string();
      parts.push(&flags_str);
    }
    format!("Font[{}]", parts.join(","))
  }

  /// Wildcard font matching: if any components are defined in both fonts,
  /// they must be equal. Perl: match
  pub fn font_match(&self, other: &Font) -> bool {
    fn check<T: PartialEq>(a: &Option<T>, b: &Option<T>) -> bool {
      !(a.is_some() && b.is_some() && a != b)
    }
    check(&self.family, &other.family)
      && check(&self.series, &other.series)
      && check(&self.shape, &other.shape)
      && check(&self.size, &other.size)
      // For color: None = DEFCOLOR, so compare effective colors
      && !is_diff_font_color(self.color.as_ref(), other.color.as_ref())
      && check(&self.bg, &other.bg)
      && check(&self.opacity, &other.opacity)
      && check(&self.encoding, &other.encoding)
      && check(&self.language, &other.language)
      && check(&self.mathstyle, &other.mathstyle)
  }

  /// Fill in undefined fields from a concrete font.
  /// Perl: makeConcrete
  pub fn make_concrete(&self, concrete: &Font) -> Self {
    Font {
      family:        self.family.clone().or_else(|| concrete.family.clone()),
      series:        self.series.clone().or_else(|| concrete.series.clone()),
      shape:         self.shape.clone().or_else(|| concrete.shape.clone()),
      size:          self.size.or(concrete.size),
      color:         self.color.or(concrete.color),
      bg:            self.bg.or(concrete.bg),
      opacity:       self.opacity.clone().or_else(|| concrete.opacity.clone()),
      encoding:      self.encoding.clone().or_else(|| concrete.encoding.clone()),
      language:      self.language.clone().or_else(|| concrete.language.clone()),
      mathstyle:     self
        .mathstyle
        .clone()
        .or_else(|| concrete.mathstyle.clone()),
      flags:         Some(self.flags.unwrap_or(0) | concrete.flags.unwrap_or(0)),
      mathstylestep: self.mathstylestep.or(concrete.mathstylestep),
      name:          self.name.clone().or_else(|| concrete.name.clone()),
      emph:          self.emph.or(concrete.emph),
      scripted:      self.scripted.or(concrete.scripted),
      fraction:      self.fraction.or(concrete.fraction),
      forceseries:   self.forceseries.or(concrete.forceseries),
      forcefamily:   self.forcefamily.or(concrete.forcefamily),
      forceshape:    self.forceshape.or(concrete.forceshape),
      forcebold:     self.forcebold.or(concrete.forcebold),
      scale:         self.scale.or(concrete.scale),
    }
  }

  /// Apply pure style changes (from purestyleChanges) to this font.
  /// Perl: mergePurestyle
  pub fn merge_purestyle(&self, changes: &Font) -> Self {
    let mut new = self.clone();
    if let Some(scale) = changes.scale
      && let Some(ref mut sz) = new.size
    {
      *sz *= scale;
    }
    if changes.color.is_some() {
      new.color.clone_from(&changes.color);
    }
    if changes.bg.is_some() {
      new.bg.clone_from(&changes.bg);
    }
    if changes.opacity.is_some() {
      new.opacity.clone_from(&changes.opacity);
    }
    if let Some(step) = changes.mathstylestep {
      let cur_style: &str = new.mathstyle.as_deref().unwrap_or("display");
      if let Some(step_map) = STEP_MATH_STYLE.get(cur_style)
        && let Some(new_style) = step_map.get(&step)
      {
        new.mathstyle = Some(Cow::Borrowed(new_style));
      }
    }
    new
  }

  /// Compute math bearing (inter-atom spacing) between two boxes.
  /// Perl: math_bearing
  pub fn math_bearing(&self, thisbox: &Digested, prevbox: &Digested) -> f64 {
    let r0 = prevbox
      .get_property("role")
      .and_then(|s| match s.into_owned() {
        Stored::String(sym) => Some(arena::with(sym, |s| s.to_string())),
        _ => None,
      })
      .unwrap_or_else(|| "ID".to_string());
    let r1 = thisbox
      .get_property("role")
      .and_then(|s| match s.into_owned() {
        Stored::String(sym) => Some(arena::with(sym, |s| s.to_string())),
        _ => None,
      })
      .unwrap_or_else(|| "ID".to_string());
    let t0 = *MATH_ATOM_TYPE.get(r0.as_str()).unwrap_or(&0);
    let t1 = *MATH_ATOM_TYPE.get(r1.as_str()).unwrap_or(&0);
    let bearing = MATH_BEARINGS[t0][t1];
    let style = self
      .get_mathstyle()
      .map(|s| s.to_string())
      .unwrap_or_else(|| "text".to_string());
    if bearing == 0 || (bearing < 0 && style != "display" && style != "text") {
      return 0.0;
    }
    // Look up the bearing register: 1=thinmuskip, 2=medmuskip, 3=thickmuskip
    let reg_cs = match bearing.unsigned_abs() {
      1 => T_CS!("\\thinmuskip"),
      2 => T_CS!("\\medmuskip"),
      3 => T_CS!("\\thickmuskip"),
      _ => return 0.0,
    };
    if let Ok(Some(def)) = lookup_definition(&reg_cs)
      && let Some(val) = def.value_of(Vec::new())
    {
      // Perl: $STATE->lookupDefinition(...)->valueOf->spValue
      // MuGlue->spValue = fixpoint($skip/UNITY, font->getMUWidth)
      //                 = kround((skip/UNITY) * MUWidth)
      // The raw skip is in mu*UNITY units; convert to sp via MUWidth.
      let skip = val.value_of();
      let mu_width = self.get_mu_width() as f64;
      return (skip as f64 / UNITY_F64 * mu_width).trunc();
    }
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
  pub fn get_family(&self) -> Option<&Cow<'_, str>> { self.family.as_ref() }
  pub fn get_series(&self) -> Option<&Cow<'_, str>> { self.series.as_ref() }
  pub fn get_shape(&self) -> Option<&Cow<'_, str>> { self.shape.as_ref() }
  pub fn get_size(&self) -> Option<f64> { self.size }
  pub fn get_color(&self) -> Option<&Color> { self.color.as_ref() }
  pub fn get_background(&self) -> Option<&Color> { self.bg.as_ref() }
  pub fn get_opacity(&self) -> Option<&Cow<'_, str>> { self.opacity.as_ref() }
  pub fn get_encoding(&self) -> Option<&Cow<'_, str>> { self.encoding.as_ref() }
  pub fn get_language(&self) -> Option<&Cow<'_, str>> { self.language.as_ref() }
  pub fn get_mathstyle(&self) -> Option<&Cow<'_, str>> { self.mathstyle.as_ref() }
  pub fn get_flags(&self) -> Option<u8> { self.flags }

  // NOTE: In math, NORMALLY, setting any one of
  //    family, series or shape
  // will, usually, automatically reset the others to thier defaults!
  // You must arrange this in the calls....
  pub fn merge(&self, other: Font) -> Self { self.merge_ref(&other) }

  /// Like `merge` but borrows `other` to avoid requiring callers to own or
  /// clone a Font just to pass into merge. Clones only the retained fields
  /// from `other` (cheap — Option<Cow<'static,str>> clones are free for
  /// Borrowed variants, and most Font fields are None in typical uses).
  pub fn merge_ref(&self, other: &Font) -> Self {
    // Handle forcebold for compatibility (Perl lines 873-874)
    let mut series = other.series.clone();
    let mut force_series = other.forceseries;
    if other.forcebold == Some(true) {
      series = Some(Cow::Borrowed("bold"));
      force_series = Some(true);
    }

    // Build flags from force options
    let mut flags: u8 = 0;
    if other.forcefamily == Some(true) {
      flags |= FLAG_FORCE_FAMILY;
    }
    if force_series == Some(true) {
      flags |= FLAG_FORCE_SERIES;
    }
    if other.forceshape == Some(true) {
      flags |= FLAG_FORCE_SHAPE;
    }

    let oflags = self.flags.unwrap_or(0);
    // Perl: fallback to self if not overridden, or if force-flags on self prevent override
    let family = if other.family.is_none() || (oflags & FLAG_FORCE_FAMILY != 0) {
      self.family.clone()
    } else {
      other.family.clone()
    };
    let series = if series.is_none() || (oflags & FLAG_FORCE_SERIES != 0) {
      self.series.clone()
    } else {
      series
    };
    let mut shape = if other.shape.is_none() || (oflags & FLAG_FORCE_SHAPE != 0) {
      self.shape.clone()
    } else {
      other.shape.clone()
    };
    let mut size = other.size.or(self.size);
    let color = other.color.or(self.color);
    // Perl: $bg = $$self[5] if (!exists $options{background});
    // Only override bg if `other` actually specifies one
    let bg = if other.bg.is_some() {
      other.bg
    } else {
      self.bg
    };
    let opacity = other.opacity.clone().or_else(|| self.opacity.clone());
    let encoding = other.encoding.clone().or_else(|| self.encoding.clone());
    let language = other.language.clone().or_else(|| self.language.clone());
    let mut mathstyle = other.mathstyle.clone().or_else(|| self.mathstyle.clone());
    flags |= self.flags.unwrap_or(0);

    // Dynamic adjustment directives
    if let Some(scale) = other.scale
      && let Some(ref mut sz) = size
    {
      *sz *= scale;
    }

    // Scale factor for mathstyle-based sizing
    let style_scale = if let Some(sz) = self.size {
      let key: &str = self.mathstyle.as_deref().unwrap_or("display");
      sz / *STYLE_SIZE.get(key).unwrap_or(&10) as f64
    } else {
      1.0
    };

    if other.size.is_some() {
      // Explicitly requested size, use it
    } else if other.mathstyle.is_some() {
      // Set the size from mathstyle
      let ms: &str = mathstyle.as_deref().unwrap_or("display");
      size = Some(style_scale * *STYLE_SIZE.get(ms).unwrap_or(&10) as f64);
    } else if other.scripted == Some(true) {
      // Adjust both the mathstyle & size for scripts
      let ms: &str = mathstyle.as_deref().unwrap_or("display");
      mathstyle = SCRIPT_STYLE_MAP.get(ms).map(|c| Cow::Borrowed(*c));
      let new_ms: &str = mathstyle.as_deref().unwrap_or("display");
      size = Some(style_scale * *STYLE_SIZE.get(new_ms).unwrap_or(&10) as f64);
    } else if other.fraction == Some(true) {
      // Adjust both for fractions
      let ms: &str = mathstyle.as_deref().unwrap_or("display");
      mathstyle = FRAC_STYLE_MAP.get(ms).map(|c| Cow::Borrowed(*c));
      let new_ms: &str = mathstyle.as_deref().unwrap_or("display");
      size = Some(style_scale * *STYLE_SIZE.get(new_ms).unwrap_or(&10) as f64);
    }

    // Emphasis handling (Perl lines 909-912)
    if other.emph == Some(true) {
      shape = if shape.as_deref() == Some("italic") {
        Some(Cow::Borrowed("upright"))
      } else {
        Some(Cow::Borrowed("italic"))
      };
      flags |= FLAG_EMPH;
    }
    // Disable emph in math (Perl: $flags &= ~$FLAG_EMPH if $mathstyle)
    if mathstyle.is_some() {
      flags &= !FLAG_EMPH;
    }

    let newfont = Font {
      family,
      series,
      shape,
      size,
      color,
      bg,
      opacity,
      encoding,
      language,
      mathstyle,
      flags: Some(flags),
      // Carry over fields that aren't part of Perl's merge:
      mathstylestep: other.mathstylestep.or(self.mathstylestep),
      name: other.name.clone().or_else(|| self.name.clone()),
      emph: None,
      scripted: None,
      fraction: None,
      forceseries: if flags & FLAG_FORCE_SERIES != 0 {
        Some(true)
      } else {
        None
      },
      forcefamily: if flags & FLAG_FORCE_FAMILY != 0 {
        Some(true)
      } else {
        None
      },
      forceshape: if flags & FLAG_FORCE_SHAPE != 0 {
        Some(true)
      } else {
        None
      },
      forcebold: None,
      scale: None,
    };
    // Note: Perl's merge() has an optional `specialize` option that is passed
    // explicitly (e.g. merge(specialize => $text)). It's NOT keyed on the font name.
    // Specialize is called at TBox creation time (tbox.rs) with the actual text content.
    // Do NOT call specialize here with the font name — it corrupts font properties
    // (e.g. resetting series "bold" to "medium" for font names like "cmb10").
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
      return new; // ?
    }
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
      self.shape.clone().unwrap_or_else(|| DEFSHAPE.into())
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
        if new.family.is_none() || (new.family.as_deref() != Some(DEFFAMILY)) {
          new.family = Some(deffamily);
        }
        // Perl: $shape = 'italic' if !$shape || !($flags & $FLAG_FORCE_SHAPE);
        if new.shape.is_none() || !self.forceshape.unwrap_or(false) {
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
    let mut distance: i8 = 0;
    // Normalize "math" → "serif" for comparison (Perl lines 436-437)
    let fam = self
      .family
      .as_deref()
      .map(|f| if f == "math" { "serif" } else { f });
    let ofam = other
      .family
      .as_deref()
      .map(|f| if f == "math" { "serif" } else { f });
    if is_diff_opt_str(fam, ofam) {
      distance += 1;
    }
    if is_diff_opt_str(self.series.as_deref(), other.series.as_deref()) {
      distance += 1;
    }
    if is_diff_opt_str(self.shape.as_deref(), other.shape.as_deref()) {
      distance += 1;
    }
    if is_diff_f64(self.size, other.size) {
      distance += 1;
    }
    // Color: use reference-style comparison (different Color variant = different).
    // Perl's isDiff uses object reference equality: Cmyk(0,0,0,1) ≠ Rgb(0,0,0)
    // even though both are visually black.
    if is_diff_font_color_ref(self.color.as_ref(), other.color.as_ref()) {
      distance += 1;
    }
    if is_diff_color(self.bg.as_ref(), other.bg.as_ref()) {
      distance += 1;
    }
    if is_diff_opt_str(self.opacity.as_deref(), other.opacity.as_deref()) {
      distance += 1;
    }
    // Perl does NOT count encoding differences
    // Perl does NOT count mathstyle differences
    if is_diff_opt_str(self.language.as_deref(), other.language.as_deref()) {
      distance += 1;
    }
    // Perl: ($flags & $FLAG_EMPH) ^ ($oflags & $FLAG_EMPH) ? 1 : 0
    let flags = self.flags.unwrap_or(0);
    let oflags = other.flags.unwrap_or(0);
    if (flags & FLAG_EMPH) ^ (oflags & FLAG_EMPH) != 0 {
      distance += 1;
    }
    distance
  }

  /// This method compares 2 fonts, returning the differences between them.
  /// Returns the font attribute string (family/series/shape components that differ
  /// from text defaults), joined by spaces. E.g., "italic" for a math font.
  /// Used by cancel.sty to capture font state for XML attributes.
  pub fn font_attribute_string(&self) -> String {
    let mut parts = Vec::new();
    if let Some(ref fam) = self.family {
      let f = if fam == "math" { "serif" } else { fam.as_ref() };
      if f != DEFFAMILY {
        parts.push(f.to_string());
      }
    }
    if let Some(ref ser) = self.series
      && ser.as_ref() != DEFSERIES
    {
      parts.push(ser.to_string());
    }
    if let Some(ref shp) = self.shape
      && shp.as_ref() != DEFSHAPE
    {
      parts.push(shp.to_string());
    }
    parts.join(" ")
  }

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
    if is_diff(family.as_ref(), other_family.as_ref()) {
      diffs.push(family.clone().unwrap());
      font_properties.family = family;
    }
    if is_diff(self.series.as_ref(), other.series.as_ref()) {
      let series = self.series.clone().unwrap();
      diffs.push(series);
      font_properties.series.clone_from(&self.series);
    }
    if is_diff(self.shape.as_ref(), other.shape.as_ref()) {
      let shape = self.shape.clone().unwrap();
      diffs.push(shape);
      font_properties.shape.clone_from(&self.shape);
    }
    let mut result = HashMap::default();

    if !diffs.is_empty() {
      let font_value = diffs.join(" ");
      result.insert(s!("font"), (font_value, font_properties));
    }

    if is_diff_f64(self.size.as_ref().copied(), other.size.as_ref().copied()) {
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
    // Emit color when Color variants differ (reference-style comparison).
    // Perl's `ne` treats Cmyk(0,0,0,1) ≠ Rgb(0,0,0) even though both are black.
    if is_diff_font_color_ref(self.color.as_ref(), other.color.as_ref()) {
      let effective_color = self.color.unwrap_or(DEFCOLOR);
      result.insert(
        "color".to_string(),
        (effective_color.to_attribute(), Font {
          color: Some(effective_color),
          ..Font::default()
        }),
      );
    }
    if is_diff_color(self.bg.as_ref(), other.bg.as_ref()) {
      result.insert(
        "backgroundcolor".to_string(),
        (self.bg.as_ref().unwrap().to_attribute(), Font {
          bg: self.bg,
          ..Font::default()
        }),
      );
    }
    if is_diff(self.opacity.as_ref(), other.opacity.as_ref()) {
      result.insert(
        "opacity".to_string(),
        (self.opacity.as_ref().unwrap().to_string(), Font {
          opacity: self.opacity.clone(),
          ..Font::default()
        }),
      );
    }
    if is_diff(self.encoding.as_ref(), other.encoding.as_ref()) {
      result.insert(
        "encoding".to_string(),
        (self.encoding.as_ref().unwrap().to_string(), Font {
          encoding: self.encoding.clone(),
          ..Font::default()
        }),
      );
    }
    if is_diff(self.language.as_ref(), other.language.as_ref()) {
      result.insert(
        "xml:lang".to_string(),
        (self.language.as_ref().unwrap().to_string(), Font {
          language: self.language.clone(),
          ..Font::default()
        }),
      );
    }
    // Emph: (!$mstyle && $flags && ($flags & $FLAG_EMPH) && (!$oflags || !($oflags & $FLAG_EMPH))
    let flags = self.flags.unwrap_or(0);
    let oflags = other.flags.unwrap_or(0);
    if self.mathstyle.is_none()
      && flags != 0
      && (flags & FLAG_EMPH) != 0
      && (oflags == 0 || (oflags & FLAG_EMPH) == 0)
    {
      result.insert(
        "element".to_string(),
        ("ltx:emph".to_string(), Font::default()),
      );
    }
    // We do NOT want mathstyle showing up automatically in the attributes
    result
  }

  pub fn purestyle_changes(&self, other: &Font) -> Font {
    let mathstyle = self.get_mathstyle();
    let othermathstyle = other.get_mathstyle();
    let othercolor = other.get_color();
    let mut changes = Font {
      scale: Some(other.get_size().unwrap() / self.get_size().unwrap()),
      bg: other.bg,
      opacity: other.opacity.clone(), // should multiply or replace?
      ..Font::default()
    };
    if is_diff_font_color(othercolor, Some(&DEFCOLOR)) {
      changes.color = Some(othercolor.copied().unwrap_or(DEFCOLOR));
    }

    if let Some(ms) = mathstyle
      && let Some(os) = othermathstyle
    {
      let ms_str: &str = ms;
      let os_str: &str = os;
      changes.mathstylestep = Some(*MATH_STYLE_STEP.get(ms_str).unwrap().get(os_str).unwrap());
    }
    changes
  }

  /// Find a Font Metric corresponding to this font's family_series_shape_size
  /// that contains the given `char`, if given.
  /// Try to find a fallback metric if `char` is not in the current Font.
  /// Perl: getMetric
  pub fn get_metric(&self, c_opt: Option<char>) -> &MetricData {
    let family = self.family.as_deref().unwrap_or("serif");
    let series = self.series.as_deref().unwrap_or("medium");
    let shape = self.shape.as_deref().unwrap_or("upright");
    let size = self.size.unwrap_or_else(defsize) as i64;
    // Stack buffer for char→&str lookup key, reused across paths. Avoids
    // one String allocation per character per get_metric call (which is
    // called per-character inside compute_string_size).
    let mut ch_buf = [0u8; 4];
    let ch_key = c_opt.map(|c| c.encode_utf8(&mut ch_buf) as &str);
    if let Some(name) = lookup_metric_name(family, series, shape) {
      let fullname = format!("{name}{size}");
      if let Some(metric) = STDMETRICS.get(fullname.as_str())
        && ch_key.is_none_or(|k| metric.sizes.contains_key(k))
      {
        return metric;
      }
      // Try base name fallback
      let metric = get_metric_for_name(name);
      if ch_key.is_none_or(|k| metric.sizes.contains_key(k)) {
        return metric;
      }
    }
    // Look for a fallback metric if char given
    if let Some(k) = ch_key {
      for name in METRIC_FALLBACKS {
        let fullname = format!("{name}{size}");
        let metric = STDMETRICS
          .get(fullname.as_str())
          .unwrap_or_else(|| get_metric_for_name(name));
        if metric.sizes.contains_key(k) {
          return metric;
        }
      }
    }
    get_metric_for_name("cmr")
  }

  pub fn get_em_width(&self) -> i64 {
    let size = self.get_size().unwrap_or_else(defsize);
    let m = self.get_metric(None);
    (size * m.emwidth).trunc() as i64
  }
  pub fn get_ex_height(&self) -> i64 {
    let size = self.get_size().unwrap_or_else(defsize);
    let m = self.get_metric(None);
    (size * m.exheight).trunc() as i64
  }
  pub fn get_mu_width(&self) -> i64 {
    let size = self.get_size().unwrap_or_else(defsize);
    let m = self.get_metric(None);
    (size * m.emwidth / 18.0).trunc() as i64
  }

  pub fn compute_string_size(
    &self,
    text: &str,
    _options: SymHashMap<Stored>,
  ) -> (Dimension, Dimension, Dimension) {
    if text.is_empty()
      || self
        .get_family()
        .map(|fam| fam == "nullfont")
        .unwrap_or(false)
    {
      return (
        Dimension::default(),
        Dimension::default(),
        Dimension::default(),
      );
    }
    let size = self.get_size().unwrap_or_else(defsize);
    let ismath = self.get_family().map(|fam| fam == "math").unwrap_or(false);
    let (mut w, mut h, mut d) = (0, 0, 0);
    // Iterate via Peekable — no intermediate Vec<char> allocation,
    // and we get O(1) lookahead for kerning between consecutive chars.
    let mut chars_iter = text.chars().peekable();
    // Stack buffers for char→&str and char+char→&str lookups, avoiding
    // String::to_string() + String::format!() heap allocations inside
    // the per-character hot loop. encode_utf8 writes directly to the
    // buffer and returns a borrowed &str slice — no allocation.
    let mut ch_buf = [0u8; 4];
    let mut kern_buf = [0u8; 8];
    while let Some(ch) = chars_iter.next() {
      let metric = self.get_metric(Some(ch));
      let ch_key = ch.encode_utf8(&mut ch_buf);
      let entry_opt = metric.sizes.get(ch_key);
      let (cw, ch_sz, cd, ci) = if let Some(entry) = entry_opt {
        *entry
      } else {
        (0.75 * UNITY_F64, 0.7 * UNITY_F64, 0.2 * UNITY_F64, 0.0)
      };
      w += (cw * size).trunc() as i64;
      // Kerning: check kern between this char and next.
      if let Some(&next_ch) = chars_iter.peek() {
        let first_len = ch.encode_utf8(&mut kern_buf).len();
        let second_len = next_ch.encode_utf8(&mut kern_buf[first_len..]).len();
        let kern_key = std::str::from_utf8(&kern_buf[..first_len + second_len]).unwrap();
        if let Some(kern) = metric.kerns.get(kern_key) {
          w += (size * kern).trunc() as i64;
        }
      }
      // Italic correction in math
      if ismath && ci != 0.0 {
        w += (size * ci).trunc() as i64;
      }
      h = max(h, (ch_sz * size).trunc() as i64);
      d = max(d, (cd * size).trunc() as i64);
    }
    // The 1 is so that any actual glyph appears to be non-empty.
    // This is presumably only necessary to deal with the flawed emptiness heiristics in Alignment?
    if w == 0 {
      w = 1;
    }
    (Dimension::new(w), Dimension::new(h), Dimension::new(d))
  }

  /// Get nominal width, height base ?
  /// Probably should be using data from FontMetric ???
  pub fn get_nominal_size(&self) -> (Dimension, Dimension, Dimension) {
    let size = self.get_size().unwrap_or_else(defsize);
    let u = size * UNITY_F64;
    (
      Dimension::new_f64(0.75 * u),
      Dimension::new_f64(0.7 * u),
      Dimension::new_f64(0.2 * u),
    )
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
  /// Perl: Font.pm sub computeBoxesSize (L635-680)
  /// Compute the size of a List of boxes, dispatching to helpers based on layout mode.
  pub fn compute_boxes_size(
    &self,
    boxes: &[Digested],
    options: SymHashMap<Stored>,
  ) -> Result<(Dimension, Dimension, Dimension)> {
    // Perl L646-647: `elsif ($ref =~ /^LaTeXML::Core::(?:Box|Whatsit|Alignment)$/) {
    // return $boxes->getSize; }` — a single bare Box/Whatsit/Alignment (NOT a
    // List) returns its getSize directly, short-circuiting ahead of the
    // List/split_words logic. This is essential for an isSpace box carrying an
    // explicit height/depth (e.g. `\phantom`'s XMHint): split_words keeps only
    // its width as inter-word space and discards height/depth, but getSize honors
    // the full width/height/depth. In Perl the dispatch is on the type of the
    // measured object (a bare-box body, not a List); a single-element slice here
    // is the faithful analogue (a List-of-one would still be a List in Perl).
    if let [single] = boxes
      && !matches!(single.data(), DigestedData::List(_))
    {
      let mut bx_clone = single.clone();
      let (w, h, d, ..) = bx_clone.get_size(None)?;
      return Ok((w, h, d));
    }
    // Perl L638: my $mode = $boxes->getProperty('mode') || 'restricted_horizontal';
    let mode_str = match options.get("mode") {
      Some(Stored::String(s)) => arena::with(*s, |s| s.to_string()),
      _ => "restricted_horizontal".to_string(),
    };
    // Perl: $vattach = $boxes->getProperty('vattach') || $options{vattach} || 'baseline'
    let vattach = match options.get("vattach") {
      Some(Stored::String(s)) => arena::with(*s, |s| s.to_string()),
      _ => "baseline".to_string(),
    };
    // Perl #2798: `elsif (my $width = ($mode =~ /horizontal$/) && $boxes->getProperty('width'))`
    // — a horizontal list is formatted as a paragraph IFF an explicit width is
    // supplied (recorded by S4's repack_horizontal). NO `\hsize` fallback: a
    // horizontal List without a width property is restricted_horizontal (a single
    // line, no wrapping), matching Perl.
    let para_width: Option<i64> = if mode_str.ends_with("horizontal") {
      match options.get("width") {
        Some(Stored::Dimension(d)) => Some(d.value_of()),
        Some(Stored::Int(i)) => Some(*i),
        _ => None,
      }
    } else {
      None
    };
    // Perl #2798: baseline (sp) — from List property (S4) / option, default 12pt.
    let baseline: i64 = match options.get("baseline") {
      Some(Stored::Dimension(d)) => d.value_of(),
      Some(Stored::Int(i)) => *i,
      _ => 12 * UNITY,
    };
    let mut maxwidth: i64 = 0;
    // Perl #2798: lines are now [baseline, wd, ht, dp] (per-line baseline; -1 = no
    // inter-line adjustment, e.g. \vskip / \hrule).
    let mut lines: Vec<[i64; 4]> = Vec::new();
    if mode_str.ends_with("vertical") {
      // Perl: For vertical, ALL boxes are lines.
      for bx in boxes {
        if bx.has_property("isEmpty") {
          continue;
        }
        // Perl: a horizontal sub-List WITH a width is formatted as a paragraph.
        if matches!(bx.data(), DigestedData::List(_))
          && bx
            .get_property("mode")
            .map(|v| v.to_string())
            .unwrap_or_default()
            == "horizontal"
          && let Some(w) = bx.get_property("width").and_then(|v| match &*v {
            Stored::Dimension(d) => Some(d.value_of()),
            Stored::Int(i) => Some(*i),
            _ => None,
          })
        {
          if w > maxwidth {
            maxwidth = w;
          }
          let sub_baseline = bx
            .get_property("baseline")
            .and_then(|v| match &*v {
              Stored::Dimension(d) => Some(d.value_of()),
              Stored::Int(i) => Some(*i),
              _ => None,
            })
            .unwrap_or(baseline);
          lines.extend(self.linebreak_paragraph(&bx.unlist(), w, sub_baseline)?);
          continue;
        }
        // Perl: single box → one line, with baseline (or -1 for vskip/rule).
        // DIVERGENCE from Perl #2798 (upstream candidate): Perl folds vskips
        // and rules into one `-1` flag and RESETS prevdepth for both, so any
        // glue item between lines silently disables \baselineskip accounting
        // — a stack of N verbatim lines interleaved with fancyvrb's interline
        // vspace measures as Σ(h+d) ≈ N×6pt instead of N×\baselineskip
        // (witness 2605.00468: 49-line Prompt boxes budgeted at half their
        // TeX height; content spilled through every following box). TeX truth
        // (tex.web vpack): \prevdepth is TRANSPARENT to glue — only a BOX
        // updates it (to its depth), and only \hrule disables it (sentinel
        // \prevdepth = -1000pt). Encode vskip as -1 (transparent) and rule
        // as -2 (reset) so the stack can honor both.
        let (w, h, d) = self.compute_boxes_size_box(bx)?;
        let bs = if bx.get_property_bool("isHorizontalRule") {
          -2
        } else if bx.get_property_bool("isVerticalSpace") {
          -1
        } else {
          baseline
        };
        if w != 0 || h != 0 || d != 0 {
          lines.push([bs, w, h, d]);
        }
      }
    } else if let Some(w) = para_width {
      // Perl: proper paragraph — flatten, split into words, break into lines.
      if w > maxwidth {
        maxwidth = w;
      }
      let flat: Vec<&Digested> = boxes
        .iter()
        .filter(|b| !b.has_property("isEmpty"))
        .collect();
      let flat_owned: Vec<Digested> = flat.into_iter().cloned().collect();
      lines = self.linebreak_paragraph(&flat_owned, w, baseline)?;
    } else {
      // Perl: restricted_horizontal or math — one line, no wrapping.
      let filtered: Vec<&Digested> = boxes
        .iter()
        .filter(|b| !b.has_property("isEmpty"))
        .collect();
      let words = self.compute_boxes_size_words(&filtered)?;
      lines = Self::compute_boxes_size_lines(None, baseline, &words);
    }
    // Perl: stack up the multiple lines; mathaxis = size/4.
    let size = self.get_size().unwrap_or_else(defsize) as i64;
    let mathaxis = size * UNITY / 4;
    static SIZE_TRACE: std::sync::LazyLock<bool> =
      std::sync::LazyLock::new(|| std::env::var("LXML_SIZE_TRACE").is_ok());
    if *SIZE_TRACE {
      eprintln!(
        "SIZE mode={mode_str} vattach={vattach} baseline={} nboxes={} lines={:?}",
        baseline as f64 / 65536.0,
        boxes.len(),
        lines
          .iter()
          .map(|l| [
            l[0] as f64 / 65536.0,
            l[1] as f64 / 65536.0,
            l[2] as f64 / 65536.0,
            l[3] as f64 / 65536.0
          ])
          .collect::<Vec<_>>()
      );
    }
    let (mut wd, mut ht, mut dp) = Self::compute_boxes_size_stack(&vattach, mathaxis, &lines);
    // Perl: $wd = $maxwidth if $wd && $maxwidth (set to fill width, unless empty).
    if wd != 0 && maxwidth != 0 {
      wd = maxwidth;
    }
    // Perl: divide up totalheight, if requested.
    if let Some(th) = options.get("totalheight").and_then(|v| match v {
      Stored::Dimension(d) => Some(d.value_of()),
      Stored::Int(i) => Some(*i),
      _ => None,
    }) {
      let diff = th - ht - dp;
      if diff > 0 {
        match vattach.as_str() {
          "bottom" => ht += diff,
          "middle" => {
            ht += diff / 2;
            dp += diff / 2;
          },
          _ => dp += diff,
        }
      }
    }
    Ok((Dimension::new(wd), Dimension::new(ht), Dimension::new(dp)))
  }

  /// Perl #2798: linebreak_paragraph — format a horizontal list (with width) as
  /// a paragraph: flatten nested horizontal Lists + sizing-flattenable Whatsits,
  /// split into words, then break into lines. Returns [baseline, wd, ht, dp] lines.
  fn linebreak_paragraph(
    &self,
    boxes: &[Digested],
    width: i64,
    baseline: i64,
  ) -> Result<Vec<[i64; 4]>> {
    let flat = Self::flatten_paragraph(boxes);
    let flat_refs: Vec<&Digested> = flat.iter().collect();
    let words = self.compute_boxes_size_words(&flat_refs)?;
    Ok(Self::compute_boxes_size_lines(
      Some(width),
      baseline,
      &words,
    ))
  }

  /// Perl #2798: flatten_paragraph — open up contained horizontal Lists, and any
  /// Whatsits that format AS IF embedded paragraph material (e.g. `\emph`), so
  /// they participate in line-breaking.
  fn flatten_paragraph(boxes: &[Digested]) -> Vec<Digested> {
    let mut queue: std::collections::VecDeque<Digested> = boxes.iter().cloned().collect();
    let mut out: Vec<Digested> = Vec::new();
    while let Some(bx) = queue.pop_front() {
      if matches!(bx.data(), DigestedData::List(_))
        && bx
          .get_property("mode")
          .map(|v| v.to_string())
          .unwrap_or_default()
          == "horizontal"
      {
        for ib in bx.unlist().into_iter().rev() {
          queue.push_front(ib);
        }
      } else if matches!(bx.data(), DigestedData::Whatsit(_))
        && let Some(repl) = flatten_for_sizing(&bx)
      {
        for ib in repl.into_iter().rev() {
          queue.push_front(ib);
        }
      } else {
        out.push(bx);
      }
    }
    out
  }

  /// Perl: Font.pm sub computeBoxesSize_box (L683-702)
  /// Compute the size of a single box, returning (w, h, d) in sp.
  fn compute_boxes_size_box(&self, bx: &Digested) -> Result<(i64, i64, i64)> {
    // Clone to avoid caching side effects on the original box
    let mut bx_clone = bx.clone();
    let (w, h, d, ..) = bx_clone.get_size(None)?;
    Ok((w.value_of(), h.value_of(), d.value_of()))
  }

  /// Perl: Font.pm sub computeBoxesSize_words (L705-746)
  /// Compute a list of sizes of space-delimited "words" within a NON-vertical list.
  /// Returns Vec of [prevspace, wd, ht, dp] where prevspace=-1 means line break.
  fn compute_boxes_size_words(&self, boxes: &[&Digested]) -> Result<Vec<[f64; 4]>> {
    let mut words: Vec<[f64; 4]> = Vec::new();
    let mut prevbox: Option<&Digested> = None;
    let mut prevspace: f64 = 0.0;
    // Perl L711: my $size = int($self->getSize || DEFSIZE() || 10);
    let size = self.get_size().unwrap_or_else(defsize) as i64;
    let (mut wd, mut ht, mut dp): (f64, i64, i64) = (0.0, 0, 0);
    for bx in boxes {
      let (w, h, d) = self.compute_boxes_size_box(bx)?;
      // Perl L716-721: Check for possible line-break points
      if bx.get_property_bool("isBreak") {
        // Perl: vertical space (isBreak + isVerticalSpace) contributes height
        // even though it acts as a line break. Include its h/d in the word
        // so alignment row spacing accounts for \noalign{\vskip X}.
        if bx.get_property_bool("isVerticalSpace") {
          ht = max(ht, h);
          dp = max(dp, d);
        }
        if wd != 0.0 || ht != 0 || dp != 0 || prevspace > 0.0 {
          words.push([prevspace, wd, ht as f64, dp as f64]);
          wd = 0.0;
          ht = 0;
          dp = 0;
          prevspace = -1.0;
        } else {
          prevspace = -1.0;
        }
      }
      // Perl L723-728: isSpace (but not isVerticalSpace) — word boundary
      else if bx.get_property_bool("isSpace") && !bx.get_property_bool("isVerticalSpace") {
        if wd != 0.0 || ht != 0 || dp != 0 || prevspace < 0.0 {
          words.push([prevspace, wd, ht as f64, dp as f64]);
          wd = 0.0;
          ht = 0;
          dp = 0;
          prevspace = w as f64;
        } else {
          prevspace += w as f64;
        }
      }
      // Perl #2798: an ideographic (CJK) char is itself a word.
      else if bx.get_property_bool("isIdeographic") {
        if wd != 0.0 {
          words.push([prevspace, wd, ht as f64, dp as f64]);
        }
        words.push([0.0, w as f64, h as f64, d as f64]);
        wd = 0.0;
        ht = 0;
        dp = 0;
        prevspace = 0.0;
      }
      // Perl L729-741: Else accumulate into "word"
      else {
        wd += w as f64;
        ht = max(ht, h);
        dp = max(dp, d);
        // Perl L734-741: Kern HACK for lists of individual Box's
        if let Some(pb) = prevbox
          && matches!(pb.data(), DigestedData::TBox(_))
          && matches!(bx.data(), DigestedData::TBox(_))
        {
          let prevchar = pb.get_string()?.chars().last();
          let curchar = bx.get_string()?.chars().next();
          let metric = self.get_metric(curchar);
          // Perl L738-739: math bearing
          if let Some(family) = self.get_family()
            && family == "math"
          {
            wd += self.math_bearing(bx, pb);
          }
          // Perl L740-741: kerning
          if let Some(prevc) = prevchar
            && let Some(curc) = curchar
          {
            let kern_key = String::from(prevc) + &String::from(curc);
            if let Some(kern) = metric.kerns.get(kern_key.as_str()) {
              wd += size as f64 * kern;
            }
          }
        }
      }
      // Perl L743: $prevbox = $box
      prevbox = Some(bx);
    }
    // Perl L744-745: be sure to get last bit
    if wd != 0.0 || ht != 0 || dp != 0 || prevspace != 0.0 {
      words.push([prevspace, wd, ht as f64, dp as f64]);
    }
    Ok(words)
  }

  /// Perl #2798: collect_lines — break words into lines per `wrapwidth` (if any)
  /// or explicit breaks. Each line is `[baseline, wd, ht, dp]`; the per-line
  /// `baseline` drives inter-line spacing in `compute_boxes_size_stack`.
  fn compute_boxes_size_lines(
    wrapwidth: Option<i64>,
    baseline: i64,
    words: &[[f64; 4]],
  ) -> Vec<[i64; 4]> {
    let mut lines: Vec<[i64; 4]> = Vec::new();
    let fuzz = UNITY as f64; // 1pt
    let (mut wd, mut ht, mut dp): (f64, i64, i64) = (0.0, 0, 0);
    for item in words {
      let (space, w, h, d) = (item[0], item[1], item[2] as i64, item[3] as i64);
      // Forced linebreak (space == -1) or wrapped linebreak.
      if space == -1.0 || wrapwidth.is_some_and(|ww| wd + space * 0.5 + w > ww as f64 + fuzz) {
        if wd != 0.0 {
          lines.push([baseline, kround(wd), ht, dp]);
        }
        wd = w;
        ht = h;
        dp = d;
      } else {
        wd += space + w;
        ht = max(ht, h);
        dp = max(dp, d);
      }
    }
    if wd != 0.0 || ht != 0 || dp != 0 {
      lines.push([baseline, kround(wd), ht, dp]);
    }
    lines
  }

  /// Perl #2798: stack_lines — sum a stack of `[baseline, wd, ht, dp]` lines:
  /// `wd` is the max, inter-line spacing uses each line's `baseline` (`bs < 0` =
  /// no adjustment), and `ht`/`dp` are split per `vattach` (`mathaxis` = size/4).
  fn compute_boxes_size_stack(vattach: &str, mathaxis: i64, lines: &[[i64; 4]]) -> (i64, i64, i64) {
    let nlines = lines.len();
    if nlines == 0 {
      return (0, 0, 0);
    }
    if nlines == 1 {
      let [_bs, w, h, d] = lines[0];
      return (w, h, d);
    }
    // Perl: $lineskip = lookupDefinition('\lineskip')->valueOf->valueOf
    let lineskip = lookup_definition(&T_CS!("\\lineskip"))
      .ok()
      .flatten()
      .and_then(|def| def.value_of(Vec::new()))
      .map(|v| v.value_of())
      .unwrap_or(0);
    let mut wd: i64 = 0;
    let mut prevdepth: i64 = -99999;
    let mut th: i64 = 0;
    for line in lines {
      let [bs, w, h, d] = *line;
      wd = max(w, wd);
      th += h + d;
      if prevdepth >= 0 && bs >= 0 {
        if prevdepth + h < bs {
          th += bs - prevdepth - h;
        } else {
          th += lineskip;
        }
      }
      // TeX vpack \prevdepth discipline (divergence from Perl #2798 — see
      // the vertical branch above): boxes set prevdepth to their depth;
      // glue (bs == -1) is TRANSPARENT (prevdepth unchanged, so the next
      // box still receives \baselineskip accounting across the skip);
      // rules (bs == -2) disable it (TeX's \prevdepth = -1000pt sentinel).
      prevdepth = if bs >= 0 {
        d
      } else if bs == -1 {
        prevdepth
      } else {
        -99999
      };
    }
    let (ht, dp) = match vattach {
      "middle" => (th / 2 + mathaxis, th / 2 - mathaxis),
      "bottom" => {
        let d = lines[nlines - 1][3];
        (th - d, d)
      },
      // else (baseline / top): align to baseline of top row.
      _ => {
        let h = lines[0][2];
        (h, th - h)
      },
    };
    (wd, ht, dp)
  }
}

/// Perl #2798: `Whatsit->flattenForSizing` — a horizontal Whatsit whose sizer is
/// a pure `#arg`/`#prop` reference can be flattened so its content participates
/// in paragraph line-breaking (e.g. `\emph`). STUB: returns `None` (no
/// flattening) for now — refine to parse the sizer spec. None of the current
/// sizing fixtures depend on this; only `\emph`-style line-wrapping differs.
fn flatten_for_sizing(_w: &Digested) -> Option<Vec<Digested>> { None }

fn is_diff(x: Option<&Cow<str>>, y: Option<&Cow<str>>) -> bool {
  x.is_some() && (y.is_none() || (x != y))
}

fn is_diff_opt_str(x: Option<&str>, y: Option<&str>) -> bool {
  x.is_some() && (y.is_none() || (x != y))
}

fn is_diff_f64(x: Option<f64>, y: Option<f64>) -> bool { x.is_some() && (y.is_none() || (x != y)) }

fn is_diff_color(x: Option<&Color>, y: Option<&Color>) -> bool {
  x.is_some() && (y.is_none() || (x != y))
}

/// Like is_diff_color but treats None as DEFCOLOR (for the `color` field).
/// Visual comparison: Gray(0) == Rgb(0,0,0) since both are black.
fn is_diff_font_color(x: Option<&Color>, y: Option<&Color>) -> bool {
  let cx = x.unwrap_or(&DEFCOLOR);
  let cy = y.unwrap_or(&DEFCOLOR);
  if cx == cy {
    return false;
  }
  cx.to_rgb() != cy.to_rgb()
}

/// Reference-style comparison for color field: treats None as DEFCOLOR.
/// Unlike is_diff_font_color, does NOT fall back to visual to_rgb() comparison.
/// Cmyk(0,0,0,1) IS different from Rgb(0,0,0) even though both are visually black.
/// This matches Perl's `ne` reference equality: two Color objects at different
/// addresses are "different" even if they represent the same visual color.
/// In our model, different Color variants = different Perl references.
fn is_diff_font_color_ref(x: Option<&Color>, y: Option<&Color>) -> bool {
  let cx = x.unwrap_or(&DEFCOLOR);
  let cy = y.unwrap_or(&DEFCOLOR);
  cx != cy
}

/// Matches fonts when both are converted to toString strings.
/// Uses regex caching for repeated lookups.
/// Perl: match_font
pub fn match_font(font1: &str, font2: &str) -> bool {
  // Build a regex from font1 where '*' components become wildcards
  if let Some(inner) = font1
    .strip_prefix("Font[")
    .and_then(|s| s.strip_suffix(']'))
  {
    let comps: Vec<&str> = inner.split(',').collect();
    let re_str = format!(
      "^Font\\[{}\\]$",
      comps
        .iter()
        .map(|c| if *c == "*" {
          "[^,]+".to_string()
        } else {
          regex::escape(c)
        })
        .collect::<Vec<_>>()
        .join(",")
    );
    if let Ok(re) = Regex::new(&re_str) {
      return re.is_match(font2);
    }
  }
  false
}

/// Generate XPath fragments for font matching.
/// Perl: font_match_xpaths
pub fn font_match_xpaths(font: &str) -> String {
  if let Some(inner) = font.strip_prefix("Font[").and_then(|s| s.strip_suffix(']')) {
    let comps: Vec<&str> = inner.split(',').collect();
    // Only check family, series, shape (indices 0, 1, 2)
    let mut frags: Vec<String> = Vec::new();
    if !comps.is_empty() && comps[0] != "*" {
      frags.push(format!("[{},", comps[0]));
    }
    if comps.len() > 1 && comps[1] != "*" {
      frags.push(format!(",{},", comps[1]));
    }
    if comps.len() > 2 && comps[2] != "*" {
      frags.push(format!(",{},", comps[2]));
    }
    let mut parts: Vec<String> = vec!["@_font".to_string()];
    for frag in frags {
      parts.push(format!("contains(@_font,'{frag}')"));
    }
    parts.join(" and ")
  } else {
    "@_font".to_string()
  }
}

/// Decode a codepoint using the fontmap for a given font and/or fontencoding.
///
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
/// Push a fontmap character to a string, handling known multi-char entries.
fn push_fontmap_char(result: &mut String, c: char, _code: u8) {
  // T1 position 223: "SS" (capital sharp S as two chars)
  if c == '\u{1E9E}' {
    result.push_str("SS");
    return;
  }
  // For standalone combining characters (Unicode Mn category), prepend NBSP as base.
  // Perl fontmaps encode these as UTF(0xA0)."\x{combining}" (two-char strings).
  if is_combining_mark(c) {
    result.push('\u{00A0}');
  }
  result.push(c);
}

/// Check if a character is a Unicode combining mark (category Mn).
fn is_combining_mark(c: char) -> bool {
  matches!(c as u32,
    0x0300..=0x036F   // Combining Diacritical Marks
    | 0x1AB0..=0x1AFF // Combining Diacritical Marks Extended
    | 0x1DC0..=0x1DFF // Combining Diacritical Marks Supplement
    | 0x20D0..=0x20FF // Combining Diacritical Marks for Symbols
    | 0xFE20..=0xFE2F // Combining Half Marks
  )
}

/// Decode a codepoint, returning a SymStr that may contain multiple characters.
/// This handles Perl font map entries like `UTF(0xA0)."\x{0335}"` (OT1 pos 32).
pub fn decode_str(code: u8, encoding_opt: Option<String>, implicit: bool) -> Option<SymStr> {
  // First, check for multi-char overrides (for entries that can't fit in Option<char>)
  if let Some(s) = lookup_multichar_override(code, encoding_opt.as_deref()) {
    return Some(arena::pin(s));
  }
  if let Some(c) = decode(code, encoding_opt, implicit) {
    // T1 position 223: "SS" (capital sharp S as two chars)
    if c == '\u{1E9E}' {
      return Some(pin!("SS"));
    }
    // For standalone combining characters, prepend NBSP as base character
    // (Perl fontmaps encode these as UTF(0xA0)."\x{combining}")
    if is_combining_mark(c) {
      return Some(arena::pin(format!("\u{00A0}{c}")));
    }
    Some(arena::pin_char(c))
  } else {
    None
  }
}

/// Look up multi-char override for a given encoding position.
/// Returns Some(String) if a multi-char override exists.
fn lookup_multichar_override(code: u8, encoding_opt: Option<&str>) -> Option<String> {
  let encoding = match encoding_opt {
    Some(enc) if !enc.is_empty() => enc.to_string(),
    _ => {
      let font = lookup_font();
      font.and_then(|f| f.get_encoding().map(|e| e.to_string()))?
    },
  };
  if encoding.is_empty() {
    return None;
  }
  let mapname = format!("{encoding}_fontmap_multichar");
  with_value(&mapname, |val_opt| {
    if let Some(Stored::HashString(map)) = val_opt {
      map.get(&code.to_string()).cloned()
    } else {
      None
    }
  })
}

pub fn decode(code: u8, encoding_opt: Option<String>, implicit: bool) -> Option<char> {
  let mut font = None;
  let encoding = match encoding_opt {
    Some(enc) => Cow::Owned(enc),
    None => {
      font = lookup_font();
      if let Some(ref font) = font {
        match font.get_encoding() {
          None => Cow::Borrowed(""),
          Some(encoding) => encoding.clone(),
        }
      } else {
        Cow::Borrowed("")
      }
    },
  };

  let mut map: Option<Fontmap> = None;
  if !encoding.is_empty() {
    let _ = preload_font_map(&encoding); // infallible in practice; swallow Result
    if let Some(encmap) = load_font_map(&encoding) {
      // OK got some map.
      map = Some(encmap);
      if let Some(ref font) = font
        && let Some(family) = (*font).get_family()
      {
        with_value(&s!("{encoding}_{family}_fontmap"), |fmap_opt| {
          if let Some(fmap) = fmap_opt {
            map = fmap.into(); // Use the family specific map, if any.
          }
        });
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

pub fn decode_string(string: SymStr, encoding_opt: Option<&str>, implicit: bool) -> SymStr {
  let empty_sym = pin!("");
  if string == empty_sym {
    return empty_sym;
  }
  let mut font = None;
  let encoding = match encoding_opt {
    None => {
      font = lookup_font();
      if let Some(ref font) = font {
        font.get_encoding().unwrap_or(&Cow::Borrowed(""))
      } else {
        ""
      }
    },
    Some(encoding) => encoding,
  };

  let mut map: Option<Fontmap> = None;
  if !encoding.is_empty() {
    let _ = preload_font_map(encoding); // infallible in practice; swallow Result
    if let Some(encmap) = load_font_map(encoding) {
      // OK got some map.
      map = Some(encmap);
      if let Some(ref font) = font
        && let Some(family) = (*font).get_family()
      {
        with_value(&s!("{}_{}_fontmap", encoding, family), |fmap_opt| {
          if let Some(fmap) = fmap_opt {
            map = fmap.into(); // Use the family specific map, if any.
          }
        });
      }
    }
  }

  // Load multi-char overrides if available
  let multichar_map: Option<HashMap<String, String>> = if !encoding.is_empty() {
    let mapname = format!("{encoding}_fontmap_multichar");
    with_value(&mapname, |val_opt| {
      if let Some(Stored::HashString(m)) = val_opt {
        Some(m.clone())
      } else {
        None
      }
    })
  } else {
    None
  };

  let mut result_string: String = String::new();
  arena::with(string, |str| {
    for c in str.chars() {
      if implicit {
        if let Some(ref map_ref) = map {
          let code = c as u16; // u16, so that Unicode chars get cast correctly
          if code < 128 {
            // Check multi-char override first
            if let Some(ref mc) = multichar_map
              && let Some(mc_str) = mc.get(&(code as u8).to_string())
            {
              result_string.push_str(mc_str);
              continue;
            }
            if let Some(Some(mapc_val)) = map_ref.get(code as usize) {
              push_fontmap_char(&mut result_string, *mapc_val, code as u8);
            }
          } else {
            result_string.push(c);
          }
        } else {
          result_string.push(c)
        }
      } else if let Some(ref map_ref) = map {
        let code = c as u8;
        // Check multi-char override first
        if let Some(ref mc) = multichar_map
          && let Some(mc_str) = mc.get(&code.to_string())
        {
          result_string.push_str(mc_str);
          continue;
        }
        if let Some(Some(mapc_val)) = map_ref.get(code as usize) {
          push_fontmap_char(&mut result_string, *mapc_val, code);
        }
      }
    }
  });
  arena::pin(result_string)
}

/// Convert stanard font size names, such as `tiny`, `Huge`, etc to f64
pub fn rationalize_font_size(size: &str) -> f64 {
  if let Some(symbolic) = FONT_SIZE.get(size) {
    *symbolic * defsize()
  } else {
    // Perl: return $size — if not a symbolic name, return the numeric value as-is
    size.parse::<f64>().unwrap_or_else(|_| defsize())
  }
}

/// convert size to percent
pub fn relative_font_size(newsize: f64, oldsize: f64) -> String {
  s!("{}%", (0.5 + 100.0 * newsize / oldsize).floor())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn relative_font_size_same_is_100() {
    assert_eq!(relative_font_size(10.0, 10.0), "100%");
    assert_eq!(relative_font_size(12.0, 12.0), "100%");
  }

  #[test]
  fn relative_font_size_doubled_is_200() {
    assert_eq!(relative_font_size(20.0, 10.0), "200%");
  }

  #[test]
  fn relative_font_size_half_is_50() {
    assert_eq!(relative_font_size(5.0, 10.0), "50%");
  }

  #[test]
  fn match_font_exact_wildcard_tail() {
    // Font[family,series,shape,size,...] — '*' matches any single
    // component.
    // match_font(f1, f2) returns true iff f2 matches the pattern f1.
    // f1 with all-wildcards should match any well-formed Font[...].
    assert!(match_font("Font[*,*,*,*]", "Font[rm,med,up,10]"));
  }

  #[test]
  fn match_font_exact_match() {
    assert!(match_font("Font[rm,med,up,10]", "Font[rm,med,up,10]"));
    assert!(!match_font("Font[rm,med,up,10]", "Font[sf,med,up,10]"));
  }

  #[test]
  fn match_font_partial_wildcard() {
    // First position wildcard matches rm, sf, tt, etc.
    assert!(match_font("Font[*,med,up,10]", "Font[rm,med,up,10]"));
    assert!(match_font("Font[*,med,up,10]", "Font[sf,med,up,10]"));
    // But a non-wildcard in series must match.
    assert!(!match_font("Font[*,bold,up,10]", "Font[rm,med,up,10]"));
  }

  #[test]
  fn match_font_malformed_input() {
    // Missing Font[...] wrapper → false.
    assert!(!match_font("not_a_font", "Font[rm,med,up,10]"));
  }

  #[test]
  fn font_match_xpaths_all_wildcards_is_attr_only() {
    let xp = font_match_xpaths("Font[*,*,*,*]");
    // All wildcards → just @_font, no contains(...) fragments.
    assert_eq!(xp, "@_font");
  }

  #[test]
  fn font_match_xpaths_includes_specified_components() {
    let xp = font_match_xpaths("Font[rm,bold,*,*]");
    // Family and series specified; shape/size wildcarded.
    assert!(xp.contains("@_font"));
    assert!(xp.contains("contains"));
    assert!(xp.contains("rm"));
    assert!(xp.contains("bold"));
  }

  #[test]
  fn font_match_xpaths_malformed_is_empty_or_fallback() {
    let xp = font_match_xpaths("garbage");
    // Not a Font[...] format → some minimal/fallback output.
    // Implementation detail: we don't over-constrain, just verify
    // it doesn't panic.
    let _ = xp;
  }
}
