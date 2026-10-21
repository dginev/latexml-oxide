use std::{borrow::Cow, fmt, fmt::Display, rc::Rc};

use once_cell::sync::Lazy;

use crate::{
  Digested,
  common::{
    arena::{self, SymStr},
    dimension::Dimension,
    error::*,
    float::Float,
    glue::Glue,
    mudimension::MuDimension,
    muglue::MuGlue,
    number::Number,
    numeric_ops::NumericOps,
    store::Stored,
  },
  definition::{Definition, register::Register},
  state,
  tokens::Tokens,
};

static CONTROLNAME: &[&str] = &[
  "NUL", "SOH", "STX", "ETX", "EOT", "ENQ", "ACK", "BEL", "BS", "HT", "LF", "VT", "FF", "CR", "SO",
  "SI", "DLE", "DC1", "DC2", "DC3", "DC4", "NAK", "SYN", "ETB", "CAN", "EM", "SUB", "ESC", "FS",
  "GS", "RS", "US",
];

/// A Token category code, as in TeX
///
/// Two bytes (`#[repr(u16)]`), so that [`Token`] (`text` u32, `code`, `prov` u16) has no padding byte: with one, every
/// move of a token on the gullet's read path (`Option<Token>`, `CheckedRead::Tok`, the `Result` slot) copied the pad
/// and `prov` as a 3-byte blob, byte by byte (`token_layout_code_at_byte_4`).
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[repr(u16)]
pub enum Catcode {
  ESCAPE,
  BEGIN,
  END,
  MATH,
  ALIGN,
  EOL,
  PARAM,
  SUPER,
  SUB,
  IGNORE,
  SPACE,
  LETTER,
  OTHER,
  ACTIVE,
  COMMENT,
  INVALID,
  CS,
  MARKER,
  ARG,
}

impl From<u8> for Catcode {
  fn from(num: u8) -> Catcode {
    use crate::token::Catcode::*;
    match num {
      0 => ESCAPE,
      1 => BEGIN,
      2 => END,
      3 => MATH,
      4 => ALIGN,
      5 => EOL,
      6 => PARAM,
      7 => SUPER,
      8 => SUB,
      9 => IGNORE,
      10 => SPACE,
      11 => LETTER,
      12 => OTHER,
      13 => ACTIVE,
      14 => COMMENT,
      15 => INVALID,
      16 => CS,
      17 => MARKER,
      18 => ARG,
      _ => {
        // let message = s!("Unrecognized catcode: {:?}", num);
        // Warn!("unknown", "catcode", None, message);
        IGNORE
      },
    }
  }
}

impl From<Catcode> for u8 {
  fn from(cc: Catcode) -> u8 {
    use crate::token::Catcode::*;
    match cc {
      ESCAPE => 0,
      BEGIN => 1,
      END => 2,
      MATH => 3,
      ALIGN => 4,
      EOL => 5,
      PARAM => 6,
      SUPER => 7,
      SUB => 8,
      IGNORE => 9,
      SPACE => 10,
      LETTER => 11,
      OTHER => 12,
      ACTIVE => 13,
      COMMENT => 14,
      INVALID => 15,
      CS => 16,
      MARKER => 17,
      ARG => 18,
    }
  }
}

impl Catcode {
  /// a debug-friendly name
  pub fn name(self) -> &'static str {
    use crate::token::Catcode::*;
    match self {
      // Primitive
      ESCAPE => "Escape",
      BEGIN => "Begin",
      END => "End",
      MATH => "Math",
      ALIGN => "Align",
      EOL => "EOL",
      PARAM => "Parameter",
      SUPER => "Superscript",
      SUB => "Subscript",
      SPACE => "Space",
      // Non-primitive
      IGNORE => "Ignore",
      LETTER => "Letter",
      OTHER => "Other",
      ACTIVE => "Active",
      COMMENT => "Comment",
      INVALID => "Invalid",
      CS => "ControlSequence",
      MARKER => "Marker",
      ARG => "Arg",
    }
  }

  /// SymStr form of `name()` — each variant caches its interned
  /// symbol via `pin!`. Used by `Token::get_cs_name` /
  /// `pin_cs_name` to avoid a per-call `pin_static` hash probe
  /// (fires on every primitive-token definition lookup).
  pub fn name_sym(self) -> SymStr {
    use crate::token::Catcode::*;
    match self {
      ESCAPE => crate::pin!("Escape"),
      BEGIN => crate::pin!("Begin"),
      END => crate::pin!("End"),
      MATH => crate::pin!("Math"),
      ALIGN => crate::pin!("Align"),
      EOL => crate::pin!("EOL"),
      PARAM => crate::pin!("Parameter"),
      SUPER => crate::pin!("Superscript"),
      SUB => crate::pin!("Subscript"),
      SPACE => crate::pin!("Space"),
      IGNORE => crate::pin!("Ignore"),
      LETTER => crate::pin!("Letter"),
      OTHER => crate::pin!("Other"),
      ACTIVE => crate::pin!("Active"),
      COMMENT => crate::pin!("Comment"),
      INVALID => crate::pin!("Invalid"),
      CS => crate::pin!("ControlSequence"),
      MARKER => crate::pin!("Marker"),
      ARG => crate::pin!("Arg"),
    }
  }
  /// a \meaning-friendly name
  pub fn meaning(self) -> &'static str {
    use crate::token::Catcode::*;
    match self {
      ESCAPE => "the escape character",
      BEGIN => "begin-group character",
      END => "end-group character",
      MATH => "math shift character",
      ALIGN => "alignment tab character",
      EOL => "end-of-line character",
      PARAM => "macro parameter character",
      SUPER => "superscript character",
      SUB => "subscript character",
      IGNORE => "ignored character",
      SPACE => "blank space",
      LETTER => "the letter",
      OTHER => "the character",
      ACTIVE => "active character",
      COMMENT => "comment character",
      INVALID => "invalid character",
      _ => "",
    }
  }
  /// a short name helpful for Token debugging
  pub fn short_name(self) -> &'static str {
    use crate::token::Catcode::*;
    match self {
      ESCAPE => "T_ESCAPE",
      BEGIN => "T_BEGIN",
      END => "T_END",
      MATH => "T_MATH",
      ALIGN => "T_ALIGN",
      EOL => "T_EOL",
      PARAM => "T_PARAM",
      SUPER => "T_SUPER",
      SUB => "T_SUB",
      IGNORE => "T_IGNORE",
      SPACE => "T_SPACE",
      LETTER => "T_LETTER",
      OTHER => "T_OTHER",
      ACTIVE => "T_ACTIVE",
      COMMENT => "T_COMMENT",
      INVALID => "T_INVALID",
      CS => "T_CS",
      MARKER => "T_MARKER",
      ARG => "T_ARG",
    }
  }

  // ======================================================================
  // Categories of Category codes.
  // For Tokens with these catcodes, only the catcode is relevant for comparison.
  // (if they even make it to a stage where they get compared)
  /// TeX-primitive codes
  pub fn is_primitive(self) -> bool {
    use crate::token::Catcode::*;
    match self {
      // Primitives
      ESCAPE | BEGIN | END | MATH | ALIGN | EOL | PARAM | SUPER | SUB | SPACE => true,
      // Non-primitive
      IGNORE | LETTER | OTHER | ACTIVE | COMMENT | INVALID | CS | MARKER | ARG => false,
    }
  }
  /// Catcodes with associated primitives
  pub fn is_executable(self) -> bool {
    use crate::token::Catcode::*;
    match self {
      // Executable
      BEGIN | END | MATH | ALIGN | SUPER | SUB | ACTIVE | CS => true,
      // Non-executable
      EOL | ESCAPE | PARAM | SPACE | IGNORE | LETTER | OTHER | COMMENT | INVALID | MARKER | ARG => {
        false
      },
    }
  }
  /// Catcodes which can be neutralized
  pub fn is_neutralizable(self) -> bool {
    use crate::token::Catcode::*;
    match self {
      // Neutralizable
      MATH | ALIGN | PARAM | SUPER | SUB | ACTIVE => true,
      // Non-neutralizable
      ESCAPE | BEGIN | END | EOL | IGNORE | SPACE | LETTER | OTHER | COMMENT | INVALID | CS
      | MARKER | ARG => false,
    }
  }
  /// Shorthand to match the "active" and "command sequence" catcodes
  pub fn is_active_or_cs(self) -> bool {
    use crate::token::Catcode::*;
    matches!(self, ACTIVE | CS)
  }
  /// Tokens which can be absorbed without side-effects
  pub fn is_absorbable(self) -> bool {
    use crate::token::Catcode::*;
    // Absorbable
    matches!(self, SPACE | LETTER | OTHER | COMMENT)
  }
  /// Gullet can only hold comment and marker tokens
  pub fn is_gullet_holdable(self) -> bool {
    use crate::token::Catcode::*;
    matches!(self, COMMENT | MARKER)
  }
  /// Catcodes that are of note for balanced reads.
  pub fn is_balanced_interesting(self) -> bool {
    use crate::token::Catcode::*;
    // `gullet::is_balanced` reacts to BEGIN,END,MARKER coded tokens
    matches!(self, BEGIN | END | MARKER)
  }
}

