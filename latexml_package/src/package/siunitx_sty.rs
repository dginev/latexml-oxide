//! siunitx.sty — SI units and number formatting
//! Perl: siunitx.sty.ltxml (1817 lines)
//!
//! Pragmatic port: defines key commands and all SI units as simple macros.
//! Number formatting and semantic unit markup not yet implemented.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("amsmath");

  // Ignore sisetup options
  DefMacro!("\\sisetup{}", "");

  //======================================================================
  // Key symbols
  DefMath!("\\SIUnitSymbolDegree", None, "\u{00B0}", meaning => "arcdegree");
  DefMath!("\\SIUnitSymbolArcminute", None, "\u{2032}", meaning => "arcminute");
  DefMath!("\\SIUnitSymbolArcsecond", None, "\u{2033}", meaning => "arcsecond");
  DefMath!("\\SIUnitSymbolCelsius", None, "\u{00B0}C");
  DefMath!("\\SIUnitSymbolOhm", None, "\u{2126}");
  DefMath!("\\SIUnitSymbolAngstrom", None, "\u{00C5}");
  DefMath!("\\SIUnitSymbolMicro", None, "\u{00B5}");

  //======================================================================
  // Number formatting — simplified
  // \num{number} — just pass through
  DefMacro!("\\num[]{}", "#2");
  // \numlist, \numrange
  DefMacro!("\\numlist[]{}", "#2");
  DefMacro!("\\numrange[]{}{}", "#2 to #3");
  // \ang{degrees;minutes;seconds} — simplified: just output the argument
  DefMacro!("\\ang[]{}", "#2\\SIUnitSymbolDegree{}");

  //======================================================================
  // Unit formatting — simplified
  // \si{unit} — format unit
  DefMacro!("\\si[]{}", "#2");
  // \SI{number}{unit} — number followed by unit
  DefMacro!("\\SI[]{}{}", "#2\\,#3");
  // \SIlist, \SIrange
  DefMacro!("\\SIlist[]{}{}", "#2\\,#3");
  DefMacro!("\\SIrange[]{}{}{}", "#2 to #3\\,#4");
  Let!("\\tablenum", "\\num");

  //======================================================================
  // Unit declaration primitives
  // Perl: \DeclareSIUnit OptionalKeyVals:SIX SkipSpaces DefToken {}
  // Reads: optional [...], then a CS token, then {body}
  DefPrimitive!("\\DeclareSIUnit[]", {
    let cs = gullet::read_token()?.unwrap_or(T_CS!("\\relax"));
    let body = gullet::read_arg(ExpansionLevel::Off)?;
    let body_str = body.to_string();
    let expansion_str = format!("\\text{{{body_str}}}");
    let expansion = Tokenize!(&expansion_str);
    def_macro(cs, None, expansion, None)?;
  });

  // Perl: \DeclareSIPrefix OptionalKeyVals:SIX SkipSpaces DefToken {}{}
  DefPrimitive!("\\DeclareSIPrefix[]", {
    let cs = gullet::read_token()?.unwrap_or(T_CS!("\\relax"));
    let body = gullet::read_arg(ExpansionLevel::Off)?;
    let _power = gullet::read_arg(ExpansionLevel::Off)?;
    let body_str = body.to_string();
    let expansion_str = format!("\\text{{{body_str}}}");
    let expansion = Tokenize!(&expansion_str);
    def_macro(cs, None, expansion, None)?;
  });

  // Perl: \DeclareSIQualifier OptionalKeyVals:SIX SkipSpaces DefToken {}
  DefPrimitive!("\\DeclareSIQualifier[]", {
    let cs = gullet::read_token()?.unwrap_or(T_CS!("\\relax"));
    let body = gullet::read_arg(ExpansionLevel::Off)?;
    let body_str = body.to_string();
    let expansion_str = format!("\\text{{{body_str}}}");
    let expansion = Tokenize!(&expansion_str);
    def_macro(cs, None, expansion, None)?;
  });

  // Perl: \DeclareBinaryPrefix, \DeclareSIPrePower, \DeclareSIPostPower — similar pattern
  DefPrimitive!("\\DeclareBinaryPrefix[]", {
    let _cs = gullet::read_token()?;
    let _body = gullet::read_arg(ExpansionLevel::Off)?;
    let _power = gullet::read_arg(ExpansionLevel::Off)?;
  });
  DefPrimitive!("\\DeclareSIPrePower[]", {
    let _cs = gullet::read_token()?;
    let _body = gullet::read_arg(ExpansionLevel::Off)?;
  });
  DefPrimitive!("\\DeclareSIPostPower[]", {
    let _cs = gullet::read_token()?;
    let _body = gullet::read_arg(ExpansionLevel::Off)?;
  });

  //======================================================================
  // \per — division separator in units
  DefMacro!("\\per", "/");
  // \highlight — color highlight in units
  DefMacro!("\\highlight{}", "#1");
  // \of — qualifier
  DefMacro!("\\of{}", "(#1)");
  // \tothe, \raiseto — power (use \textsuperscript for text-safe superscript)
  DefMacro!("\\tothe{}", "\\textsuperscript{#1}");
  DefMacro!("\\raiseto{}", "\\textsuperscript{#1}");

  //======================================================================
  // SI Base Units
  DefMacro!("\\ampere", r"\text{A}");
  DefMacro!("\\candela", r"\text{cd}");
  DefMacro!("\\kelvin", r"\text{K}");
  DefMacro!("\\kilogram", r"\text{kg}");
  DefMacro!("\\metre", r"\text{m}");
  DefMacro!("\\meter", r"\text{m}");
  DefMacro!("\\mole", r"\text{mol}");
  DefMacro!("\\second", r"\text{s}");

  // SI Derived Units
  DefMacro!("\\becquerel", r"\text{Bq}");
  DefMacro!("\\degreeCelsius", r"\text{{}^{\circ}C}");
  DefMacro!("\\coulomb", r"\text{C}");
  DefMacro!("\\farad", r"\text{F}");
  DefMacro!("\\gray", r"\text{Gy}");
  DefMacro!("\\hertz", r"\text{Hz}");
  DefMacro!("\\henry", r"\text{H}");
  DefMacro!("\\joule", r"\text{J}");
  DefMacro!("\\katal", r"\text{kat}");
  DefMacro!("\\lumen", r"\text{lm}");
  DefMacro!("\\lux", r"\text{lx}");
  DefMacro!("\\newton", r"\text{N}");
  DefMacro!("\\ohm", r"\text{\Omega}");
  DefMacro!("\\pascal", r"\text{Pa}");
  DefMacro!("\\radian", r"\text{rad}");
  DefMacro!("\\siemens", r"\text{S}");
  DefMacro!("\\sievert", r"\text{Sv}");
  DefMacro!("\\steradian", r"\text{sr}");
  DefMacro!("\\tesla", r"\text{T}");
  DefMacro!("\\volt", r"\text{V}");
  DefMacro!("\\watt", r"\text{W}");
  DefMacro!("\\weber", r"\text{Wb}");

  // Non-SI units accepted for use
  DefMacro!("\\day", r"\text{d}");
  DefMacro!("\\hectare", r"\text{ha}");
  DefMacro!("\\hour", r"\text{h}");
  DefMacro!("\\litre", r"\text{L}");
  DefMacro!("\\liter", r"\text{L}");
  DefMacro!("\\minute", r"\text{min}");
  DefMacro!("\\tonne", r"\text{t}");

  // Additional units
  DefMacro!("\\angstrom", r"\text{\AA}");
  DefMacro!("\\arcminute", r"\text{'}");
  DefMacro!("\\arcsecond", r"\text{''}");
  DefMacro!("\\astronomicalunit", r"\text{au}");
  DefMacro!("\\atomicmassunit", r"\text{u}");
  DefMacro!("\\barn", r"\text{b}");
  DefMacro!("\\bel", r"\text{B}");
  DefMacro!("\\bohr", r"\text{a_0}");
  DefMacro!("\\dalton", r"\text{Da}");
  DefMacro!("\\decibel", r"\text{dB}");
  DefMacro!("\\degree", r"^{\circ}");
  DefMacro!("\\electronvolt", r"\text{eV}");
  DefMacro!("\\hartree", r"\text{E_h}");
  DefMacro!("\\knot", r"\text{kn}");
  DefMacro!("\\neper", r"\text{Np}");
  DefMacro!("\\percent", r"\%");

  // SI Prefixes
  DefMacro!("\\yocto", r"\text{y}");
  DefMacro!("\\zepto", r"\text{z}");
  DefMacro!("\\atto", r"\text{a}");
  DefMacro!("\\femto", r"\text{f}");
  DefMacro!("\\pico", r"\text{p}");
  DefMacro!("\\nano", r"\text{n}");
  DefMacro!("\\micro", r"\text{\mu}");
  DefMacro!("\\milli", r"\text{m}");
  DefMacro!("\\centi", r"\text{c}");
  DefMacro!("\\deci", r"\text{d}");
  DefMacro!("\\deca", r"\text{da}");
  DefMacro!("\\hecto", r"\text{h}");
  DefMacro!("\\kilo", r"\text{k}");
  DefMacro!("\\mega", r"\text{M}");
  DefMacro!("\\giga", r"\text{G}");
  DefMacro!("\\tera", r"\text{T}");
  DefMacro!("\\peta", r"\text{P}");
  DefMacro!("\\exa", r"\text{E}");
  DefMacro!("\\zetta", r"\text{Z}");
  DefMacro!("\\yotta", r"\text{Y}");

  // Abbreviations (commonly used short forms)
  DefMacro!("\\fg", r"\text{fg}");
  DefMacro!("\\pg", r"\text{pg}");
  DefMacro!("\\ng", r"\text{ng}");
  DefMacro!("\\ug", r"\text{\mu g}");
  DefMacro!("\\mg", r"\text{mg}");
  DefMacro!("\\g", r"\text{g}");
  DefMacro!("\\kg", r"\text{kg}");
  DefMacro!("\\gram", r"\text{g}");

  // Table column types S and s — treat as centered columns
  DefColumnType!("S", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(latexml_core::alignment::cell::Cell {
        ..latexml_core::alignment::cell::Cell::default()
      })
    });
  });
  DefColumnType!("s", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(latexml_core::alignment::cell::Cell {
        ..latexml_core::alignment::cell::Cell::default()
      })
    });
  });
  DefMacro!("\\ProvidesExplFile{}{}{}{}", "");
});
