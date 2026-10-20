//! t3enc.def (tipa's T3 encoding, FUKUI Rei): the raw file, with its accent machinery set as Unicode combining marks.
//!
//! t3enc.def sets every tipa diacritic by stacking boxes: `\upper@accent`/`\lower@accent` (t3enc.def:268-294, the
//! same as exaccent.sty's) put `\hbox{#2\char#4}` — slot `#4` of the font `#2` selects (`\tipaencoding`, or the current
//! font for `\empty`) — above or below `\hbox{#5}` in a `\vbox`/`\vtop`, `\ipaclap` (:330-333) overlaps two boxes and
//! `\ipabar` (:343-354) rules a bar across one. Read raw, the accent box came apart from its letter or was lost
//! (`\textsubring{n}` printed "n", `\textbarb` a two-row table). Here the slot's glyph is the combining mark the tipa
//! font draws there (t3_fontmap.rs `t3_accent_mark`: `\textsubring{n}` n̥, `\~*{a}` a̰, `\textroundcap{a}` ȃ,
//! `\t{ts}` t͡s), applied as the kernel applies `\accent` (`\lx@applyaccent`); a box accent (`\Upperaccent`'s `b`
//! form: tipa.sty:477 `\sups`) stays raw. In T3 the kernel accents `\'`, `\~`, `\r` … are t3enc.def's own (:489-516,
//! with their `*` forms below the letter), as LaTeX's encoding-dispatched accents are. OXIDIZED_DESIGN_DIVERGENCES
//! #781. Witnesses 1802.05444, 2405.01972.
use crate::{
  package::t3_fontmap::{AccentPlace, t3_accent_mark},
  prelude::*,
};

/// The state value carrying the mark `\lx@tipa@accent@mark` decoded to `\lx@tipa@accent@apply`: the combining mark and
/// the slot's standalone glyph, tab-separated; empty when the slot has none.
const MARK_KEY: &str = "lx@tipa@accent@mark";

/// The kernel accents t3enc.def:489-516 declares for T3, and tipa.sty:391's `\|` (the math ∥ elsewhere).
const T3_ACCENTS: [&str; 17] = [
  "\\`", "\\'", "\\^", "\\~", "\\\"", "\\H", "\\r", "\\v", "\\u", "\\=", "\\.", "\\b", "\\c",
  "\\d", "\\t", "\\k", "\\|",
];

/// The macro-made T3 symbols (t3enc.def:523-560) Unicode has a letter for: barred b, d and g, the crossed two and
/// crossed inverted glottal stop, and the kerned ligatures ʣ ʥ ʨ (tipauni.sty's `\textdzlig` "02A3 …). The others keep
/// t3enc.def's construction (`\ipabar` → a stroke overlay below; `\textctdctzlig` the two letters).
const T3_MADE_LETTERS: [(&str, &str); 8] = [
  ("\\T3\\textbarb", "\u{0180}"),
  ("\\T3\\textbard", "\u{0111}"),
  ("\\T3\\textbarg", "\u{01E5}"),
  ("\\T3\\textcrtwo", "\u{01BB}"),
  ("\\T3\\textcrinvglotstop", "\u{01BE}"),
  ("\\T3\\textdzlig", "\u{02A3}"),
  ("\\T3\\textdctzlig", "\u{02A5}"),
  ("\\T3\\texttctclig", "\u{02A8}"),
];

fn current_encoding() -> String {
  lookup_font()
    .and_then(|font| font.get_encoding().map(|encoding| encoding.to_string()))
    .unwrap_or_else(|| "OT1".to_string())
}

/// The mark of slot `slot` set `place` in the current font: T3's from its glyph table, another encoding's (an
/// exaccent-style `\upperaccent` in T1 text) from the kernel's accent combiners of the slot's standalone glyph.
fn accent_mark(slot: i64, place: AccentPlace) -> Option<(char, String)> {
  let encoding = current_encoding();
  let standalone = u8::try_from(slot)
    .ok()
    .and_then(|code| font::decode_str(code, Some(encoding.clone()), false))
    .map(|glyph| with(glyph, |g| g.to_string()))
    .unwrap_or_default();
  let mark = if encoding == "T3" {
    t3_accent_mark(slot, place)
  } else {
    let combiner = match place {
      AccentPlace::Below => "accent_combiner_below",
      AccentPlace::Above | AccentPlace::Over => "accent_combiner_above",
    };
    lookup_mapping(combiner, &standalone)
      .map(|mark| mark.to_string())
      .and_then(|mark| mark.chars().next())
  }?;
  let standalone = if standalone.is_empty() {
    mark.to_string()
  } else {
    standalone
  };
  Some((mark, standalone))
}