/// The core immutable syntactic primitive resulting from TeX's read-in and expansion process
/// We allow the fields to be public, so that we can use builder macros such as
/// ```
/// macro_rules! T_SPACE(() => {
///     Token { text: arena::pin_static(" "), code: Catcode::SPACE}
///   });
/// ```
///
/// `#[repr(C)]` pins the field order `text`, `code`, `prov` (`loc`): `code` stays at byte 4, where it sat before
/// `prov` existed, and with a two-byte [`Catcode`] the struct has no padding (`prov` at 6). Rust's default layout put
/// `prov` at 4 and `code` at 6, which made every move of a token through `Option<Token>`, `CheckedRead::Tok` and the
/// read path's `Result` slot a 4-piece field-wise copy (`gullet::read_internal_token_checked`: +1.29 % instructions
/// on an empty article; +1.14 % with `repr(C)` alone, the pad byte still copied apart; +0.65 % gap-free). The guard
/// is `token_layout_code_at_byte_4`.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Token {
  /// an arena id the character content for this token
  pub text: SymStr,
  /// a TeX catcode
  pub code: Catcode,
  /// Out-of-band field provenance (`0` = none): the frontmatter field whose stored value this token
  /// is a token of (the kernel `\title`/`\author` stamp their store, latexml_engine `\lx@field@mark`), so the
  /// ink digested from it is known for that field's print. Not part of TeX's token: **excluded from `PartialEq`**
  /// (`\ifx`, `\meaning`, `\pdfstrcmp`, xstring compare as pdflatex does), never written, never dumped. It sits in
  /// what was the struct's padding: `Token` stays 8 bytes (`token_size_invariant`). OXIDIZED_DESIGN #715.
  pub prov: u16,
  /// Origin handle into the per-conversion token-origin side arena (1-based;
  /// `0` = no recorded origin). Present only under the `token-locators` feature
  /// (the opt-in source-map precision build); `Token` stays 8 bytes otherwise.
  /// Set in `read_token`; carried through expansion so a digested run can recover
  /// its exact source span. **Excluded from `PartialEq`** (tokens compare by
  /// meaning, not origin — see `impl PartialEq`). See docs/performance/SOURCE_PROVENANCE.md
  /// §3.1.1.
  #[cfg(feature = "token-locators")]
  pub loc:  u32,
}

impl fmt::Debug for Token {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if self.code == Catcode::ARG {
      self.with_str(|text| write!(f, "\"#{}\"", text))
    } else {
      self.with_str(|text| write!(f, "{:?}", text))
    }
  }
}

impl Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if self.code == Catcode::ARG {
      write!(f, "#")?;
    }
    self.with_str(|text| write!(f, "{}", text))
  }
}

/// Compare two tokens; They are equal if they both have same catcode & string
// [We pretend all SPACE's are the same, since we'd like to hide newline's in there!]
// NOTE: That another popular equality checks whether the "meaning" (defn) are the same.
// That is NOT done here; see Equals(x,y) and XEquals(x,y)
impl PartialEq for Token {
  fn eq(&self, other: &Token) -> bool {
    self.code == other.code && (self.code == Catcode::SPACE || (self.text == other.text))
  }
}

// Per-symbol memo for `Token::is_noexpand_family`, indexed by the symbol's
// arena index: 0 = not yet computed, 1 = not in the family, 2 = in the family.
// Sound because the arena is append-only (a symbol's text never changes);
// MUST be cleared alongside `arena::reset()` (see
// `reset_noexpand_family_memo`, called from `reset_thread_engine`) since a
// reset renumbers symbols. A symbol's index is its byte offset in the arena's
// buffer (not a dense index), so the memo covers only the first
// `NOEXPAND_MEMO_SPAN` bytes and a later symbol is checked directly: unbounded,
// the memo grew with every byte interned, to 1.5 GB after a long pgf path
// (tkz-grapheur), and in `cortex_worker`, which never resets the arena, with
// the worker's whole lifetime. Ordinary papers' arenas are 8-21 MB, inside the
// span, so for them the memo is as before. A map keyed by symbol bounds it
// too, but costs +0.5 % instructions on typical papers (the 24-paper C2 set).
thread_local! {
  static NOEXPAND_FAMILY_MEMO: std::cell::RefCell<Vec<u8>> =
    const { std::cell::RefCell::new(Vec::new()) };
}
/// The arena byte span the [`Token::is_noexpand_family`] memo covers (64 MiB).
const NOEXPAND_MEMO_SPAN: usize = 1 << 26;

/// Clear the [`Token::is_noexpand_family`] per-symbol memo. Companion to
/// `arena::reset()` — symbol indices are reused after a reset, so stale
/// entries would alias unrelated strings (same bug class as the REPORT-map
/// fix, `7b64a48ad1`).
pub fn reset_noexpand_family_memo() { NOEXPAND_FAMILY_MEMO.with(|m| m.borrow_mut().clear()); }

/// Name of the bare no-op control sequence `\noexpand` collapses to, and the
/// prefix of every member of the no-expand family. See [`Token::is_noexpand_family`].
pub const NOEXPAND_PREFIX: &str = "\\special_relax";
/// Separator byte between the `\special_relax` prefix and the shadowed token's
/// text in a family token's name. `\x01` is never valid in a CS name or active
/// char, so `\special_relax\x01\foo` is unambiguous and cannot collide with a
/// real CS such as a (hypothetical) `\special_relaxfoo`.
pub const NOEXPAND_SEP: u8 = 1;

/// Build the no-expand family token shadowing `shadowed` — the per-token,
/// dump-safe representation of `\noexpand <shadowed>` (faithful to TeX's
/// `no_expand_flag`: the shadowed identity is preserved in the name; the family
/// resolves to `\relax` meaning). `shadowed` is an expandable/undefined CS or
/// active token (the only things `\noexpand` wraps; see `is_dont_expandable`).
pub fn noexpand_family(shadowed: &Token) -> Token {
  let text = shadowed.with_str(|s| format!("{NOEXPAND_PREFIX}{}{s}", NOEXPAND_SEP as char));
  Token {
    text: arena::pin(text),
    code: Catcode::CS,
    prov: 0,
    #[cfg(feature = "token-locators")]
    loc: 0,
  }
}

// Note: given that we are pinning the strings in an arena,
//  once we have a token of a certain kind it is now faster to clone
//  a known token than it is to build a new one
//  (as the arena lookup is a hair slower than copying a u32)

/// constant for a BEGIN "{" token
#[thread_local]
pub static TOKEN_BEGIN: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static("{"),
  code: Catcode::BEGIN,
  prov: 0,
  #[cfg(feature = "token-locators")]
  loc: 0,
});
/// constant for an END "}" token
#[thread_local]
pub static TOKEN_END: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static("}"),
  code: Catcode::END,
  prov: 0,
  #[cfg(feature = "token-locators")]
  loc: 0,
});
/// constant for a MATH "$" token
#[thread_local]
pub static TOKEN_MATH: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static("$"),
  code: Catcode::MATH,
  prov: 0,
  #[cfg(feature = "token-locators")]
  loc: 0,
});
/// constant for an ALIGN "&" token
#[thread_local]
pub static TOKEN_ALIGN: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static("&"),
  code: Catcode::ALIGN,
  prov: 0,
  #[cfg(feature = "token-locators")]
  loc: 0,
});
/// constant for a PARAM "#" token
#[thread_local]
pub static TOKEN_PARAM: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static("#"),
  code: Catcode::PARAM,
  prov: 0,
  #[cfg(feature = "token-locators")]
  loc: 0,
});
/// constant for a SUPER "^" token
#[thread_local]
pub static TOKEN_SUPER: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static("^"),
  code: Catcode::SUPER,
  prov: 0,
  #[cfg(feature = "token-locators")]
  loc: 0,
});
/// constant for a SUB "_" token
#[thread_local]
pub static TOKEN_SUB: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static("_"),
  code: Catcode::SUB,
  prov: 0,
  #[cfg(feature = "token-locators")]
  loc: 0,
});
/// constant for a SPACE " " token
#[thread_local]
pub static TOKEN_SPACE: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static(" "),
  code: Catcode::SPACE,
  prov: 0,
  #[cfg(feature = "token-locators")]
  loc: 0,
});
/// constant for a CR "\n" token
#[thread_local]
pub static TOKEN_CR: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static("\n"),
  code: Catcode::SPACE,
  prov: 0,
  #[cfg(feature = "token-locators")]
  loc: 0,
});
/// constant for T_CS("\relax")
#[thread_local]
pub static TOKEN_RELAX: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static("\\relax"),
  code: Catcode::CS,
  prov: 0,
  #[cfg(feature = "token-locators")]
  loc: 0,
});
/// constant for T_CS("\expandafter")
#[thread_local]
pub static TOKEN_EXPANDAFTER: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static("\\expandafter"),
  code: Catcode::CS,
  prov: 0,
  #[cfg(feature = "token-locators")]
  loc: 0,
});
/// constant for T_CS("\endcsname")
#[thread_local]
pub static TOKEN_ENDCSNAME: Lazy<Token> = Lazy::new(|| Token {
  text: arena::pin_static("\\endcsname"),
  code: Catcode::CS,
  prov: 0,
  #[cfg(feature = "token-locators")]
  loc: 0,
});

