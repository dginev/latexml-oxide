use std::{
  fmt,
  hash::{Hash, Hasher},
};

/// Color in a specific color model, matching Perl's LaTeXML::Common::Color hierarchy.
///
/// Core models: rgb, cmy, cmyk, hsb, gray.
/// PartialEq compares by model + components, matching Perl's Object::ne
/// behavior via toString (e.g., cmyk(0,0,0,1) ≠ rgb(0,0,0)).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Color {
  // Eq is manually implemented below since f64 doesn't derive Eq,
  // but our floats are always valid (no NaN).
  Rgb(f64, f64, f64),
  Cmy(f64, f64, f64),
  Cmyk(f64, f64, f64, f64),
  Hsb(f64, f64, f64),
  Gray(f64),
}

/// Perl: use constant Black => bless ['rgb', 0, 0, 0], '...::rgb';
pub const BLACK: Color = Color::Rgb(0.0, 0.0, 0.0);
/// Perl: use constant White => bless ['rgb', 1, 1, 1], '...::rgb';
pub const WHITE: Color = Color::Rgb(1.0, 1.0, 1.0);

// f64 doesn't implement Eq, but our color components are always valid floats (no NaN).
impl Eq for Color {}

impl Hash for Color {
  fn hash<H: Hasher>(&self, hasher: &mut H) {
    std::mem::discriminant(self).hash(hasher);
    for c in self.components() {
      ((c * 100000.0) as i64).hash(hasher);
    }
  }
}

/// Perl: toString → "model(c1,c2,...)"
impl fmt::Display for Color {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let model = self.model();
    let comps: Vec<String> = self
      .components()
      .iter()
      .map(|c| format_component(*c))
      .collect();
    write!(f, "{model}({})", comps.join(","))
  }
}

