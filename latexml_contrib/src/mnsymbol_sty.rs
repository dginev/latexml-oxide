//! MnSymbol.sty — Achim Blumensath's symbol fonts (TeX Live `tex/latex/mnsymbol/MnSymbol.sty`, v1.4 2007/01/21).
//!
//! No Perl binding exists (Perl reads the package raw, where its seven NFSS symbol fonts give no glyphs). MnSymbol
//! replaces the math symbol fonts and declares every name in them, one font slot after another, with
//! `\Decl@Mn@Char`/`\Decl@Mn@Alias` (`\DeclareMathSymbol`), the big operators with `\Decl@Mn@Op`/`\Decl@Mn@Int`, and
//! the delimiters with `\Decl@Mn@Open`/`\Decl@Mn@Close` (`\DeclareMathDelimiter`). The binding gives each declared name
//! a math symbol: the tables below hold all 1,307 `\Decl@Mn@Char`/`\Decl@Mn@Alias` names of MnSymbol.sty:250-1584 in
//! declaration order, the 55 `\Decl@Mn@Op`/`\Decl@Mn@Int` operators of :1586-1650, and the delimiters, accents and
//! commands of :1651-1853 are bound by hand below.
//!
//! **Glyphs.** A name's code point is unicode-math's for the same name (`unicode-math-table.tex`), else the Unicode
//! TeX4ht gives the name's font slot (`tex4ht/ht-fonts/unicode/MnSymbol/MnSymbol{A,B,C,D,F}-Bold10.htf`, HTML entities
//! decoded: `&gt;` is `\gtr`'s >; a slot is the running `\Set@Mn@Sym` count), else the base glyph with U+0338 for a negated `\n…` name, else a glyph chosen
//! from MnSymbol.pdf's tables, checked against its drawings (the footlines' bar on the side the name says,
//! `\rightfootline` ⟞; `\upmodels` ⫫, 2604.25507; `\ocirc` ⊚, where unicode-math's `\ocirc` is the ring accent). A glyph Unicode lacks (most of the diagonal arrows, the filled spoons, the
//! smile/frown stacks) takes the code point of its family's right-pointing member — the XMTok keeps the command's
//! name. **Roles** follow the declared class as the sibling symbol bindings do: `\mathrel` an ARROW for an
//! arrow/harpoon/spoon/mapsto family, else RELOP; `\mathbin` ADDOP for the plus/minus/cup/cap/vee/wedge/triangle/
//! diamond families (math_common's `\cup`, `\vee`, `\diamond`), else MULOP; `\mathord` none (ID for the dots);
//! `\mathop` SUMOP or INTOP.
//!
//! **Names already defined keep their definition** (OXIDIZED_DESIGN_DIVERGENCES #545). MnSymbol redeclares the whole
//! AMS set (`\Decl@Mn@Sym` first `\let`s the name undefined); here a name the kernel, amsmath or amssymb already
//! defines keeps that definition, whose role and meaning are richer than the slot's class, and a name sharing an
//! already defined name's slot and class is `\let` to it (`\nin` to `\notin`, `\rightvdash` to `\vdash`). So
//! `\Mn@Fake@Missing` (:29-78) has nothing left to fake, and its undefining of `\eth`, `\mho`, `\Finv`… is not
//! carried out: a document that uses them keeps them. The package's own declarators stay available to a document
//! under the same rule (2410.20575 `\Decl@Mn@Open{\lsem}{MNlargesymbols}{'102}` after loading MnSymbol); a new name
//! takes the glyph the tables give its slot (`slot_glyph`), and a slot they do not map goes to the kernel's
//! `\DeclareMathSymbol` with an `Info:unmapped`.
//!
//! Witnesses (run 340: 13,470 papers load MnSymbol, 3,724 ended in Error, their first errors `\nin` 175, `\udots`
//! 173, `\sumint` 165, `\cupdot` 130, `\strokedint` 113, `\lsem` 108): 2410.20575, 2210.15345 (`\let\vec\undefined`
//! before MnSymbol, whose `\DeclareMathAccent\vec` defines it again), 2508.12496 (`\checkmark`), 2609.07645
//! (`\llangle`), 2604.25507 (`\upmodels`), atableau (`\bigcircle`), biblatex-apa6-test (`\rcurvearrowse`),
//! univie-ling (`\blacktriangleright`).
use latexml_package::prelude::*;

/// One `\Decl@Mn@Char`/`\Decl@Mn@Alias` name: the command, its glyph, its role (`""` for none), the names declared
/// in the same font slot with the same math class, and the slot itself (symbol font, position).
type MnChar = (
  &'static str,
  &'static str,
  &'static str,
  &'static [&'static str],
  &'static str,
  u8,
);
/// A fixed-size big operator of the `symbols` font (`\tsum`, `\dsumint`): the command, its glyph, its role, its
/// math style (`text` for the `\t…` forms, `display` for the `\d…` ones), and its slot.
type MnSizedOp = (&'static str, &'static str, &'static str, &'static str, u8);
/// A `\Decl@Mn@Op`/`\Decl@Mn@Int` operator: the command, its glyph, its role, its meaning (`""` for none).
type MnOp = (&'static str, &'static str, &'static str, &'static str);
/// A `\Decl@Mn@Open`/`\Decl@Mn@Close`/`\Decl@Mn@Delim` delimiter of the `largesymbols` font: the symbol, its glyph,
/// its role, and its slot.
type MnDelim = (&'static str, &'static str, &'static str, u8);

/// Every name the tables declare (the `\Decl@Mn@Char`/`\Decl@Mn@Alias` names, the sized and the variable-size
/// operators): the checklist of `perfect_kernel_batch65l::mnsymbol_declares_every_name`.
pub fn declared_names() -> impl Iterator<Item = &'static str> {
  MN_CHARS
    .iter()
    .map(|row| row.0)
    .chain(MN_SIZED_OPS.iter().map(|row| row.0))
    .chain(MN_OPS.iter().map(|row| row.0))
}

/// latex.ltx's `\@ifundefined` sense: no meaning, or `\relax`.
fn unclaimed(cs: &str) -> bool { !is_defined(cs) }

fn role_of(role: &str) -> Option<String> { (!role.is_empty()).then(|| role.to_string()) }

/// Bind one `\Decl@Mn@Char`: an already defined name keeps its definition; a name whose slot-and-class sibling is
/// defined becomes that sibling (`\let`); any other is a math symbol of its glyph and role.
fn declare_char(&(cs, glyph, role, same_as, ..): &MnChar) -> Result<()> {
  if !unclaimed(cs) {
    return Ok(());
  }
  if let Some(sibling) = same_as.iter().find(|name| !unclaimed(name)) {
    let_i(&T_CS!(cs), &T_CS!(*sibling), None);
    return Ok(());
  }
  def_math(T_CS!(cs), None, glyph.to_string(), MathPrimitiveOptions {
    role: role_of(role),
    ..Default::default()
  })
}

/// The math class a table role stands for (the inverse of the module comment's role rule).
fn class_of_role(role: &str) -> &'static str {
  match role {
    "ARROW" | "RELOP" => "\\mathrel",
    "ADDOP" | "MULOP" => "\\mathbin",
    "SUMOP" | "INTOP" => "\\mathop",
    "PUNCT" => "\\mathpunct",
    "OPEN" => "\\mathopen",
    "CLOSE" => "\\mathclose",
    _ => "\\mathord",
  }
}

/// The role of a document-declared name: the slot's own role when the declared class is the slot's, else the
/// class's role as `\DeclareMathSymbol` gives it (latex_constructs `\DeclareMathSymbol`).
fn declared_role(class: &str, slot_role: &str) -> Option<String> {
  if slot_role == "VERTBAR" || class == class_of_role(slot_role) {
    return role_of(slot_role);
  }
  match class {
    "\\mathord" | "0" | "\\mathalpha" | "7" => Some("ID"),
    "\\mathop" | "1" => Some("BIGOP"),
    "\\mathbin" | "2" => Some("BINOP"),
    "\\mathrel" | "3" => Some("RELOP"),
    "\\mathopen" | "4" => Some("OPEN"),
    "\\mathclose" | "5" => Some("CLOSE"),
    "\\mathpunct" | "6" => Some("PUNCT"),
    _ => None,
  }
  .map(str::to_string)
}

/// The glyph and role the tables give a slot of MnSymbol's symbol fonts (`MnSyA`…`MnSyD`, `symbols`,
/// `largesymbols`), for a name a document declares with the package's own declarators.
fn slot_glyph(font: &str, slot: u8) -> Option<(&'static str, &'static str)> {
  match font {
    "symbols" => MN_SIZED_OPS
      .iter()
      .find(|row| row.4 == slot)
      .map(|row| (row.1, row.2)),
    "largesymbols" => MN_DELIMS
      .iter()
      .find(|row| row.3 == slot)
      .map(|row| (row.1, row.2)),
    _ => MN_CHARS
      .iter()
      .find(|row| row.4 == font && row.5 == slot)
      .map(|row| (row.1, row.2)),
  }
}

/// Info for a slot the tables do not map: the name is then declared by the kernel's `\DeclareMathSymbol`, whose
/// glyph for an unknown font is the slot's code point.
fn report_unmapped(cs: &str, font: &str, slot: impl std::fmt::Display) {
  Info!(
    "unmapped",
    "MnSymbol.sty",
    s!(
      "{cs}: MnSymbol has no Unicode glyph for slot {slot} of the symbol font {font}; declared as a generic symbol."
    )
  );
}

/// `\cmd{arg}…{slot}`: the kernel declaration for a slot the tables do not map (`cmd` empty for a bare argument list).
fn kernel_declaration(cmd: &str, args: &[&Tokens], slot: i64) -> Tokens {
  let mut toks = Vec::new();
  if !cmd.is_empty() {
    toks.push(T_CS!(cmd));
  }
  for arg in args {
    toks.push(T_BEGIN!());
    toks.extend((*arg).clone().unlist());
    toks.push(T_END!());
  }
  toks.push(T_BEGIN!());
  toks.extend(Explode!(slot.to_string()));
  toks.push(T_END!());
  Tokens::new(toks)
}

LoadDefinitions!({
  // MnSymbol.sty:30-31 `\Mn@Fake@Missing` loads textcomp and eufrak, :83 the package amsmath.
  RequirePackage!("textcomp");
  RequirePackage!("eufrak");
  RequirePackage!("amsmath");
  // The AMS names MnSymbol redeclares are amssymb's; their definitions stand (see the module comment). Guard:
  // `perfect_kernel_batch56::font_symbol_packages_carry_amssymb`.
  RequirePackage!("amssymb");
  // :14-28, :79-82: `mnsy`/`cmsy`/`abx` choose the `\mathcal` font, `retainmissing` skips `\Mn@Fake@Missing`; the
  // calligraphic alphabet is the kernel's either way.
  for option in ["mnsy", "cmsy", "abx", "retainmissing"] {
    DeclareOption!(option, None);
  }
  ProcessOptions!();
  // MnSymbol.sty never sets `\slimits@`: amsmath's `\displaylimits` stands, so a `\Decl@Mn@Op` operator takes its
  // limits above and below in display math.
  // A native binding, not a stub: an unconditional Warn would mark every MnSymbol paper (cf. OXIDIZED_DESIGN_DIVERGENCES
  // #62, biblatex).
  Info!(
    "bindings",
    "MnSymbol.sty",
    "MnSymbol.sty is provided by a native binding, not interpreted raw."
  );

  // :250-1584 every `\Decl@Mn@Char`/`\Decl@Mn@Alias` name.
  for row in MN_CHARS {
    declare_char(row)?;
  }
  // :1446-1583 the `symbols` font: text- and display-size big operators.
  for &(cs, glyph, role, style, _) in MN_SIZED_OPS {
    if unclaimed(cs) {
      def_math(T_CS!(cs), None, glyph.to_string(), MathPrimitiveOptions {
        role: role_of(role),
        mathstyle: Some(style.to_string()),
        dynamic_scriptpos: role == "SUMOP",
        ..Default::default()
      })?;
    }
  }
  // :1586-1650 `\Decl@Mn@Int\sumint\dsumint\tsumint` (`\DOTSI\tsumint\ilimits@`) and `\Decl@Mn@Op\bigplus…`
  // (`\DOTSB\tbigplus\slimits@`): variable-size operators, as math_common's `\int` and `\sum`.
  for &(cs, glyph, role, meaning) in MN_OPS {
    if unclaimed(cs) {
      def_math(T_CS!(cs), None, glyph.to_string(), MathPrimitiveOptions {
        role: role_of(role),
        meaning: role_of(meaning),
        dynamic_scriptpos: role == "SUMOP",
        variablesize_op: true,
        ..Default::default()
      })?;
    }
  }
  // :1628 `\let\biguplus\bigcupplus` — math_common's `\biguplus` stands.

  // :1651-1710 delimiters (`\Decl@Mn@Open`/`\Decl@Mn@Close`): fences after `\left`/`\right` as well (a math symbol is
  // a delimiter to `scan_delimiter`, OXIDIZED_DESIGN_DIVERGENCES #530). The names the kernel and amsmath define keep
  // their definitions.
  for &(cs, glyph, role, _) in MN_DELIMS {
    if cs.starts_with('\\') && unclaimed(cs) {
      def_math(T_CS!(cs), None, glyph.to_string(), MathPrimitiveOptions {
        role: role_of(role),
        stretchy: Some(false),
        ..Default::default()
      })?;
    }
  }
  // :1674-1675, :1680 `\mvert` (`\let\mid\mvert`) and `\mVert`: the `\mathrel` bars, as math_common's `\mid`.
  if unclaimed("\\mvert") {
    DefMath!("\\mvert", None, "\u{2223}", role => "VERTBAR");
  }
  if unclaimed("\\mVert") {
    DefMath!("\\mVert", None, "\u{2225}", role => "VERTBAR");
  }
  // :1711-1727 the pieces of the horizontal braces: `\downbrace…` form `\overbrace` (⏞), `\upbrace…` `\underbrace`.
  for (cs, glyph) in [
    ("\\downbrace", "\u{23DE}"),
    ("\\downbraceg", "\u{23DE}"),
    ("\\downbracegg", "\u{23DE}"),
    ("\\downbraceggg", "\u{23DE}"),
    ("\\downbracegggg", "\u{23DE}"),
    ("\\upbrace", "\u{23DF}"),
    ("\\upbraceg", "\u{23DF}"),
    ("\\upbracegg", "\u{23DF}"),
    ("\\upbraceggg", "\u{23DF}"),
    ("\\upbracegggg", "\u{23DF}"),
    ("\\braceld", "\u{23DE}"),
    ("\\bracelu", "\u{23DF}"),
    ("\\bracerd", "\u{23DE}"),
    ("\\braceru", "\u{23DF}"),
    ("\\bracemd", "\u{23DE}"),
    ("\\bracemu", "\u{23DF}"),
    ("\\bracemid", "\u{23AF}"),
  ] {
    if unclaimed(cs) {
      def_math(
        T_CS!(cs),
        None,
        glyph.to_string(),
        MathPrimitiveOptions::default(),
      )?;
    }
  }
  // :1822-1827 the wide over- and under-accents, as mathabx's `\overgroup`/`\undergroup`.
  if unclaimed("\\overgroup") {
    DefMath!("\\overgroup{}", "\u{23DC}", operator_role => "OVERACCENT", operator_stretchy => true);
  }
  if unclaimed("\\undergroup") {
    DefMath!("\\undergroup{}", "\u{23DD}", operator_role => "UNDERACCENT", operator_stretchy => true);
  }
  if unclaimed("\\overlinesegment") {
    DefMath!("\\overlinesegment{}", "\u{20E9}", operator_role => "OVERACCENT", operator_stretchy => true);
  }
  if unclaimed("\\underlinesegment") {
    DefMath!("\\underlinesegment{}", "\u{032A}", operator_role => "UNDERACCENT", operator_stretchy => true);
  }
  if unclaimed("\\overleftharpoon") {
    DefMath!("\\overleftharpoon{}", "\u{21BC}", operator_role => "OVERACCENT", operator_stretchy => true);
  }
  if unclaimed("\\overrightharpoon") {
    DefMath!("\\overrightharpoon{}", "\u{21C0}", operator_role => "OVERACCENT", operator_stretchy => true);
  }
  // :1845-1851 `\DeclareMathAccent`: `\widehat`, `\widetilde` and `\vec` as math_common has them (a document may have
  // undefined them so that MnSymbol could declare them, 2210.15345 `\let\vec\undefined`), `\wideparen` as mathabx's;
  // `\middlebar`, `\middleslash` and `\strokethrough` overlay their argument (MnSymbol's `\hbar` is
  // `\middlebar h`), set here as accents of the overlay character.
  if unclaimed("\\widehat") {
    DefMath!("\\widehat Digested", "\u{005E}", operator_role => "OVERACCENT");
  }
  if unclaimed("\\widetilde") {
    DefMath!("\\widetilde Digested", "\u{007E}", operator_role => "OVERACCENT");
  }
  if unclaimed("\\vec") {
    DefMath!("\\vec Digested", "\u{2192}", operator_role => "OVERACCENT", operator_stretchy => false);
  }
  if unclaimed("\\wideparen") {
    DefMath!("\\wideparen Digested", "\u{23DC}", operator_role => "OVERACCENT");
  }
  if unclaimed("\\middlebar") {
    DefMath!("\\middlebar Digested", "\u{0335}", operator_role => "OVERACCENT");
  }
  if unclaimed("\\middleslash") {
    DefMath!("\\middleslash Digested", "\u{0337}", operator_role => "OVERACCENT");
  }
  if unclaimed("\\strokethrough") {
    DefMath!("\\strokethrough Digested", "\u{0338}", operator_role => "OVERACCENT");
  }
  // :555-558 the spoons with an empty and a filled end (`\leftspoon\mkern-14mu\rightfilledspoon`): one relation, the
  // double-ended multimap.
  for cs in [
    "\\emptyfilledspoon",
    "\\filledemptyspoon",
    "\\largeemptyfilledspoon",
    "\\largefilledemptyspoon",
  ] {
    if unclaimed(cs) {
      def_math(
        T_CS!(cs),
        None,
        "\u{29DF}".to_string(),
        MathPrimitiveOptions {
          role: Some("ARROW".to_string()),
          ..Default::default()
        },
      )?;
    }
  }
  // :1445 `\coloneq` = `\mathrel{{\vdotdot}{\equal}}`, as mathabx's.
  if unclaimed("\\coloneq") {
    DefMath!("\\coloneq", None, "\u{2254}", role => "RELOP");
  }

  // :85-107 the package's own declarators, for a document that declares more names with them after loading MnSymbol
  // (2410.20575). Under the binding's rule a name already defined keeps its definition: `\Mn@ifundefined` stands for
  // the `\let#1\undefined` with which the declarators clear it.
  DefMacro!("\\Mn@ifundefined Token {}{}", sub[(cs, yes, no)] {
    Ok(if is_defined_token(&cs) { no } else { yes })
  });
  // A name a document declares gets the glyph the tables give its slot (OXIDIZED_DESIGN_DIVERGENCES #545); a slot
  // they do not map is declared by the kernel, with an Info.
  DefPrimitive!("\\Mn@DeclareSymbol{}{}{}{Number}", sub[(cs, class, font, slot)] {
    let (class_name, font_name, slot) = (class.to_string(), font.to_string(), slot.value_of());
    let Some(&cs_token) = cs.clone().unlist().first() else {
      return Ok(Vec::new());
    };
    let found = u8::try_from(slot).ok().and_then(|slot| slot_glyph(font_name.trim(), slot));
    if let Some((glyph, slot_role)) = found {
      def_math(cs_token, None, glyph.to_string(), MathPrimitiveOptions {
        role: declared_role(class_name.trim(), slot_role),
        ..Default::default()
      })?;
    } else {
      report_unmapped(&cs_token.to_string(), font_name.trim(), slot);
      unread(kernel_declaration("\\DeclareMathSymbol", &[&cs, &class, &font], slot));
    }
    Ok(Vec::new())
  });
  DefPrimitive!("\\Mn@DeclareDelimiter{}{}{}{Number}", sub[(sym, class, font, slot)] {
    let (class_name, font_name, slot) = (class.to_string(), font.to_string(), slot.value_of());
    let toks = sym.clone().unlist();
    let found = u8::try_from(slot).ok().and_then(|slot| slot_glyph(font_name.trim(), slot));
    match (toks.as_slice(), found) {
      ([cs_token], Some((glyph, slot_role))) if cs_token.get_catcode() == Catcode::CS => {
        def_math(*cs_token, None, glyph.to_string(), MathPrimitiveOptions {
          role: declared_role(class_name.trim(), slot_role),
          stretchy: Some(false),
          ..Default::default()
        })?;
      },
      _ => {
        if found.is_none() {
          report_unmapped(&sym.to_string(), font_name.trim(), slot);
        }
        // `\DeclareMathDelimiter{sym}{class}{font}{slot}{font}{slot}`.
        let mut toks = kernel_declaration("\\DeclareMathDelimiter", &[&sym, &class, &font], slot).unlist();
        toks.extend(kernel_declaration("", &[&font], slot).unlist());
        unread(Tokens::new(toks));
      },
    }
    Ok(Vec::new())
  });
  RawTeX!(
    r"\def\Set@Mn@Sym#1{\@tempcnta #1\relax}
\def\Next@Mn@Sym{\advance\@tempcnta 1\relax}
\def\Prev@Mn@Sym{\advance\@tempcnta-1\relax}
\def\@Decl@Mn@Sym#1#2#3#4{\Mn@DeclareSymbol{#2}{#3}{#4}{#1}}
\def\Decl@Mn@Sym#1#2#3{\Mn@ifundefined#1{\expandafter\@Decl@Mn@Sym\expandafter{\the\@tempcnta}{#1}{#3}{#2}}{}\Next@Mn@Sym}
\def\Decl@Mn@Alias#1#2#3{\Prev@Mn@Sym\Decl@Mn@Sym{#1}{#2}{#3}}
\let\Decl@Mn@Char\Decl@Mn@Sym
\def\Decl@Mn@Op#1#2#3{\Mn@ifundefined#1{\def#1{\DOTSB#3\slimits@}}{}}
\def\Decl@Mn@Int#1#2#3{\Mn@ifundefined#1{\def#1{\DOTSI#3\ilimits@}}{}}
\def\Decl@Mn@Delim#1#2#3#4{\Mn@ifundefined#1{\Mn@DeclareDelimiter{#1}{#2}{#3}{#4}}{}}
\def\Decl@Mn@Open#1#2#3{\Decl@Mn@Delim{#1}{\mathopen}{#2}{#3}}
\def\Decl@Mn@Close#1#2#3{\Decl@Mn@Delim{#1}{\mathclose}{#2}{#3}}"
  );
});