/// Eagerly initialize this thread's pre-built `#[thread_local]` token
/// constants. Each one's `Lazy` initializer interns its control-sequence
/// name via `arena::pin_static`, so they must be forced AFTER
/// [`arena::force_init`](crate::common::arena::force_init) and before any
/// code accesses them during another root's initialization — otherwise the
/// first access re-entrantly initializes the token from within that other
/// root's init, the macOS `#[thread_local]` hazard (rust-lang/rust#29594,
/// issue #217). No behavioral change on Linux.
pub(crate) fn force_init() {
  Lazy::force(&TOKEN_BEGIN);
  Lazy::force(&TOKEN_END);
  Lazy::force(&TOKEN_MATH);
  Lazy::force(&TOKEN_ALIGN);
  Lazy::force(&TOKEN_PARAM);
  Lazy::force(&TOKEN_SUPER);
  Lazy::force(&TOKEN_SUB);
  Lazy::force(&TOKEN_SPACE);
  Lazy::force(&TOKEN_CR);
  Lazy::force(&TOKEN_RELAX);
  Lazy::force(&TOKEN_EXPANDAFTER);
  Lazy::force(&TOKEN_ENDCSNAME);
}

#[macro_export]
/// macro for a BEGIN "{" token
macro_rules! T_BEGIN(() => { *$crate::token::TOKEN_BEGIN });
#[macro_export]
/// macro for a new END "{" token
macro_rules! T_END(() => { *$crate::token::TOKEN_END });
/// macro for a MATH "$" token
#[macro_export]
macro_rules! T_MATH(() => { *$crate::token::TOKEN_MATH });
/// macro for an ALIGN "&" token
#[macro_export]
macro_rules! T_ALIGN(() => { *$crate::token::TOKEN_ALIGN });
/// macro for a PARAM "#" token
#[macro_export]
macro_rules! T_PARAM(() => { *$crate::token::TOKEN_PARAM });
/// macro for a SUPER "^" token
#[macro_export]
macro_rules! T_SUPER(() => { *$crate::token::TOKEN_SUPER });
/// macro for a SUB "_" token
#[macro_export]
macro_rules! T_SUB(() => { *$crate::token::TOKEN_SUB });
/// macro for a SPACE token (default " ")
#[macro_export]
macro_rules! T_SPACE(() => { *$crate::token::TOKEN_SPACE };
($text:literal) => {
  Token { text: $crate::pin!($text), code: Catcode::SPACE,
      prov: 0, #[cfg(feature = "token-locators")] loc: 0
    }
});
/// macro for a CR "\n" token
#[macro_export]
macro_rules! T_CR(() => { *$crate::token::TOKEN_CR });
/// macro for a LETTER token
#[macro_export]
macro_rules! T_LETTER {
  ($text:literal) => {
    Token {
      text: $crate::pin!($text),
      code: Catcode::LETTER,
      prov: 0,
      #[cfg(feature = "token-locators")]
      loc: 0,
    }
  };
  ($text:expr_2021) => {
    Token {
      text: $crate::common::arena::pin($text),
      code: Catcode::LETTER,
      prov: 0,
      #[cfg(feature = "token-locators")]
      loc: 0,
    }
  };
}
/// macro for an OTHER code token
#[macro_export]
macro_rules! T_OTHER {
  ($text:literal) => {
    Token {
      text: $crate::pin!($text),
      code: Catcode::OTHER,
      prov: 0,
      #[cfg(feature = "token-locators")]
      loc: 0,
    }
  };
  ($text:expr_2021) => {
    Token {
      text: $crate::common::arena::pin($text),
      code: Catcode::OTHER,
      prov: 0,
      #[cfg(feature = "token-locators")]
      loc: 0,
    }
  };
}
/// T_OTHER from a single character
#[macro_export]
macro_rules! T_OTHER_CHAR {
  ($text:literal) => {
    Token {
      text: $crate::common::arena::pin_char($text),
      code: Catcode::OTHER,
      prov: 0,
      #[cfg(feature = "token-locators")]
      loc: 0,
    }
  };
}
/// macro for an ACTIVE char token
#[macro_export]
macro_rules! T_ACTIVE {
  ($c:expr_2021) => {{
    let mut tmp = [0u8; 4];
    let s = $c.encode_utf8(&mut tmp);
    Token {
      text: $crate::common::arena::pin(s),
      code: Catcode::ACTIVE,
      prov: 0,
      #[cfg(feature = "token-locators")]
      loc: 0,
    }
  }};
}
/// macro for a COMMENT content token
#[macro_export]
macro_rules! T_COMMENT {
  ($text:expr_2021) => {
    Token {
      text: $crate::common::arena::pin($text),
      code: Catcode::COMMENT,
      prov: 0,
      #[cfg(feature = "token-locators")]
      loc: 0,
    }
  };
}
/// macro for a command sequence token
#[macro_export]
macro_rules! T_CS {
  ($text:literal) => {
    $crate::token::Token {
      text: $crate::pin!($text),
      code: $crate::token::Catcode::CS,
      prov: 0,
      #[cfg(feature = "token-locators")]
      loc: 0,
    }
  };
  ($text:expr_2021) => {
    $crate::token::Token {
      text: $crate::common::arena::pin($text),
      code: $crate::token::Catcode::CS,
      prov: 0,
      #[cfg(feature = "token-locators")]
      loc: 0,
    }
  };
}

/// macro for T_CS("\\relax")
#[macro_export]
macro_rules! T_RELAX(() => { $crate::token::TOKEN_RELAX.clone() });

/// macro for a tracing MARKER token
#[macro_export]
macro_rules! T_MARKER {
  ($text:expr_2021) => {
    Token {
      text: $crate::common::arena::pin($text),
      code: Catcode::MARKER,
      prov: 0,
      #[cfg(feature = "token-locators")]
      loc: 0,
    }
  };
}

/// macro for a numbered ARG token
#[macro_export]
macro_rules! T_ARG {
  ($text:expr_2021) => {
    Token {
      text: $crate::common::arena::pin($text.to_string()),
      code: Catcode::ARG,
      prov: 0,
      #[cfg(feature = "token-locators")]
      loc: 0,
    }
  };
}

/// Token constructor macro (defaults to OTHER code). `Token!(sym <SymStr>, <catcode>)` takes an
/// already-interned symbol (a token's `text`, a `Stored::String`), without interning it again.
#[macro_export]
macro_rules! Token {
  (sym $sym:expr_2021, $cc:expr_2021) => {
    Token {
      text: $sym,
      code: $cc,
      prov: 0,
      #[cfg(feature = "token-locators")]
      loc: 0,
    }
  };
  ($text:expr_2021) => {
    Token!($text, Catcode::OTHER)
  };
  ($text:literal, $cc:expr_2021) => {
    Token {
      text: $crate::pin!($text),
      code: $cc,
      prov: 0,
      #[cfg(feature = "token-locators")]
      loc: 0,
    }
  };
  ($text:expr_2021, $cc:expr_2021) => {
    Token {
      text: $crate::common::arena::pin($text),
      code: $cc,
      prov: 0,
      #[cfg(feature = "token-locators")]
      loc: 0,
    }
  };
}

/// Special case: a character needs swift string conversion, so let's use a dedicated macro
#[macro_export]
macro_rules! CharToken {
  ($c:expr_2021) => {
    CharToken!($c, $crate::token::Catcode::OTHER)
  };
  ($c:expr_2021, $cc:expr_2021) => {
    $crate::token::Token {
      text: $crate::common::arena::pin_char($c),
      code: $cc,
      prov: 0,
      #[cfg(feature = "token-locators")]
      loc: 0,
    }
  };
}