LoadDefinitions!({
  InputDefinitions!("t3enc", extension => Some("def".into()), noltxml => true);

  // Run inside `\begingroup <font switch> … \endgroup`: the slot is decoded in the font `#2` selected, as
  // t3enc.def:269 `\setbox\@ne\hbox{#2\char#4}` sets it, and handed on past the group.
  DefPrimitive!("\\lx@tipa@accent@mark{}{}", sub[(place, slot)] {
    let place = match place.to_string().as_str() {
      "below" => AccentPlace::Below,
      "over" => AccentPlace::Over,
      _ => AccentPlace::Above,
    };
    let (slot, _) = read_braced(Tokens::new(slot.revert()), read_number)?;
    let slot = slot.value_of();
    let value = match accent_mark(slot, place) {
      Some((mark, standalone)) => s!("{mark}\t{standalone}"),
      None => {
        let encoding = current_encoding();
        if !already_reported(&s!("tipa_accent_{encoding}_{slot}_{place:?}")) {
          Warn!("missing_font_encoding", encoding,
            s!("Font encoding {encoding} has no combining mark for the accent in slot {slot} set {place:?}: \
                the letter is printed without it"));
        }
        String::new()
      },
    };
    assign_value(MARK_KEY, value, Some(Scope::Global));
  });
  // The letter with the mark decoded just before: the kernel's `\accent` composition (tex_character.rs
  // `apply_accent`, precomposed where Unicode has the letter: `\'a` á). Read when it is expanded, before the letter is
  // digested, so a nested accent (`\textdotbreve`, t3enc.def:452-454) decodes its own.
  DefMacro!("\\lx@tipa@accent@apply{}", sub[(letter)] {
    let value = lookup_string(MARK_KEY);
    assign_value(MARK_KEY, String::new(), Some(Scope::Global));
    let letter = letter.unlist();
    match value.split_once('\t') {
      Some((mark, standalone)) => {
        let mut tokens = vec![
          T_CS!("\\lx@applyaccent"), T_CS!("\\lx@tipa@accent@apply"),
          Token::new(mark, Catcode::OTHER), T_OTHER!(standalone), T_BEGIN!(),
        ];
        tokens.extend(letter);
        tokens.push(T_END!());
        Tokens::new(tokens)
      },
      None => Tokens::new(letter),
    }
  }, protected => true);

  // t3enc.def:268-294: the `a` form's accent is slot #4 of the font #2 selects; the `b` form's is material, set raw.
  Let!("\\lx@tipa@raw@upper@accent", "\\upper@accent");
  Let!("\\lx@tipa@raw@lower@accent", "\\lower@accent");
  for (cs, raw, place) in [
    ("\\upper@accent", "\\lx@tipa@raw@upper@accent", "above"),
    ("\\lower@accent", "\\lx@tipa@raw@lower@accent", "below"),
  ] {
    DefMacro!(T_CS!(cs), "{}{}[]{}{}", sub[(form, font, kern, accent, letter)] {
      let form = form.unlist_cow().into_owned();
      let font = font.unlist_cow().into_owned();
      let accent = accent.unlist_cow().into_owned();
      let letter = letter.unlist_cow().into_owned();
      let mut tokens = Vec::new();
      if matches!(form.as_slice(), [t] if t.to_string() == "a") {
        tokens.push(T_CS!("\\begingroup"));
        tokens.extend(font);
        tokens.push(T_CS!("\\lx@tipa@accent@mark"));
        tokens.extend([T_BEGIN!()]);
        tokens.extend(Explode!(place));
        tokens.extend([T_END!(), T_BEGIN!()]);
        tokens.extend(accent);
        tokens.extend([T_END!(), T_CS!("\\endgroup"), T_CS!("\\lx@tipa@accent@apply"), T_BEGIN!()]);
        tokens.extend(letter);
        tokens.push(T_END!());
      } else {
        tokens.push(T_CS!(raw));
        tokens.extend(form);
        tokens.extend(font);
        tokens.push(T_OTHER!("["));
        tokens.extend(kern.unlist_cow().into_owned());
        tokens.push(T_OTHER!("]"));
        tokens.push(T_BEGIN!());
        tokens.extend(accent);
        tokens.extend([T_END!(), T_BEGIN!()]);
        tokens.extend(letter);
        tokens.push(T_END!());
      }
      Tokens::new(tokens)
    });
  }
  // t3enc.def:431-432: the tilde of slot 38 over the letter (`\ipaclap`), U+0334.
  DefMacro!(
    T_CS!("\\T3\\textsuperimposetilde"),
    "{}",
    "\\begingroup\\tipaencoding\\lx@tipa@accent@mark{over}{38}\\endgroup\\lx@tipa@accent@apply{#1}"
  );
  // t3enc.def:343-354: a bar ruled across the symbol, the short stroke overlay U+0335 — over a character run
  // (`\textbarc`'s `{\tipaencoding c}`). Anything else keeps t3enc.def's construction: `\textdoublebaresh`
  // (:535-537) bars an `\hb@xt@` box of another `\ipabar`, which the overlay printed as TeX source (rev77x).
  // `\providecommand`ed: read again, t3enc.def keeps this binding's, and the raw one saved first stays.
  if !IsDefined!(&T_CS!("\\lx@tipa@raw@ipabar")) {
    Let!("\\lx@tipa@raw@ipabar", "\\ipabar");
  }
  DefMacro!("\\ipabar{}{}{}{}{}", sub[(symbol, height, width, left, right)] {
    let symbol = symbol.unlist();
    let character_run = symbol.iter().all(|t| {
      matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER | Catcode::BEGIN | Catcode::END)
        || *t == T_CS!("\\tipaencoding")
    });
    let mut tokens = if character_run {
      vec![
        T_CS!("\\lx@applyaccent"), T_CS!("\\ipabar"), T_OTHER!("\u{0335}"), T_OTHER!("\u{00A0}\u{0335}"),
        T_BEGIN!(),
      ]
    } else {
      vec![T_CS!("\\lx@tipa@raw@ipabar"), T_BEGIN!()]
    };
    tokens.extend(symbol);
    tokens.push(T_END!());
    if !character_run {
      for arg in [height, width, left, right] {
        tokens.push(T_BEGIN!());
        tokens.extend(arg.unlist());
        tokens.push(T_END!());
      }
    }
    Tokens::new(tokens)
  });
  for (cs, glyph) in T3_MADE_LETTERS {
    def_primitive(
      T_CS!(cs),
      None,
      Some(PrimitiveBody::String(pin(glyph))),
      PrimitiveOptions::default(),
    )?;
  }

  // t3enc.def read again (`\input{t3enc.def}` after tipa.sty) re-declares its T3 commands, and the kernel ones saved
  // the first time stay saved: saving again would save these wrappers, which then expand to themselves (rev77x;
  // guard `perfect_kernel_batch77x::t3enc_def_loaded_twice`).
  if IsDefined!(&T_CS!("\\lx@tipa@native@'")) {
    return Ok(());
  }

  // In T3 text the kernel accents are t3enc.def's (latex.ltx's text commands dispatch on `\cf@encoding`; ours are
  // native and take the encoding's command only here), and `\|` is tipa.sty's diacritic shorthand (`\|[{t}` t̪), once
  // tipa.sty has declared it: elsewhere, and in math, they are what they were. The command stays an expandable macro
  // that expands to the native one (a `\csname` or `\edef` sees what it saw before, OXIDIZED_DESIGN #170); the T3 one
  // is reached through a protected step, as LaTeX's dispatcher is robust.
  for name in T3_ACCENTS {
    let cs = T_CS!(name);
    let native = T_CS!(s!("\\lx@tipa@native@{}", &name[1..]));
    let t3 = T_CS!(s!("\\T3{name}"));
    let t3_step = T_CS!(s!("\\lx@tipa@T3@{}", &name[1..]));
    Let!(native, cs);
    DefMacro!(t3_step, "", sub[_args] { Tokens!(t3) }, protected => true);
    DefMacro!(cs, "", sub[_args] {
      if !lookup_bool("IN_MATH") && current_encoding() == "T3" && IsDefined!(&t3) {
        Tokens!(t3_step)
      } else {
        Tokens!(native)
      }
    });
  }
});
