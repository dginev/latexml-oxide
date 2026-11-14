use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: SIunits.sty.ltxml
  // Apparently siunitx is a revision and extension of SIunits
  // Not quite backwards compatible, but worth a try...
  RequirePackage!("siunitx");
  // SIunits.sty:173.
  RequirePackage!("amstext");

  // Apparently similar, but expects the numbers to be already formatted?
  // (things like \times, ^, etc appear)
  Let!("\\unit", "\\SI");

  RawTeX!("\\sisetup{parse-numbers = false, input-product = \\times,}");

  DefMacro!("\\squaren{}", "{#1}^{2}");

  // SIunits.sty's own commands and units, which papers use outside `\SI`/`\unit` (`\addunit{\Pa}{Pa}`,
  // `$221\usk\meter$`, `\metre\usk\reciprocal\second`, `\degreecelsius`; witnesses 0709.1649, 2103.01586, 1904.06264,
  // 2407.13230). Each is defined with `\providecommand`, which keeps a definition a paper or another package made
  // first and replaces the `\relax` the siunitx shim leaves its unit names outside `\si` (`\si` rebinds them to its
  // own unit objects inside its argument). The unit-building commands and aliases (`\meter`, `\deka`) are SIunits.sty
  // L51-193 and 266; `\no@qsk`
  // (L117) removes the quantity skip of SIunits' own `\unit`, which is siunitx's here; `\SI@square` (L162) is
  // `\squaren` or the `\square` SIunits defines, both `\power{#1}{2}`. Beyond Perl's shim: OXIDIZED_DESIGN_DIVERGENCES #565.
  RawTeX!(r"\providecommand\SI@fstyle[1]{\mathrm{#1}}
\providecommand{\addunit}[2]{\newcommand{#1}{\ensuremath{\SI@fstyle{#2}}}}
\providecommand{\addprefix}[2]{\newcommand{#1}{\ensuremath{\SI@fstyle{#2}}}}
\def\lx@siunits@unit#1#2{\providecommand{#1}{\ensuremath{\SI@fstyle{#2}}}}
\providecommand*{\usk}{\ensuremath{\;}}
\providecommand*{\unitskip}[1]{\renewcommand{\usk}{\ensuremath{#1}}}
\providecommand*{\per}{\ensuremath{\SI@fstyle{/}}}
\providecommand{\power}[2]{\ensuremath{\SI@fstyle{#1}^{\SI@fstyle{#2}}}}
\providecommand*{\squared}{\ensuremath{^{\SI@fstyle{2}}}}
\providecommand*{\cubic}[1]{\power{#1}{3}}
\providecommand*{\cubed}{\ensuremath{^{\SI@fstyle{3}}}}
\providecommand*{\fourth}[1]{\power{#1}{4}}
\providecommand*{\SIminus}{\ensuremath{-}}
\providecommand*{\reciprocal}[1]{\power{#1}{\SIminus1}}
\providecommand*{\rp}[1]{\power{#1}{\SIminus1}}
\providecommand*{\rpsquare}[1]{\power{#1}{\SIminus2}}
\providecommand*{\rpsquared}{\ensuremath{^{\SI@fstyle{\SIminus2}}}}
\providecommand*{\rpcubic}[1]{\power{#1}{\SIminus3}}
\providecommand*{\rpcubed}{\ensuremath{^{\SI@fstyle{\SIminus3}}}}
\providecommand*{\rpfourth}[1]{\power{#1}{\SIminus4}}
\providecommand{\degreecelsius}{\ensuremath{\lx@siunits@degreecelsius}}
\providecommand*{\SImu}{\textmu}
\def\no@qsk{}
\providecommand*{\meter}{\metre}
\providecommand*{\deka}{\deca}
\providecommand*{\dekad}{\decad}
\providecommand*{\NoAMS}{\def\micro{\ensuremath{\SI@fstyle{\mbox{\SImu}}}}}
\providecommand*{\quantityskip}[1]{}
\def\SI@square#1{\power{#1}{2}}
\newif\if@redefsquare \newif\if@defsquaren \newif\if@redefGray \newif\if@defGray \newif\if@optionbinary
\newif\if@optionNoAMS
\providecommand*\SIunits[1][thickspace,thickqspace]{\@for\SIunits@@:=#1\do{\SIunits@execopt\SIunits@@}}
\providecommand*\SIunits@execopt[1]{\@ifundefined{SIunits@opt@#1}%
  {\PackageWarning{SIunits}{Requested option `#1' not provided}}%
  {\@nameuse{SIunits@opt@#1}}}
\providecommand*\SIunits@opt@cdot{\renewcommand*{\usk}{\ensuremath{\cdot}}}
\providecommand*\SIunits@opt@thickspace{\renewcommand*{\usk}{\ensuremath{\;}}}
\providecommand*\SIunits@opt@mediumspace{\renewcommand*{\usk}{\ensuremath{\:}}}
\providecommand*\SIunits@opt@thinspace{\renewcommand*{\usk}{\ensuremath{\,}}}
\providecommand*\SIunits@opt@thickqspace{}
\providecommand*\SIunits@opt@mediumqspace{}
\providecommand*\SIunits@opt@thinqspace{}");
  // `\degreecelsius` (L266, `\degree\Celsius`: a raised ring before an upright C) as the one glyph ℃, as gensymb's
  // `\celsius`, so a number before it parses as the quantity it is.
  DefMath!("\\lx@siunits@degreecelsius", None, "\u{2103}", alias => "\\degreecelsius");
  // `\DeclareMathSymbol{\upmu}{\mathord}{greek}{"16}` (L51), as upgreek's; a package that defined it first keeps it.
  if !is_defined("\\upmu") {
    DefMath!("\\upmu", '\u{03BC}', font => { shape => "upright", forceshape => true });
  }
  // The raised `{}^{\circ}`, `'` and `''` of `\degree`, `\arcminute` and `\arcsecond` (L398-402) as their glyphs, as
  // `\degreecelsius`: a number before them parses as the angle it is.
  DefMath!("\\lx@siunits@degree", None, "\u{00B0}", alias => "\\degree");
  DefMath!("\\lx@siunits@arcminute", None, "\u{2032}", alias => "\\arcminute");
  DefMath!("\\lx@siunits@arcsecond", None, "\u{2033}", alias => "\\arcsecond");
  let mut units = String::new();
  for &(name, body) in SIUNITS_UNITS {
    let body = match name {
      r"\degree" => r"\lx@siunits@degree",
      r"\arcminute" | r"\paminute" => r"\lx@siunits@arcminute",
      r"\arcsecond" | r"\pasecond" => r"\lx@siunits@arcsecond",
      _ => body,
    };
    units.push_str(&format!("\\lx@siunits@unit{name}{{{body}}}\n"));
  }
  RawTeX!(&units);
  for (option, table) in [("derivedinbase", SIUNITS_DERIVED_IN_BASE), ("derived", SIUNITS_DERIVED)] {
    let mut body = format!("\\def\\SIunits@opt@{option}{{");
    for (name, unit) in table {
      body.push_str(&format!("\\lx@siunits@unit{name}{{{unit}}}"));
    }
    body.push('}');
    RawTeX!(&body);
  }

  // SIunits.sty:545-569, the package options.
  DeclareOption!("cdot", "\\SIunits@opt@cdot");
  DeclareOption!("thickspace", "\\SIunits@opt@thickspace");
  DeclareOption!("mediumspace", "\\SIunits@opt@mediumspace");
  DeclareOption!("thinspace", "\\SIunits@opt@thinspace");
  DeclareOption!("thickqspace", "\\SIunits@opt@thickqspace");
  DeclareOption!("mediumqspace", "\\SIunits@opt@mediumqspace");
  DeclareOption!("thinqspace", "\\SIunits@opt@thinqspace");
  DeclareOption!("derivedinbase", "\\SIunits@opt@derivedinbase");
  DeclareOption!("derived", "\\SIunits@opt@derived");
  DeclareOption!("textstyle", "\\renewcommand\\SI@fstyle[1]{\\text{#1}}");
  DeclareOption!("amssymb", "\\@redefsquaretrue");
  DeclareOption!("squaren", "\\@defsquarentrue");
  DeclareOption!("pstricks", "\\@redefGraytrue");
  DeclareOption!("Gray", "\\@defGraytrue");
  DeclareOption!("italian", None);
  DeclareOption!("binary", "\\@optionbinarytrue");
  DeclareOption!("noams", "\\@optionNoAMStrue");
  ProcessOptions!();
  // L566 loads binary.sty for option `binary`; L201-209, 125-150 and 271-301 define `\micro`, `\square` and `\gray`
  // (or `\Gray`) at the document's start, where the options and the packages loaded meanwhile decide them (an
  // amssymb `\square` is kept: SIunits' error for it is not repeated).
  RawTeX!(r"\if@optionbinary\RequirePackage{binary}\fi
\AtBeginDocument{%
\if@optionNoAMS\lx@siunits@unit\micro{\mbox{\SImu}}\else\lx@siunits@unit\micro{\upmu}\fi
\if@redefsquare\def\square#1{\power{#1}{2}}\else\if@defsquaren\else
  \@ifundefined{square}{\def\square#1{\power{#1}{2}}}{}\fi\fi
\if@redefGray\def\gray{\ensuremath{\SI@fstyle{Gy}}}\else\if@defGray\def\Gray{\ensuremath{\SI@fstyle{Gy}}}\else
  \lx@siunits@unit\gray{Gy}\fi\fi}");

  // NOTE: do NOT `six_enable_unit_macros(true)` here. siunitx's unit macros include short abbreviations (`\m`, `\s`,
  // `\g`, `\A`, …) that SIunits never defines (its shortest names are `\rad`, `\ton`, `\lux`, `\bel`); enabling them
  // clobbered the extremely common user macro `\newcommand{\m}[1]{\(#1\)}` (a LaTeX `\newcommand` of an
  // already-defined CS is silently ignored), so every `\m{…}` rendered as the metre symbol in TEXT mode → 753× `^`/`_`
  // "can only appear in math mode" (witness 1509.04521, `\usepackage[squaren]{SIunits}` + `\newcommand{\m}`). The
  // table above defines SIunits' own names only.
});

/// SIunits.sty's units and prefixes: its top-level `\addunit`/`\addprefix` calls (L195-543) in file order, as
/// `(name, body)`. The begin-document `\micro`/`\gray` and the option sets below are defined apart.
const SIUNITS_UNITS: &[(&str, &str)] = &[
  (r"\yocto", r"y"),                                              // :195
  (r"\zepto", r"z"),                                              // :196
  (r"\atto", r"a"),                                               // :197
  (r"\femto", r"f"),                                              // :198
  (r"\pico", r"p"),                                               // :199
  (r"\nano", r"n"),                                               // :200
  (r"\milli", r"m"),                                              // :210
  (r"\centi", r"c"),                                              // :211
  (r"\deci", r"d"),                                               // :212
  (r"\deca", r"da"),                                              // :213
  (r"\hecto", r"h"),                                              // :214
  (r"\kilo", r"k"),                                               // :215
  (r"\mega", r"M"),                                               // :216
  (r"\giga", r"G"),                                               // :217
  (r"\tera", r"T"),                                               // :218
  (r"\peta", r"P"),                                               // :219
  (r"\exa", r"E"),                                                // :220
  (r"\zetta", r"Z"),                                              // :221
  (r"\yotta", r"Y"),                                              // :222
  (r"\gram", r"g"),                                               // :243
  (r"\metre", r"m"),                                              // :244
  (r"\kilogram", r"\kilo\gram"),                                  // :245
  (r"\second", r"s"),                                             // :246
  (r"\ampere", r"A"),                                             // :247
  (r"\kelvin", r"K"),                                             // :248
  (r"\mole", r"mol"),                                             // :249
  (r"\candela", r"cd"),                                           // :250
  (r"\radian", r"rad"),                                           // :251
  (r"\steradian", r"sr"),                                         // :252
  (r"\hertz", r"Hz"),                                             // :253
  (r"\newton", r"N"),                                             // :254
  (r"\pascal", r"Pa"),                                            // :255
  (r"\joule", r"J"),                                              // :256
  (r"\watt", r"W"),                                               // :257
  (r"\coulomb", r"C"),                                            // :258
  (r"\volt", r"V"),                                               // :259
  (r"\farad", r"F"),                                              // :260
  (r"\ohm", r"\ensuremath{\Omega}"),                              // :261
  (r"\siemens", r"S"),                                            // :262
  (r"\weber", r"Wb"),                                             // :263
  (r"\tesla", r"T"),                                              // :264
  (r"\henry", r"H"),                                              // :265
  (r"\celsius", r"\degreecelsius"),                               // :267
  (r"\lumen", r"lm"),                                             // :268
  (r"\lux", r"lx"),                                               // :269
  (r"\becquerel", r"Bq"),                                         // :270
  (r"\sievert", r"Sv"),                                           // :299
  (r"\katal", r"kat"),                                            // :300
  (r"\minute", r"min"),                                           // :395
  (r"\hour", r"h"),                                               // :396
  (r"\dday", r"d"),                                               // :397
  (r"\degree", r"\no@qsk\ensuremath{^{\circ}}"),                  // :398
  (r"\paminute", r"\no@qsk\ensuremath{'}"),                       // :399
  (r"\arcminute", r"\no@qsk\ensuremath{'}"),                      // :400
  (r"\pasecond", r"\no@qsk\ensuremath{''}"),                      // :401
  (r"\arcsecond", r"\no@qsk\ensuremath{''}"),                     // :402
  (r"\ton", r"t"),                                                // :403
  (r"\tonne", r"t"),                                              // :404
  (r"\liter", r"L"),                                              // :405
  (r"\litre", r"l"),                                              // :406
  (r"\neper", r"Np"),                                             // :407
  (r"\bel", r"B"),                                                // :408
  (r"\curie", r"Ci"),                                             // :409
  (r"\rad", r"rad"),                                              // :410
  (r"\arad", r"rd"),                                              // :411
  (r"\rem", r"rem"),                                              // :412
  (r"\roentgen", r"R"),                                           // :413
  (r"\electronvolt", r"e\kern-0.05ex\volt"),                      // :414
  (r"\atomicmass", r"u"),                                         // :415
  (r"\atomicmassunit", r"u"),                                     // :416
  (r"\dalton", r"Da"),                                            // :417
  (r"\are", r"a"),                                                // :418
  (r"\hectare", r"\hecto\are"),                                   // :419
  (r"\barn", r"b"),                                               // :420
  (r"\bbar", r"bar"),                                             // :421
  (r"\gal", r"Gal"),                                              // :422
  (r"\angstrom", r"\mbox{{\AA}}"),                                // :423
  (r"\rperminute", r"r\per\minute"),                              // :424
  (r"\rpersecond", r"r\per\second"),                              // :425
  (r"\squaremetre", r"\SI@square\metre"),                         // :426
  (r"\cubicmetre", r"\cubic\metre"),                              // :427
  (r"\graypersecond", r"\gray\per\second"),                       // :428
  (r"\graypersecondnp", r"\gray\usk\reciprocal\second"),          // :429
  (r"\metrepersquaresecond", r"\metre\per\second\squared"),       // :430
  (r"\metrepersquaresecondnp", r"\metre\usk\second\rpsquared"),   // :431
  (r"\joulepermole", r"\joule\per\mole"),                         // :432
  (r"\joulepermolenp", r"\joule\usk\reciprocal\mole"),            // :433
  (r"\molepercubicmetre", r"\mole\per\cubic\metre"),              // :434
  (r"\molepercubicmetrenp", r"\mole\usk\rpcubic\metre"),          // :435
  (r"\radianpersquaresecond", r"\radian\per\second\squared"),     // :436
  (r"\radianpersquaresecondnp", r"\radian\usk\second\rpsquared"), // :437
  (
    r"\kilogramsquaremetrepersecond",
    r"\kilogram\usk\squaremetre\per\second",
  ), // :438
  (
    r"\kilogramsquaremetrepersecondnp",
    r"\kilogram\usk\squaremetre\usk\reciprocal\second",
  ), // :439
  (r"\radianpersecond", r"\radian\per\second"),                   // :440
  (r"\radianpersecondnp", r"\radian\usk\reciprocal\second"),      // :441
  (
    r"\squaremetrepercubicmetre",
    r"\squaremetre\per\cubic\metre",
  ), // :442
  (
    r"\squaremetrepercubicmetrenp",
    r"\squaremetre\usk\rpcubic\metre",
  ), // :443
  (r"\katalpercubicmetre", r"\katal\per\cubic\metre"),            // :444
  (r"\katalpercubicmetrenp", r"\katal\usk\rpcubic\metre"),        // :445
  (r"\coulombpermol", r"\coulomb\per\mole"),                      // :446
  (r"\coulombpermolnp", r"\coulomb\usk\reciprocal\mole"),         // :447
  (r"\amperepersquaremetre", r"\ampere\per\squaremetre"),         // :448
  (r"\amperepersquaremetrenp", r"\ampere\usk\rpsquare\metre"),    // :449
  (r"\kilogrampercubicmetre", r"\kilogram\per\cubic\metre"),      // :450
  (r"\kilogrampercubicmetrenp", r"\kilogram\usk\rpcubic\metre"),  // :451
  (
    r"\squaremetrepernewtonsecond",
    r"\squaremetre\per\newton\usk\second",
  ), // :452
  (
    r"\squaremetrepernewtonsecondnp",
    r"\squaremetre\usk\reciprocal\newton\usk\reciprocal\second",
  ), // :453
  (r"\pascalsecond", r"\pascal\usk\second"),                      // :454
  (r"\coulombpercubicmetre", r"\coulomb\per\cubic\metre"),        // :455
  (r"\coulombpercubicmetrenp", r"\coulomb\usk\rpcubic\metre"),    // :456
  (r"\amperemetresecond", r"\ampere\usk\metre\usk\second"),       // :457
  (r"\voltpermetre", r"\volt\per\metre"),                         // :458
  (r"\voltpermetrenp", r"\volt\usk\reciprocal\metre"),            // :459
  (r"\coulombpersquaremetre", r"\coulomb\per\squaremetre"),       // :460
  (r"\coulombpersquaremetrenp", r"\coulomb\usk\rpsquare\metre"),  // :461
  (r"\faradpermetre", r"\farad\per\metre"),                       // :462
  (r"\faradpermetrenp", r"\farad\usk\reciprocal\metre"),          // :463
  (r"\ohmmetre", r"\ohm\usk\metre"),                              // :464
  (r"\kilowatthour", r"\kilo\watt\hour"),                         // :465
  (r"\wattpersquaremetre", r"\watt\per\squaremetre"),             // :466
  (r"\wattpersquaremetrenp", r"\watt\usk\rpsquare\metre"),        // :467
  (r"\joulepersquaremetre", r"\joule\per\squaremetre"),           // :468
  (r"\joulepersquaremetrenp", r"\joule\usk\rpsquare\metre"),      // :469
  (r"\newtonpercubicmetre", r"\newton\per\cubic\metre"),          // :470
  (r"\newtonpercubicmetrenp", r"\newton\usk\rpcubic\metre"),      // :471
  (r"\newtonperkilogram", r"\newton\per\kilogram"),               // :472
  (r"\newtonperkilogramnp", r"\newton\usk\reciprocal\kilogram"),  // :473
  (r"\jouleperkelvin", r"\joule\per\kelvin"),                     // :474
  (r"\jouleperkelvinnp", r"\joule\usk\reciprocal\kelvin"),        // :475
  (r"\jouleperkilogram", r"\joule\per\kilogram"),                 // :476
  (r"\jouleperkilogramnp", r"\joule\usk\reciprocal\kilogram"),    // :477
  (r"\coulombperkilogram", r"\coulomb\per\kilogram"),             // :478
  (
    r"\coulombperkilogramnp",
    r"\coulomb\usk\reciprocal\kilogram",
  ), // :479
  (r"\squaremetrepersecond", r"\squaremetre\per\second"),         // :480
  (
    r"\squaremetrepersecondnp",
    r"\squaremetre\usk\reciprocal\second",
  ), // :481
  (
    r"\squaremetrepersquaresecond",
    r"\squaremetre\per\second\squared",
  ), // :482
  (
    r"\squaremetrepersquaresecondnp",
    r"\squaremetre\usk\second\rpsquared",
  ), // :483
  (
    r"\kilogrammetrepersecond",
    r"\kilogram\usk\metre\per\second",
  ), // :484
  (
    r"\kilogrammetrepersecondnp",
    r"\kilogram\usk\metre\usk\reciprocal\second",
  ), // :485
  (r"\candelapersquaremetre", r"\candela\per\squaremetre"),       // :486
  (r"\candelapersquaremetrenp", r"\candela\usk\rpsquare\metre"),  // :487
  (r"\amperepermetre", r"\ampere\per\metre"),                     // :488
  (r"\amperepermetrenp", r"\ampere\usk\reciprocal\metre"),        // :489
  (r"\joulepertesla", r"\joule\per\tesla"),                       // :490
  (r"\jouleperteslanp", r"\joule\usk\reciprocal\tesla"),          // :491
  (r"\henrypermetre", r"\henry\per\metre"),                       // :492
  (r"\henrypermetrenp", r"\henry\usk\reciprocal\metre"),          // :493
  (r"\kilogrampersecond", r"\kilogram\per\second"),               // :494
  (r"\kilogrampersecondnp", r"\kilogram\usk\reciprocal\second"),  // :495
  (
    r"\kilogrampersquaremetresecond",
    r"\kilogram\per\squaremetre\usk\second",
  ), // :496
  (
    r"\kilogrampersquaremetresecondnp",
    r"\kilogram\usk\rpsquare\metre\usk\reciprocal\second",
  ), // :497
  (r"\kilogrampersquaremetre", r"\kilogram\per\squaremetre"),     // :498
  (
    r"\kilogrampersquaremetrenp",
    r"\kilogram\usk\rpsquare\metre",
  ), // :499
  (r"\kilogrampermetre", r"\kilogram\per\metre"),                 // :500
  (r"\kilogrampermetrenp", r"\kilogram\usk\reciprocal\metre"),    // :501
  (r"\joulepermolekelvin", r"\joule\per\mole\usk\kelvin"),        // :502
  (
    r"\joulepermolekelvinnp",
    r"\joule\usk\reciprocal\mole\usk\reciprocal\kelvin",
  ), // :503
  (r"\kilogramperkilomole", r"\kilogram\per\kilo\mole"),          // :504
  (
    r"\kilogramperkilomolenp",
    r"\kilogram\usk\kilo\reciprocal\mole",
  ), // :505
  (r"\kilogramsquaremetre", r"\kilogram\usk\squaremetre"),        // :506
  (r"\kilogramsquaremetrenp", r"\kilogramsquaremetre"),           // :507
  (
    r"\kilogrammetrepersquaresecond",
    r"\kilogram\usk\metre\per\second\squared",
  ), // :508
  (
    r"\kilogrammetrepersquaresecondnp",
    r"\kilogram\usk\metre\usk\second\rpsquared",
  ), // :509
  (r"\newtonpersquaremetre", r"\newton\per\squaremetre"),         // :510
  (r"\newtonpersquaremetrenp", r"\newton\usk\rpsquare\metre"),    // :511
  (r"\persquaremetresecond", r"1\per\squaremetre\usk\second"),    // :512
  (
    r"\persquaremetresecondnp",
    r"\rpsquare\metre\usk\reciprocal\second",
  ), // :513
  (r"\wattperkilogram", r"\watt\per\kilogram"),                   // :514
  (r"\wattperkilogramnp", r"\watt\usk\reciprocal\kilogram"),      // :515
  (r"\wattpercubicmetre", r"\watt\per\cubic\metre"),              // :516
  (r"\wattpercubicmetrenp", r"\watt\usk\rpcubic\metre"),          // :517
  (
    r"\wattpersquaremetresteradian",
    r"\watt\per\squaremetre\usk\steradian",
  ), // :518
  (
    r"\wattpersquaremetresteradiannp",
    r"\watt\usk\rpsquare\metre\usk\rp\steradian",
  ), // :519
  (
    r"\jouleperkilogramkelvin",
    r"\joule\per\kilogram\usk\kelvin",
  ), // :520
  (
    r"\jouleperkilogramkelvinnp",
    r"\joule\usk\reciprocal\kilogram\usk\reciprocal\kelvin",
  ), // :521
  (r"\squaremetreperkilogram", r"\squaremetre\per\kilogram"),     // :522
  (
    r"\rpsquaremetreperkilogram",
    r"\squaremetre\usk\reciprocal\kilogram",
  ), // :523
  (r"\cubicmetreperkilogram", r"\cubic\metre\per\kilogram"),      // :524
  (
    r"\rpcubicmetreperkilogram",
    r"\cubic\metre\usk\reciprocal\kilogram",
  ), // :525
  (r"\newtonpermetre", r"\newton\per\metre"),                     // :526
  (r"\newtonpermetrenp", r"\newton\usk\reciprocal\metre"),        // :527
  (r"\Celsius", r"\ensuremath{\SI@fstyle{C}}"),                   // :528
  (r"\wattpermetrekelvin", r"\watt\per\metre\usk\kelvin"),        // :529
  (
    r"\wattpermetrekelvinnp",
    r"\watt\usk\reciprocal\metre\usk\reciprocal\kelvin",
  ), // :530
  (r"\newtonmetre", r"\newton\usk\metre"),                        // :531
  (r"\newtonmetrenp", r"\newtonmetre"),                           // :531
  (
    r"\squaremetrepercubicsecond",
    r"\squaremetre\per\cubic\second",
  ), // :532
  (
    r"\squaremetrepercubicsecondnp",
    r"\squaremetre\usk\rpcubic\second",
  ), // :533
  (r"\metrepersecond", r"\metre\per\second"),                     // :534
  (r"\metrepersecondnp", r"\metre\usk\reciprocal\second"),        // :535
  (r"\joulepercubicmetre", r"\joule\per\cubicmetre"),             // :536
  (r"\joulepercubicmetrenp", r"\joule\usk\rpcubic\metre"),        // :537
  (
    r"\kilogrampercubicmetrecoulomb",
    r"\kilogram\per\cubic\metre\usk\coulomb",
  ), // :538
  (
    r"\kilogrampercubicmetrecoulombnp",
    r"\kilogram\usk\rpcubic\metre\usk\reciprocal\coulomb",
  ), // :539
  (r"\cubicmetrepersecond", r"\cubicmetre\per\second"),           // :540
  (
    r"\rpcubicmetrepersecond",
    r"\cubicmetre\usk\reciprocal\second",
  ), // :541
  (
    r"\kilogrampersecondcubicmetre",
    r"\kilogram\per\second\usk\cubicmetre",
  ), // :542
  (
    r"\kilogrampersecondcubicmetrenp",
    r"\kilogram\usk\reciprocal\second\usk\rpcubic\metre",
  ), // :543
];

/// `\SIunits@opt@derivedinbase` (L302-347): option `derivedinbase`, the derived units in base units.
const SIUNITS_DERIVED_IN_BASE: &[(&str, &str)] = &[
  (r"\radianbase", r"\metre\usk\reciprocal\metre"), // :304
  (r"\steradianbase", r"\squaremetre\usk\rpsquare\metre"), // :306
  (r"\hertzbase", r"\reciprocal\second"),           // :308
  (r"\newtonbase", r"\metre\usk\kilogram\usk\second\rpsquared"), // :310
  (
    r"\pascalbase",
    r"\reciprocal\metre\usk\kilogram\usk\second\rpsquared",
  ), // :312
  (
    r"\joulebase",
    r"\squaremetre\usk\kilogram\usk\second\rpsquared",
  ), // :314
  (
    r"\wattbase",
    r"\squaremetre\usk\kilogram\usk\rpcubic\second",
  ), // :316
  (r"\coulombbase", r"\ampere\usk\second"),         // :318
  (
    r"\voltbase",
    r"\squaremetre\usk\kilogram\usk\rpcubic\second\usk\reciprocal\ampere",
  ), // :320
  (
    r"\faradbase",
    r"\rpsquare\metre\usk\reciprocal\kilogram\usk\fourth\second\usk\ampere\squared",
  ), // :322
  (
    r"\ohmbase",
    r"\squaremetre\usk\kilogram\usk\rpcubic\second\usk\rpsquare\ampere",
  ), // :324
  (
    r"\siemensbase",
    r"\rpsquare\metre\usk\reciprocal\kilogram\usk\cubic\second\usk\ampere\squared",
  ), // :326
  (
    r"\weberbase",
    r"\squaremetre\usk\kilogram\usk\second\rpsquared\usk\reciprocal\ampere",
  ), // :328
  (
    r"\teslabase",
    r"\kilogram\usk\second\rpsquared\usk\reciprocal\ampere",
  ), // :330
  (
    r"\henrybase",
    r"\squaremetre\usk\kilogram\usk\second\rpsquared\usk\rpsquare\ampere",
  ), // :332
  (r"\celsiusbase", r"\kelvin"),                    // :334
  (
    r"\lumenbase",
    r"\candela\usk\squaremetre\usk\rpsquare\metre",
  ), // :336
  (r"\luxbase", r"\candela\usk\squaremetre\usk\rpfourth\metre"), // :338
  (r"\becquerelbase", r"\hertzbase"),               // :340
  (r"\graybase", r"\squaremetre\usk\second\rpsquared"), // :342
  (r"\sievertbase", r"\graybase"),                  // :344
  (r"\katalbase", r"\rp\second\usk\mole"),          // :346
];

/// `\SIunits@opt@derived` (L349-393): option `derived`, the derived units in derived units.
const SIUNITS_DERIVED: &[(&str, &str)] = &[
  (r"\derradian", r"\metre\usk\reciprocal\metre"), // :350
  (r"\dersteradian", r"\squaremetre\usk\rpsquare\metre"), // :352
  (r"\derhertz", r"\reciprocal\second"),           // :354
  (r"\dernewton", r"\metre\usk\kilogram\usk\second\rpsquared"), // :356
  (r"\derpascal", r"\newton\usk\rpsquare\metre"),  // :358
  (r"\derjoule", r"\newton\usk\metre"),            // :360
  (r"\derwatt", r"\joule\usk\reciprocal\second"),  // :362
  (r"\dercoulomb", r"\ampere\usk\second"),         // :364
  (r"\dervolt", r"\watt\usk\reciprocal\ampere"),   // :366
  (r"\derfarad", r"\coulomb\usk\reciprocal\volt"), // :368
  (r"\derohm", r"\volt\usk\reciprocal\ampere"),    // :370
  (r"\dersiemens", r"\ampere\usk\reciprocal\volt"), // :372
  (
    r"\derweber",
    r"\squaremetre\usk\kilogram\usk\second\rpsquared\usk\reciprocal\ampere",
  ), // :374
  (r"\dertesla", r"\weber\usk\rpsquare\metre"),    // :376
  (r"\derhenry", r"\weber\usk\reciprocal\ampere"), // :378
  (r"\dercelsius", r"\kelvin"),                    // :380
  (r"\derlumen", r"\candela\usk\steradian"),       // :382
  (r"\derlux", r"\lumen\usk\rpsquare\metre"),      // :384
  (r"\derbecquerel", r"\derhertz"),                // :386
  (r"\dergray", r"\joule\usk\reciprocal\kilogram"), // :388
  (r"\dersievert", r"\dergray"),                   // :390
  (r"\derkatal", r"\katalbase"),                   // :392
];