/// Explode a string into a list of tokens, all w/catcode OTHER (except space).
/// Note: newlines are converted to OTHER, NOT SPACE (Perl #2700 reverted #2646).
/// ^^J in TeX decodes to CC_OTHER by default; let the tokenizer handle catcode
/// reassignment if needed.
#[macro_export]
macro_rules! Explode(($text:expr_2021) => (
  $text.to_string().chars().map(|c|
    if c==' ' { T_SPACE!() }
    else {
      CharToken!(c)
    }
  ).collect::<Vec<Token>>()
));

#[macro_export]
macro_rules! ExplodeChars(($text:expr_2021) => (
  $text.as_str().chars().map(|c|
    if c==' ' { T_SPACE!() }
    else {
      CharToken!(c)
    }
  ).collect::<Vec<Token>>()
));

/// Similar to Explode, but convert letters to catcode LETTER and others to OTHER
/// Hopefully, this is essentially correct WITHOUT resorting to catcode lookup?
/// Perl sync: newlines are OTHER, not SPACE (matches Perl #2700 revert of #2646).
#[macro_export]
macro_rules! ExplodeText(
  ($text:expr_2021) => ({
  use $crate::token::{Catcode,Token};
  $text.to_string().chars().map(|c|
    if c==' ' { T_SPACE!() }
    else {
      let mut tmp = [0u8; 4];
      let s = c.encode_utf8(&mut tmp);
      if c.is_alphabetic() {
      T_LETTER!(s) }
    else { T_OTHER!(s) }}
  ).collect::<Vec<Token>>()
}));

#[macro_export]
macro_rules! SymExplodeText(
  ($sym:expr_2021) => ({
  use $crate::token::{Catcode,Token};
  let chars : Vec<char> = arena::with($sym, |text| text.chars().collect());
  chars.into_iter().map(|c|
    if c==' ' { T_SPACE!() }
    else {
      let mut tmp = [0u8; 4];
      let s = c.encode_utf8(&mut tmp);
      if c.is_alphabetic() {
      T_LETTER!(s) }
    else { T_OTHER!(s) }}
  ).collect::<Vec<Token>>()
}));

// static UNTEX_LINELENGTH: usize = 78; // [CONSTANT]

impl Default for Token {
  fn default() -> Self {
    Token {
      text: arena::pin_static("EXPECTED_TOKEN"),
      code: Catcode::OTHER,
      prov: 0,
      #[cfg(feature = "token-locators")]
      loc: 0,
    }
  }
}

///======================================================================
/// Accessors.
/// The bits of a [`Token::prov`] below the field: the token's position in the field's stored value (saturating).
pub const FIELD_PROV_POS_BITS: u16 = 14;
/// The largest position a [`Token::prov`] holds.
pub const FIELD_PROV_POS_MASK: u16 = (1 << FIELD_PROV_POS_BITS) - 1;
/// The field provenance of the token at `pos` of field `field`'s stored value (`field` 1..=3; `0` is no field).
pub fn field_prov(field: u16, pos: usize) -> u16 {
  (field << FIELD_PROV_POS_BITS) | (pos.min(FIELD_PROV_POS_MASK as usize) as u16)
}
/// The field of a [`Token::prov`] (`0` = none).
#[inline]
pub fn field_prov_field(prov: u16) -> u16 { prov >> FIELD_PROV_POS_BITS }
/// The position in its field's stored value of a [`Token::prov`].
#[inline]
pub fn field_prov_pos(prov: u16) -> u16 { prov & FIELD_PROV_POS_MASK }

/// The expansion of a token with a field provenance (`prov`, [`Token::prov`]) is that field's too: each of its tokens
/// without a provenance of its own takes `prov` (an `\and` in `\@author` expands to ink of the author field). A no-op
/// for `prov` 0, the case of every token outside a frontmatter field's store.
#[inline]
pub fn inherit_field_prov(prov: u16, expansion: &mut Tokens) {
  if prov != 0 {
    for t in expansion.unlist_mut() {
      if t.prov == 0 {
        t.prov = prov;
      }
    }
  }
}

/// A cover of a field's stored value: the positions `start..end` (end exclusive) of one field (`field`, `0` = none)
/// that an invocation consumed, its own position and its arguments. See [`field_cover_pending_merge`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FieldCover {
  /// the field (`1` title, `2` author; `0` = no cover)
  pub field: u16,
  /// the first position covered
  pub start: u16,
  /// one past the last position covered
  pub end:   u16,
}

/// The cover of a field token's invocation that produced no box (a font switch, `\relax`, a macro's expansion still
/// to be digested): the next box digested from that field's tokens within it takes it (a field's print covers its
/// font switches and the arguments a macro or primitive consumed, OXIDIZED_DESIGN #715).
#[thread_local]
static FIELD_COVER_PENDING: std::cell::Cell<FieldCover> =
  std::cell::Cell::new(FieldCover { field: 0, start: 0, end: 0 });

/// Merge `cover` with the pending cover of a boxless invocation when they touch (the same field, `cover` starting
/// inside or at the end of the pending one), clearing the pending cover either way: what a box digested from a field
/// token covers of its stored value.
pub fn field_cover_pending_merge(cover: FieldCover) -> FieldCover {
  let pending = FIELD_COVER_PENDING.replace(FieldCover::default());
  if pending.field == cover.field && pending.start <= cover.start && cover.start <= pending.end {
    FieldCover {
      field: cover.field,
      start: pending.start,
      end:   pending.end.max(cover.end),
    }
  } else {
    cover
  }
}

/// Leave `cover` pending for the next box of its field ([`field_cover_pending_merge`]), merged with the pending one
/// it continues.
pub fn field_cover_set_pending(cover: FieldCover) {
  let merged = field_cover_pending_merge(cover);
  FIELD_COVER_PENDING.set(merged);
}

/// One entry of the field ledger ([`field_ledger_push`]): the cover of a box digested from a field token, and whether
/// the structure the print is set in took the box instead of the document absorbing it ([`field_ledger_take`]).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FieldLedgerEntry {
  /// what the box covers of its field's value
  pub cover: FieldCover,
  /// taken by an alignment the document shows: a `tabular` row's end marker (a value's `\\`), digested and consumed by
  /// the alignment body, never absorbed, so no run of ink holds it (taken when the alignment's whatsit is absorbed,
  /// core `document::FIELD_TAKEN_PROPERTY`)
  pub taken: bool,
}

thread_local! {
  /// The covers of the boxes digested from field tokens, in digestion order ([`field_ledger_push`]). What a run of ink
  /// covers is the entries of the boxes it absorbed (core `Document::absorb`'s field runs keep their serials), and of
  /// those an alignment took right after them ([`field_ledger_take`]); never an entry absorbed elsewhere (another run,
  /// a formula) or never absorbed (an unused `\setbox`, `\sbox`, `{lrbox}`), OXIDIZED_DESIGN #715.
  static FIELD_LEDGER: std::cell::RefCell<Vec<FieldLedgerEntry>> =
    const { std::cell::RefCell::new(Vec::new()) };
}
/// Set once the frontmatter placement settled the field ink (core `Document::settle_field_ink`): the ledger records
/// nothing more, and nothing digested after it carries a field provenance property.
#[thread_local]
static FIELD_LEDGER_SETTLED: std::cell::Cell<bool> = std::cell::Cell::new(false);
/// The most covers the ledger keeps (a full ledger records no more: a print digested past it is no whole print).
const FIELD_LEDGER_CAP: usize = 1 << 16;

/// Is the field ledger recording ([`field_ledger_settle`])? The stomach records a field token's box only while it is.
#[inline]
pub fn field_ledger_live() -> bool { !FIELD_LEDGER_SETTLED.get() }

/// Stop recording field covers: the frontmatter placement settled the field ink (no run is marked after it).
pub fn field_ledger_settle() { FIELD_LEDGER_SETTLED.set(true); }

/// Record the cover of a box digested from a field token (the field ledger); its serial, the box's `fieldSerial`.
pub fn field_ledger_push(cover: FieldCover) -> Option<u32> {
  if FIELD_LEDGER_SETTLED.get() {
    return None;
  }
  FIELD_LEDGER.with(|ledger| {
    let mut ledger = ledger.borrow_mut();
    if ledger.len() >= FIELD_LEDGER_CAP {
      return None;
    }
    ledger.push(FieldLedgerEntry { cover, taken: false });
    u32::try_from(ledger.len() - 1).ok()
  })
}