impl Color {
  /// Return the color model name. Perl: $self->model
  pub fn model(&self) -> &'static str {
    match self {
      Color::Rgb(..) => "rgb",
      Color::Cmy(..) => "cmy",
      Color::Cmyk(..) => "cmyk",
      Color::Hsb(..) => "hsb",
      Color::Gray(..) => "gray",
    }
  }

  /// Return the component values. Perl: $self->components
  pub fn components(&self) -> Vec<f64> {
    match self {
      Color::Rgb(r, g, b) => vec![*r, *g, *b],
      Color::Cmy(c, m, y) => vec![*c, *m, *y],
      Color::Cmyk(c, m, y, k) => vec![*c, *m, *y, *k],
      Color::Hsb(h, s, b) => vec![*h, *s, *b],
      Color::Gray(g) => vec![*g],
    }
  }

  /// Convert to RGB model. Perl: $self->rgb
  pub fn to_rgb(&self) -> Color {
    match self {
      Color::Rgb(..) => *self,
      Color::Cmy(c, m, y) => Color::Rgb(1.0 - c, 1.0 - m, 1.0 - y),
      Color::Cmyk(..) => self.to_cmy().to_rgb(),
      Color::Hsb(h, s, b) => {
        let i = (6.0 * h) as i32;
        let f = 6.0 * h - i as f64;
        let u = b * (1.0 - s * (1.0 - f));
        let v = b * (1.0 - s * f);
        let w = b * (1.0 - s);
        match i {
          0 => Color::Rgb(*b, u, w),
          1 => Color::Rgb(v, *b, w),
          2 => Color::Rgb(w, *b, u),
          3 => Color::Rgb(w, v, *b),
          4 => Color::Rgb(u, w, *b),
          5 => Color::Rgb(*b, w, v),
          6 => Color::Rgb(*b, w, w),
          _ => Color::Rgb(*b, w, w), // fallback
        }
      },
      Color::Gray(g) => Color::Rgb(*g, *g, *g),
    }
  }

  /// Convert to CMY model. Perl: $self->cmy
  pub fn to_cmy(&self) -> Color {
    match self {
      Color::Cmy(..) => *self,
      Color::Rgb(r, g, b) => Color::Cmy(1.0 - r, 1.0 - g, 1.0 - b),
      Color::Cmyk(c, m, y, k) => Color::Cmy((c + k).min(1.0), (m + k).min(1.0), (y + k).min(1.0)),
      Color::Hsb(..) => self.to_rgb().to_cmy(),
      Color::Gray(g) => Color::Cmy(1.0 - g, 1.0 - g, 1.0 - g),
    }
  }

  /// Convert to CMYK model. Perl: $self->cmyk
  pub fn to_cmyk(&self) -> Color {
    match self {
      Color::Cmyk(..) => *self,
      Color::Cmy(c, m, y) => {
        // Perl: undercolor-removal with beta parameters all = 1
        let k = c.min(*m).min(*y);
        Color::Cmyk(
          (c - k).clamp(0.0, 1.0),
          (m - k).clamp(0.0, 1.0),
          (y - k).clamp(0.0, 1.0),
          k,
        )
      },
      Color::Rgb(..) => self.to_cmy().to_cmyk(),
      Color::Hsb(..) => self.to_rgb().to_cmyk(),
      Color::Gray(g) => Color::Cmyk(0.0, 0.0, 0.0, 1.0 - g),
    }
  }

  /// Convert to HSB model. Perl: $self->hsb
  pub fn to_hsb(&self) -> Color {
    match self {
      Color::Hsb(..) => *self,
      Color::Rgb(r, g, b) => {
        // Perl: rgb.pm Phi function + hsb dispatch
        let i = 4 * (if *r >= *g { 1 } else { 0 })
          + 2 * (if *g >= *b { 1 } else { 0 })
          + (if *b >= *r { 1 } else { 0 });
        match i {
          1 => phi(*b, *g, *r, 3.0, 1.0),
          2 => phi(*g, *r, *b, 1.0, 1.0),
          3 => phi(*g, *b, *r, 3.0, -1.0),
          4 => phi(*r, *b, *g, 5.0, 1.0),
          5 => phi(*b, *r, *g, 5.0, -1.0),
          6 => phi(*r, *g, *b, 1.0, -1.0),
          7 => Color::Hsb(0.0, 0.0, *b),
          _ => Color::Hsb(0.0, 0.0, 0.0),
        }
      },
      Color::Cmy(..) => self.to_rgb().to_hsb(),
      Color::Cmyk(..) => self.to_cmy().to_hsb(),
      Color::Gray(g) => Color::Hsb(0.0, 0.0, *g),
    }
  }

  /// Convert to gray model. Perl: $self->gray
  pub fn to_gray(&self) -> Color {
    match self {
      Color::Gray(..) => *self,
      Color::Rgb(r, g, b) => Color::Gray(0.3 * r + 0.59 * g + 0.11 * b),
      Color::Cmy(c, m, y) => Color::Gray(1.0 - (0.3 * c + 0.59 * m + 0.11 * y)),
      Color::Cmyk(c, m, y, k) => Color::Gray(1.0 - (0.3 * c + 0.59 * m + 0.11 * y + k).min(1.0)),
      Color::Hsb(..) => self.to_rgb().to_gray(),
    }
  }

  /// Convert to another model by name. Perl: $self->convert($tomodel)
  /// Handles both core models (rgb, cmy, cmyk, hsb, gray) and
  /// extended models (HTML, RGB, Hsb, HSB, Gray, tHsb, wave).
  pub fn convert(&self, to_model: &str) -> Color {
    match to_model {
      "rgb" => self.to_rgb(),
      "cmy" => self.to_cmy(),
      "cmyk" => self.to_cmyk(),
      "hsb" => self.to_hsb(),
      "gray" => self.to_gray(),
      // Extended models map to their core equivalent
      "HTML" | "RGB" => self.to_rgb(),
      "Hsb" | "HSB" | "tHsb" => self.to_hsb(),
      "Gray" => self.to_gray(),
      _ => *self,
    }
  }

  /// Return components scaled to the target model's native range.
  /// Core models use 0-1 range. Extended models use their native ranges:
  /// HTML/RGB: 0-255, Hsb: h=0-360/s=0-1/b=0-1, HSB: 0-240, Gray: 0-15.
  /// Perl: Color objects in extended models store components in native range.
  pub fn components_for_model(&self, model: &str) -> Vec<f64> {
    let core = self.convert(model);
    let comps = core.components();
    match model {
      "HTML" | "RGB" => comps.iter().map(|c| (c * 255.0).round()).collect(),
      "Hsb" => vec![(comps[0] * 360.0).round(), comps[1], comps[2]],
      "HSB" => comps.iter().map(|c| (c * 240.0).round()).collect(),
      "Gray" => vec![(comps[0] * 15.0).round()],
      _ => comps,
    }
  }

  /// Convert to hex attribute string. Perl: $self->toAttribute() = $self->rgb->toHex()
  pub fn to_attribute(&self) -> String {
    let rgb = self.to_rgb();
    if let Color::Rgb(r, g, b) = rgb {
      format!(
        "#{:02X}{:02X}{:02X}",
        component_to_u8(r),
        component_to_u8(g),
        component_to_u8(b)
      )
    } else {
      unreachable!()
    }
  }

  /// Complement. Perl: $self->complement
  pub fn complement(&self) -> Color {
    match self {
      Color::Rgb(r, g, b) => Color::Rgb(1.0 - r, 1.0 - g, 1.0 - b),
      Color::Cmy(c, m, y) => Color::Cmy(1.0 - c, 1.0 - m, 1.0 - y),
      Color::Cmyk(..) => self.to_cmy().complement().to_cmyk(),
      Color::Hsb(h, s, b) => {
        let hp = if *h < 0.5 { h + 0.5 } else { h - 0.5 };
        let bp = 1.0 - b * (1.0 - s);
        let sp = if bp == 0.0 { 0.0 } else { b * s / bp };
        Color::Hsb(hp, sp, bp)
      },
      Color::Gray(g) => Color::Gray(1.0 - g),
    }
  }

  /// Mix self*fraction + other*(1-fraction). Perl: $self->mix($other, $fraction)
  pub fn mix(&self, other: &Color, fraction: f64) -> Color {
    let (base, other) = self.align_models(other);
    // Hsb: mix in rgb space then convert back
    if matches!(&base, Color::Hsb(..)) {
      return base.to_rgb().mix(&other, fraction).to_hsb();
    }
    let a = base.components();
    let b = other.components();
    // Allow extrapolation (fraction outside [0,1]) so callers can express
    // xcolor's `c!p` for p>100 ("darker than base"); clamp the resulting
    // components back into the model's valid [0,1] range.
    let mixed: Vec<f64> = a
      .iter()
      .zip(b.iter())
      .map(|(ai, bi)| (fraction * ai + (1.0 - fraction) * bi).clamp(0.0, 1.0))
      .collect();
    from_model_components(base.model(), &mixed)
  }

  /// Add component-wise. Perl: $self->add($other)
  pub fn add(&self, other: &Color) -> Color {
    let (base, other) = self.align_models(other);
    let a = base.components();
    let b = other.components();
    let added: Vec<f64> = a.iter().zip(b.iter()).map(|(ai, bi)| ai + bi).collect();
    from_model_components(base.model(), &added)
  }

  /// Scale all components. Perl: $self->scale($m)
  pub fn scale(&self, m: f64) -> Color {
    let scaled: Vec<f64> = self.components().iter().map(|c| m * c).collect();
    from_model_components(self.model(), &scaled)
  }

  /// Multiply by component vector. Perl: $self->multiply(@m)
  pub fn multiply(&self, factors: &[f64]) -> Color {
    let comps = self.components();
    let result: Vec<f64> = comps
      .iter()
      .zip(factors.iter())
      .map(|(c, f)| c * f)
      .collect();
    from_model_components(self.model(), &result)
  }

  /// Align two colors to the same model for operations.
  /// Perl: if base is gray, convert to other's model; else convert other to base's model.
  fn align_models(&self, other: &Color) -> (Color, Color) {
    if self.model() == other.model() {
      return (*self, *other);
    }
    if self.model() == "gray" {
      (self.convert(other.model()), *other)
    } else {
      (*self, other.convert(self.model()))
    }
  }

  /// Format the RGB components as a comma-separated string for reversion.
  /// Perl: join(',', $color->rgb->components)
  pub fn rgb_components_string(&self) -> String {
    let rgb = self.to_rgb();
    let comps = rgb.components();
    comps
      .iter()
      .map(|c| format_component(*c))
      .collect::<Vec<_>>()
      .join(",")
  }

  /// Encode for state storage: "model c1 c2 ..."
  pub fn to_stored(&self) -> String {
    let model = self.model();
    let comps: Vec<String> = self
      .components()
      .iter()
      .map(|c| format_component(*c))
      .collect();
    format!("{model} {}", comps.join(" "))
  }

  /// Decode from state storage format "model c1 c2 ..."
  pub fn from_stored(s: &str) -> Option<Color> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.is_empty() {
      return None;
    }
    let model = parts[0];
    let comps: Vec<f64> = parts[1..]
      .iter()
      .filter_map(|p| p.parse::<f64>().ok())
      .collect();
    Some(from_model_components(model, &comps))
  }
}