// MnSymbol.sty:1651-1710 the `largesymbols` delimiters, one row per slot (octal in the comments) and the names the
// binding declares. The glyphs Unicode lacks: `\ullcorner` (⌜ over ⌞) and `\ulrcorner` take the upper corners,
// `\lwavy`/`\rwavy` the vertical wavy line, `\lWavy`/`\rWavy` the vertical zigzag, `\langlebar`/`\ranglebar` the
// Z notation binding brackets (an angle with a bar); `\llangle`/`\rrangle` are set as `\langle` is (2609.07645).
#[rustfmt::skip]
static MN_DELIMS: &[MnDelim] = &[
  ("[", "[", "OPEN", 0),                       // '000
  ("]", "]", "CLOSE", 5),                      // '005
  ("\\lfloor", "\u{230A}", "OPEN", 10),        // '012
  ("\\rfloor", "\u{230B}", "CLOSE", 15),       // '017
  ("\\lceil", "\u{2308}", "OPEN", 20),         // '024
  ("\\rceil", "\u{2309}", "CLOSE", 25),        // '031
  ("\\ulcorner", "\u{231C}", "OPEN", 30),      // '036
  ("\\urcorner", "\u{231D}", "CLOSE", 35),     // '043
  ("\\llcorner", "\u{231E}", "OPEN", 40),      // '050
  ("\\lrcorner", "\u{231F}", "CLOSE", 45),     // '055
  ("\\ullcorner", "\u{231C}", "OPEN", 50),     // '062
  ("\\ulrcorner", "\u{231D}", "CLOSE", 55),    // '067
  ("\\lsem", "\u{27E6}", "OPEN", 66),          // '102
  ("\\rsem", "\u{27E7}", "CLOSE", 71),         // '107
  ("\\arrowvert", "\u{23D0}", "VERTBAR", 82),  // '122
  ("|", "|", "VERTBAR", 83),                   // '123 also \vert, \mvert, \lvert, \rvert
  ("\\Arrowvert", "\u{2016}", "VERTBAR", 88),  // '130
  ("\\Vert", "\u{2016}", "VERTBAR", 89),       // '131 also \mVert, \lVert, \rVert
  ("\\lwavy", "\u{2307}", "OPEN", 94),         // '136
  ("\\rwavy", "\u{2307}", "CLOSE", 94),
  ("\\lWavy", "\u{299A}", "OPEN", 95),         // '137
  ("\\rWavy", "\u{299A}", "CLOSE", 95),
  ("\\langle", "\u{27E8}", "OPEN", 96),        // '140 also <
  ("\\rangle", "\u{27E9}", "CLOSE", 101),      // '145 also >
  ("\\langlebar", "\u{2989}", "OPEN", 106),    // '152
  ("\\ranglebar", "\u{298A}", "CLOSE", 111),   // '157
  ("\\llangle", "\u{27EA}", "OPEN", 116),      // '164
  ("\\rrangle", "\u{27EB}", "CLOSE", 121),     // '171
  ("/", "/", "", 126),                         // '176
  ("\\backslash", "\\", "", 131),              // '203
  ("(", "(", "OPEN", 136),                     // '210
  (")", ")", "CLOSE", 141),                    // '215
  ("\\lbrace", "{", "OPEN", 152),              // '230
  ("\\rbrace", "}", "CLOSE", 157),             // '235
  ("\\lgroup", "\u{27EE}", "OPEN", 162),       // '242
  ("\\rgroup", "\u{27EF}", "CLOSE", 163),      // '243
  ("\\rmoustache", "\u{23B1}", "CLOSE", 164),  // '244
  ("\\lmoustache", "\u{23B0}", "OPEN", 165),   // '245
  ("\\bracevert", "\u{23AA}", "VERTBAR", 168), // '250
];