/// Note that the structure a print is set in took the box recorded with `serial` (an alignment's row-end marker, a
/// value's `\\`): no run absorbs it, and the run its row ends may cover it ([`FieldLedgerEntry::taken`]).
pub fn field_ledger_take(serial: u32) {
  FIELD_LEDGER.with(|ledger| {
    if let Some(entry) = ledger.borrow_mut().get_mut(serial as usize) {
      entry.taken = true;
    }
  });
}

/// The positions of field `field`'s value covered by the ledger entries with the serials in `serials` (inclusive
/// `first..=last` intervals): sorted disjoint `start..end` intervals, merged where they touch.
pub fn field_ledger_cover(field: u16, serials: &[(u32, u32)]) -> Vec<(u16, u16)> {
  let mut cover: Vec<(u16, u16)> = FIELD_LEDGER.with(|ledger| {
    let ledger = ledger.borrow();
    let mut cover = Vec::new();
    for &(first, last) in serials {
      let (first, last) = (first as usize, last as usize);
      if first > last || first >= ledger.len() {
        continue;
      }
      cover.extend(
        ledger[first..=last.min(ledger.len() - 1)]
          .iter()
          .filter(|e| e.cover.field == field)
          .map(|e| (e.cover.start, e.cover.end.max(e.cover.start + 1))),
      );
    }
    cover
  });
  cover.sort_unstable();
  let mut merged: Vec<(u16, u16)> = Vec::with_capacity(cover.len());
  for (start, end) in cover {
    match merged.last_mut() {
      Some((_, e)) if start <= *e => *e = (*e).max(end),
      _ => merged.push((start, end)),
    }
  }
  merged
}

/// The entry the field ledger recorded with `serial` ([`field_ledger_push`]).
pub fn field_ledger_entry(serial: u32) -> Option<FieldLedgerEntry> {
  FIELD_LEDGER.with(|ledger| ledger.borrow().get(serial as usize).copied())
}

/// A stretch of digestion the document never keeps (a rehearsal of a frontmatter field's value, latexml_engine's
/// frontmatter observer), over which the field ledger records ([`field_ledger_live`], even once the placement settled
/// it) and after which it is as it was: the entries pushed meanwhile are dropped, the pending cover and the settled
/// flag put back, and the gate of field stamping (`document::field_prov_stamped`: a `\title` the rehearsal
/// stamps) as it was (on drop). Its serials are read while it is open ([`field_ledger_entry`]); a print the document
/// makes later is numbered as if it had never run.
pub struct FieldLedgerScope {
  len:     usize,
  pending: FieldCover,
  settled: bool,
  stamped: bool,
}

impl FieldLedgerScope {
  /// Open the scope: the ledger records from here.
  pub fn begin() -> Self {
    let len = FIELD_LEDGER.with(|ledger| ledger.borrow().len());
    FieldLedgerScope {
      len,
      pending: FIELD_COVER_PENDING.replace(FieldCover::default()),
      settled: FIELD_LEDGER_SETTLED.replace(false),
      stamped: crate::document::field_prov_stamped(),
    }
  }
}

impl Drop for FieldLedgerScope {
  fn drop(&mut self) {
    FIELD_LEDGER.with(|ledger| ledger.borrow_mut().truncate(self.len));
    FIELD_COVER_PENDING.set(self.pending);
    FIELD_LEDGER_SETTLED.set(self.settled);
    crate::document::restore_field_prov_stamped(self.stamped);
  }
}

/// Clear the pending field cover and the ledger, and record again (conversion start).
pub fn reset_field_cover_pending() {
  FIELD_COVER_PENDING.set(FieldCover::default());
  FIELD_LEDGER.with(|ledger| ledger.borrow_mut().clear());
  FIELD_LEDGER_SETTLED.set(false);
}

impl Token {
  /// simple Token constructor, wrapping over text and catcode
  pub fn new<T: AsRef<str>>(text: T, code: Catcode) -> Self {
    Token {
      text: arena::pin(text),
      code,
      prov: 0,
      #[cfg(feature = "token-locators")]
      loc: 0,
    }
  }