/// Convert color component float (0.0-1.0) to u8 (0-255).
/// Matches Perl's `roundto($n * 255, 0)` which adds a small epsilon factor.
fn component_to_u8(v: f64) -> u8 {
  let scaled = v.clamp(0.0, 1.0) * 255.0 * (1.0 + 100.0 * f64::EPSILON);
  scaled.round() as u8
}

/// Format a float component like Perl: integers without decimal point.
pub fn format_component(v: f64) -> String {
  if (v - v.round()).abs() < 1e-10 {
    format!("{}", v.round() as i64)
  } else {
    format!("{v}")
  }
}

/// Perl rgb.pm: Phi function for RGB→HSB conversion
fn phi(x: f64, y: f64, z: f64, u: f64, v: f64) -> Color {
  Color::Hsb(
    (u * (x - z) + v * (x - y)) / (6.0 * (x - z)),
    (x - z) / x,
    x,
  )
}

/// Parse color components from a spec string (comma or space separated)
fn parse_components(spec: &str) -> Vec<f64> {
  // Perl commit a8b75dbb (#2551): support mixed-delimiter input. When the spec
  // contains a comma, split on comma first, then allow whitespace splits inside
  // each component so e.g. `153 153, 192` for {RGB}{153 153, 192} yields 3 values.
  if spec.contains(',') {
    spec
      .split(',')
      .flat_map(|s| s.split_whitespace())
      .filter_map(|s| s.trim().parse::<f64>().ok())
      .collect()
  } else {
    spec
      .split_whitespace()
      .filter_map(|s| s.parse::<f64>().ok())
      .collect()
  }
}