// The tables, derived from MnSymbol.sty, unicode-math-table.tex and TeX4ht's MnSymbol .htf files as the module comment
// describes; rows in MnSymbol.sty's declaration order.
#[rustfmt::skip]
static MN_CHARS: &[MnChar] = &[
  ("\\rightarrow", "\u{2192}", "ARROW", &["\\to"], "MnSyA", 0),
  ("\\to", "\u{2192}", "ARROW", &["\\rightarrow"], "MnSyA", 0),
  ("\\uparrow", "\u{2191}", "ARROW", &[], "MnSyA", 1),
  ("\\leftarrow", "\u{2190}", "ARROW", &["\\gets"], "MnSyA", 2),
  ("\\gets", "\u{2190}", "ARROW", &["\\leftarrow"], "MnSyA", 2),
  ("\\downarrow", "\u{2193}", "ARROW", &[], "MnSyA", 3),
  ("\\nearrow", "\u{2197}", "ARROW", &[], "MnSyA", 4),
  ("\\nwarrow", "\u{2196}", "ARROW", &[], "MnSyA", 5),
  ("\\swarrow", "\u{2199}", "ARROW", &[], "MnSyA", 6),
  ("\\searrow", "\u{2198}", "ARROW", &[], "MnSyA", 7),
  ("\\Rightarrow", "\u{21D2}", "ARROW", &[], "MnSyA", 8),
  ("\\Uparrow", "\u{21D1}", "ARROW", &[], "MnSyA", 9),
  ("\\Leftarrow", "\u{21D0}", "ARROW", &[], "MnSyA", 10),
  ("\\Downarrow", "\u{21D3}", "ARROW", &[], "MnSyA", 11),
  ("\\Nearrow", "\u{21D7}", "ARROW", &[], "MnSyA", 12),
  ("\\Nwarrow", "\u{21D6}", "ARROW", &[], "MnSyA", 13),
  ("\\Swarrow", "\u{21D9}", "ARROW", &[], "MnSyA", 14),
  ("\\Searrow", "\u{21D8}", "ARROW", &[], "MnSyA", 15),
  ("\\leftrightarrow", "\u{2194}", "ARROW", &[], "MnSyA", 16),
  ("\\updownarrow", "\u{2195}", "ARROW", &[], "MnSyA", 17),
  ("\\neswarrow", "\u{2922}", "ARROW", &[], "MnSyA", 18),
  ("\\nwsearrow", "\u{2921}", "ARROW", &[], "MnSyA", 19),
  ("\\Leftrightarrow", "\u{21D4}", "ARROW", &[], "MnSyA", 20),
  ("\\Updownarrow", "\u{21D5}", "ARROW", &[], "MnSyA", 21),
  ("\\Neswarrow", "\u{2922}", "ARROW", &[], "MnSyA", 22),
  ("\\Nwsearrow", "\u{2921}", "ARROW", &[], "MnSyA", 23),
  ("\\twoheadrightarrow", "\u{21A0}", "ARROW", &[], "MnSyA", 24),
  ("\\twoheaduparrow", "\u{219F}", "ARROW", &[], "MnSyA", 25),
  ("\\twoheadleftarrow", "\u{219E}", "ARROW", &[], "MnSyA", 26),
  ("\\twoheaddownarrow", "\u{21A1}", "ARROW", &[], "MnSyA", 27),
  ("\\twoheadnearrow", "\u{21A0}", "ARROW", &[], "MnSyA", 28),
  ("\\twoheadnwarrow", "\u{21A0}", "ARROW", &[], "MnSyA", 29),
  ("\\twoheadswarrow", "\u{21A0}", "ARROW", &[], "MnSyA", 30),
  ("\\twoheadsearrow", "\u{21A0}", "ARROW", &[], "MnSyA", 31),
  ("\\rightarrowtail", "\u{21A3}", "ARROW", &[], "MnSyA", 32),
  ("\\uparrowtail", "\u{21A3}", "ARROW", &[], "MnSyA", 33),
  ("\\leftarrowtail", "\u{21A2}", "ARROW", &[], "MnSyA", 34),
  ("\\downarrowtail", "\u{21A3}", "ARROW", &[], "MnSyA", 35),
  ("\\nearrowtail", "\u{21A3}", "ARROW", &[], "MnSyA", 36),
  ("\\nwarrowtail", "\u{21A3}", "ARROW", &[], "MnSyA", 37),
  ("\\swarrowtail", "\u{21A3}", "ARROW", &[], "MnSyA", 38),
  ("\\searrowtail", "\u{21A3}", "ARROW", &[], "MnSyA", 39),
  ("\\rightmapsto", "\u{21A6}", "ARROW", &["\\mapsto"], "MnSyA", 40),
  ("\\mapsto", "\u{21A6}", "ARROW", &["\\rightmapsto"], "MnSyA", 40),
  ("\\upmapsto", "\u{21A5}", "ARROW", &[], "MnSyA", 41),
  ("\\leftmapsto", "\u{21A4}", "ARROW", &[], "MnSyA", 42),
  ("\\downmapsto", "\u{21A7}", "ARROW", &[], "MnSyA", 43),
  ("\\nemapsto", "\u{21A6}", "ARROW", &[], "MnSyA", 44),
  ("\\nwmapsto", "\u{21A6}", "ARROW", &[], "MnSyA", 45),
  ("\\swmapsto", "\u{21A6}", "ARROW", &[], "MnSyA", 46),
  ("\\semapsto", "\u{21A6}", "ARROW", &[], "MnSyA", 47),
  ("\\lhookrightarrow", "\u{21AA}", "ARROW", &["\\hookrightarrow"], "MnSyA", 48),
  ("\\hookrightarrow", "\u{21AA}", "ARROW", &["\\lhookrightarrow"], "MnSyA", 48),
  ("\\lhookuparrow", "\u{21AA}", "ARROW", &[], "MnSyA", 49),
  ("\\lhookleftarrow", "\u{21A9}", "ARROW", &[], "MnSyA", 50),
  ("\\lhookdownarrow", "\u{21AA}", "ARROW", &[], "MnSyA", 51),
  ("\\lhooknearrow", "\u{21AA}", "ARROW", &[], "MnSyA", 52),
  ("\\lhooknwarrow", "\u{2923}", "ARROW", &[], "MnSyA", 53),
  ("\\lhookswarrow", "\u{21AA}", "ARROW", &[], "MnSyA", 54),
  ("\\lhooksearrow", "\u{2925}", "ARROW", &[], "MnSyA", 55),
  ("\\rhookrightarrow", "\u{21AA}", "ARROW", &[], "MnSyA", 56),
  ("\\rhookuparrow", "\u{21AA}", "ARROW", &[], "MnSyA", 57),
  ("\\rhookleftarrow", "\u{21A9}", "ARROW", &["\\hookleftarrow"], "MnSyA", 58),
  ("\\hookleftarrow", "\u{21A9}", "ARROW", &["\\rhookleftarrow"], "MnSyA", 58),
  ("\\rhookdownarrow", "\u{21AA}", "ARROW", &[], "MnSyA", 59),
  ("\\rhooknearrow", "\u{2924}", "ARROW", &[], "MnSyA", 60),
  ("\\rhooknwarrow", "\u{21AA}", "ARROW", &[], "MnSyA", 61),
  ("\\rhookswarrow", "\u{2926}", "ARROW", &[], "MnSyA", 62),
  ("\\rhooksearrow", "\u{21AA}", "ARROW", &[], "MnSyA", 63),
  ("\\rightharpoonccw", "\u{21C0}", "ARROW", &["\\rightharpoonup"], "MnSyA", 64),
  ("\\rightharpoonup", "\u{21C0}", "ARROW", &["\\rightharpoonccw"], "MnSyA", 64),
  ("\\upharpoonccw", "\u{21BF}", "ARROW", &["\\upharpoonleft"], "MnSyA", 65),
  ("\\upharpoonleft", "\u{21BF}", "ARROW", &["\\upharpoonccw"], "MnSyA", 65),
  ("\\leftharpoonccw", "\u{21BD}", "ARROW", &["\\leftharpoondown"], "MnSyA", 66),
  ("\\leftharpoondown", "\u{21BD}", "ARROW", &["\\leftharpoonccw"], "MnSyA", 66),
  ("\\downharpoonccw", "\u{21C2}", "ARROW", &["\\downharpoonright"], "MnSyA", 67),
  ("\\downharpoonright", "\u{21C2}", "ARROW", &["\\downharpoonccw"], "MnSyA", 67),
  ("\\neharpoonccw", "\u{21C0}", "ARROW", &[], "MnSyA", 68),
  ("\\nwharpoonccw", "\u{21C0}", "ARROW", &[], "MnSyA", 69),
  ("\\swharpoonccw", "\u{21C0}", "ARROW", &[], "MnSyA", 70),
  ("\\seharpoonccw", "\u{21C0}", "ARROW", &[], "MnSyA", 71),
  ("\\rightharpooncw", "\u{21C1}", "ARROW", &["\\rightharpoondown"], "MnSyA", 72),
  ("\\rightharpoondown", "\u{21C1}", "ARROW", &["\\rightharpooncw"], "MnSyA", 72),
  ("\\upharpooncw", "\u{21BE}", "ARROW", &["\\upharpoonright", "\\restriction"], "MnSyA", 73),
  ("\\upharpoonright", "\u{21BE}", "ARROW", &["\\upharpooncw", "\\restriction"], "MnSyA", 73),
  ("\\restriction", "\u{21BE}", "RELOP", &["\\upharpooncw", "\\upharpoonright"], "MnSyA", 73),
  ("\\leftharpooncw", "\u{21BC}", "ARROW", &["\\leftharpoonup"], "MnSyA", 74),
  ("\\leftharpoonup", "\u{21BC}", "ARROW", &["\\leftharpooncw"], "MnSyA", 74),
  ("\\downharpooncw", "\u{21C3}", "ARROW", &["\\downharpoonleft"], "MnSyA", 75),
  ("\\downharpoonleft", "\u{21C3}", "ARROW", &["\\downharpooncw"], "MnSyA", 75),
  ("\\neharpooncw", "\u{21C1}", "ARROW", &[], "MnSyA", 76),
  ("\\nwharpooncw", "\u{21C1}", "ARROW", &[], "MnSyA", 77),
  ("\\swharpooncw", "\u{21C1}", "ARROW", &[], "MnSyA", 78),
  ("\\seharpooncw", "\u{21C1}", "ARROW", &[], "MnSyA", 79),
  ("\\leftrightharpoonupdown", "\u{294A}", "ARROW", &[], "MnSyA", 80),
  ("\\updownharpoonleftright", "\u{294D}", "ARROW", &[], "MnSyA", 81),
  ("\\neswharpoonnwse", "\u{294B}", "ARROW", &[], "MnSyA", 82),
  ("\\nwseharpoonnesw", "\u{294B}", "ARROW", &[], "MnSyA", 83),
  ("\\leftrightharpoondownup", "\u{294B}", "ARROW", &[], "MnSyA", 84),
  ("\\updownharpoonrightleft", "\u{294C}", "ARROW", &[], "MnSyA", 85),
  ("\\neswharpoonsenw", "\u{294A}", "ARROW", &[], "MnSyA", 86),
  ("\\nwseharpoonswne", "\u{294A}", "ARROW", &[], "MnSyA", 87),
  ("\\rightleftharpoons", "\u{21CC}", "ARROW", &[], "MnSyA", 88),
  ("\\updownharpoons", "\u{296E}", "ARROW", &[], "MnSyA", 89),
  ("\\neswharpoons", "\u{21CB}", "ARROW", &[], "MnSyA", 90),
  ("\\senwharpoons", "\u{21CB}", "ARROW", &[], "MnSyA", 91),
  ("\\leftrightharpoons", "\u{21CB}", "ARROW", &[], "MnSyA", 92),
  ("\\downupharpoons", "\u{296F}", "ARROW", &[], "MnSyA", 93),
  ("\\swneharpoons", "\u{21CB}", "ARROW", &[], "MnSyA", 94),
  ("\\nwseharpoons", "\u{21CB}", "ARROW", &[], "MnSyA", 95),
  ("\\dashedrightarrow", "\u{21E2}", "ARROW", &["\\dashrightarrow", "\\dasharrow"], "MnSyA", 96),
  ("\\dashrightarrow", "\u{21E2}", "ARROW", &["\\dashedrightarrow", "\\dasharrow"], "MnSyA", 96),
  ("\\dasharrow", "\u{21E2}", "ARROW", &["\\dashedrightarrow", "\\dashrightarrow"], "MnSyA", 96),
  ("\\dasheduparrow", "\u{21E1}", "ARROW", &[], "MnSyA", 97),
  ("\\dashedleftarrow", "\u{21E0}", "ARROW", &["\\dashleftarrow"], "MnSyA", 98),
  ("\\dashleftarrow", "\u{21E0}", "ARROW", &["\\dashedleftarrow"], "MnSyA", 98),
  ("\\dasheddownarrow", "\u{21E3}", "ARROW", &[], "MnSyA", 99),
  ("\\dashednearrow", "\u{21E2}", "ARROW", &[], "MnSyA", 100),
  ("\\dashednwarrow", "\u{21E2}", "ARROW", &[], "MnSyA", 101),
  ("\\dashedswarrow", "\u{21E2}", "ARROW", &[], "MnSyA", 102),
  ("\\dashedsearrow", "\u{21E2}", "ARROW", &[], "MnSyA", 103),
  ("\\rightspoon", "\u{22B8}", "ARROW", &["\\multimap"], "MnSyA", 104),
  ("\\multimap", "\u{22B8}", "ARROW", &["\\rightspoon"], "MnSyA", 104),
  ("\\upspoon", "\u{2AEF}", "ARROW", &[], "MnSyA", 105),
  ("\\leftspoon", "\u{27DC}", "ARROW", &[], "MnSyA", 106),
  ("\\downspoon", "\u{2AF0}", "ARROW", &[], "MnSyA", 107),
  ("\\nespoon", "\u{22B8}", "ARROW", &[], "MnSyA", 108),
  ("\\nwspoon", "\u{22B8}", "ARROW", &[], "MnSyA", 109),
  ("\\swspoon", "\u{22B8}", "ARROW", &[], "MnSyA", 110),
  ("\\sespoon", "\u{22B8}", "ARROW", &[], "MnSyA", 111),
  ("\\rightfilledspoon", "\u{22B8}", "ARROW", &[], "MnSyA", 112),
  ("\\upfilledspoon", "\u{2AEF}", "ARROW", &[], "MnSyA", 113),
  ("\\leftfilledspoon", "\u{27DC}", "ARROW", &[], "MnSyA", 114),
  ("\\downfilledspoon", "\u{2AF0}", "ARROW", &[], "MnSyA", 115),
  ("\\nefilledspoon", "\u{22B8}", "ARROW", &[], "MnSyA", 116),
  ("\\nwfilledspoon", "\u{22B8}", "ARROW", &[], "MnSyA", 117),
  ("\\swfilledspoon", "\u{22B8}", "ARROW", &[], "MnSyA", 118),
  ("\\sefilledspoon", "\u{22B8}", "ARROW", &[], "MnSyA", 119),
  ("\\rightfootline", "\u{27DE}", "RELOP", &[], "MnSyA", 120),
  ("\\upfootline", "\u{27D9}", "RELOP", &[], "MnSyA", 121),
  ("\\leftfootline", "\u{27DD}", "RELOP", &[], "MnSyA", 122),
  ("\\downfootline", "\u{27D8}", "RELOP", &[], "MnSyA", 123),
  ("\\nefootline", "\u{27DE}", "RELOP", &[], "MnSyA", 124),
  ("\\nwfootline", "\u{27DE}", "RELOP", &[], "MnSyA", 125),
  ("\\swfootline", "\u{27DE}", "RELOP", &[], "MnSyA", 126),
  ("\\sefootline", "\u{27DE}", "RELOP", &[], "MnSyA", 127),
  ("\\rightfree", "\u{2ADD}", "RELOP", &[], "MnSyA", 128),
  ("\\upfree", "\u{2ADD}", "RELOP", &[], "MnSyA", 129),
  ("\\leftfree", "\u{2ADD}", "RELOP", &[], "MnSyA", 130),
  ("\\downfree", "\u{2ADD}", "RELOP", &[], "MnSyA", 131),
  ("\\nefree", "\u{2ADD}", "RELOP", &[], "MnSyA", 132),
  ("\\nwfree", "\u{2ADD}", "RELOP", &[], "MnSyA", 133),
  ("\\swfree", "\u{2ADD}", "RELOP", &[], "MnSyA", 134),
  ("\\sefree", "\u{2ADD}", "RELOP", &[], "MnSyA", 135),
  ("\\rightpitchfork", "\u{22D4}", "RELOP", &[], "MnSyA", 136),
  ("\\uppitchfork", "\u{22D4}", "RELOP", &["\\pitchfork"], "MnSyA", 137),
  ("\\pitchfork", "\u{22D4}", "RELOP", &["\\uppitchfork"], "MnSyA", 137),
  ("\\leftpitchfork", "\u{22D4}", "RELOP", &[], "MnSyA", 138),
  ("\\downpitchfork", "\u{2ADB}", "RELOP", &[], "MnSyA", 139),
  ("\\nepitchfork", "\u{22D4}", "RELOP", &[], "MnSyA", 140),
  ("\\nwpitchfork", "\u{22D4}", "RELOP", &[], "MnSyA", 141),
  ("\\swpitchfork", "\u{22D4}", "RELOP", &[], "MnSyA", 142),
  ("\\sepitchfork", "\u{22D4}", "RELOP", &[], "MnSyA", 143),
  ("\\rightrightarrows", "\u{21C9}", "ARROW", &[], "MnSyA", 144),
  ("\\upuparrows", "\u{21C8}", "ARROW", &[], "MnSyA", 145),
  ("\\leftleftarrows", "\u{21C7}", "ARROW", &[], "MnSyA", 146),
  ("\\downdownarrows", "\u{21CA}", "ARROW", &[], "MnSyA", 147),
  ("\\nenearrows", "\u{21C9}", "ARROW", &[], "MnSyA", 148),
  ("\\nwnwarrows", "\u{21C7}", "ARROW", &[], "MnSyA", 149),
  ("\\swswarrows", "\u{21C7}", "ARROW", &[], "MnSyA", 150),
  ("\\sesearrows", "\u{21C9}", "ARROW", &[], "MnSyA", 151),
  ("\\rightleftarrows", "\u{21C4}", "ARROW", &[], "MnSyA", 152),
  ("\\updownarrows", "\u{21C5}", "ARROW", &[], "MnSyA", 153),
  ("\\neswarrows", "\u{21C6}", "ARROW", &[], "MnSyA", 154),
  ("\\nwsearrows", "\u{21C6}", "ARROW", &[], "MnSyA", 155),
  ("\\leftrightarrows", "\u{21C6}", "ARROW", &[], "MnSyA", 156),
  ("\\downuparrows", "\u{21F5}", "ARROW", &[], "MnSyA", 157),
  ("\\swnearrows", "\u{21C6}", "ARROW", &[], "MnSyA", 158),
  ("\\senwarrows", "\u{21C6}", "ARROW", &[], "MnSyA", 159),
  ("\\rightlsquigarrow", "\u{219D}", "ARROW", &["\\leadsto", "\\rightsquigarrow"], "MnSyA", 160),
  ("\\leadsto", "\u{219D}", "ARROW", &["\\rightlsquigarrow", "\\rightsquigarrow"], "MnSyA", 160),
  ("\\rightsquigarrow", "\u{21DD}", "ARROW", &["\\rightlsquigarrow", "\\leadsto"], "MnSyA", 160),
  ("\\uplsquigarrow", "\u{219D}", "ARROW", &[], "MnSyA", 161),
  ("\\leftlsquigarrow", "\u{219C}", "ARROW", &[], "MnSyA", 162),
  ("\\downlsquigarrow", "\u{219D}", "ARROW", &[], "MnSyA", 163),
  ("\\nelsquigarrow", "\u{219D}", "ARROW", &[], "MnSyA", 164),
  ("\\nwlsquigarrow", "\u{219D}", "ARROW", &[], "MnSyA", 165),
  ("\\swlsquigarrow", "\u{219D}", "ARROW", &[], "MnSyA", 166),
  ("\\selsquigarrow", "\u{219D}", "ARROW", &[], "MnSyA", 167),
  ("\\rightrsquigarrow", "\u{219D}", "ARROW", &[], "MnSyA", 168),
  ("\\uprsquigarrow", "\u{219D}", "ARROW", &[], "MnSyA", 169),
  ("\\leftrsquigarrow", "\u{219C}", "ARROW", &[], "MnSyA", 170),
  ("\\downrsquigarrow", "\u{219D}", "ARROW", &[], "MnSyA", 171),
  ("\\nersquigarrow", "\u{219D}", "ARROW", &[], "MnSyA", 172),
  ("\\nwrsquigarrow", "\u{219D}", "ARROW", &[], "MnSyA", 173),
  ("\\swrsquigarrow", "\u{219D}", "ARROW", &[], "MnSyA", 174),
  ("\\sersquigarrow", "\u{219D}", "ARROW", &[], "MnSyA", 175),
  ("\\squigarrowleftright", "\u{21AD}", "ARROW", &["\\leftrightsquigarrow"], "MnSyA", 176),
  ("\\leftrightsquigarrow", "\u{21AD}", "ARROW", &["\\squigarrowleftright"], "MnSyA", 176),
  ("\\squigarrowupdown", "\u{21AD}", "ARROW", &[], "MnSyA", 177),
  ("\\squigarrowrightleft", "\u{21AD}", "ARROW", &[], "MnSyA", 178),
  ("\\squigarrowdownup", "\u{21AD}", "ARROW", &[], "MnSyA", 179),
  ("\\squigarrownesw", "\u{21AD}", "ARROW", &[], "MnSyA", 180),
  ("\\squigarrownwse", "\u{21AD}", "ARROW", &[], "MnSyA", 181),
  ("\\squigarrowswne", "\u{21AD}", "ARROW", &[], "MnSyA", 182),
  ("\\squigarrowsenw", "\u{21AD}", "ARROW", &[], "MnSyA", 183),
  ("\\lcurvearrowright", "\u{21B7}", "ARROW", &["\\curvearrowright"], "MnSyA", 184),
  ("\\curvearrowright", "\u{21B7}", "ARROW", &["\\lcurvearrowright"], "MnSyA", 184),
  ("\\lcurvearrowup", "\u{21B7}", "ARROW", &[], "MnSyA", 185),
  ("\\lcurvearrowleft", "\u{21B7}", "ARROW", &[], "MnSyA", 186),
  ("\\lcurvearrowdown", "\u{2938}", "ARROW", &[], "MnSyA", 187),
  ("\\lcurvearrowne", "\u{21B7}", "ARROW", &[], "MnSyA", 188),
  ("\\lcurvearrownw", "\u{21B7}", "ARROW", &[], "MnSyA", 189),
  ("\\lcurvearrowsw", "\u{21B7}", "ARROW", &[], "MnSyA", 190),
  ("\\lcurvearrowse", "\u{21B7}", "ARROW", &[], "MnSyA", 191),
  ("\\rcurvearrowright", "\u{293B}", "ARROW", &[], "MnSyA", 192),
  ("\\rcurvearrowup", "\u{293B}", "ARROW", &[], "MnSyA", 193),
  ("\\rcurvearrowleft", "\u{21B6}", "ARROW", &["\\curvearrowleft"], "MnSyA", 194),
  ("\\curvearrowleft", "\u{21B6}", "ARROW", &["\\rcurvearrowleft"], "MnSyA", 194),
  ("\\rcurvearrowdown", "\u{2939}", "ARROW", &[], "MnSyA", 195),
  ("\\rcurvearrowne", "\u{293B}", "ARROW", &[], "MnSyA", 196),
  ("\\rcurvearrownw", "\u{293B}", "ARROW", &[], "MnSyA", 197),
  ("\\rcurvearrowsw", "\u{293B}", "ARROW", &[], "MnSyA", 198),
  ("\\rcurvearrowse", "\u{293B}", "ARROW", &[], "MnSyA", 199),
  ("\\curvearrowleftright", "\u{21B7}", "ARROW", &[], "MnSyA", 200),
  ("\\curvearrowupdown", "\u{21B7}", "ARROW", &[], "MnSyA", 201),
  ("\\curvearrowrightleft", "\u{21B7}", "ARROW", &[], "MnSyA", 202),
  ("\\curvearrowdownup", "\u{21B7}", "ARROW", &[], "MnSyA", 203),
  ("\\curvearrownesw", "\u{21B7}", "ARROW", &[], "MnSyA", 204),
  ("\\curvearrownwse", "\u{21B7}", "ARROW", &[], "MnSyA", 205),
  ("\\curvearrowswne", "\u{21B7}", "ARROW", &[], "MnSyA", 206),
  ("\\curvearrowsenw", "\u{21B7}", "ARROW", &[], "MnSyA", 207),
  ("\\leftrightline", "\u{2212}", "RELOP", &["\\relbar"], "MnSyA", 208),
  ("\\relbar", "\u{2212}", "RELOP", &["\\leftrightline"], "MnSyA", 208),
  ("\\updownline", "\u{2223}", "RELOP", &["\\divides"], "MnSyA", 209),
  ("\\divides", "\u{2223}", "RELOP", &["\\updownline"], "MnSyA", 209),
  ("\\neswline", "\u{27CB}", "RELOP", &["\\diagup"], "MnSyA", 210),
  ("\\diagup", "\u{27CB}", "RELOP", &["\\neswline"], "MnSyA", 210),
  ("\\nwseline", "\u{27CD}", "RELOP", &["\\diagdown"], "MnSyA", 211),
  ("\\diagdown", "\u{27CD}", "RELOP", &["\\nwseline"], "MnSyA", 211),
  ("\\Leftrightline", "=", "RELOP", &["\\Relbar"], "MnSyA", 212),
  ("\\Relbar", "=", "RELOP", &["\\Leftrightline"], "MnSyA", 212),
  ("\\Updownline", "\u{2225}", "RELOP", &["\\parallel"], "MnSyA", 213),
  ("\\parallel", "\u{2225}", "RELOP", &["\\Updownline"], "MnSyA", 213),
  ("\\Neswline", "\u{2AFD}", "RELOP", &[], "MnSyA", 214),
  ("\\Nwseline", "\u{29F5}\u{29F5}", "RELOP", &[], "MnSyA", 215),
  ("\\rightvdash", "\u{22A2}", "RELOP", &["\\vdash"], "MnSyA", 216),
  ("\\vdash", "\u{22A2}", "RELOP", &["\\rightvdash"], "MnSyA", 216),
  ("\\upvdash", "\u{22A5}", "RELOP", &["\\perp"], "MnSyA", 217),
  ("\\perp", "\u{27C2}", "RELOP", &["\\upvdash"], "MnSyA", 217),
  ("\\leftvdash", "\u{22A3}", "RELOP", &["\\dashv"], "MnSyA", 218),
  ("\\dashv", "\u{22A3}", "RELOP", &["\\leftvdash"], "MnSyA", 218),
  ("\\downvdash", "\u{22A4}", "RELOP", &[], "MnSyA", 219),
  ("\\nevdash", "\u{22A2}", "RELOP", &[], "MnSyA", 220),
  ("\\nwvdash", "\u{22A2}", "RELOP", &[], "MnSyA", 221),
  ("\\swvdash", "\u{22A2}", "RELOP", &[], "MnSyA", 222),
  ("\\sevdash", "\u{22A2}", "RELOP", &[], "MnSyA", 223),
  ("\\rightmodels", "\u{22A7}", "RELOP", &["\\models", "\\vDash"], "MnSyA", 224),
  ("\\models", "\u{22A7}", "RELOP", &["\\rightmodels", "\\vDash"], "MnSyA", 224),
  ("\\vDash", "\u{22A8}", "RELOP", &["\\rightmodels", "\\models"], "MnSyA", 224),
  ("\\upmodels", "\u{2AEB}", "RELOP", &[], "MnSyA", 225),
  ("\\leftmodels", "\u{2AE4}", "RELOP", &[], "MnSyA", 226),
  ("\\downmodels", "\u{2AEA}", "RELOP", &[], "MnSyA", 227),
  ("\\nemodels", "\u{22A7}", "RELOP", &[], "MnSyA", 228),
  ("\\nwmodels", "\u{22A7}", "RELOP", &[], "MnSyA", 229),
  ("\\swmodels", "\u{22A7}", "RELOP", &[], "MnSyA", 230),
  ("\\semodels", "\u{22A7}", "RELOP", &[], "MnSyA", 231),
  ("\\rightVdash", "\u{22A9}", "RELOP", &["\\Vdash"], "MnSyA", 232),
  ("\\Vdash", "\u{22A9}", "RELOP", &["\\rightVdash"], "MnSyA", 232),
  ("\\upVdash", "\u{234A}", "RELOP", &[], "MnSyA", 233),
  ("\\leftVdash", "\u{2AE3}", "RELOP", &[], "MnSyA", 234),
  ("\\downVdash", "\u{2351}", "RELOP", &[], "MnSyA", 235),
  ("\\neVdash", "\u{22A9}", "RELOP", &[], "MnSyA", 236),
  ("\\nwVdash", "\u{22A9}", "RELOP", &[], "MnSyA", 237),
  ("\\swVdash", "\u{22A9}", "RELOP", &[], "MnSyA", 238),
  ("\\seVdash", "\u{22A9}", "RELOP", &[], "MnSyA", 239),
  ("\\rightModels", "\u{22AB}", "RELOP", &["\\VDash"], "MnSyA", 240),
  ("\\VDash", "\u{22AB}", "RELOP", &["\\rightModels"], "MnSyA", 240),
  ("\\upModels", "\u{22AB}", "RELOP", &[], "MnSyA", 241),
  ("\\leftModels", "\u{2AE5}", "RELOP", &[], "MnSyA", 242),
  ("\\downModels", "\u{22AB}", "RELOP", &[], "MnSyA", 243),
  ("\\neModels", "\u{22AB}", "RELOP", &[], "MnSyA", 244),
  ("\\nwModels", "\u{22AB}", "RELOP", &[], "MnSyA", 245),
  ("\\swModels", "\u{22AB}", "RELOP", &[], "MnSyA", 246),
  ("\\seModels", "\u{22AB}", "RELOP", &[], "MnSyA", 247),
  ("\\rcirclearrowright", "\u{293F}", "ARROW", &[], "MnSyA", 248),
  ("\\rcirclearrowup", "\u{21BA}", "ARROW", &["\\circlearrowleft"], "MnSyA", 249),
  ("\\circlearrowleft", "\u{21BA}", "ARROW", &["\\rcirclearrowup"], "MnSyA", 249),
  ("\\rcirclearrowleft", "\u{27F2}", "ARROW", &[], "MnSyA", 250),
  ("\\rcirclearrowdown", "\u{293F}", "ARROW", &[], "MnSyA", 251),
  ("\\lcirclearrowright", "\u{27F3}", "ARROW", &[], "MnSyA", 252),
  ("\\lcirclearrowup", "\u{21BB}", "ARROW", &["\\circlearrowright"], "MnSyA", 253),
  ("\\circlearrowright", "\u{21BB}", "ARROW", &["\\lcirclearrowup"], "MnSyA", 253),
  ("\\lcirclearrowleft", "\u{293E}", "ARROW", &[], "MnSyA", 254),
  ("\\lcirclearrowdown", "\u{27F3}", "ARROW", &[], "MnSyA", 255),
  ("\\nrightarrow", "\u{219B}", "ARROW", &["\\nto"], "MnSyB", 0),
  ("\\nto", "\u{219B}", "ARROW", &["\\nrightarrow"], "MnSyB", 0),
  ("\\nuparrow", "\u{2191}\u{338}", "ARROW", &[], "MnSyB", 1),
  ("\\nleftarrow", "\u{219A}", "ARROW", &["\\ngets"], "MnSyB", 2),
  ("\\ngets", "\u{219A}", "ARROW", &["\\nleftarrow"], "MnSyB", 2),
  ("\\ndownarrow", "\u{2193}\u{338}", "ARROW", &[], "MnSyB", 3),
  ("\\nnearrow", "\u{2197}\u{338}", "ARROW", &[], "MnSyB", 4),
  ("\\nnwarrow", "\u{2196}\u{338}", "ARROW", &[], "MnSyB", 5),
  ("\\nswarrow", "\u{2199}\u{338}", "ARROW", &[], "MnSyB", 6),
  ("\\nsearrow", "\u{2198}\u{338}", "ARROW", &[], "MnSyB", 7),
  ("\\nRightarrow", "\u{21CF}", "ARROW", &[], "MnSyB", 8),
  ("\\nUparrow", "\u{21D1}\u{338}", "ARROW", &[], "MnSyB", 9),
  ("\\nLeftarrow", "\u{21CD}", "ARROW", &[], "MnSyB", 10),
  ("\\nDownarrow", "\u{21D3}\u{338}", "ARROW", &[], "MnSyB", 11),
  ("\\nNearrow", "\u{21D7}\u{338}", "ARROW", &[], "MnSyB", 12),
  ("\\nNwarrow", "\u{21D6}\u{338}", "ARROW", &[], "MnSyB", 13),
  ("\\nSwarrow", "\u{21D9}\u{338}", "ARROW", &[], "MnSyB", 14),
  ("\\nSearrow", "\u{21D8}\u{338}", "ARROW", &[], "MnSyB", 15),
  ("\\nleftrightarrow", "\u{21AE}", "ARROW", &[], "MnSyB", 16),
  ("\\nupdownarrow", "\u{2195}\u{338}", "ARROW", &[], "MnSyB", 17),
  ("\\nneswarrow", "\u{2921}\u{338}", "ARROW", &[], "MnSyB", 18),
  ("\\nnwsearrow", "\u{2922}\u{338}", "ARROW", &[], "MnSyB", 19),
  ("\\nLeftrightarrow", "\u{21CE}", "ARROW", &[], "MnSyB", 20),
  ("\\nUpdownarrow", "\u{21D5}\u{338}", "ARROW", &[], "MnSyB", 21),
  ("\\nNeswarrow", "\u{2922}\u{338}", "ARROW", &[], "MnSyB", 22),
  ("\\nNwsearrow", "\u{2921}\u{338}", "ARROW", &[], "MnSyB", 23),
  ("\\ntwoheadrightarrow", "\u{21A0}\u{338}", "ARROW", &[], "MnSyB", 24),
  ("\\ntwoheaduparrow", "\u{219F}\u{338}", "ARROW", &[], "MnSyB", 25),
  ("\\ntwoheadleftarrow", "\u{219E}\u{338}", "ARROW", &[], "MnSyB", 26),
  ("\\ntwoheaddownarrow", "\u{21A1}\u{338}", "ARROW", &[], "MnSyB", 27),
  ("\\ntwoheadnearrow", "\u{21A0}\u{338}", "ARROW", &[], "MnSyB", 28),
  ("\\ntwoheadnwarrow", "\u{21A0}\u{338}", "ARROW", &[], "MnSyB", 29),
  ("\\ntwoheadswarrow", "\u{21A0}\u{338}", "ARROW", &[], "MnSyB", 30),
  ("\\ntwoheadsearrow", "\u{21A0}\u{338}", "ARROW", &[], "MnSyB", 31),
  ("\\nrightarrowtail", "\u{21A3}\u{338}", "ARROW", &[], "MnSyB", 32),
  ("\\nuparrowtail", "\u{21A3}\u{338}", "ARROW", &[], "MnSyB", 33),
  ("\\nleftarrowtail", "\u{21A2}\u{338}", "ARROW", &[], "MnSyB", 34),
  ("\\ndownarrowtail", "\u{21A3}\u{338}", "ARROW", &[], "MnSyB", 35),
  ("\\nnearrowtail", "\u{21A3}\u{338}", "ARROW", &[], "MnSyB", 36),
  ("\\nnwarrowtail", "\u{21A3}\u{338}", "ARROW", &[], "MnSyB", 37),
  ("\\nswarrowtail", "\u{21A3}\u{338}", "ARROW", &[], "MnSyB", 38),
  ("\\nsearrowtail", "\u{21A3}\u{338}", "ARROW", &[], "MnSyB", 39),
  ("\\nrightmapsto", "\u{21A6}\u{338}", "ARROW", &["\\nmapsto"], "MnSyB", 40),
  ("\\nmapsto", "\u{21A6}\u{338}", "ARROW", &["\\nrightmapsto"], "MnSyB", 40),
  ("\\nupmapsto", "\u{21A5}\u{338}", "ARROW", &[], "MnSyB", 41),
  ("\\nleftmapsto", "\u{21A4}\u{338}", "ARROW", &[], "MnSyB", 42),
  ("\\ndownmapsto", "\u{21A7}\u{338}", "ARROW", &[], "MnSyB", 43),
  ("\\nnemapsto", "\u{21A6}\u{338}", "ARROW", &[], "MnSyB", 44),
  ("\\nnwmapsto", "\u{21A6}\u{338}", "ARROW", &[], "MnSyB", 45),
  ("\\nswmapsto", "\u{21A6}\u{338}", "ARROW", &[], "MnSyB", 46),
  ("\\nsemapsto", "\u{21A6}\u{338}", "ARROW", &[], "MnSyB", 47),
  ("\\nlhookrightarrow", "\u{21AA}\u{338}", "ARROW", &["\\nhookrightarrow"], "MnSyB", 48),
  ("\\nhookrightarrow", "\u{21AA}\u{338}", "ARROW", &["\\nlhookrightarrow"], "MnSyB", 48),
  ("\\nlhookuparrow", "\u{21AA}\u{338}", "ARROW", &[], "MnSyB", 49),
  ("\\nlhookleftarrow", "\u{21A9}\u{338}", "ARROW", &[], "MnSyB", 50),
  ("\\nlhookdownarrow", "\u{21AA}\u{338}", "ARROW", &[], "MnSyB", 51),
  ("\\nlhooknearrow", "\u{21AA}\u{338}", "ARROW", &[], "MnSyB", 52),
  ("\\nlhooknwarrow", "\u{2923}\u{338}", "ARROW", &[], "MnSyB", 53),
  ("\\nlhookswarrow", "\u{21AA}\u{338}", "ARROW", &[], "MnSyB", 54),
  ("\\nlhooksearrow", "\u{2925}\u{338}", "ARROW", &[], "MnSyB", 55),
  ("\\nrhookrightarrow", "\u{21AA}\u{338}", "ARROW", &[], "MnSyB", 56),
  ("\\nrhookuparrow", "\u{21AA}\u{338}", "ARROW", &[], "MnSyB", 57),
  ("\\nrhookleftarrow", "\u{21A9}\u{338}", "ARROW", &["\\nhookleftarrow"], "MnSyB", 58),
  ("\\nhookleftarrow", "\u{21A9}\u{338}", "ARROW", &["\\nrhookleftarrow"], "MnSyB", 58),
  ("\\nrhookdownarrow", "\u{21AA}\u{338}", "ARROW", &[], "MnSyB", 59),
  ("\\nrhooknearrow", "\u{2924}\u{338}", "ARROW", &[], "MnSyB", 60),
  ("\\nrhooknwarrow", "\u{21AA}\u{338}", "ARROW", &[], "MnSyB", 61),
  ("\\nrhookswarrow", "\u{2926}\u{338}", "ARROW", &[], "MnSyB", 62),
  ("\\nrhooksearrow", "\u{21AA}\u{338}", "ARROW", &[], "MnSyB", 63),
  ("\\nrightharpoonccw", "\u{21C0}\u{338}", "ARROW", &["\\nrightharpoonup"], "MnSyB", 64),
  ("\\nrightharpoonup", "\u{21C0}\u{338}", "ARROW", &["\\nrightharpoonccw"], "MnSyB", 64),
  ("\\nupharpoonccw", "\u{21BF}\u{338}", "ARROW", &["\\nupharpoonleft"], "MnSyB", 65),
  ("\\nupharpoonleft", "\u{21BF}\u{338}", "ARROW", &["\\nupharpoonccw"], "MnSyB", 65),
  ("\\nleftharpoonccw", "\u{21BD}\u{338}", "ARROW", &["\\nleftharpoondown"], "MnSyB", 66),
  ("\\nleftharpoondown", "\u{21BD}\u{338}", "ARROW", &["\\nleftharpoonccw"], "MnSyB", 66),
  ("\\ndownharpoonccw", "\u{21C2}\u{338}", "ARROW", &["\\ndownharpoonright"], "MnSyB", 67),
  ("\\ndownharpoonright", "\u{21C2}\u{338}", "ARROW", &["\\ndownharpoonccw"], "MnSyB", 67),
  ("\\nneharpoonccw", "\u{21C0}\u{338}", "ARROW", &[], "MnSyB", 68),
  ("\\nnwharpoonccw", "\u{21C0}\u{338}", "ARROW", &[], "MnSyB", 69),
  ("\\nswharpoonccw", "\u{21C0}\u{338}", "ARROW", &[], "MnSyB", 70),
  ("\\nseharpoonccw", "\u{21C0}\u{338}", "ARROW", &[], "MnSyB", 71),
  ("\\nrightharpooncw", "\u{21C1}\u{338}", "ARROW", &["\\nrightharpoondown"], "MnSyB", 72),
  ("\\nrightharpoondown", "\u{21C1}\u{338}", "ARROW", &["\\nrightharpooncw"], "MnSyB", 72),
  ("\\nupharpooncw", "\u{21BE}\u{338}", "ARROW", &["\\nupharpoonright", "\\nrestriction"], "MnSyB", 73),
  ("\\nupharpoonright", "\u{21BE}\u{338}", "ARROW", &["\\nupharpooncw", "\\nrestriction"], "MnSyB", 73),
  ("\\nrestriction", "\u{21BE}\u{338}", "RELOP", &["\\nupharpooncw", "\\nupharpoonright"], "MnSyB", 73),
  ("\\nleftharpooncw", "\u{21BC}\u{338}", "ARROW", &["\\nleftharpoonup"], "MnSyB", 74),
  ("\\nleftharpoonup", "\u{21BC}\u{338}", "ARROW", &["\\nleftharpooncw"], "MnSyB", 74),
  ("\\ndownharpooncw", "\u{21C3}\u{338}", "ARROW", &["\\ndownharpoonleft"], "MnSyB", 75),
  ("\\ndownharpoonleft", "\u{21C3}\u{338}", "ARROW", &["\\ndownharpooncw"], "MnSyB", 75),
  ("\\nneharpooncw", "\u{21C1}\u{338}", "ARROW", &[], "MnSyB", 76),
  ("\\nnwharpooncw", "\u{21C1}\u{338}", "ARROW", &[], "MnSyB", 77),
  ("\\nswharpooncw", "\u{21C1}\u{338}", "ARROW", &[], "MnSyB", 78),
  ("\\nseharpooncw", "\u{21C1}\u{338}", "ARROW", &[], "MnSyB", 79),
  ("\\nleftrightharpoonupdown", "\u{294B}\u{338}", "ARROW", &[], "MnSyB", 80),
  ("\\nupdownharpoonleftright", "\u{294D}\u{338}", "ARROW", &[], "MnSyB", 81),
  ("\\nneswharpoonnwse", "\u{294B}\u{338}", "ARROW", &[], "MnSyB", 82),
  ("\\nnwseharpoonnesw", "\u{294B}\u{338}", "ARROW", &[], "MnSyB", 83),
  ("\\nleftrightharpoondownup", "\u{294A}\u{338}", "ARROW", &[], "MnSyB", 84),
  ("\\nupdownharpoonrightleft", "\u{294C}\u{338}", "ARROW", &[], "MnSyB", 85),
  ("\\nneswharpoonsenw", "\u{294A}\u{338}", "ARROW", &[], "MnSyB", 86),
  ("\\nnwseharpoonswne", "\u{294A}\u{338}", "ARROW", &[], "MnSyB", 87),
  ("\\nrightleftharpoons", "\u{21CC}\u{338}", "ARROW", &[], "MnSyB", 88),
  ("\\nupdownharpoons", "\u{296E}\u{338}", "ARROW", &[], "MnSyB", 89),
  ("\\nneswharpoons", "\u{21CB}\u{338}", "ARROW", &[], "MnSyB", 90),
  ("\\nsenwharpoons", "\u{21CB}\u{338}", "ARROW", &[], "MnSyB", 91),
  ("\\nleftrightharpoons", "\u{21CB}\u{338}", "ARROW", &[], "MnSyB", 92),
  ("\\ndownupharpoons", "\u{296F}\u{338}", "ARROW", &[], "MnSyB", 93),
  ("\\nswneharpoons", "\u{21CB}\u{338}", "ARROW", &[], "MnSyB", 94),
  ("\\nnwseharpoons", "\u{21CB}\u{338}", "ARROW", &[], "MnSyB", 95),
  ("\\ndashedrightarrow", "\u{21E2}\u{338}", "ARROW", &["\\ndashrightarrow", "\\ndasharrow"], "MnSyB", 96),
  ("\\ndashrightarrow", "\u{21E2}\u{338}", "ARROW", &["\\ndashedrightarrow", "\\ndasharrow"], "MnSyB", 96),
  ("\\ndasharrow", "\u{21E2}\u{338}", "ARROW", &["\\ndashedrightarrow", "\\ndashrightarrow"], "MnSyB", 96),
  ("\\ndasheduparrow", "\u{21E1}\u{338}", "ARROW", &[], "MnSyB", 97),
  ("\\ndashedleftarrow", "\u{21E0}\u{338}", "ARROW", &["\\ndashleftarrow"], "MnSyB", 98),
  ("\\ndashleftarrow", "\u{21E0}\u{338}", "ARROW", &["\\ndashedleftarrow"], "MnSyB", 98),
  ("\\ndasheddownarrow", "\u{21E3}\u{338}", "ARROW", &[], "MnSyB", 99),
  ("\\ndashednearrow", "\u{21E2}\u{338}", "ARROW", &[], "MnSyB", 100),
  ("\\ndashednwarrow", "\u{21E2}\u{338}", "ARROW", &[], "MnSyB", 101),
  ("\\ndashedswarrow", "\u{21E2}\u{338}", "ARROW", &[], "MnSyB", 102),
  ("\\ndashedsearrow", "\u{21E2}\u{338}", "ARROW", &[], "MnSyB", 103),
  ("\\nrightspoon", "\u{22B8}\u{338}", "ARROW", &["\\nmultimap"], "MnSyB", 104),
  ("\\nmultimap", "\u{22B8}\u{338}", "ARROW", &["\\nrightspoon"], "MnSyB", 104),
  ("\\nupspoon", "\u{2AEF}\u{338}", "ARROW", &[], "MnSyB", 105),
  ("\\nleftspoon", "\u{27DC}\u{338}", "ARROW", &[], "MnSyB", 106),
  ("\\ndownspoon", "\u{2AF0}\u{338}", "ARROW", &[], "MnSyB", 107),
  ("\\nnespoon", "\u{22B8}\u{338}", "ARROW", &[], "MnSyB", 108),
  ("\\nnwspoon", "\u{22B8}\u{338}", "ARROW", &[], "MnSyB", 109),
  ("\\nswspoon", "\u{22B8}\u{338}", "ARROW", &[], "MnSyB", 110),
  ("\\nsespoon", "\u{22B8}\u{338}", "ARROW", &[], "MnSyB", 111),
  ("\\nrightfilledspoon", "\u{22B8}\u{338}", "ARROW", &[], "MnSyB", 112),
  ("\\nupfilledspoon", "\u{2AEF}\u{338}", "ARROW", &[], "MnSyB", 113),
  ("\\nleftfilledspoon", "\u{27DC}\u{338}", "ARROW", &[], "MnSyB", 114),
  ("\\ndownfilledspoon", "\u{2AF0}\u{338}", "ARROW", &[], "MnSyB", 115),
  ("\\nnefilledspoon", "\u{22B8}\u{338}", "ARROW", &[], "MnSyB", 116),
  ("\\nnwfilledspoon", "\u{22B8}\u{338}", "ARROW", &[], "MnSyB", 117),
  ("\\nswfilledspoon", "\u{22B8}\u{338}", "ARROW", &[], "MnSyB", 118),
  ("\\nsefilledspoon", "\u{22B8}\u{338}", "ARROW", &[], "MnSyB", 119),
  ("\\nrightfootline", "\u{27DE}\u{338}", "RELOP", &[], "MnSyB", 120),
  ("\\nupfootline", "\u{27D9}\u{338}", "RELOP", &[], "MnSyB", 121),
  ("\\nleftfootline", "\u{27DD}\u{338}", "RELOP", &[], "MnSyB", 122),
  ("\\ndownfootline", "\u{27D8}\u{338}", "RELOP", &[], "MnSyB", 123),
  ("\\nnefootline", "\u{27DE}\u{338}", "RELOP", &[], "MnSyB", 124),
  ("\\nnwfootline", "\u{27DE}\u{338}", "RELOP", &[], "MnSyB", 125),
  ("\\nswfootline", "\u{27DE}\u{338}", "RELOP", &[], "MnSyB", 126),
  ("\\nsefootline", "\u{27DE}\u{338}", "RELOP", &[], "MnSyB", 127),
  ("\\nrightfree", "\u{2ADD}\u{338}", "RELOP", &[], "MnSyB", 128),
  ("\\nupfree", "\u{2ADD}\u{338}", "RELOP", &[], "MnSyB", 129),
  ("\\nleftfree", "\u{2ADD}\u{338}", "RELOP", &[], "MnSyB", 130),
  ("\\ndownfree", "\u{2ADC}", "RELOP", &[], "MnSyB", 131),
  ("\\nnefree", "\u{2ADD}\u{338}", "RELOP", &[], "MnSyB", 132),
  ("\\nnwfree", "\u{2ADD}\u{338}", "RELOP", &[], "MnSyB", 133),
  ("\\nswfree", "\u{2ADD}\u{338}", "RELOP", &[], "MnSyB", 134),
  ("\\nsefree", "\u{2ADD}\u{338}", "RELOP", &[], "MnSyB", 135),
  ("\\nrightpitchfork", "\u{22D4}\u{338}", "RELOP", &[], "MnSyB", 136),
  ("\\nuppitchfork", "\u{22D4}\u{338}", "RELOP", &["\\npitchfork"], "MnSyB", 137),
  ("\\npitchfork", "\u{22D4}\u{338}", "RELOP", &["\\nuppitchfork"], "MnSyB", 137),
  ("\\nleftpitchfork", "\u{22D4}\u{338}", "RELOP", &[], "MnSyB", 138),
  ("\\ndownpitchfork", "\u{2ADB}\u{338}", "RELOP", &[], "MnSyB", 139),
  ("\\nnepitchfork", "\u{22D4}\u{338}", "RELOP", &[], "MnSyB", 140),
  ("\\nnwpitchfork", "\u{22D4}\u{338}", "RELOP", &[], "MnSyB", 141),
  ("\\nswpitchfork", "\u{22D4}\u{338}", "RELOP", &[], "MnSyB", 142),
  ("\\nsepitchfork", "\u{22D4}\u{338}", "RELOP", &[], "MnSyB", 143),
  ("\\nrightrightarrows", "\u{21C9}\u{338}", "ARROW", &[], "MnSyB", 144),
  ("\\nupuparrows", "\u{21C8}\u{338}", "ARROW", &[], "MnSyB", 145),
  ("\\nleftleftarrows", "\u{21C7}\u{338}", "ARROW", &[], "MnSyB", 146),
  ("\\ndowndownarrows", "\u{21CA}\u{338}", "ARROW", &[], "MnSyB", 147),
  ("\\nnenearrows", "\u{21C9}\u{338}", "ARROW", &[], "MnSyB", 148),
  ("\\nnwnwarrows", "\u{21C7}\u{338}", "ARROW", &[], "MnSyB", 149),
  ("\\nswswarrows", "\u{21C7}\u{338}", "ARROW", &[], "MnSyB", 150),
  ("\\nsesearrows", "\u{21C9}\u{338}", "ARROW", &[], "MnSyB", 151),
  ("\\nrightleftarrows", "\u{21C4}\u{338}", "ARROW", &[], "MnSyB", 152),
  ("\\nupdownarrows", "\u{21C5}\u{338}", "ARROW", &[], "MnSyB", 153),
  ("\\nneswarrows", "\u{21C6}\u{338}", "ARROW", &[], "MnSyB", 154),
  ("\\nnwsearrows", "\u{21C6}\u{338}", "ARROW", &[], "MnSyB", 155),
  ("\\nleftrightarrows", "\u{21C6}\u{338}", "ARROW", &[], "MnSyB", 156),
  ("\\ndownuparrows", "\u{21F5}\u{338}", "ARROW", &[], "MnSyB", 157),
  ("\\nswnearrows", "\u{21C6}\u{338}", "ARROW", &[], "MnSyB", 158),
  ("\\nsenwarrows", "\u{21C6}\u{338}", "ARROW", &[], "MnSyB", 159),
  ("\\nrightlsquigarrow", "\u{219D}\u{338}", "ARROW", &["\\nleadsto", "\\nrightsquigarrow"], "MnSyB", 160),
  ("\\nleadsto", "\u{219D}\u{338}", "ARROW", &["\\nrightlsquigarrow", "\\nrightsquigarrow"], "MnSyB", 160),
  ("\\nrightsquigarrow", "\u{219D}\u{338}", "ARROW", &["\\nrightlsquigarrow", "\\nleadsto"], "MnSyB", 160),
  ("\\nuplsquigarrow", "\u{219D}\u{338}", "ARROW", &[], "MnSyB", 161),
  ("\\nleftlsquigarrow", "\u{219C}\u{338}", "ARROW", &[], "MnSyB", 162),
  ("\\ndownlsquigarrow", "\u{219D}\u{338}", "ARROW", &[], "MnSyB", 163),
  ("\\nnelsquigarrow", "\u{219D}\u{338}", "ARROW", &[], "MnSyB", 164),
  ("\\nnwlsquigarrow", "\u{219D}\u{338}", "ARROW", &[], "MnSyB", 165),
  ("\\nswlsquigarrow", "\u{219D}\u{338}", "ARROW", &[], "MnSyB", 166),
  ("\\nselsquigarrow", "\u{219D}\u{338}", "ARROW", &[], "MnSyB", 167),
  ("\\nrightrsquigarrow", "\u{219D}\u{338}", "ARROW", &[], "MnSyB", 168),
  ("\\nuprsquigarrow", "\u{219D}\u{338}", "ARROW", &[], "MnSyB", 169),
  ("\\nleftrsquigarrow", "\u{219C}\u{338}", "ARROW", &[], "MnSyB", 170),
  ("\\ndownrsquigarrow", "\u{219D}\u{338}", "ARROW", &[], "MnSyB", 171),
  ("\\nnersquigarrow", "\u{219D}\u{338}", "ARROW", &[], "MnSyB", 172),
  ("\\nnwrsquigarrow", "\u{219D}\u{338}", "ARROW", &[], "MnSyB", 173),
  ("\\nswrsquigarrow", "\u{219D}\u{338}", "ARROW", &[], "MnSyB", 174),
  ("\\nsersquigarrow", "\u{219D}\u{338}", "ARROW", &[], "MnSyB", 175),
  ("\\nsquigarrowleftright", "\u{21AD}\u{338}", "ARROW", &["\\nleftrightsquigarrow"], "MnSyB", 176),
  ("\\nleftrightsquigarrow", "\u{21AD}\u{338}", "ARROW", &["\\nsquigarrowleftright"], "MnSyB", 176),
  ("\\nsquigarrowupdown", "\u{21AD}\u{338}", "ARROW", &[], "MnSyB", 177),
  ("\\nsquigarrowrightleft", "\u{21AD}\u{338}", "ARROW", &[], "MnSyB", 178),
  ("\\nsquigarrowdownup", "\u{21AD}\u{338}", "ARROW", &[], "MnSyB", 179),
  ("\\nsquigarrownesw", "\u{21AD}\u{338}", "ARROW", &[], "MnSyB", 180),
  ("\\nsquigarrownwse", "\u{21AD}\u{338}", "ARROW", &[], "MnSyB", 181),
  ("\\nsquigarrowswne", "\u{21AD}\u{338}", "ARROW", &[], "MnSyB", 182),
  ("\\nsquigarrowsenw", "\u{21AD}\u{338}", "ARROW", &[], "MnSyB", 183),
  ("\\nlcurvearrowright", "\u{21B7}\u{338}", "ARROW", &["\\ncurvearrowright"], "MnSyB", 184),
  ("\\ncurvearrowright", "\u{21B7}\u{338}", "ARROW", &["\\nlcurvearrowright"], "MnSyB", 184),
  ("\\nlcurvearrowup", "\u{21B7}\u{338}", "ARROW", &[], "MnSyB", 185),
  ("\\nlcurvearrowleft", "\u{21B7}\u{338}", "ARROW", &[], "MnSyB", 186),
  ("\\nlcurvearrowdown", "\u{2938}\u{338}", "ARROW", &[], "MnSyB", 187),
  ("\\nlcurvearrowne", "\u{21B7}\u{338}", "ARROW", &[], "MnSyB", 188),
  ("\\nlcurvearrownw", "\u{21B7}\u{338}", "ARROW", &[], "MnSyB", 189),
  ("\\nlcurvearrowsw", "\u{21B7}\u{338}", "ARROW", &[], "MnSyB", 190),
  ("\\nlcurvearrowse", "\u{21B7}\u{338}", "ARROW", &[], "MnSyB", 191),
  ("\\nrcurvearrowright", "\u{293B}\u{338}", "ARROW", &[], "MnSyB", 192),
  ("\\nrcurvearrowup", "\u{293B}\u{338}", "ARROW", &[], "MnSyB", 193),
  ("\\nrcurvearrowleft", "\u{21B6}\u{338}", "ARROW", &["\\ncurvearrowleft"], "MnSyB", 194),
  ("\\ncurvearrowleft", "\u{21B6}\u{338}", "ARROW", &["\\nrcurvearrowleft"], "MnSyB", 194),
  ("\\nrcurvearrowdown", "\u{2939}\u{338}", "ARROW", &[], "MnSyB", 195),
  ("\\nrcurvearrowne", "\u{293B}\u{338}", "ARROW", &[], "MnSyB", 196),
  ("\\nrcurvearrownw", "\u{293B}\u{338}", "ARROW", &[], "MnSyB", 197),
  ("\\nrcurvearrowsw", "\u{293B}\u{338}", "ARROW", &[], "MnSyB", 198),
  ("\\nrcurvearrowse", "\u{293B}\u{338}", "ARROW", &[], "MnSyB", 199),
  ("\\ncurvearrowleftright", "\u{21B7}\u{338}", "ARROW", &[], "MnSyB", 200),
  ("\\ncurvearrowupdown", "\u{21B7}\u{338}", "ARROW", &[], "MnSyB", 201),
  ("\\ncurvearrowrightleft", "\u{21B7}\u{338}", "ARROW", &[], "MnSyB", 202),
  ("\\ncurvearrowdownup", "\u{21B7}\u{338}", "ARROW", &[], "MnSyB", 203),
  ("\\ncurvearrownesw", "\u{21B7}\u{338}", "ARROW", &[], "MnSyB", 204),
  ("\\ncurvearrownwse", "\u{21B7}\u{338}", "ARROW", &[], "MnSyB", 205),
  ("\\ncurvearrowswne", "\u{21B7}\u{338}", "ARROW", &[], "MnSyB", 206),
  ("\\ncurvearrowsenw", "\u{21B7}\u{338}", "ARROW", &[], "MnSyB", 207),
  ("\\nleftrightline", "\u{2212}\u{338}", "RELOP", &["\\nrelbar"], "MnSyB", 208),
  ("\\nrelbar", "\u{2212}\u{338}", "RELOP", &["\\nleftrightline"], "MnSyB", 208),
  ("\\nupdownline", "\u{2224}", "RELOP", &["\\ndivides", "\\nmid"], "MnSyB", 209),
  ("\\ndivides", "\u{2224}", "RELOP", &["\\nupdownline", "\\nmid"], "MnSyB", 209),
  ("\\nmid", "\u{2224}", "RELOP", &["\\nupdownline", "\\ndivides"], "MnSyB", 209),
  ("\\nneswline", "\u{27CB}\u{338}", "RELOP", &["\\ndiagup"], "MnSyB", 210),
  ("\\ndiagup", "\u{27CB}\u{338}", "RELOP", &["\\nneswline"], "MnSyB", 210),
  ("\\nnwseline", "\u{27CD}\u{338}", "RELOP", &["\\ndiagdown"], "MnSyB", 211),
  ("\\ndiagdown", "\u{27CD}\u{338}", "RELOP", &["\\nnwseline"], "MnSyB", 211),
  ("\\nLeftrightline", "=\u{338}", "RELOP", &["\\nRelbar"], "MnSyB", 212),
  ("\\nRelbar", "=\u{338}", "RELOP", &["\\nLeftrightline"], "MnSyB", 212),
  ("\\nUpdownline", "\u{2226}", "RELOP", &["\\nparallel"], "MnSyB", 213),
  ("\\nparallel", "\u{2226}", "RELOP", &["\\nUpdownline"], "MnSyB", 213),
  ("\\nNeswline", "\u{2AFD}\u{338}", "RELOP", &[], "MnSyB", 214),
  ("\\nNwseline", "\u{29F5}\u{29F5}\u{338}", "RELOP", &[], "MnSyB", 215),
  ("\\nrightvdash", "\u{22AC}", "RELOP", &["\\nvdash"], "MnSyB", 216),
  ("\\nvdash", "\u{22AC}", "RELOP", &["\\nrightvdash"], "MnSyB", 216),
  ("\\nupvdash", "\u{22A5}\u{338}", "RELOP", &["\\nperp"], "MnSyB", 217),
  ("\\nperp", "\u{27C2}\u{338}", "RELOP", &["\\nupvdash"], "MnSyB", 217),
  ("\\nleftvdash", "\u{22A3}\u{338}", "RELOP", &["\\ndashv"], "MnSyB", 218),
  ("\\ndashv", "\u{22A3}\u{338}", "RELOP", &["\\nleftvdash"], "MnSyB", 218),
  ("\\ndownvdash", "\u{22A4}\u{338}", "RELOP", &[], "MnSyB", 219),
  ("\\nnevdash", "\u{22A2}\u{338}", "RELOP", &[], "MnSyB", 220),
  ("\\nnwvdash", "\u{22A2}\u{338}", "RELOP", &[], "MnSyB", 221),
  ("\\nswvdash", "\u{22A2}\u{338}", "RELOP", &[], "MnSyB", 222),
  ("\\nsevdash", "\u{22A2}\u{338}", "RELOP", &[], "MnSyB", 223),
  ("\\nrightmodels", "\u{22AD}", "RELOP", &["\\nmodels", "\\nvDash"], "MnSyB", 224),
  ("\\nmodels", "\u{22AD}", "RELOP", &["\\nrightmodels", "\\nvDash"], "MnSyB", 224),
  ("\\nvDash", "\u{22AD}", "RELOP", &["\\nrightmodels", "\\nmodels"], "MnSyB", 224),
  ("\\nupmodels", "\u{2AEB}\u{338}", "RELOP", &[], "MnSyB", 225),
  ("\\nleftmodels", "\u{2AE4}\u{338}", "RELOP", &[], "MnSyB", 226),
  ("\\ndownmodels", "\u{2AEA}\u{338}", "RELOP", &[], "MnSyB", 227),
  ("\\nnemodels", "\u{22A7}\u{338}", "RELOP", &[], "MnSyB", 228),
  ("\\nnwmodels", "\u{22A7}\u{338}", "RELOP", &[], "MnSyB", 229),
  ("\\nswmodels", "\u{22A7}\u{338}", "RELOP", &[], "MnSyB", 230),
  ("\\nsemodels", "\u{22A7}\u{338}", "RELOP", &[], "MnSyB", 231),
  ("\\nrightVdash", "\u{22AE}", "RELOP", &["\\nVdash"], "MnSyB", 232),
  ("\\nVdash", "\u{22AE}", "RELOP", &["\\nrightVdash"], "MnSyB", 232),
  ("\\nupVdash", "\u{234A}\u{338}", "RELOP", &[], "MnSyB", 233),
  ("\\nleftVdash", "\u{2AE3}\u{338}", "RELOP", &[], "MnSyB", 234),
  ("\\ndownVdash", "\u{2351}\u{338}", "RELOP", &[], "MnSyB", 235),
  ("\\nneVdash", "\u{22A9}\u{338}", "RELOP", &[], "MnSyB", 236),
  ("\\nnwVdash", "\u{22A9}\u{338}", "RELOP", &[], "MnSyB", 237),
  ("\\nswVdash", "\u{22A9}\u{338}", "RELOP", &[], "MnSyB", 238),
  ("\\nseVdash", "\u{22A9}\u{338}", "RELOP", &[], "MnSyB", 239),
  ("\\nrightModels", "\u{22AF}", "RELOP", &["\\nVDash"], "MnSyB", 240),
  ("\\nVDash", "\u{22AF}", "RELOP", &["\\nrightModels"], "MnSyB", 240),
  ("\\nupModels", "\u{22AB}\u{338}", "RELOP", &[], "MnSyB", 241),
  ("\\nleftModels", "\u{2AE5}\u{338}", "RELOP", &[], "MnSyB", 242),
  ("\\ndownModels", "\u{22AB}\u{338}", "RELOP", &[], "MnSyB", 243),
  ("\\nneModels", "\u{22AB}\u{338}", "RELOP", &[], "MnSyB", 244),
  ("\\nnwModels", "\u{22AB}\u{338}", "RELOP", &[], "MnSyB", 245),
  ("\\nswModels", "\u{22AB}\u{338}", "RELOP", &[], "MnSyB", 246),
  ("\\nseModels", "\u{22AB}\u{338}", "RELOP", &[], "MnSyB", 247),
  ("\\nrcirclearrowright", "\u{293F}\u{338}", "ARROW", &[], "MnSyB", 248),
  ("\\nrcirclearrowup", "\u{21BA}\u{338}", "ARROW", &["\\ncirclearrowleft"], "MnSyB", 249),
  ("\\ncirclearrowleft", "\u{21BA}\u{338}", "ARROW", &["\\nrcirclearrowup"], "MnSyB", 249),
  ("\\nrcirclearrowleft", "\u{27F2}\u{338}", "ARROW", &[], "MnSyB", 250),
  ("\\nrcirclearrowdown", "\u{293F}\u{338}", "ARROW", &[], "MnSyB", 251),
  ("\\nlcirclearrowright", "\u{27F3}\u{338}", "ARROW", &[], "MnSyB", 252),
  ("\\nlcirclearrowup", "\u{21BB}\u{338}", "ARROW", &["\\ncirclearrowright"], "MnSyB", 253),
  ("\\ncirclearrowright", "\u{21BB}\u{338}", "ARROW", &["\\nlcirclearrowup"], "MnSyB", 253),
  ("\\nlcirclearrowleft", "\u{293E}\u{338}", "ARROW", &[], "MnSyB", 254),
  ("\\nlcirclearrowdown", "\u{27F3}\u{338}", "ARROW", &[], "MnSyB", 255),
  ("\\cdot", "\u{22C5}", "MULOP", &[], "MnSyC", 0),
  ("\\cdotp", "\u{B7}", "PUNCT", &[], "MnSyC", 0),
  ("\\hdotdot", "\u{2025}", "MULOP", &[], "MnSyC", 1),
  ("\\vdotdot", "\u{2236}", "MULOP", &[], "MnSyC", 2),
  ("\\colon", "\u{2236}", "PUNCT", &[], "MnSyC", 2),
  ("\\udotdot", "\u{2025}", "MULOP", &[], "MnSyC", 3),
  ("\\ddotdot", "\u{2025}", "MULOP", &[], "MnSyC", 4),
  ("\\hdots", "\u{22EF}", "ID", &[], "MnSyC", 5),
  ("\\udots", "\u{22F0}", "ID", &[], "MnSyC", 7),
  ("\\ddots", "\u{22F1}", "ID", &[], "MnSyC", 8),
  ("\\righttherefore", "\u{2234}", "MULOP", &[], "MnSyC", 9),
  ("\\uptherefore", "\u{2234}", "MULOP", &["\\therefore"], "MnSyC", 10),
  ("\\therefore", "\u{2234}", "MULOP", &["\\uptherefore"], "MnSyC", 10),
  ("\\lefttherefore", "\u{2234}", "MULOP", &[], "MnSyC", 11),
  ("\\downtherefore", "\u{2235}", "MULOP", &["\\because"], "MnSyC", 12),
  ("\\because", "\u{2235}", "MULOP", &["\\downtherefore"], "MnSyC", 12),
  ("\\diamonddots", "\u{2058}", "ADDOP", &[], "MnSyC", 13),
  ("\\squaredots", "\u{2237}", "MULOP", &[], "MnSyC", 14),
  ("\\fivedots", "\u{2059}", "MULOP", &[], "MnSyC", 15),
  ("\\minus", "\u{2212}", "ADDOP", &[], "MnSyC", 16),
  ("\\medvert", "\u{2223}", "MULOP", &[], "MnSyC", 17),
  ("\\shortmid", "\u{2223}", "RELOP", &[], "MnSyC", 17),
  ("\\medslash", "\u{2215}", "MULOP", &[], "MnSyC", 18),
  ("\\medbackslash", "\u{2216}", "MULOP", &["\\setminus", "\\smallsetminus"], "MnSyC", 19),
  ("\\setminus", "\u{2216}", "ADDOP", &["\\medbackslash", "\\smallsetminus"], "MnSyC", 19),
  ("\\smallsetminus", "\u{2216}", "ADDOP", &["\\medbackslash", "\\setminus"], "MnSyC", 19),
  ("\\plus", "+", "ADDOP", &[], "MnSyC", 20),
  ("\\times", "\u{D7}", "MULOP", &[], "MnSyC", 21),
  ("\\pm", "\u{B1}", "MULOP", &[], "MnSyC", 22),
  ("\\mp", "\u{2213}", "MULOP", &[], "MnSyC", 23),
  ("\\dotminus", "\u{2238}", "ADDOP", &[], "MnSyC", 24),
  ("\\dotmedvert", "\u{2223}", "MULOP", &[], "MnSyC", 25),
  ("\\minusdot", "\u{2A2A}", "ADDOP", &[], "MnSyC", 26),
  ("\\medvertdot", "\u{2223}", "MULOP", &[], "MnSyC", 27),
  ("\\div", "\u{F7}", "MULOP", &[], "MnSyC", 28),
  ("\\slashdiv", "\u{F7}", "MULOP", &[], "MnSyC", 29),
  ("\\vertdiv", "\u{F7}", "MULOP", &[], "MnSyC", 30),
  ("\\backslashdiv", "\u{F7}", "MULOP", &[], "MnSyC", 31),
  ("\\neg", "\u{AC}", "", &["\\lnot"], "MnSyC", 32),
  ("\\lnot", "\u{AC}", "", &["\\neg"], "MnSyC", 32),
  ("\\minushookdown", "\u{AC}", "ADDOP", &[], "MnSyC", 32),
  ("\\backneg", "\u{2310}", "", &[], "MnSyC", 33),
  ("\\hookdownminus", "\u{2310}", "ADDOP", &[], "MnSyC", 33),
  ("\\dtimes", "\u{D7}", "MULOP", &[], "MnSyC", 34),
  ("\\rtimes", "\u{22CA}", "MULOP", &[], "MnSyC", 35),
  ("\\utimes", "\u{D7}", "MULOP", &[], "MnSyC", 36),
  ("\\ltimes", "\u{22C9}", "MULOP", &[], "MnSyC", 37),
  ("\\bowtie", "\u{22C8}", "MULOP", &["\\Join"], "MnSyC", 38),
  ("\\Join", "\u{2A1D}", "MULOP", &["\\bowtie"], "MnSyC", 38),
  ("\\vertbowtie", "\u{29D6}", "MULOP", &[], "MnSyC", 39),
  ("\\rightY", "\u{227B}", "ADDOP", &[], "MnSyC", 40),
  ("\\upY", "\u{2144}", "ADDOP", &[], "MnSyC", 41),
  ("\\leftY", "\u{227A}", "ADDOP", &[], "MnSyC", 42),
  ("\\downY", "Y", "ADDOP", &[], "MnSyC", 43),
  ("\\wedge", "\u{2227}", "ADDOP", &["\\land"], "MnSyC", 44),
  ("\\land", "\u{2227}", "MULOP", &["\\wedge"], "MnSyC", 44),
  ("\\vee", "\u{2228}", "ADDOP", &["\\lor"], "MnSyC", 45),
  ("\\lor", "\u{2228}", "MULOP", &["\\vee"], "MnSyC", 45),
  ("\\wedgedot", "\u{27D1}", "ADDOP", &[], "MnSyC", 46),
  ("\\veedot", "\u{27C7}", "ADDOP", &[], "MnSyC", 47),
  ("\\doublewedge", "\u{2A55}", "ADDOP", &[], "MnSyC", 48),
  ("\\doublevee", "\u{2A54}", "ADDOP", &[], "MnSyC", 49),
  ("\\curlywedge", "\u{22CF}", "ADDOP", &[], "MnSyC", 50),
  ("\\curlyvee", "\u{22CE}", "ADDOP", &[], "MnSyC", 51),
  ("\\curlywedgedot", "\u{22CF}", "ADDOP", &[], "MnSyC", 52),
  ("\\curlyveedot", "\u{22CE}", "ADDOP", &[], "MnSyC", 53),
  ("\\doublecurlywedge", "\u{22CF}", "ADDOP", &[], "MnSyC", 54),
  ("\\doublecurlyvee", "\u{22CE}", "ADDOP", &[], "MnSyC", 55),
  ("\\cup", "\u{222A}", "ADDOP", &[], "MnSyC", 56),
  ("\\cap", "\u{2229}", "ADDOP", &[], "MnSyC", 57),
  ("\\doublecup", "\u{22D3}", "ADDOP", &["\\Cup"], "MnSyC", 58),
  ("\\Cup", "\u{22D3}", "MULOP", &["\\doublecup"], "MnSyC", 58),
  ("\\doublecap", "\u{22D2}", "ADDOP", &["\\Cap"], "MnSyC", 59),
  ("\\Cap", "\u{22D2}", "MULOP", &["\\doublecap"], "MnSyC", 59),
  ("\\cupdot", "\u{228D}", "ADDOP", &[], "MnSyC", 60),
  ("\\capdot", "\u{2A40}", "ADDOP", &[], "MnSyC", 61),
  ("\\cupplus", "\u{228E}", "ADDOP", &["\\uplus"], "MnSyC", 62),
  ("\\uplus", "\u{228E}", "ADDOP", &["\\cupplus"], "MnSyC", 62),
  ("\\capplus", "\u{2229}", "ADDOP", &[], "MnSyC", 63),
  ("\\sqcup", "\u{2294}", "ADDOP", &[], "MnSyC", 64),
  ("\\sqcap", "\u{2293}", "ADDOP", &[], "MnSyC", 65),
  ("\\doublesqcup", "\u{2A4F}", "ADDOP", &[], "MnSyC", 66),
  ("\\doublesqcap", "\u{2A4E}", "ADDOP", &[], "MnSyC", 67),
  ("\\sqcupdot", "\u{2294}", "ADDOP", &[], "MnSyC", 68),
  ("\\sqcapdot", "\u{2293}", "ADDOP", &[], "MnSyC", 69),
  ("\\sqcupplus", "\u{2294}", "ADDOP", &[], "MnSyC", 70),
  ("\\sqcapplus", "\u{2293}", "ADDOP", &[], "MnSyC", 71),
  ("\\smalltriangleright", "\u{25B9}", "ADDOP", &[], "MnSyC", 72),
  ("\\smalltriangleup", "\u{25B5}", "ADDOP", &[], "MnSyC", 73),
  ("\\smalltriangleleft", "\u{25C3}", "ADDOP", &[], "MnSyC", 74),
  ("\\smalltriangledown", "\u{25BF}", "ADDOP", &[], "MnSyC", 75),
  ("\\filledtriangleright", "\u{25B8}", "ADDOP", &["\\blacktriangleright"], "MnSyC", 76),
  ("\\blacktriangleright", "\u{25B6}", "ADDOP", &["\\filledtriangleright"], "MnSyC", 76),
  ("\\filledtriangleup", "\u{25B4}", "ADDOP", &[], "MnSyC", 77),
  ("\\filledtriangleleft", "\u{25C2}", "ADDOP", &["\\blacktriangleleft"], "MnSyC", 78),
  ("\\blacktriangleleft", "\u{25C0}", "ADDOP", &["\\filledtriangleleft"], "MnSyC", 78),
  ("\\filledtriangledown", "\u{25BE}", "ADDOP", &[], "MnSyC", 79),
  ("\\medtriangleright", "\u{25B7}", "ADDOP", &["\\triangleright"], "MnSyC", 80),
  ("\\triangleright", "\u{25B7}", "ADDOP", &["\\medtriangleright"], "MnSyC", 80),
  ("\\medtriangleup", "\u{25B3}", "ADDOP", &["\\triangle", "\\vartriangle", "\\bigtriangleup"], "MnSyC", 81),
  ("\\triangle", "\u{25B3}", "ADDOP", &["\\medtriangleup", "\\vartriangle", "\\bigtriangleup"], "MnSyC", 81),
  ("\\vartriangle", "\u{25B5}", "ADDOP", &["\\medtriangleup", "\\triangle", "\\bigtriangleup"], "MnSyC", 81),
  ("\\bigtriangleup", "\u{25B3}", "ADDOP", &["\\medtriangleup", "\\triangle", "\\vartriangle"], "MnSyC", 81),
  ("\\medtriangleleft", "\u{25C1}", "ADDOP", &["\\triangleleft"], "MnSyC", 82),
  ("\\triangleleft", "\u{25C1}", "ADDOP", &["\\medtriangleleft"], "MnSyC", 82),
  ("\\medtriangledown", "\u{25BD}", "ADDOP", &["\\bigtriangledown", "\\triangledown"], "MnSyC", 83),
  ("\\bigtriangledown", "\u{25BD}", "ADDOP", &["\\medtriangledown", "\\triangledown"], "MnSyC", 83),
  ("\\triangledown", "\u{25BF}", "ADDOP", &["\\medtriangledown", "\\bigtriangledown"], "MnSyC", 83),
  ("\\largetriangleright", "\u{25B7}", "", &[], "MnSyC", 84),
  ("\\largetriangleup", "\u{25B3}", "", &[], "MnSyC", 85),
  ("\\largetriangleleft", "\u{25C1}", "", &[], "MnSyC", 86),
  ("\\largetriangledown", "\u{25BD}", "", &[], "MnSyC", 87),
  ("\\circ", "\u{25CB}", "MULOP", &[], "MnSyC", 88),
  ("\\bullet", "\u{25CF}", "MULOP", &[], "MnSyC", 89),
  ("\\medcircle", "\u{25EF}", "MULOP", &[], "MnSyC", 90),
  ("\\largecircle", "\u{25EF}", "", &["\\bigcirc"], "MnSyC", 91),
  ("\\bigcirc", "\u{25EF}", "", &["\\largecircle"], "MnSyC", 91),
  ("\\ominus", "\u{2296}", "ADDOP", &["\\circleddash"], "MnSyC", 92),
  ("\\circleddash", "\u{229D}", "MULOP", &["\\ominus"], "MnSyC", 92),
  ("\\overt", "\u{29B6}", "MULOP", &[], "MnSyC", 93),
  ("\\oslash", "\u{2298}", "MULOP", &[], "MnSyC", 94),
  ("\\obackslash", "\u{29B8}", "MULOP", &[], "MnSyC", 95),
  ("\\oplus", "\u{2295}", "ADDOP", &[], "MnSyC", 96),
  ("\\otimes", "\u{2297}", "MULOP", &[], "MnSyC", 97),
  ("\\odot", "\u{2299}", "MULOP", &[], "MnSyC", 98),
  ("\\ocirc", "\u{229A}", "MULOP", &["\\circledcirc"], "MnSyC", 99),
  ("\\circledcirc", "\u{229A}", "MULOP", &["\\ocirc"], "MnSyC", 99),
  ("\\otriangle", "\u{25B3}\u{20DD}", "ADDOP", &[], "MnSyC", 100),
  ("\\oast", "\u{229B}", "MULOP", &["\\circledast"], "MnSyC", 101),
  ("\\circledast", "\u{229B}", "MULOP", &["\\oast"], "MnSyC", 101),
  ("\\ostar", "\u{235F}", "MULOP", &[], "MnSyC", 102),
  ("\\diameter", "\u{2300}", "", &["\\emptyset", "\\varnothing"], "MnSyC", 103),
  ("\\emptyset", "\u{2205}", "", &["\\diameter", "\\varnothing"], "MnSyC", 103),
  ("\\varnothing", "\u{2205}", "", &["\\diameter", "\\emptyset"], "MnSyC", 103),
  ("\\smallsquare", "\u{25FD}", "MULOP", &[], "MnSyC", 104),
  ("\\filledsquare", "\u{25FE}", "MULOP", &[], "MnSyC", 105),
  ("\\medsquare", "\u{25FB}", "MULOP", &["\\square", "\\Box"], "MnSyC", 106),
  ("\\square", "\u{25FB}", "MULOP", &["\\medsquare", "\\Box"], "MnSyC", 106),
  ("\\Box", "\u{25FB}", "MULOP", &["\\medsquare", "\\square"], "MnSyC", 106),
  ("\\largesquare", "\u{25FB}", "", &[], "MnSyC", 107),
  ("\\smalldiamond", "\u{25C7}", "ADDOP", &["\\diamond"], "MnSyC", 108),
  ("\\diamond", "\u{25C7}", "ADDOP", &["\\smalldiamond"], "MnSyC", 108),
  ("\\filleddiamond", "\u{25C6}", "ADDOP", &[], "MnSyC", 109),
  ("\\meddiamond", "\u{25C7}", "ADDOP", &["\\Diamond"], "MnSyC", 110),
  ("\\Diamond", "\u{25C7}", "MULOP", &["\\meddiamond"], "MnSyC", 110),
  ("\\largediamond", "\u{25C7}", "", &[], "MnSyC", 111),
  ("\\boxminus", "\u{229F}", "ADDOP", &[], "MnSyC", 112),
  ("\\boxvert", "\u{25EB}", "MULOP", &[], "MnSyC", 113),
  ("\\boxslash", "\u{29C4}", "MULOP", &[], "MnSyC", 114),
  ("\\boxbackslash", "\u{29C5}", "MULOP", &[], "MnSyC", 115),
  ("\\boxplus", "\u{229E}", "ADDOP", &[], "MnSyC", 116),
  ("\\boxtimes", "\u{22A0}", "MULOP", &[], "MnSyC", 117),
  ("\\boxdot", "\u{22A1}", "MULOP", &[], "MnSyC", 118),
  ("\\boxbox", "\u{29C8}", "MULOP", &[], "MnSyC", 119),
  ("\\diamondminus", "\u{2212}\u{20DF}", "ADDOP", &[], "MnSyC", 120),
  ("\\diamondvert", "|\u{20DF}", "ADDOP", &[], "MnSyC", 121),
  ("\\diamondslash", "\u{2215}\u{20DF}", "ADDOP", &[], "MnSyC", 122),
  ("\\diamondbackslash", "\u{2216}\u{20DF}", "ADDOP", &[], "MnSyC", 123),
  ("\\diamondplus", "+\u{20DF}", "ADDOP", &[], "MnSyC", 124),
  ("\\diamondtimes", "\u{D7}\u{20DF}", "ADDOP", &[], "MnSyC", 125),
  ("\\diamonddot", "\u{27D0}", "ADDOP", &[], "MnSyC", 126),
  ("\\diamonddiamond", "\u{25C8}", "ADDOP", &[], "MnSyC", 127),
  ("\\smallstar", "\u{2606}", "MULOP", &[], "MnSyC", 128),
  ("\\filledstar", "\u{2605}", "MULOP", &[], "MnSyC", 129),
  ("\\medstar", "\u{2606}", "MULOP", &[], "MnSyC", 130),
  ("\\largestar", "\u{2606}", "", &[], "MnSyC", 131),
  ("\\pentagram", "\u{26E4}", "MULOP", &[], "MnSyC", 132),
  ("\\largepentagram", "\u{26E4}", "", &[], "MnSyC", 133),
  ("\\thinstar", "\u{22C6}", "MULOP", &["\\star"], "MnSyC", 134),
  ("\\star", "\u{22C6}", "MULOP", &["\\thinstar"], "MnSyC", 134),
  ("\\ast", "\u{2217}", "MULOP", &[], "MnSyC", 135),
  ("\\hbipropto", "\u{29DC}", "MULOP", &[], "MnSyC", 136),
  ("\\neswbipropto", "\u{29DC}", "MULOP", &[], "MnSyC", 137),
  ("\\vbipropto", "\u{29DC}", "MULOP", &[], "MnSyC", 138),
  ("\\nwsebipropto", "\u{29DC}", "MULOP", &[], "MnSyC", 139),
  ("\\leftpropto", "\u{221D}", "RELOP", &["\\propto", "\\varpropto"], "MnSyC", 140),
  ("\\propto", "\u{221D}", "RELOP", &["\\leftpropto", "\\varpropto"], "MnSyC", 140),
  ("\\varpropto", "\u{221D}", "RELOP", &["\\leftpropto", "\\propto"], "MnSyC", 140),
  ("\\downpropto", "\u{221D}", "RELOP", &[], "MnSyC", 141),
  ("\\rightpropto", "\u{221D}", "RELOP", &[], "MnSyC", 142),
  ("\\uppropto", "\u{221D}", "RELOP", &[], "MnSyC", 143),
  ("\\hcrossing", "\u{292B}", "RELOP", &[], "MnSyC", 144),
  ("\\neswcrossing", "\u{292B}", "RELOP", &[], "MnSyC", 145),
  ("\\vcrossing", "\u{292B}", "RELOP", &[], "MnSyC", 146),
  ("\\nwsecrossing", "\u{292B}", "RELOP", &[], "MnSyC", 147),
  ("\\between", "\u{226C}", "RELOP", &[], "MnSyC", 148),
  ("\\separated", ")(", "RELOP", &[], "MnSyC", 149),
  ("\\bot", "\u{22A5}", "", &[], "MnSyC", 150),
  ("\\top", "\u{22A4}", "", &[], "MnSyC", 151),
  ("\\intercal", "\u{22BA}", "MULOP", &[], "MnSyC", 151),
  ("\\wreath", "\u{2240}", "MULOP", &["\\wr"], "MnSyC", 152),
  ("\\wr", "\u{2240}", "MULOP", &["\\wreath"], "MnSyC", 152),
  ("\\angle", "\u{2220}", "", &[], "MnSyC", 153),
  ("\\measuredangle", "\u{2221}", "", &[], "MnSyC", 154),
  ("\\sphericalangle", "\u{2222}", "", &[], "MnSyC", 155),
  ("\\prime", "\u{2032}", "", &[], "MnSyC", 156),
  ("\\backprime", "\u{2035}", "", &[], "MnSyC", 157),
  ("\\smallprod", "\u{220F}", "MULOP", &[], "MnSyC", 158),
  ("\\amalg", "\u{2A3F}", "MULOP", &[], "MnSyC", 159),
  ("\\checkmark", "\u{2713}", "", &[], "MnSyC", 160),
  ("\\lightning", "\u{2607}", "", &[], "MnSyC", 161),
  ("\\diamondsuit", "\u{2662}", "", &[], "MnSyC", 162),
  ("\\heartsuit", "\u{2661}", "", &[], "MnSyC", 163),
  ("\\spadesuit", "\u{2660}", "", &[], "MnSyC", 164),
  ("\\clubsuit", "\u{2663}", "", &[], "MnSyC", 165),
  ("\\forall", "\u{2200}", "", &[], "MnSyC", 166),
  ("\\exists", "\u{2203}", "", &[], "MnSyC", 167),
  ("\\nexists", "\u{2204}", "", &[], "MnSyC", 168),
  ("\\nabla", "\u{2207}", "", &[], "MnSyC", 169),
  ("\\infty", "\u{221E}", "", &[], "MnSyC", 170),
  ("\\smallint", "\u{222B}", "INTOP", &[], "MnSyC", 171),
  ("\\flat", "\u{266D}", "SUMOP", &[], "MnSyC", 172),
  ("\\natural", "\u{266E}", "SUMOP", &[], "MnSyC", 173),
  ("\\sharp", "\u{266F}", "SUMOP", &[], "MnSyC", 174),
  ("\\aleph", "\u{2135}", "", &[], "MnSyC", 175),
  ("\\beth", "\u{2136}", "", &[], "MnSyC", 176),
  ("\\gimel", "\u{2137}", "", &[], "MnSyC", 177),
  ("\\daleth", "\u{2138}", "", &[], "MnSyC", 178),
  ("\\wp", "\u{2118}", "", &[], "MnSyC", 179),
  ("\\powerset", "\u{2118}", "", &[], "MnSyC", 180),
  ("\\invneg", "\u{2A3C}", "", &[], "MnSyC", 181),
  ("\\minushookup", "\u{2A3C}", "ADDOP", &[], "MnSyC", 181),
  ("\\invbackneg", "\u{2A3D}", "", &[], "MnSyC", 182),
  ("\\hookupminus", "\u{2A3D}", "ADDOP", &[], "MnSyC", 182),
  ("\\lefthalfcup", "\u{231E}", "ADDOP", &[], "MnSyC", 183),
  ("\\righthalfcup", "\u{231F}", "ADDOP", &[], "MnSyC", 184),
  ("\\lefthalfcap", "\u{231C}", "ADDOP", &[], "MnSyC", 185),
  ("\\righthalfcap", "\u{231D}", "ADDOP", &[], "MnSyC", 186),
  ("\\closedsucc", "\u{227B}", "RELOP", &[], "MnSyC", 187),
  ("\\closedcurlywedge", "\u{22CF}", "ADDOP", &[], "MnSyC", 188),
  ("\\closedprec", "\u{227A}", "RELOP", &[], "MnSyC", 189),
  ("\\closedcurlyvee", "\u{22CE}", "ADDOP", &[], "MnSyC", 190),
  ("\\rightslice", "\u{2AA7}", "RELOP", &[], "MnSyC", 191),
  ("\\upslice", "\u{2AA7}", "MULOP", &[], "MnSyC", 192),
  ("\\leftslice", "\u{2AA6}", "RELOP", &[], "MnSyC", 193),
  ("\\downslice", "\u{2AA7}", "MULOP", &[], "MnSyC", 194),
  ("\\smalllozenge", "\u{25CA}", "", &[], "MnSyC", 195),
  ("\\filledlozenge", "\u{29EB}", "", &[], "MnSyC", 196),
  ("\\medlozenge", "\u{25CA}", "", &["\\lozenge"], "MnSyC", 197),
  ("\\lozenge", "\u{25CA}", "", &["\\medlozenge"], "MnSyC", 197),
  ("\\filledmedlozenge", "\u{29EB}", "", &["\\blacklozenge"], "MnSyC", 198),
  ("\\blacklozenge", "\u{29EB}", "", &["\\filledmedlozenge"], "MnSyC", 198),
  ("\\largelozenge", "\u{25CA}", "", &[], "MnSyC", 199),
  ("\\filledmedtriangleright", "\u{25B6}", "ADDOP", &[], "MnSyC", 200),
  ("\\filledmedtriangleup", "\u{25B2}", "ADDOP", &["\\blacktriangle"], "MnSyC", 201),
  ("\\blacktriangle", "\u{25B4}", "ADDOP", &["\\filledmedtriangleup"], "MnSyC", 201),
  ("\\filledmedtriangleleft", "\u{25C0}", "ADDOP", &[], "MnSyC", 202),
  ("\\filledmedtriangledown", "\u{25BC}", "ADDOP", &["\\blacktriangledown"], "MnSyC", 203),
  ("\\blacktriangledown", "\u{25BE}", "ADDOP", &["\\filledmedtriangledown"], "MnSyC", 203),
  ("\\filledmedsquare", "\u{220E}", "MULOP", &["\\blacksquare"], "MnSyC", 204),
  ("\\blacksquare", "\u{220E}", "MULOP", &["\\filledmedsquare"], "MnSyC", 204),
  ("\\filledlargestar", "\u{2605}", "", &["\\bigstar"], "MnSyC", 205),
  ("\\bigstar", "\u{2605}", "", &["\\filledlargestar"], "MnSyC", 205),
  ("\\medstarofdavid", "\u{2721}", "", &[], "MnSyC", 206),
  ("\\largestarofdavid", "\u{2721}", "", &[], "MnSyC", 207),
  ("\\maltese", "\u{2720}", "", &[], "MnSyC", 208),
  ("\\closedequal", "=", "RELOP", &[], "MnSyC", 220),
  ("\\equalclosed", "=", "RELOP", &[], "MnSyC", 221),
  ("\\equivclosed", "\u{2261}", "RELOP", &[], "MnSyC", 222),
  ("\\lesssim", "\u{2272}", "RELOP", &[], "MnSyC", 223),
  ("\\gtrsim", "\u{2273}", "RELOP", &[], "MnSyC", 224),
  ("\\lessapprox", "\u{2A85}", "RELOP", &[], "MnSyC", 225),
  ("\\gtrapprox", "\u{2A86}", "RELOP", &[], "MnSyC", 226),
  ("\\eqslantless", "\u{2A95}", "RELOP", &[], "MnSyC", 227),
  ("\\eqslantgtr", "\u{2A96}", "RELOP", &[], "MnSyC", 228),
  ("\\curlyeqprec", "\u{22DE}", "RELOP", &[], "MnSyC", 229),
  ("\\curlyeqsucc", "\u{22DF}", "RELOP", &[], "MnSyC", 230),
  ("\\Rrightarrow", "\u{21DB}", "ARROW", &[], "MnSyC", 231),
  ("\\Lleftarrow", "\u{21DA}", "ARROW", &[], "MnSyC", 232),
  ("\\nclosedequal", "=\u{338}", "RELOP", &[], "MnSyC", 233),
  ("\\nequalclosed", "=\u{338}", "RELOP", &[], "MnSyC", 234),
  ("\\nequivclosed", "\u{2261}\u{338}", "RELOP", &[], "MnSyC", 235),
  ("\\lnsim", "\u{22E6}", "RELOP", &[], "MnSyC", 236),
  ("\\gnsim", "\u{22E7}", "RELOP", &[], "MnSyC", 237),
  ("\\lnapprox", "\u{2A89}", "RELOP", &[], "MnSyC", 238),
  ("\\gnapprox", "\u{2A8A}", "RELOP", &[], "MnSyC", 239),
  ("\\neqslantless", "\u{2A95}\u{338}", "RELOP", &[], "MnSyC", 240),
  ("\\neqslantgtr", "\u{2A96}\u{338}", "RELOP", &[], "MnSyC", 241),
  ("\\ncurlyeqprec", "\u{22DE}\u{338}", "RELOP", &[], "MnSyC", 242),
  ("\\ncurlyeqsucc", "\u{22DF}\u{338}", "RELOP", &[], "MnSyC", 243),
  ("\\nRrightarrow", "\u{21DB}\u{338}", "ARROW", &[], "MnSyC", 244),
  ("\\nLleftarrow", "\u{21DA}\u{338}", "ARROW", &[], "MnSyC", 245),
  ("\\Rsh", "\u{21B1}", "ARROW", &[], "MnSyC", 246),
  ("\\Lsh", "\u{21B0}", "ARROW", &[], "MnSyC", 247),
  ("\\looparrowright", "\u{21AC}", "ARROW", &[], "MnSyC", 248),
  ("\\looparrowleft", "\u{21AB}", "ARROW", &[], "MnSyC", 249),
  ("\\leftthreetimes", "\u{22CB}", "MULOP", &[], "MnSyC", 250),
  ("\\rightthreetimes", "\u{22CC}", "MULOP", &[], "MnSyC", 251),
  ("\\Vvdash", "\u{22AA}", "RELOP", &[], "MnSyC", 252),
  ("\\nshortmid", "\u{2224}", "RELOP", &[], "MnSyC", 253),
  ("\\shortparallel", "\u{2225}", "RELOP", &[], "MnSyC", 254),
  ("\\nshortparallel", "\u{2226}", "RELOP", &[], "MnSyC", 255),
  ("\\equal", "=", "RELOP", &[], "MnSyD", 0),
  ("\\equiv", "\u{2261}", "RELOP", &[], "MnSyD", 1),
  ("\\sim", "\u{223C}", "RELOP", &[], "MnSyD", 2),
  ("\\backsim", "\u{223D}", "RELOP", &[], "MnSyD", 3),
  ("\\approx", "\u{2248}", "RELOP", &[], "MnSyD", 4),
  ("\\backapprox", "\u{2248}", "RELOP", &[], "MnSyD", 5),
  ("\\triplesim", "\u{224B}", "RELOP", &[], "MnSyD", 6),
  ("\\backtriplesim", "\u{224B}", "RELOP", &[], "MnSyD", 7),
  ("\\simeq", "\u{2243}", "RELOP", &[], "MnSyD", 8),
  ("\\backsimeq", "\u{22CD}", "RELOP", &[], "MnSyD", 9),
  ("\\eqsim", "\u{2242}", "RELOP", &[], "MnSyD", 10),
  ("\\backeqsim", "\u{2242}", "RELOP", &[], "MnSyD", 11),
  ("\\cong", "\u{2245}", "RELOP", &[], "MnSyD", 12),
  ("\\backcong", "\u{224C}", "RELOP", &[], "MnSyD", 13),
  ("\\approxeq", "\u{224A}", "RELOP", &[], "MnSyD", 14),
  ("\\backapproxeq", "\u{224A}", "RELOP", &[], "MnSyD", 15),
  ("\\bumpeq", "\u{224F}", "RELOP", &[], "MnSyD", 16),
  ("\\eqbump", "\u{224F}", "RELOP", &[], "MnSyD", 17),
  ("\\Bumpeq", "\u{224E}", "RELOP", &[], "MnSyD", 18),
  ("\\doteq", "\u{2250}", "RELOP", &[], "MnSyD", 19),
  ("\\eqdot", "\u{2A66}", "RELOP", &[], "MnSyD", 20),
  ("\\Doteq", "\u{2251}", "RELOP", &[], "MnSyD", 21),
  ("\\doteqdot", "\u{2251}", "MULOP", &[], "MnSyD", 21),
  ("\\fallingdotseq", "\u{2252}", "RELOP", &[], "MnSyD", 22),
  ("\\risingdotseq", "\u{2253}", "RELOP", &[], "MnSyD", 23),
  ("\\smile", "\u{2323}", "RELOP", &["\\smallsmile"], "MnSyD", 24),
  ("\\smallsmile", "\u{2323}", "RELOP", &["\\smile"], "MnSyD", 24),
  ("\\frown", "\u{2322}", "RELOP", &["\\smallfrown"], "MnSyD", 25),
  ("\\smallfrown", "\u{2322}", "RELOP", &["\\frown"], "MnSyD", 25),
  ("\\doublesmile", "\u{2323}", "RELOP", &[], "MnSyD", 26),
  ("\\doublefrown", "\u{2322}", "RELOP", &[], "MnSyD", 27),
  ("\\triplesmile", "\u{2323}", "RELOP", &[], "MnSyD", 28),
  ("\\triplefrown", "\u{2322}", "RELOP", &[], "MnSyD", 29),
  ("\\smilefrown", "\u{224D}", "RELOP", &["\\asymp"], "MnSyD", 30),
  ("\\asymp", "\u{224D}", "RELOP", &["\\smilefrown"], "MnSyD", 30),
  ("\\frownsmile", "\u{2322}", "RELOP", &[], "MnSyD", 31),
  ("\\smileeq", "\u{2323}", "RELOP", &[], "MnSyD", 32),
  ("\\frowneq", "\u{2322}", "RELOP", &[], "MnSyD", 33),
  ("\\eqsmile", "\u{2323}", "RELOP", &[], "MnSyD", 34),
  ("\\eqfrown", "\u{2322}", "RELOP", &[], "MnSyD", 35),
  ("\\doublesmileeq", "\u{2323}", "RELOP", &[], "MnSyD", 36),
  ("\\doublefrowneq", "\u{2322}", "RELOP", &[], "MnSyD", 37),
  ("\\smileeqfrown", "\u{2323}", "RELOP", &[], "MnSyD", 38),
  ("\\frowneqsmile", "\u{2322}", "RELOP", &[], "MnSyD", 39),
  ("\\smilefrowneq", "\u{2323}", "RELOP", &[], "MnSyD", 40),
  ("\\frownsmileeq", "\u{2322}", "RELOP", &[], "MnSyD", 41),
  ("\\sqsmile", "\u{2323}", "RELOP", &[], "MnSyD", 42),
  ("\\sqfrown", "\u{2322}", "RELOP", &[], "MnSyD", 43),
  ("\\sqdoublesmile", "\u{2323}", "RELOP", &[], "MnSyD", 44),
  ("\\sqdoublefrown", "\u{2322}", "RELOP", &[], "MnSyD", 45),
  ("\\sqtriplesmile", "\u{2323}", "RELOP", &[], "MnSyD", 46),
  ("\\sqtriplefrown", "\u{2322}", "RELOP", &[], "MnSyD", 47),
  ("\\sqsmilefrown", "\u{2323}", "RELOP", &[], "MnSyD", 48),
  ("\\sqfrownsmile", "\u{2322}", "RELOP", &[], "MnSyD", 49),
  ("\\sqsmileeq", "\u{2323}", "RELOP", &[], "MnSyD", 50),
  ("\\sqfrowneq", "\u{2322}", "RELOP", &[], "MnSyD", 51),
  ("\\sqeqsmile", "\u{2323}", "RELOP", &[], "MnSyD", 52),
  ("\\sqeqfrown", "\u{2322}", "RELOP", &[], "MnSyD", 53),
  ("\\sqdoublesmileeq", "\u{2323}", "RELOP", &[], "MnSyD", 54),
  ("\\sqdoublefrowneq", "\u{2322}", "RELOP", &[], "MnSyD", 55),
  ("\\sqsmileeqfrown", "\u{2323}", "RELOP", &[], "MnSyD", 56),
  ("\\sqfrowneqsmile", "\u{2322}", "RELOP", &[], "MnSyD", 57),
  ("\\eqcirc", "\u{2256}", "RELOP", &[], "MnSyD", 58),
  ("\\circeq", "\u{2257}", "RELOP", &[], "MnSyD", 59),
  ("\\triangleeq", "\u{225C}", "RELOP", &["\\triangleq"], "MnSyD", 60),
  ("\\triangleq", "\u{225C}", "RELOP", &["\\triangleeq"], "MnSyD", 60),
  ("\\hateq", "\u{2259}", "RELOP", &[], "MnSyD", 61),
  ("\\in", "\u{2208}", "RELOP", &[], "MnSyD", 62),
  ("\\owns", "\u{220B}", "RELOP", &["\\ni"], "MnSyD", 63),
  ("\\ni", "\u{220B}", "RELOP", &["\\owns"], "MnSyD", 63),
  ("\\less", "<", "RELOP", &[], "MnSyD", 64),
  ("\\gtr", ">", "RELOP", &[], "MnSyD", 65),
  ("\\leq", "\u{2264}", "RELOP", &["\\le"], "MnSyD", 66),
  ("\\le", "\u{2264}", "RELOP", &["\\leq"], "MnSyD", 66),
  ("\\geq", "\u{2265}", "RELOP", &["\\ge"], "MnSyD", 67),
  ("\\ge", "\u{2265}", "RELOP", &["\\geq"], "MnSyD", 67),
  ("\\leqslant", "\u{2A7D}", "RELOP", &[], "MnSyD", 68),
  ("\\geqslant", "\u{2A7E}", "RELOP", &[], "MnSyD", 69),
  ("\\leqq", "\u{2266}", "RELOP", &[], "MnSyD", 70),
  ("\\geqq", "\u{2267}", "RELOP", &[], "MnSyD", 71),
  ("\\lessgtr", "\u{2276}", "RELOP", &[], "MnSyD", 72),
  ("\\gtrless", "\u{2277}", "RELOP", &[], "MnSyD", 73),
  ("\\lesseqgtr", "\u{22DA}", "RELOP", &[], "MnSyD", 74),
  ("\\gtreqless", "\u{22DB}", "RELOP", &[], "MnSyD", 75),
  ("\\lesseqqgtr", "\u{2A8B}", "RELOP", &[], "MnSyD", 76),
  ("\\gtreqqless", "\u{2A8C}", "RELOP", &[], "MnSyD", 77),
  ("\\lesseqgtrslant", "\u{22DA}", "RELOP", &[], "MnSyD", 78),
  ("\\gtreqlessslant", "\u{22DB}", "RELOP", &[], "MnSyD", 79),
  ("\\ll", "\u{226A}", "RELOP", &[], "MnSyD", 80),
  ("\\gg", "\u{226B}", "RELOP", &[], "MnSyD", 81),
  ("\\lll", "\u{22D8}", "RELOP", &["\\llless"], "MnSyD", 82),
  ("\\llless", "\u{22D8}", "RELOP", &["\\lll"], "MnSyD", 82),
  ("\\ggg", "\u{22D9}", "RELOP", &["\\gggtr"], "MnSyD", 83),
  ("\\gggtr", "\u{22D9}", "RELOP", &["\\ggg"], "MnSyD", 83),
  ("\\lessclosed", "\u{22B2}", "RELOP", &["\\lhd", "\\vartriangleleft"], "MnSyD", 84),
  ("\\lhd", "\u{22B2}", "RELOP", &["\\lessclosed", "\\vartriangleleft"], "MnSyD", 84),
  ("\\vartriangleleft", "\u{22B2}", "RELOP", &["\\lessclosed", "\\lhd"], "MnSyD", 84),
  ("\\gtrclosed", "\u{22B3}", "RELOP", &["\\rhd", "\\vartriangleright"], "MnSyD", 85),
  ("\\rhd", "\u{22B3}", "RELOP", &["\\gtrclosed", "\\vartriangleright"], "MnSyD", 85),
  ("\\vartriangleright", "\u{22B3}", "RELOP", &["\\gtrclosed", "\\rhd"], "MnSyD", 85),
  ("\\leqclosed", "\u{22B4}", "RELOP", &["\\unlhd", "\\trianglelefteq"], "MnSyD", 86),
  ("\\unlhd", "\u{22B4}", "RELOP", &["\\leqclosed", "\\trianglelefteq"], "MnSyD", 86),
  ("\\trianglelefteq", "\u{22B4}", "RELOP", &["\\leqclosed", "\\unlhd"], "MnSyD", 86),
  ("\\geqclosed", "\u{22B5}", "RELOP", &["\\unrhd", "\\trianglerighteq"], "MnSyD", 87),
  ("\\unrhd", "\u{22B5}", "RELOP", &["\\geqclosed", "\\trianglerighteq"], "MnSyD", 87),
  ("\\trianglerighteq", "\u{22B5}", "RELOP", &["\\geqclosed", "\\unrhd"], "MnSyD", 87),
  ("\\sqsubset", "\u{228F}", "RELOP", &[], "MnSyD", 88),
  ("\\sqsupset", "\u{2290}", "RELOP", &[], "MnSyD", 89),
  ("\\sqsubseteq", "\u{2291}", "RELOP", &[], "MnSyD", 90),
  ("\\sqsupseteq", "\u{2292}", "RELOP", &[], "MnSyD", 91),
  ("\\sqsubseteqq", "\u{2291}", "RELOP", &[], "MnSyD", 92),
  ("\\sqsupseteqq", "\u{2292}", "RELOP", &[], "MnSyD", 93),
  ("\\Sqsubset", "\u{228F}", "RELOP", &[], "MnSyD", 94),
  ("\\Sqsupset", "\u{2290}", "RELOP", &[], "MnSyD", 95),
  ("\\subset", "\u{2282}", "RELOP", &[], "MnSyD", 96),
  ("\\supset", "\u{2283}", "RELOP", &[], "MnSyD", 97),
  ("\\subseteq", "\u{2286}", "RELOP", &[], "MnSyD", 98),
  ("\\supseteq", "\u{2287}", "RELOP", &[], "MnSyD", 99),
  ("\\subseteqq", "\u{2AC5}", "RELOP", &[], "MnSyD", 100),
  ("\\supseteqq", "\u{2AC6}", "RELOP", &[], "MnSyD", 101),
  ("\\Subset", "\u{22D0}", "RELOP", &[], "MnSyD", 102),
  ("\\Supset", "\u{22D1}", "RELOP", &[], "MnSyD", 103),
  ("\\prec", "\u{227A}", "RELOP", &[], "MnSyD", 104),
  ("\\succ", "\u{227B}", "RELOP", &[], "MnSyD", 105),
  ("\\preceq", "\u{2AAF}", "RELOP", &[], "MnSyD", 106),
  ("\\succeq", "\u{2AB0}", "RELOP", &[], "MnSyD", 107),
  ("\\preccurlyeq", "\u{227C}", "RELOP", &[], "MnSyD", 108),
  ("\\succcurlyeq", "\u{227D}", "RELOP", &[], "MnSyD", 109),
  ("\\precsim", "\u{227E}", "RELOP", &[], "MnSyD", 110),
  ("\\succsim", "\u{227F}", "RELOP", &[], "MnSyD", 111),
  ("\\precapprox", "\u{2AB7}", "RELOP", &[], "MnSyD", 112),
  ("\\succapprox", "\u{2AB8}", "RELOP", &[], "MnSyD", 113),
  ("\\lessdot", "\u{22D6}", "RELOP", &[], "MnSyD", 114),
  ("\\gtrdot", "\u{22D7}", "RELOP", &[], "MnSyD", 115),
  ("\\leqdot", "\u{2A7F}", "RELOP", &[], "MnSyD", 116),
  ("\\geqdot", "\u{2A80}", "RELOP", &[], "MnSyD", 117),
  ("\\leqslantdot", "\u{2A7F}", "RELOP", &[], "MnSyD", 118),
  ("\\geqslantdot", "\u{2A80}", "RELOP", &[], "MnSyD", 119),
  ("\\nequal", "\u{2260}", "RELOP", &["\\neq", "\\ne"], "MnSyD", 120),
  ("\\neq", "\u{2260}", "RELOP", &["\\nequal", "\\ne"], "MnSyD", 120),
  ("\\ne", "\u{2260}", "RELOP", &["\\nequal", "\\neq"], "MnSyD", 120),
  ("\\nequiv", "\u{2262}", "RELOP", &[], "MnSyD", 121),
  ("\\nsim", "\u{2241}", "RELOP", &[], "MnSyD", 122),
  ("\\nbacksim", "\u{223D}\u{338}", "RELOP", &[], "MnSyD", 123),
  ("\\napprox", "\u{2249}", "RELOP", &[], "MnSyD", 124),
  ("\\nbackapprox", "\u{2248}\u{338}", "RELOP", &[], "MnSyD", 125),
  ("\\ntriplesim", "\u{224B}\u{338}", "RELOP", &[], "MnSyD", 126),
  ("\\nbacktriplesim", "\u{224B}\u{338}", "RELOP", &[], "MnSyD", 127),
  ("\\nsimeq", "\u{2244}", "RELOP", &[], "MnSyD", 128),
  ("\\nbacksimeq", "\u{22CD}\u{338}", "RELOP", &[], "MnSyD", 129),
  ("\\neqsim", "\u{2242}\u{338}", "RELOP", &[], "MnSyD", 130),
  ("\\nbackeqsim", "\u{2242}\u{338}", "RELOP", &[], "MnSyD", 131),
  ("\\ncong", "\u{2247}", "RELOP", &[], "MnSyD", 132),
  ("\\nbackcong", "\u{224C}\u{338}", "RELOP", &[], "MnSyD", 133),
  ("\\napproxeq", "\u{224A}\u{338}", "RELOP", &[], "MnSyD", 134),
  ("\\nbackapproxeq", "\u{224A}\u{338}", "RELOP", &[], "MnSyD", 135),
  ("\\nbumpeq", "\u{224F}\u{338}", "RELOP", &[], "MnSyD", 136),
  ("\\neqbump", "\u{224F}\u{338}", "RELOP", &[], "MnSyD", 137),
  ("\\nBumpeq", "\u{224E}\u{338}", "RELOP", &[], "MnSyD", 138),
  ("\\ndoteq", "\u{2250}\u{338}", "RELOP", &[], "MnSyD", 139),
  ("\\neqdot", "\u{2A66}\u{338}", "RELOP", &[], "MnSyD", 140),
  ("\\nDoteq", "\u{2251}\u{338}", "RELOP", &[], "MnSyD", 141),
  ("\\nfallingdotseq", "\u{2252}\u{338}", "RELOP", &[], "MnSyD", 142),
  ("\\nrisingdotseq", "\u{2253}\u{338}", "RELOP", &[], "MnSyD", 143),
  ("\\nsmile", "\u{2323}\u{338}", "RELOP", &[], "MnSyD", 144),
  ("\\nfrown", "\u{2322}\u{338}", "RELOP", &[], "MnSyD", 145),
  ("\\ndoublesmile", "\u{2323}\u{338}", "RELOP", &[], "MnSyD", 146),
  ("\\ndoublefrown", "\u{2322}\u{338}", "RELOP", &[], "MnSyD", 147),
  ("\\ntriplesmile", "\u{2323}\u{338}", "RELOP", &[], "MnSyD", 148),
  ("\\ntriplefrown", "\u{2322}\u{338}", "RELOP", &[], "MnSyD", 149),
  ("\\nsmilefrown", "\u{226D}", "RELOP", &["\\nasymp"], "MnSyD", 150),
  ("\\nasymp", "\u{226D}", "RELOP", &["\\nsmilefrown"], "MnSyD", 150),
  ("\\nfrownsmile", "\u{2322}\u{338}", "RELOP", &[], "MnSyD", 151),
  ("\\nsmileeq", "\u{2323}\u{338}", "RELOP", &[], "MnSyD", 152),
  ("\\nfrowneq", "\u{2322}\u{338}", "RELOP", &[], "MnSyD", 153),
  ("\\neqsmile", "\u{2323}\u{338}", "RELOP", &[], "MnSyD", 154),
  ("\\neqfrown", "\u{2322}\u{338}", "RELOP", &[], "MnSyD", 155),
  ("\\ndoublesmileeq", "\u{2323}\u{338}", "RELOP", &[], "MnSyD", 156),
  ("\\ndoublefrowneq", "\u{2322}\u{338}", "RELOP", &[], "MnSyD", 157),
  ("\\nsmileeqfrown", "\u{2323}\u{338}", "RELOP", &[], "MnSyD", 158),
  ("\\nfrowneqsmile", "\u{2322}\u{338}", "RELOP", &[], "MnSyD", 159),
  ("\\nsmilefrowneq", "\u{2323}\u{338}", "RELOP", &[], "MnSyD", 160),
  ("\\nfrownsmileeq", "\u{2322}\u{338}", "RELOP", &[], "MnSyD", 161),
  ("\\nsqsmile", "\u{2323}\u{338}", "RELOP", &[], "MnSyD", 162),
  ("\\nsqfrown", "\u{2322}\u{338}", "RELOP", &[], "MnSyD", 163),
  ("\\nsqdoublesmile", "\u{2323}\u{338}", "RELOP", &[], "MnSyD", 164),
  ("\\nsqdoublefrown", "\u{2322}\u{338}", "RELOP", &[], "MnSyD", 165),
  ("\\nsqtriplesmile", "\u{2323}\u{338}", "RELOP", &[], "MnSyD", 166),
  ("\\nsqtriplefrown", "\u{2322}\u{338}", "RELOP", &[], "MnSyD", 167),
  ("\\nsqsmilefrown", "\u{2323}\u{338}", "RELOP", &[], "MnSyD", 168),
  ("\\nsqfrownsmile", "\u{2322}\u{338}", "RELOP", &[], "MnSyD", 169),
  ("\\nsqsmileeq", "\u{2323}\u{338}", "RELOP", &[], "MnSyD", 170),
  ("\\nsqfrowneq", "\u{2322}\u{338}", "RELOP", &[], "MnSyD", 171),
  ("\\nsqeqsmile", "\u{2323}\u{338}", "RELOP", &[], "MnSyD", 172),
  ("\\nsqeqfrown", "\u{2322}\u{338}", "RELOP", &[], "MnSyD", 173),
  ("\\nsqdoublesmileeq", "\u{2323}\u{338}", "RELOP", &[], "MnSyD", 174),
  ("\\nsqdoublefrowneq", "\u{2322}\u{338}", "RELOP", &[], "MnSyD", 175),
  ("\\nsqsmileeqfrown", "\u{2323}\u{338}", "RELOP", &[], "MnSyD", 176),
  ("\\nsqfrowneqsmile", "\u{2322}\u{338}", "RELOP", &[], "MnSyD", 177),
  ("\\neqcirc", "\u{2256}\u{338}", "RELOP", &[], "MnSyD", 178),
  ("\\ncirceq", "\u{2257}\u{338}", "RELOP", &[], "MnSyD", 179),
  ("\\ntriangleeq", "\u{225C}\u{338}", "RELOP", &[], "MnSyD", 180),
  ("\\nhateq", "\u{2259}\u{338}", "RELOP", &[], "MnSyD", 181),
  ("\\nin", "\u{2209}", "RELOP", &["\\notin"], "MnSyD", 182),
  ("\\notin", "\u{2209}", "RELOP", &["\\nin"], "MnSyD", 182),
  ("\\nowns", "\u{220C}", "RELOP", &[], "MnSyD", 183),
  ("\\nless", "\u{226E}", "RELOP", &[], "MnSyD", 184),
  ("\\ngtr", "\u{226F}", "RELOP", &[], "MnSyD", 185),
  ("\\nleq", "\u{2270}", "RELOP", &[], "MnSyD", 186),
  ("\\ngeq", "\u{2271}", "RELOP", &[], "MnSyD", 187),
  ("\\nleqslant", "\u{2270}", "RELOP", &[], "MnSyD", 188),
  ("\\ngeqslant", "\u{2271}", "RELOP", &[], "MnSyD", 189),
  ("\\nleqq", "\u{2266}\u{338}", "RELOP", &[], "MnSyD", 190),
  ("\\ngeqq", "\u{2267}\u{338}", "RELOP", &[], "MnSyD", 191),
  ("\\nlessgtr", "\u{2278}", "RELOP", &[], "MnSyD", 192),
  ("\\ngtrless", "\u{2279}", "RELOP", &[], "MnSyD", 193),
  ("\\nlesseqgtr", "\u{22DA}\u{338}", "RELOP", &[], "MnSyD", 194),
  ("\\ngtreqless", "\u{22DB}\u{338}", "RELOP", &[], "MnSyD", 195),
  ("\\nlesseqqgtr", "\u{2A8B}\u{338}", "RELOP", &[], "MnSyD", 196),
  ("\\ngtreqqless", "\u{2A8C}\u{338}", "RELOP", &[], "MnSyD", 197),
  ("\\nlesseqgtrslant", "\u{22DA}\u{338}", "RELOP", &[], "MnSyD", 198),
  ("\\ngtreqlessslant", "\u{22DB}\u{338}", "RELOP", &[], "MnSyD", 199),
  ("\\nll", "\u{226A}\u{338}", "RELOP", &[], "MnSyD", 200),
  ("\\ngg", "\u{226B}\u{338}", "RELOP", &[], "MnSyD", 201),
  ("\\nlll", "\u{22D8}\u{338}", "RELOP", &[], "MnSyD", 202),
  ("\\nggg", "\u{22D9}\u{338}", "RELOP", &[], "MnSyD", 203),
  ("\\nlessclosed", "\u{22EA}", "RELOP", &["\\ntriangleleft"], "MnSyD", 204),
  ("\\ntriangleleft", "\u{22EA}", "RELOP", &["\\nlessclosed"], "MnSyD", 204),
  ("\\ngtrclosed", "\u{22EB}", "RELOP", &["\\ntriangleright"], "MnSyD", 205),
  ("\\ntriangleright", "\u{22EB}", "RELOP", &["\\ngtrclosed"], "MnSyD", 205),
  ("\\nleqclosed", "\u{22EC}", "RELOP", &["\\ntrianglelefteq"], "MnSyD", 206),
  ("\\ntrianglelefteq", "\u{22EC}", "RELOP", &["\\nleqclosed"], "MnSyD", 206),
  ("\\ngeqclosed", "\u{22ED}", "RELOP", &["\\ntrianglerighteq"], "MnSyD", 207),
  ("\\ntrianglerighteq", "\u{22ED}", "RELOP", &["\\ngeqclosed"], "MnSyD", 207),
  ("\\nsqsubset", "\u{228F}\u{338}", "RELOP", &[], "MnSyD", 208),
  ("\\nsqsupset", "\u{2290}\u{338}", "RELOP", &[], "MnSyD", 209),
  ("\\nsqsubseteq", "\u{22E2}", "RELOP", &[], "MnSyD", 210),
  ("\\nsqsupseteq", "\u{22E3}", "RELOP", &[], "MnSyD", 211),
  ("\\nsqsubseteqq", "\u{2291}\u{338}", "RELOP", &[], "MnSyD", 212),
  ("\\nsqsupseteqq", "\u{2292}\u{338}", "RELOP", &[], "MnSyD", 213),
  ("\\nSqsubset", "\u{228F}\u{338}", "RELOP", &[], "MnSyD", 214),
  ("\\nSqsupset", "\u{2290}\u{338}", "RELOP", &[], "MnSyD", 215),
  ("\\nsubset", "\u{2284}", "RELOP", &[], "MnSyD", 216),
  ("\\nsupset", "\u{2285}", "RELOP", &[], "MnSyD", 217),
  ("\\nsubseteq", "\u{2288}", "RELOP", &[], "MnSyD", 218),
  ("\\nsupseteq", "\u{2289}", "RELOP", &[], "MnSyD", 219),
  ("\\nsubseteqq", "\u{2AC5}\u{338}", "RELOP", &[], "MnSyD", 220),
  ("\\nsupseteqq", "\u{2AC6}\u{338}", "RELOP", &[], "MnSyD", 221),
  ("\\nSubset", "\u{22D0}\u{338}", "RELOP", &[], "MnSyD", 222),
  ("\\nSupset", "\u{22D1}\u{338}", "RELOP", &[], "MnSyD", 223),
  ("\\nprec", "\u{2280}", "RELOP", &[], "MnSyD", 224),
  ("\\nsucc", "\u{2281}", "RELOP", &[], "MnSyD", 225),
  ("\\npreceq", "\u{2AAF}\u{338}", "RELOP", &[], "MnSyD", 226),
  ("\\nsucceq", "\u{2AB0}\u{338}", "RELOP", &[], "MnSyD", 227),
  ("\\npreccurlyeq", "\u{22E0}", "RELOP", &[], "MnSyD", 228),
  ("\\nsucccurlyeq", "\u{22E1}", "RELOP", &[], "MnSyD", 229),
  ("\\nprecsim", "\u{227E}\u{338}", "RELOP", &[], "MnSyD", 230),
  ("\\nsuccsim", "\u{227F}\u{338}", "RELOP", &[], "MnSyD", 231),
  ("\\nprecapprox", "\u{2AB7}\u{338}", "RELOP", &[], "MnSyD", 232),
  ("\\nsuccapprox", "\u{2AB8}\u{338}", "RELOP", &[], "MnSyD", 233),
  ("\\nlessdot", "\u{22D6}\u{338}", "RELOP", &[], "MnSyD", 234),
  ("\\ngtrdot", "\u{22D7}\u{338}", "RELOP", &[], "MnSyD", 235),
  ("\\nleqdot", "\u{2A7F}\u{338}", "RELOP", &[], "MnSyD", 236),
  ("\\ngeqdot", "\u{2A80}\u{338}", "RELOP", &[], "MnSyD", 237),
  ("\\nleqslantdot", "\u{2A7F}\u{338}", "RELOP", &[], "MnSyD", 238),
  ("\\ngeqslantdot", "\u{2A80}\u{338}", "RELOP", &[], "MnSyD", 239),
  ("\\lneqq", "\u{2268}", "RELOP", &["\\lvertneqq"], "MnSyD", 240),
  ("\\lvertneqq", "\u{2268}", "RELOP", &["\\lneqq"], "MnSyD", 240),
  ("\\gneqq", "\u{2269}", "RELOP", &["\\gvertneqq"], "MnSyD", 241),
  ("\\gvertneqq", "\u{2269}", "RELOP", &["\\gneqq"], "MnSyD", 241),
  ("\\lessneqqgtr", "\u{2A8B}\u{338}", "RELOP", &[], "MnSyD", 242),
  ("\\gtrneqqless", "\u{2A8C}\u{338}", "RELOP", &[], "MnSyD", 243),
  ("\\sqsubsetneq", "\u{22E4}", "RELOP", &[], "MnSyD", 244),
  ("\\sqsupsetneq", "\u{22E5}", "RELOP", &[], "MnSyD", 245),
  ("\\sqsubsetneqq", "\u{22E4}", "RELOP", &[], "MnSyD", 246),
  ("\\sqsupsetneqq", "\u{22E5}", "RELOP", &[], "MnSyD", 247),
  ("\\subsetneq", "\u{228A}", "RELOP", &["\\varsubsetneq"], "MnSyD", 248),
  ("\\varsubsetneq", "\u{228A}", "RELOP", &["\\subsetneq"], "MnSyD", 248),
  ("\\supsetneq", "\u{228B}", "RELOP", &["\\varsupsetneq"], "MnSyD", 249),
  ("\\varsupsetneq", "\u{228B}", "RELOP", &["\\supsetneq"], "MnSyD", 249),
  ("\\subsetneqq", "\u{2ACB}", "RELOP", &["\\varsubsetneqq"], "MnSyD", 250),
  ("\\varsubsetneqq", "\u{2ACB}", "RELOP", &["\\subsetneqq"], "MnSyD", 250),
  ("\\supsetneqq", "\u{2ACC}", "RELOP", &["\\varsupsetneqq"], "MnSyD", 251),
  ("\\varsupsetneqq", "\u{2ACC}", "RELOP", &["\\supsetneqq"], "MnSyD", 251),
  ("\\precnsim", "\u{22E8}", "RELOP", &[], "MnSyD", 252),
  ("\\succnsim", "\u{22E9}", "RELOP", &[], "MnSyD", 253),
  ("\\precnapprox", "\u{2AB9}", "RELOP", &[], "MnSyD", 254),
  ("\\succnapprox", "\u{2ABA}", "RELOP", &[], "MnSyD", 255),
];
#[rustfmt::skip]
static MN_SIZED_OPS: &[MnSizedOp] = &[
  ("\\tbigplus", "+", "SUMOP", "text", 0),
  ("\\dbigplus", "+", "SUMOP", "display", 1),
  ("\\tbigtimes", "\u{2A09}", "SUMOP", "text", 2),
  ("\\dbigtimes", "\u{2A09}", "SUMOP", "display", 3),
  ("\\tbigwedge", "\u{22C0}", "SUMOP", "text", 4),
  ("\\dbigwedge", "\u{22C0}", "SUMOP", "display", 5),
  ("\\tbigvee", "\u{22C1}", "SUMOP", "text", 6),
  ("\\dbigvee", "\u{22C1}", "SUMOP", "display", 7),
  ("\\tbigwedgedot", "\u{27D1}", "SUMOP", "text", 8),
  ("\\dbigwedgedot", "\u{27D1}", "SUMOP", "display", 9),
  ("\\tbigveedot", "\u{27C7}", "SUMOP", "text", 10),
  ("\\dbigveedot", "\u{27C7}", "SUMOP", "display", 11),
  ("\\tbigdoublewedge", "\u{2A55}", "SUMOP", "text", 12),
  ("\\dbigdoublewedge", "\u{2A55}", "SUMOP", "display", 13),
  ("\\tbigdoublevee", "\u{2A54}", "SUMOP", "text", 14),
  ("\\dbigdoublevee", "\u{2A54}", "SUMOP", "display", 15),
  ("\\tbigcurlywedge", "\u{22CF}", "SUMOP", "text", 16),
  ("\\dbigcurlywedge", "\u{22CF}", "SUMOP", "display", 17),
  ("\\tbigcurlyvee", "\u{22CE}", "SUMOP", "text", 18),
  ("\\dbigcurlyvee", "\u{22CE}", "SUMOP", "display", 19),
  ("\\tbigcurlywedgedot", "\u{22CF}", "SUMOP", "text", 20),
  ("\\dbigcurlywedgedot", "\u{22CF}", "SUMOP", "display", 21),
  ("\\tbigcurlyveedot", "\u{22CE}", "SUMOP", "text", 22),
  ("\\dbigcurlyveedot", "\u{22CE}", "SUMOP", "display", 23),
  ("\\tbigdoublecurlywedge", "\u{22CF}", "SUMOP", "text", 24),
  ("\\dbigdoublecurlywedge", "\u{22CF}", "SUMOP", "display", 25),
  ("\\tbigdoublecurlyvee", "\u{22CE}", "SUMOP", "text", 26),
  ("\\dbigdoublecurlyvee", "\u{22CE}", "SUMOP", "display", 27),
  ("\\tbigcap", "\u{22C2}", "SUMOP", "text", 28),
  ("\\dbigcap", "\u{22C2}", "SUMOP", "display", 29),
  ("\\tbigcup", "\u{22C3}", "SUMOP", "text", 30),
  ("\\dbigcup", "\u{22C3}", "SUMOP", "display", 31),
  ("\\tbigcapdot", "\u{2A40}", "SUMOP", "text", 32),
  ("\\dbigcapdot", "\u{2A40}", "SUMOP", "display", 33),
  ("\\tbigcupdot", "\u{2A03}", "SUMOP", "text", 34),
  ("\\dbigcupdot", "\u{2A03}", "SUMOP", "display", 35),
  ("\\tbigcapplus", "\u{22C2}", "SUMOP", "text", 36),
  ("\\dbigcapplus", "\u{22C2}", "SUMOP", "display", 37),
  ("\\tbigcupplus", "\u{2A04}", "SUMOP", "text", 38),
  ("\\dbigcupplus", "\u{2A04}", "SUMOP", "display", 39),
  ("\\tbigsqcap", "\u{2A05}", "SUMOP", "text", 40),
  ("\\dbigsqcap", "\u{2A05}", "SUMOP", "display", 41),
  ("\\tbigsqcup", "\u{2A06}", "SUMOP", "text", 42),
  ("\\dbigsqcup", "\u{2A06}", "SUMOP", "display", 43),
  ("\\tbigsqcapdot", "\u{2A05}", "SUMOP", "text", 44),
  ("\\dbigsqcapdot", "\u{2A05}", "SUMOP", "display", 45),
  ("\\tbigsqcupdot", "\u{2A06}", "SUMOP", "text", 46),
  ("\\dbigsqcupdot", "\u{2A06}", "SUMOP", "display", 47),
  ("\\tbigsqcapplus", "\u{2A05}", "SUMOP", "text", 48),
  ("\\dbigsqcapplus", "\u{2A05}", "SUMOP", "display", 49),
  ("\\tbigsqcupplus", "\u{2A06}", "SUMOP", "text", 50),
  ("\\dbigsqcupplus", "\u{2A06}", "SUMOP", "display", 51),
  ("\\tbigcircle", "\u{25EF}", "SUMOP", "text", 52),
  ("\\dbigcircle", "\u{25EF}", "SUMOP", "display", 53),
  ("\\tbigominus", "\u{2296}", "SUMOP", "text", 54),
  ("\\dbigominus", "\u{2296}", "SUMOP", "display", 55),
  ("\\tbigovert", "\u{29B6}", "SUMOP", "text", 56),
  ("\\dbigovert", "\u{29B6}", "SUMOP", "display", 57),
  ("\\tbigoslash", "\u{2298}", "SUMOP", "text", 58),
  ("\\dbigoslash", "\u{2298}", "SUMOP", "display", 59),
  ("\\tbigobackslash", "\u{29B8}", "SUMOP", "text", 60),
  ("\\dbigobackslash", "\u{29B8}", "SUMOP", "display", 61),
  ("\\tbigoplus", "\u{2A01}", "SUMOP", "text", 62),
  ("\\dbigoplus", "\u{2A01}", "SUMOP", "display", 63),
  ("\\tbigotimes", "\u{2A02}", "SUMOP", "text", 64),
  ("\\dbigotimes", "\u{2A02}", "SUMOP", "display", 65),
  ("\\tbigodot", "\u{2A00}", "SUMOP", "text", 66),
  ("\\dbigodot", "\u{2A00}", "SUMOP", "display", 67),
  ("\\tbigocirc", "\u{229A}", "SUMOP", "text", 68),
  ("\\dbigocirc", "\u{229A}", "SUMOP", "display", 69),
  ("\\tbigotriangle", "\u{25B3}\u{20DD}", "SUMOP", "text", 70),
  ("\\dbigotriangle", "\u{25B3}\u{20DD}", "SUMOP", "display", 71),
  ("\\tbigoast", "\u{229B}", "SUMOP", "text", 72),
  ("\\dbigoast", "\u{229B}", "SUMOP", "display", 73),
  ("\\tbigostar", "\u{235F}", "SUMOP", "text", 74),
  ("\\dbigostar", "\u{235F}", "SUMOP", "display", 75),
  ("\\tprod", "\u{220F}", "SUMOP", "text", 76),
  ("\\dprod", "\u{220F}", "SUMOP", "display", 77),
  ("\\tcoprod", "\u{2210}", "SUMOP", "text", 78),
  ("\\dcoprod", "\u{2210}", "SUMOP", "display", 79),
  ("\\tsum", "\u{2211}", "SUMOP", "text", 80),
  ("\\dsum", "\u{2211}", "SUMOP", "display", 81),
  ("\\tint", "\u{222B}", "INTOP", "text", 82),
  ("\\dint", "\u{222B}", "INTOP", "display", 83),
  ("\\tiint", "\u{222C}", "INTOP", "text", 84),
  ("\\diint", "\u{222C}", "INTOP", "display", 85),
  ("\\tiiint", "\u{222D}", "INTOP", "text", 86),
  ("\\diiint", "\u{222D}", "INTOP", "display", 87),
  ("\\tiiiint", "\u{2A0C}", "INTOP", "text", 88),
  ("\\diiiint", "\u{2A0C}", "INTOP", "display", 89),
  ("\\tidotsint", "\u{222B}\u{2026}\u{222B}", "INTOP", "text", 90),
  ("\\didotsint", "\u{222B}\u{2026}\u{222B}", "INTOP", "display", 91),
  ("\\tlandupint", "\u{2A19}", "INTOP", "text", 92),
  ("\\dlandupint", "\u{2A19}", "INTOP", "display", 93),
  ("\\tlanddownint", "\u{2A1A}", "INTOP", "text", 94),
  ("\\dlanddownint", "\u{2A1A}", "INTOP", "display", 95),
  ("\\tstrokedint", "\u{2A0F}", "INTOP", "text", 96),
  ("\\dstrokedint", "\u{2A0F}", "INTOP", "display", 97),
  ("\\toint", "\u{222E}", "INTOP", "text", 98),
  ("\\doint", "\u{222E}", "INTOP", "display", 99),
  ("\\toiint", "\u{222F}", "INTOP", "text", 100),
  ("\\doiint", "\u{222F}", "INTOP", "display", 101),
  ("\\trcirclerightint", "\u{2233}", "INTOP", "text", 102),
  ("\\drcirclerightint", "\u{2233}", "INTOP", "display", 103),
  ("\\tlcirclerightint", "\u{2232}", "INTOP", "text", 104),
  ("\\dlcirclerightint", "\u{2232}", "INTOP", "display", 105),
  ("\\trcircleleftint", "\u{2233}", "INTOP", "text", 106),
  ("\\drcircleleftint", "\u{2233}", "INTOP", "display", 107),
  ("\\tlcircleleftint", "\u{2232}", "INTOP", "text", 108),
  ("\\dlcircleleftint", "\u{2232}", "INTOP", "display", 109),
  ("\\tsumint", "\u{2A0B}", "INTOP", "text", 110),
  ("\\dsumint", "\u{2A0B}", "INTOP", "display", 111),
  ("\\partialvartint", "\u{222B}\u{2026}\u{222B}", "INTOP", "text", 112),
  ("\\partialvardint", "\u{222B}\u{2026}\u{222B}", "INTOP", "display", 113),
  ("\\partialvartlandupint", "\u{2A19}", "INTOP", "text", 114),
  ("\\partialvardlandupint", "\u{2A19}", "INTOP", "display", 115),
  ("\\partialvartlanddownint", "\u{2A1A}", "INTOP", "text", 116),
  ("\\partialvardlanddownint", "\u{2A1A}", "INTOP", "display", 117),
  ("\\partialvartstrokedint", "\u{2A0F}", "INTOP", "text", 118),
  ("\\partialvardstrokedint", "\u{2A0F}", "INTOP", "display", 119),
  ("\\partialvartoint", "\u{222E}", "INTOP", "text", 120),
  ("\\partialvardoint", "\u{222E}", "INTOP", "display", 121),
  ("\\partialvartoiint", "\u{222F}", "INTOP", "text", 122),
  ("\\partialvardoiint", "\u{222F}", "INTOP", "display", 123),
  ("\\partialvartrcirclerightint", "\u{2233}", "INTOP", "text", 124),
  ("\\partialvardrcirclerightint", "\u{2233}", "INTOP", "display", 125),
  ("\\partialvartlcirclerightint", "\u{2232}", "INTOP", "text", 126),
  ("\\partialvardlcirclerightint", "\u{2232}", "INTOP", "display", 127),
  ("\\partialvartrcircleleftint", "\u{2233}", "INTOP", "text", 128),
  ("\\partialvardrcircleleftint", "\u{2233}", "INTOP", "display", 129),
  ("\\partialvartlcircleleftint", "\u{2232}", "INTOP", "text", 130),
  ("\\partialvardlcircleleftint", "\u{2232}", "INTOP", "display", 131),
  ("\\partialvartsumint", "\u{2A0B}", "INTOP", "text", 132),
  ("\\partialvardsumint", "\u{2A0B}", "INTOP", "display", 133),
  ("\\tcomplement", "\u{2201}", "SUMOP", "text", 134),
  ("\\dcomplement", "\u{2201}", "SUMOP", "display", 135),
];
#[rustfmt::skip]
static MN_OPS: &[MnOp] = &[
  ("\\iint", "\u{222C}", "INTOP", "double-integral"),
  ("\\iiint", "\u{222D}", "INTOP", "triple-integral"),
  ("\\iiiint", "\u{2A0C}", "INTOP", "quadruple-integral"),
  ("\\idotsint", "\u{222B}\u{2026}\u{222B}", "INTOP", "multiple-integral"),
  ("\\landupint", "\u{2A19}", "INTOP", ""),
  ("\\landdownint", "\u{2A1A}", "INTOP", ""),
  ("\\strokedint", "\u{2A0F}", "INTOP", ""),
  ("\\oiint", "\u{222F}", "INTOP", "double-contour-integral"),
  ("\\rcirclerightint", "\u{2233}", "INTOP", ""),
  ("\\lcirclerightint", "\u{2232}", "INTOP", ""),
  ("\\rcircleleftint", "\u{2233}", "INTOP", ""),
  ("\\lcircleleftint", "\u{2232}", "INTOP", ""),
  ("\\sumint", "\u{2A0B}", "INTOP", ""),
  ("\\bigplus", "+", "SUMOP", "nary-plus"),
  ("\\bigtimes", "\u{2A09}", "SUMOP", "nary-times"),
  ("\\bigwedge", "\u{22C0}", "SUMOP", ""),
  ("\\bigvee", "\u{22C1}", "SUMOP", ""),
  ("\\bigwedgedot", "\u{27D1}", "SUMOP", ""),
  ("\\bigveedot", "\u{27C7}", "SUMOP", ""),
  ("\\bigdoublewedge", "\u{2A55}", "SUMOP", ""),
  ("\\bigdoublevee", "\u{2A54}", "SUMOP", ""),
  ("\\bigcurlywedge", "\u{22CF}", "SUMOP", ""),
  ("\\bigcurlyvee", "\u{22CE}", "SUMOP", ""),
  ("\\bigcurlywedgedot", "\u{22CF}", "SUMOP", ""),
  ("\\bigcurlyveedot", "\u{22CE}", "SUMOP", ""),
  ("\\bigdoublecurlywedge", "\u{22CF}", "SUMOP", ""),
  ("\\bigdoublecurlyvee", "\u{22CE}", "SUMOP", ""),
  ("\\bigcap", "\u{22C2}", "SUMOP", ""),
  ("\\bigcup", "\u{22C3}", "SUMOP", ""),
  ("\\bigcapdot", "\u{2A40}", "SUMOP", ""),
  ("\\bigcupdot", "\u{2A03}", "SUMOP", ""),
  ("\\bigcapplus", "\u{22C2}", "SUMOP", ""),
  ("\\bigcupplus", "\u{2A04}", "SUMOP", ""),
  ("\\bigsqcap", "\u{2A05}", "SUMOP", ""),
  ("\\bigsqcup", "\u{2A06}", "SUMOP", ""),
  ("\\bigsqcapdot", "\u{2A05}", "SUMOP", ""),
  ("\\bigsqcupdot", "\u{2A06}", "SUMOP", ""),
  ("\\bigsqcapplus", "\u{2A05}", "SUMOP", ""),
  ("\\bigsqcupplus", "\u{2A06}", "SUMOP", ""),
  ("\\bigcircle", "\u{25EF}", "SUMOP", ""),
  ("\\bigominus", "\u{2296}", "SUMOP", ""),
  ("\\bigovert", "\u{29B6}", "SUMOP", ""),
  ("\\bigoslash", "\u{2298}", "SUMOP", ""),
  ("\\bigobackslash", "\u{29B8}", "SUMOP", ""),
  ("\\bigoplus", "\u{2A01}", "SUMOP", ""),
  ("\\bigotimes", "\u{2A02}", "SUMOP", ""),
  ("\\bigodot", "\u{2A00}", "SUMOP", ""),
  ("\\bigocirc", "\u{229A}", "SUMOP", ""),
  ("\\bigotriangle", "\u{25B3}\u{20DD}", "SUMOP", ""),
  ("\\bigoast", "\u{229B}", "SUMOP", ""),
  ("\\bigostar", "\u{235F}", "SUMOP", ""),
  ("\\sum", "\u{2211}", "SUMOP", ""),
  ("\\prod", "\u{220F}", "SUMOP", ""),
  ("\\coprod", "\u{2210}", "SUMOP", ""),
  ("\\complement", "\u{2201}", "SUMOP", ""),
];