  /// A cheap structural fingerprint for the cycle-detection guard
  /// ([`crate::cycle_guard`]). Matches [`PartialEq`] semantics: SPACE tokens
  /// fingerprint by catcode alone (their text is irrelevant to equality).
  /// NOT a stable hash across processes — for in-run loop detection only.
  #[inline]
  pub fn cycle_fingerprint(&self) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = rustc_hash::FxHasher::default();
    self.code.hash(&mut h);
    if self.code != Catcode::SPACE {
      self.text.hash(&mut h);
    }
    h.finish()
  }

  /// Get the CS Name of the token. This is the name that definitions will be
  /// stored under; It's the same for various `different' BEGIN tokens, eg.
  pub fn get_cs_name(&self) -> SymStr {
    if self.code.is_primitive() {
      self.code.name_sym()
    } else {
      self.get_sym()
    }
  }

  /// Execute a closure using the CS Name of the token.
  /// This is the name that definitions will be stored under;
  /// It's the same for various `different' BEGIN tokens, eg.
  pub fn with_cs_name<R, FnR>(&self, caller: FnR) -> R
  where FnR: FnOnce(&str) -> R {
    if self.code.is_primitive() {
      caller(self.code.name())
    } else {
      self.with_str(caller)
    }
  }

  /// artificial, but avoids the data race
  pub fn pin_cs_name(&self) -> SymStr {
    if self.code.is_primitive() {
      self.code.name_sym()
    } else {
      self.get_sym()
    }
  }

  /// Get the fixed name of a primitive catcode, or empty string otherwise
  pub fn get_primitive_name(&self) -> Option<&'static str> {
    if self.code.is_primitive() {
      Some(self.code.name())
    } else {
      None
    }
  }

  /// Get the CS name only if the catcode is executable!
  pub fn get_executable_name(&self) -> String {
    let cc = self.code;
    if cc.is_executable() {
      self
        .get_primitive_name()
        .map(ToString::to_string)
        .unwrap_or_else(|| self.with_str(|text| text.to_string()))
    } else {
      String::new()
    }
  }

  /// Intersect executable and primitive
  pub fn get_executable_primitive_name(&self) -> Option<&'static str> {
    let cc = self.code;
    if cc.is_executable() && cc.is_primitive() {
      Some(self.code.name())
    } else {
      None
    }
  }

  /// Use the ticket representing the interned "text" of the token
  pub fn get_sym(&self) -> SymStr { self.text }
  /// Use the interned &str "text" of the token
  /// use `to_string` instead for an owned String with simpler
  pub fn with_str<R, FnR>(&self, caller: FnR) -> R
  where FnR: FnOnce(&str) -> R {
    arena::with(self.text, caller)
  }

  /// True for any member of the `\special_relax` no-expand family — the
  /// representation of a `\noexpand`'d (expandable or undefined) CS/active token.
  /// Such a token is a CS whose NAME is `\special_relax` `\x01` `<shadowed text>`,
  /// carrying the shadowed token's identity PER-TOKEN (faithful to TeX's
  /// `no_expand_flag`, which preserves `cur_cs`), while the whole family resolves
  /// to `\special_relax`'s `\relax` meaning ([`crate::state::lookup_meaning`]
  /// fallback). The bare `\special_relax` (no suffix) is the no-shadow case
  /// (`\dont_expand` at end-of-input). `\x01` is never valid in a CS name or as
  /// an active char, so the encoding is unambiguous.
  pub fn is_noexpand_family(&self) -> bool {
    if self.code != Catcode::CS {
      return false;
    }
    // Per-symbol memo: this runs ×2 per CS token via `state::meaning_key`
    // (read_x_token decides whether to expand, invoke_token how to invoke),
    // and the string-prefix probe was ~2% of digest self-time (2026-08-23
    // audit). A symbol's text never changes (append-only arena), so the
    // answer is memoized by symbol index: 0 = unknown, 1 = no, 2 = yes, for
    // the indices below `NOEXPAND_MEMO_SPAN`. Cleared with the arena in
    // `reset_thread_engine` (symbol indices are reused after `arena::reset`).
    use string_interner::Symbol;
    let idx = self.text.to_usize();
    let memoized = idx < NOEXPAND_MEMO_SPAN;
    if memoized {
      let cached = NOEXPAND_FAMILY_MEMO.with(|m| m.borrow().get(idx).copied().unwrap_or(0));
      if cached != 0 {
        return cached == 2;
      }
    }
    let is_family = self.with_str(|s| {
      s.starts_with(NOEXPAND_PREFIX)
        && (s.len() == NOEXPAND_PREFIX.len() || s.as_bytes()[NOEXPAND_PREFIX.len()] == NOEXPAND_SEP)
    });
    if memoized {
      NOEXPAND_FAMILY_MEMO.with(|m| {
        let mut memo = m.borrow_mut();
        if memo.len() <= idx {
          memo.resize(idx + 1, 0);
        }
        memo[idx] = if is_family { 2 } else { 1 };
      });
    }
    is_family
  }

  /// Recover the shadowed token from a `\special_relax`-family token, if it
  /// shadows one (i.e. not the bare `\special_relax`). The shadowed token is a
  /// CS (text begins `\`) or an active char.
  pub fn noexpand_shadowed(&self) -> Option<Token> {
    if self.code != Catcode::CS {
      return None;
    }
    // Compute the owned shadowed name + its catcode INSIDE the `with_str`
    // borrow, then `arena::pin` it OUTSIDE. `with_str`'s `s` (hence `rest`)
    // aliases the arena's append-only buffer; a re-entrant `arena::pin(rest)`
    // could reallocate that buffer mid-intern and read freed memory. Owning the
    // name first (and pinning after the borrow ends) removes the hazard — and
    // satisfies clippy::unnecessary_to_owned, which can't see the aliasing.
    let (name, code) = self.with_str(|s| {
      let rest = s.strip_prefix(NOEXPAND_PREFIX)?;
      let rest = rest.strip_prefix(NOEXPAND_SEP as char)?;
      let code = if rest.starts_with('\\') {
        Catcode::CS
      } else {
        Catcode::ACTIVE
      };
      Some((rest.to_string(), code))
    })?;
    Some(Token {
      text: arena::pin(name),
      code,
      prov: 0,
      #[cfg(feature = "token-locators")]
      loc: 0,
    })
  }

  /// Return the character code of  character part of the token, or 256 if it is a control
  /// sequence. A SPACE token's code is always 32: tex.web §289 `space_token =
  /// 2^8·spacer + " "` and §349 (`mid_line + car_ret` → `cur_chr := " "`) — an
  /// end-of-line becomes a space with character code 32, whatever produced it.
  /// This token keeps its `"\n"` TEXT (Perl's data model, `PRESERVE_NEWLINES`,
  /// the `tex=` reversion) but Perl's `getCharcode` returned `ord("\n")` = 10
  /// for it (Token.pm:243), so `\if` compared an end-of-line space unequal to a
  /// typed one: stringstrings' `\if\BlankSpace#1` took the wrong branch on a
  /// `\@for` item that was a lone newline and desynchronised the conditional
  /// stack (`Extra \else`), then tikzviolinplots' KDE loop ran away
  /// (batch 56gc, OXIDIZED_DESIGN #260).
  pub fn get_charcode(&self) -> u32 {
    if self.code == Catcode::CS {
      256
    } else if self.code == Catcode::SPACE {
      32
    } else {
      self.with_str(|text| {
        if let Some(c) = text.chars().next() {
          c as u32
        } else {
          0
        }
      })
    }
  }

  /// Return the catcode of the token.
  pub fn get_catcode(&self) -> Catcode { self.code }
  /// is the current one
  pub fn is_executable(&self) -> bool { self.code.is_executable() }

  /// neutralize really should only retroactively imitate what Semiverbatim would have done.
  /// So, it needs to neutralize those in SPECIALS
  /// NOTE that although '%' gets it's catcode changed in Semiverbatim,
  /// I'm pretty sure we do NOT want to neutralize comments (turn them into Catcode::OTHER)
  /// here, since if comments do get into the Tokens, that will introduce weird crap into the
  /// stream.
  pub fn neutralize(self, extraspecials: &[char]) -> Token {
    let first_c: Option<char> = self.with_str(|text| text.chars().next());
    let ch = match first_c {
      Some(ch) => ch,
      None => return self,
    };
    let cc = self.code;
    if cc.is_neutralizable() {
      for extra in extraspecials {
        if extra == &ch {
          let mut tmp = [0u8; 4];
          let s = ch.encode_utf8(&mut tmp);
          return T_OTHER!(s);
        }
      }
      let maybe_return = state::with_value("SPECIALS", |specials_opt| {
        if let Some(Stored::Chars(specials_list)) = specials_opt {
          for special in specials_list.iter() {
            if *special == ch {
              let mut tmp = [0u8; 4];
              let s = ch.encode_utf8(&mut tmp);
              return Some(T_OTHER!(s));
            }
          }
        }
        None
      });
      if let Some(token) = maybe_return {
        return token;
      }
    }
    self
  }

  pub fn as_other(&self) -> Token {
    Token {
      text: self.text,
      code: Catcode::OTHER,
      prov: 0,
      #[cfg(feature = "token-locators")]
      loc: 0,
    }
  }
  pub fn as_cs(&self) -> Token {
    Token {
      text: self.text,
      code: Catcode::CS,
      prov: 0,
      #[cfg(feature = "token-locators")]
      loc: 0,
    }
  }

  pub fn substitute_parameters(self, args: &[&Token]) -> Self {
    if self.code == Catcode::ARG {
      self.with_str(|text| {
        let arg_idx = text
          .parse::<usize>()
          .expect("ARG catcode tokens should always contain numeric literals as text");
        *args[arg_idx - 1]
      })
    } else {
      self
    }
  }

  /// A Token reverts to itself
  pub fn revert(self) -> Token { self }

  /// A string form which is primarily used for error-reporting
  pub fn stringify(&self) -> String {
    self.with_str(|text| {
      // Make the token's char content more printable, since this is for a visual messages.
      let display_text = if text.len() == 1 {
        let c = text.chars().next().unwrap() as u16;
        if c < 0x020 {
          Cow::Owned(s!("U+{:04x}/{}", c, CONTROLNAME[c as usize]))
        } else {
          Cow::Borrowed(text)
        }
      } else {
        Cow::Borrowed(text)
      };
      s!("{}[{}]", self.code.short_name(), display_text)
    })
  }

  pub fn to_register(&self) -> Option<Rc<Register>> { state::lookup_register_definition(self) }

  pub fn to_number(&self) -> Number {
    Number::new(self.with_str(|text| text.parse::<i64>()).unwrap_or(0))
  }

  pub fn to_dimension(&self) -> Dimension {
    Dimension::new_f64(self.with_str(|text| text.parse::<f64>().unwrap_or(0.0)))
  }

  pub fn to_mu_dimension(&self) -> MuDimension {
    MuDimension::new_f64(self.with_str(|s| s.parse::<f64>()).unwrap_or(0.0))
  }

  pub fn to_glue(&self) -> Glue {
    Glue::new_f64(self.with_str(|s| s.parse::<f64>()).unwrap_or(0.0))
  }

  pub fn to_mu_glue(&self) -> MuGlue {
    MuGlue::new_f64(self.with_str(|s| s.parse::<f64>()).unwrap_or(0.0))
  }

  pub fn to_float(&self) -> Float {
    Float::new_f64(self.with_str(|s| s.parse::<f64>()).unwrap_or(0.0))
  }

  pub fn be_digested(self) -> Result<Digested> { crate::stomach::digest(Tokens::new(vec![self])) }

  /// Check whether the current token is defined as `other`.
  /// That is, whether it is equal to `other`, or \let to `other`.
  /// `other` is is presumed to be some "constant", explicit token,
  /// such as  `T_SPACE` or `T_CS!("\\endcsname")`.
  pub fn defined_as(&self, other: &Token) -> bool {
    let cc = self.code;
    let occ = other.get_catcode();
    if (cc == occ) && ((occ == Catcode::SPACE) || (self.text == other.get_sym())) {
      return true;
    }
    if matches!(cc, Catcode::CS | Catcode::ACTIVE) {
      // Use the closure-based `with_meaning` — Token is `Copy`, so
      // extracting a `Token` from the borrowed Stored is an implicit
      // copy, not a clone. Avoids a full `Stored::clone()` per call
      // (defined_as fires ~1% of total instructions on siunitx/
      // physics-heavy docs).
      let letto_opt: Option<Token> = state::with_meaning(self, |defn_opt| {
        defn_opt.and_then(|defn| match defn {
          Stored::Token(t) => Some(*t),
          Stored::Expandable(inner) => Some(*inner.get_cs()),
          Stored::Primitive(inner) => Some(*inner.get_cs()),
          Stored::MathPrimitive(inner) => Some(*inner.get_cs()),
          Stored::Register(inner) => Some(*inner.get_cs()),
          Stored::Conditional(inner) => Some(*inner.get_cs()),
          Stored::Constructor(inner) => Some(*inner.get_cs()),
          _ => None,
        })
      });
      if let Some(letto) = letto_opt
        && (letto.get_catcode() == occ)
        && ((occ == Catcode::SPACE) || letto.get_sym() == other.get_sym())
      {
        return true;
      }
    }
    false
  }
}