/// Create a Color from model name and component values
pub fn from_model_components(model: &str, comps: &[f64]) -> Color {
  match model {
    "rgb" if comps.len() >= 3 => Color::Rgb(comps[0], comps[1], comps[2]),
    "cmy" if comps.len() >= 3 => Color::Cmy(comps[0], comps[1], comps[2]),
    "cmyk" if comps.len() >= 4 => Color::Cmyk(comps[0], comps[1], comps[2], comps[3]),
    "hsb" if comps.len() >= 3 => Color::Hsb(comps[0], comps[1], comps[2]),
    "gray" if !comps.is_empty() => Color::Gray(comps[0]),
    _ => BLACK,
  }
}

/// Parse a color from model name + spec string.
/// Perl: Color($model, components)->toCore
pub fn color_from_model_spec(model: &str, spec: &str) -> Color {
  let spec = spec.trim().trim_matches(|c| c == '{' || c == '}').trim();
  let c = parse_components(spec);
  from_model_components(model, &c)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn eq_close(a: f64, b: f64) -> bool { (a - b).abs() < 1e-6 }

  #[test]
  fn model_names() {
    assert_eq!(Color::Rgb(0.0, 0.0, 0.0).model(), "rgb");
    assert_eq!(Color::Cmy(0.0, 0.0, 0.0).model(), "cmy");
    assert_eq!(Color::Cmyk(0.0, 0.0, 0.0, 0.0).model(), "cmyk");
    assert_eq!(Color::Hsb(0.0, 0.0, 0.0).model(), "hsb");
    assert_eq!(Color::Gray(0.0).model(), "gray");
  }

  #[test]
  fn components_match_variant() {
    assert_eq!(Color::Rgb(0.1, 0.2, 0.3).components(), vec![0.1, 0.2, 0.3]);
    assert_eq!(Color::Cmyk(0.1, 0.2, 0.3, 0.4).components(), vec![
      0.1, 0.2, 0.3, 0.4
    ]);
    assert_eq!(Color::Gray(0.5).components(), vec![0.5]);
  }

  #[test]
  fn black_and_white_constants() {
    assert_eq!(BLACK, Color::Rgb(0.0, 0.0, 0.0));
    assert_eq!(WHITE, Color::Rgb(1.0, 1.0, 1.0));
  }

  #[test]
  fn to_rgb_idempotent() {
    let c = Color::Rgb(0.25, 0.5, 0.75);
    assert_eq!(c.to_rgb(), c);
  }

  #[test]
  fn rgb_cmy_invert_components() {
    let cmy = Color::Cmy(0.25, 0.5, 0.75);
    let rgb = cmy.to_rgb();
    if let Color::Rgb(r, g, b) = rgb {
      assert!(
        eq_close(r, 0.75) && eq_close(g, 0.5) && eq_close(b, 0.25),
        "got {rgb:?}"
      );
    } else {
      panic!("expected Rgb after cmy.to_rgb(), got {rgb:?}");
    }
  }

  #[test]
  fn complement_flips_rgb() {
    let c = Color::Rgb(0.1, 0.2, 0.3);
    let comp = c.complement();
    if let Color::Rgb(r, g, b) = comp {
      assert!(
        eq_close(r, 0.9) && eq_close(g, 0.8) && eq_close(b, 0.7),
        "got {comp:?}"
      );
    } else {
      panic!("Rgb complement should return Rgb");
    }
  }

  #[test]
  fn display_format_parenthesized() {
    let c = Color::Rgb(0.5, 0.5, 0.5);
    let s = format!("{c}");
    assert!(s.starts_with("rgb(") && s.ends_with(')'), "got {s:?}");
  }

  #[test]
  fn from_model_components_rgb() {
    let c = from_model_components("rgb", &[0.1, 0.2, 0.3]);
    assert_eq!(c, Color::Rgb(0.1, 0.2, 0.3));
  }

  #[test]
  fn from_model_components_gray_single() {
    let c = from_model_components("gray", &[0.5]);
    assert_eq!(c, Color::Gray(0.5));
  }

  #[test]
  fn from_model_components_unknown_is_black() {
    // Fallback: unknown model returns BLACK.
    let c = from_model_components("nonesuch", &[1.0, 1.0, 1.0]);
    assert_eq!(c, BLACK);
  }

  #[test]
  fn from_model_components_insufficient_comps_is_black() {
    // rgb needs 3 comps; giving 1 falls through to BLACK.
    let c = from_model_components("rgb", &[0.5]);
    assert_eq!(c, BLACK);
  }

  #[test]
  fn color_from_model_spec_parses_braced() {
    // Braces are stripped before parsing.
    let c = color_from_model_spec("rgb", "{0.1, 0.2, 0.3}");
    assert_eq!(c, Color::Rgb(0.1, 0.2, 0.3));
  }

  #[test]
  fn color_from_model_spec_parses_spaces() {
    // Spaces work as separators too.
    let c = color_from_model_spec("rgb", "0.1 0.2 0.3");
    assert_eq!(c, Color::Rgb(0.1, 0.2, 0.3));
  }

  #[test]
  fn color_inequality_across_models() {
    // rgb(0,0,0) ≠ cmyk(0,0,0,1) — different models never compare equal.
    assert_ne!(BLACK, Color::Cmyk(0.0, 0.0, 0.0, 1.0));
    assert_ne!(WHITE, Color::Gray(1.0));
  }
}