#[cfg(test)]
mod tests {
  use super::*;

  /// A combining mark (U+0300-036F, U+1AB0-1AFF, U+1DC0-1DFF, U+20D0-20FF, U+FE20-FE2F) cannot start a symbol: only an
  /// accent is a lone mark, and the tables hold none.
  fn is_combining(c: char) -> bool {
    matches!(c as u32, 0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF | 0x20D0..=0x20FF | 0xFE20..=0xFE2F)
  }

  /// Every glyph of the tables is real text: no HTML entity left from TeX4ht's `.htf` files (`&gt;` for `\gtr`), no
  /// control character or U+FFFD, and no leading combining mark (unicode-math's `\ocirc` is the ring accent, not
  /// MnSymbol's ⊚).
  #[test]
  fn mnsymbol_table_glyphs_are_characters() {
    let glyphs = MN_CHARS
      .iter()
      .map(|row| (row.0, row.1))
      .chain(MN_SIZED_OPS.iter().map(|row| (row.0, row.1)))
      .chain(MN_OPS.iter().map(|row| (row.0, row.1)))
      .chain(MN_DELIMS.iter().map(|row| (row.0, row.1)));
    for (cs, glyph) in glyphs {
      let first = glyph.chars().next();
      assert!(first.is_some(), "{cs}: empty glyph");
      assert!(!glyph.contains(['&', ';']), "{cs}: escaped glyph {glyph:?}");
      assert!(
        !glyph.chars().any(|c| c < ' ' || c == '\u{FFFD}'),
        "{cs}: control or replacement character in {glyph:?}"
      );
      assert!(
        !first.is_some_and(is_combining),
        "{cs}: glyph {glyph:?} starts with a combining mark"
      );
    }
  }

  /// A slot resolves to its first declared name's glyph; aliases share it (`\gtr`, MnSyD slot 65; `\upmodels` ⫫).
  #[test]
  fn mnsymbol_slots_resolve_to_glyphs() {
    assert_eq!(slot_glyph("MnSyD", 65), Some((">", "RELOP")));
    assert_eq!(slot_glyph("MnSyA", 0), Some(("\u{2192}", "ARROW")));
    assert_eq!(
      slot_glyph("largesymbols", 0o102),
      Some(("\u{27E6}", "OPEN"))
    );
    assert_eq!(slot_glyph("MnSyZ", 0), None);
    let upmodels = MN_CHARS.iter().find(|row| row.0 == "\\upmodels").unwrap();
    assert_eq!(upmodels.1, "\u{2AEB}");
  }
}