// A simple (constant!) auto-cast for &str to Token. Beware this will not respect the current
// catcodes in state (and @ is OTHER).
impl From<&str> for Token {
  fn from(text: &str) -> Token {
    match text.chars().next() {
      Some('{') => T_BEGIN!(),
      Some('}') => T_END!(),
      Some('$') => T_MATH!(),
      Some('#') => T_PARAM!(),
      Some('&') => T_ALIGN!(),
      Some('^') => T_SUPER!(),
      Some('_') => T_SUB!(),
      Some('\\') => T_CS!(text),
      Some('%') => T_COMMENT!(text),
      _ => {
        // tex.web §464 `str_toks`: a string becomes tokens with catcode
        // OTHER, except the space character (catcode SPACE). No catcode is
        // ever inferred from the character class — U+00A0/U+0085 are Unicode
        // whitespace but under the pdfTeX byte mouth (K10) they are the bytes
        // 0xA0/0x85 of 素 or 銀紅, and letters are letters only through a
        // mouth's catcode table. The punctuation arms above are the callers'
        // alias contract (`{`, `}`, `\cs`…), not a catcode guess.
        if text.chars().all(|c| c.is_ascii_whitespace()) {
          T_SPACE!()
        } else {
          T_OTHER!(text)
        }
      },
    }
  }
}

// ── Token-origin side arena (token-locators feature) ───────────────────────
// Per-conversion store mapping a Token's `loc` handle (1-based; 0 = none) to its
// captured source start. Tokens carry only the u32 handle (Token stays 12 bytes);
// this holds the (source, line, col). Appended in `read_token`, read by the
// digestion consumer to give a text run its true span. Cleared per conversion.
// See docs/performance/SOURCE_PROVENANCE.md §3.1.1.
#[cfg(feature = "token-locators")]
#[derive(Clone, Copy, Debug)]
pub struct TokenStart {
  pub source:    SymStr,
  pub line:      u32,
  pub col:       u32,
  /// `true` when this origin was *inherited* from a macro invocation rather
  /// than read directly from a mouth — i.e. the token is synthesized
  /// expansion output (`\today → "May 25, 2026"`) attributed to its `\today`
  /// call site. The content-range recovery in `constructor::child_span`
  /// prefers genuine (read-from-source) origins and only falls back to
  /// inherited ones, so a `\section{Intro}`'s structural body literals — now
  /// carrying an inherited origin — never widen the title's content-exact
  /// span. See docs/performance/SOURCE_PROVENANCE.md §3.1.3.
  pub inherited: bool,
}

#[cfg(feature = "token-locators")]
thread_local! {
  static TOKEN_ORIGINS: std::cell::RefCell<Vec<TokenStart>> =
    const { std::cell::RefCell::new(Vec::new()) };
}

/// Append a token's source start, returning its 1-based handle (`0` is reserved
/// for "no origin"). Only called on the source-map precision path.
#[cfg(feature = "token-locators")]
pub fn push_token_origin(source: SymStr, line: u32, col: u32) -> u32 {
  TOKEN_ORIGINS.with(|o| {
    let mut v = o.borrow_mut();
    v.push(TokenStart {
      source,
      line,
      col,
      inherited: false,
    });
    v.len() as u32 // index + 1
  })
}

/// Derive an *inherited* origin from an existing handle: look up the
/// invocation token's start, push a copy flagged `inherited`, and return its
/// new handle (`0` if `handle` is the no-origin sentinel or out of range). One
/// call per macro expansion; the returned handle is shared by every synthesized
/// result token. See `push_token_origin` and docs/performance/SOURCE_PROVENANCE.md §3.1.3.
#[cfg(feature = "token-locators")]
pub fn push_inherited_origin(handle: u32) -> u32 {
  if handle == 0 {
    return 0;
  }
  TOKEN_ORIGINS.with(|o| {
    let mut v = o.borrow_mut();
    let Some(mut start) = v.get((handle - 1) as usize).copied() else {
      return 0;
    };
    start.inherited = true;
    v.push(start);
    v.len() as u32
  })
}

/// Resolve a token `loc` handle to its origin (`None` for the `0` sentinel or an
/// out-of-range handle).
#[cfg(feature = "token-locators")]
pub fn get_token_origin(handle: u32) -> Option<TokenStart> {
  if handle == 0 {
    return None;
  }
  TOKEN_ORIGINS.with(|o| o.borrow().get((handle - 1) as usize).copied())
}

/// Reset the arena at the start of a conversion (handles are per-conversion).
#[cfg(feature = "token-locators")]
pub fn clear_token_origins() { TOKEN_ORIGINS.with(|o| o.borrow_mut().clear()); }

#[cfg(test)]
mod tests {
  use super::*;

  /// The `is_noexpand_family` memo is indexed by arena byte offset, so it covers
  /// only the first `NOEXPAND_MEMO_SPAN` bytes: a control sequence interned after
  /// more text than that is checked directly and the memo stays within the span
  /// (unbounded, it grew to the arena's size, 1.5 GB after tkz-grapheur's long
  /// pgf path). The answers are the same on both sides of the span, and on a
  /// memo hit.
  #[test]
  fn noexpand_memo_stays_within_its_span() {
    std::thread::spawn(|| {
      let early = T_CS!(NOEXPAND_PREFIX);
      assert!(early.is_noexpand_family());
      let _bulk = arena::pin("x".repeat(NOEXPAND_MEMO_SPAN + (1 << 20)));
      let late_family = T_CS!(format!("{NOEXPAND_PREFIX}{}\\foo", NOEXPAND_SEP as char));
      let late_other = T_CS!("\\noexpandmemoprobe");
      assert!(early.is_noexpand_family());
      assert!(late_family.is_noexpand_family());
      assert!(!late_other.is_noexpand_family());
      let len = NOEXPAND_FAMILY_MEMO.with(|m| m.borrow().len());
      assert!(len <= NOEXPAND_MEMO_SPAN, "memo grew to {len} bytes");
      reset_noexpand_family_memo();
      arena::reset();
    })
    .join()
    .unwrap();
  }

  /// `Token` size invariant (docs/performance/SOURCE_PROVENANCE.md §3.1.1): 8 bytes by
  /// default (`SymStr` + `Catcode`), 12 only under the `token-locators`
  /// precision build (+ the `u32` origin handle). Guards the corpus/parity/
  /// distribution builds against an accidental widening.
  #[test]
  fn token_size_invariant() {
    #[cfg(not(feature = "token-locators"))]
    assert_eq!(
      size_of::<Token>(),
      8,
      "default Token must stay 8 bytes (SymStr + Catcode + prov)"
    );
    #[cfg(feature = "token-locators")]
    assert_eq!(
      std::mem::size_of::<Token>(),
      12,
      "token-locators Token is 8 + a u32 origin handle"
    );
  }

  /// The field ledger (OXIDIZED_DESIGN #715): an entry an alignment took is marked so, a run's cover is the entries of
  /// the serials it holds (never an entry between them it does not), and once the frontmatter placement settled the
  /// field ink nothing more is recorded (fms1 round-3 review finding 5).
  #[test]
  fn field_ledger_take_cover_and_settle() {
    reset_field_cover_pending();
    let cover = |start, end| FieldCover { field: 1, start, end };
    let a = field_ledger_push(cover(0, 5)).unwrap();
    let unused = field_ledger_push(cover(5, 9)).unwrap();
    let marker = field_ledger_push(cover(9, 10)).unwrap();
    field_ledger_take(marker);
    assert!(field_ledger_entry(marker).is_some_and(|e| e.taken));
    assert!(field_ledger_entry(unused).is_some_and(|e| !e.taken));
    assert_eq!(field_ledger_cover(1, &[(a, a)]), vec![(0, 5)]);
    assert_eq!(field_ledger_cover(1, &[(a, a), (marker, marker)]), vec![
      (0, 5),
      (9, 10)
    ]);
    assert!(field_ledger_live());
    field_ledger_settle();
    assert!(!field_ledger_live());
    assert_eq!(field_ledger_push(cover(10, 11)), None);
    reset_field_cover_pending();
    assert!(field_ledger_live());
    assert_eq!(field_ledger_entry(a), None);
  }

  /// `#[repr(C)]` and a two-byte `Catcode` keep `Token` gap-free with `code` at byte 4 (`text` u32 at 0, `code` u16
  /// at 4, `prov` u16 at 6, `loc` at 8 under token-locators). Why: under Rust's default layout `prov` took byte 4 and
  /// `code` moved to byte 6, and a pad byte stayed, so every token move on the gullet's read path became a field-wise
  /// copy (+1.29 % instructions on an empty article, +3.27 % on 2311.18489; gap-free +0.65 % / +1.66 %, fms1 round 3).
  /// OXIDIZED_DESIGN #715.
  #[test]
  fn token_layout_code_at_byte_4() {
    assert_eq!(std::mem::offset_of!(Token, text), 0);
    assert_eq!(
      std::mem::offset_of!(Token, code),
      4,
      "Token.code must sit at byte 4 (read-path codegen)"
    );
    assert_eq!(
      size_of::<Catcode>(),
      2,
      "Catcode fills bytes 4-5: no pad byte before prov"
    );
    assert_eq!(std::mem::offset_of!(Token, prov), 6);
    #[cfg(feature = "token-locators")]
    assert_eq!(std::mem::offset_of!(Token, loc), 8);
    assert_eq!(
      size_of::<Option<Token>>(),
      size_of::<Token>(),
      "Option<Token> keeps its niche"
    );
  }

  /// Per-token origin capture (token-locators): each char token read from a
  /// mouth carries a handle resolving to its exact (line, col). This is the leaf
  /// accuracy that mouth-snapshot (Experiments 1–2) and digested-child assembly
  /// (Experiment 3) could not provide — the position now travels *with the
  /// token*. See docs/performance/SOURCE_PROVENANCE.md §3.1.1.
  #[cfg(feature = "token-locators")]
  #[test]
  fn token_origin_capture() {
    super::clear_token_origins();
    // "Hello" — five letters at 1-indexed columns 1..=5 on line 1.
    let toks = crate::mouth::tokenize("Hello");
    let got: Vec<(u32, u32)> = toks
      .unlist_ref()
      .iter()
      .map(|t| {
        let o = super::get_token_origin(t.loc).expect("token carries an origin handle");
        (o.line, o.col)
      })
      .collect();
    assert_eq!(
      got,
      vec![(1, 1), (1, 2), (1, 3), (1, 4), (1, 5)],
      "each letter's captured (line, col) must be exact"
    );
  }

  #[test]
  fn catcode_name_covers_all_variants() {
    // Ensure every Catcode variant produces a non-empty name.
    use Catcode::*;
    for cc in [
      ESCAPE, BEGIN, END, MATH, ALIGN, EOL, PARAM, SUPER, SUB, SPACE, IGNORE, LETTER, OTHER,
      ACTIVE, COMMENT, INVALID, CS, MARKER, ARG,
    ] {
      assert!(!cc.name().is_empty(), "{cc:?}.name() is empty");
    }
  }

  #[test]
  fn catcode_name_specific_values() {
    assert_eq!(Catcode::ESCAPE.name(), "Escape");
    assert_eq!(Catcode::BEGIN.name(), "Begin");
    assert_eq!(Catcode::CS.name(), "ControlSequence");
    assert_eq!(Catcode::LETTER.name(), "Letter");
  }

  #[test]
  fn catcode_short_name_starts_with_t_prefix() {
    use Catcode::*;
    for cc in [
      ESCAPE, BEGIN, END, MATH, ALIGN, EOL, PARAM, SUPER, SUB, SPACE, IGNORE, LETTER, OTHER,
      ACTIVE, COMMENT, INVALID, CS, MARKER, ARG,
    ] {
      assert!(
        cc.short_name().starts_with("T_"),
        "{cc:?}.short_name() = {} lacks T_ prefix",
        cc.short_name()
      );
    }
  }

  #[test]
  fn catcode_meaning_mostly_nonempty() {
    // Most variants have a TeX-meaning description.
    assert!(!Catcode::ESCAPE.meaning().is_empty());
    assert!(!Catcode::LETTER.meaning().is_empty());
    assert!(!Catcode::OTHER.meaning().is_empty());
    // A few (e.g. CS/MARKER/ARG) fall through to "".
    assert_eq!(Catcode::CS.meaning(), "");
  }

  #[test]
  fn is_primitive_checks() {
    use Catcode::*;
    // TeX primitives:
    for cc in [
      ESCAPE, BEGIN, END, MATH, ALIGN, EOL, PARAM, SUPER, SUB, SPACE,
    ] {
      assert!(cc.is_primitive(), "{cc:?} should be primitive");
    }
    // Non-primitives:
    for cc in [
      IGNORE, LETTER, OTHER, ACTIVE, COMMENT, INVALID, CS, MARKER, ARG,
    ] {
      assert!(!cc.is_primitive(), "{cc:?} should not be primitive");
    }
  }

  #[test]
  fn is_executable_checks() {
    use Catcode::*;
    for cc in [BEGIN, END, MATH, ALIGN, SUPER, SUB, ACTIVE, CS] {
      assert!(cc.is_executable(), "{cc:?} should be executable");
    }
    for cc in [
      EOL, ESCAPE, PARAM, SPACE, IGNORE, LETTER, OTHER, COMMENT, INVALID, MARKER, ARG,
    ] {
      assert!(!cc.is_executable(), "{cc:?} should not be executable");
    }
  }

  #[test]
  fn is_neutralizable_set() {
    use Catcode::*;
    for cc in [MATH, ALIGN, PARAM, SUPER, SUB, ACTIVE] {
      assert!(cc.is_neutralizable(), "{cc:?}");
    }
    assert!(!CS.is_neutralizable());
    assert!(!LETTER.is_neutralizable());
  }

  #[test]
  fn is_active_or_cs_narrow_set() {
    assert!(Catcode::ACTIVE.is_active_or_cs());
    assert!(Catcode::CS.is_active_or_cs());
    assert!(!Catcode::LETTER.is_active_or_cs());
    assert!(!Catcode::ESCAPE.is_active_or_cs());
  }

  #[test]
  fn is_absorbable_space_letter_other_comment() {
    use Catcode::*;
    assert!(SPACE.is_absorbable());
    assert!(LETTER.is_absorbable());
    assert!(OTHER.is_absorbable());
    assert!(COMMENT.is_absorbable());
    // All else not absorbable.
    assert!(!CS.is_absorbable());
    assert!(!BEGIN.is_absorbable());
  }

  #[test]
  fn is_gullet_holdable_comment_marker_only() {
    assert!(Catcode::COMMENT.is_gullet_holdable());
    assert!(Catcode::MARKER.is_gullet_holdable());
    assert!(!Catcode::SPACE.is_gullet_holdable());
    assert!(!Catcode::LETTER.is_gullet_holdable());
  }

  #[test]
  fn is_balanced_interesting_begin_end_marker() {
    assert!(Catcode::BEGIN.is_balanced_interesting());
    assert!(Catcode::END.is_balanced_interesting());
    assert!(Catcode::MARKER.is_balanced_interesting());
    assert!(!Catcode::LETTER.is_balanced_interesting());
    assert!(!Catcode::MATH.is_balanced_interesting());
  }

  #[test]
  fn catcode_u8_roundtrip() {
    // From<Catcode> for u8 + From<u8> for Catcode should round-trip
    // (at least for the documented range 0..=18).
    use Catcode::*;
    for cc in [
      ESCAPE, BEGIN, END, MATH, ALIGN, EOL, PARAM, SUPER, SUB, SPACE, IGNORE, LETTER, OTHER,
      ACTIVE, COMMENT, INVALID, CS, MARKER, ARG,
    ] {
      let b: u8 = cc.into();
      let cc2: Catcode = b.into();
      assert_eq!(cc, cc2, "roundtrip broke for {cc:?} (u8={b})");
    }
  }

  #[test]
  fn token_new_and_display() {
    let t = Token::new("foo", Catcode::LETTER);
    assert_eq!(format!("{t}"), "foo");
    assert_eq!(t.code, Catcode::LETTER);
  }

  #[test]
  fn token_arg_display_prepends_hash() {
    // ARG catcode prepends # in Display.
    let t = Token::new("1", Catcode::ARG);
    assert_eq!(format!("{t}"), "#1");
  }
}

#[cfg(test)]
mod charcode_tests {
  use super::{Catcode, Token};

  /// tex.web §289/§349: every space token has character code 32, an
  /// end-of-line space included (its text stays "\n").
  #[test]
  fn end_of_line_space_has_character_code_32() {
    assert_eq!(Token::new("\n", Catcode::SPACE).get_charcode(), 32);
    assert_eq!(Token::new(" ", Catcode::SPACE).get_charcode(), 32);
    assert_eq!(Token::new("A", Catcode::LETTER).get_charcode(), 65);
  }
}
