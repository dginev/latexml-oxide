use crate::prelude::*;

LoadDefinitions!({
  DefConstructor!("\\mathbb{}", "#1",
    bounded => true, require_math => true,
    font => {encoding => "U", family => "blackboard", series => "medium", shape => "upright"});

  DefConstructor!("\\textbb{}", "#1",
    bounded => true, forbid_math => true,
    mode => "restricted_horizontal", enter_horizontal => true,
    font => {encoding => "U", family => "blackboard"});

  DefConstructor!("\\bbfamily", "",
    font => {encoding => "U", family => "blackboard", series => "medium", shape => "upright"});

  // The blackboard layout is bbold's own font's, so its map is keyed to that font's family — the one this binding's
  // `\mathbb`/`\textbb`/`\bbfamily` name (`blackboard`, which `\fontfamily{bbold}` also selects, font.rs) and the one
  // Ubbold.fd and bbold.sty name in NFSS (`bbold`, a `\DeclareSymbolFont{…}{U}{bbold}{m}{n}`) — not to the shared `U`
  // encoding. Perl registers it for `U` itself (bbold.sty.ltxml:30), so every other `U` font without a map of its own
  // decoded through it: mathx's `\widehat` (mathabx.dcl:944) printed 𝕡, stmaryrd's symbols blackboard letters, all
  // silently (KNOWN_PERL_ERRORS #1203, OXIDIZED_DESIGN_DIVERGENCES #914; guard
  // `perfect_kernel_batch89d::bbold_map_is_its_own_familys`). Still reached: bbm's and bbmss's fonts, which font.rs
  // also names `blackboard` (SYNC_STATUS Font-Selection residual 12; RED repro
  // fonts-nfss/bbold_map_reaches_bbm_through_the_blackboard_alias).
  #[rustfmt::skip]
  let blackboard: Rc<[Option<char>]> = mixrc![
    // \Gamma     \Delta      \Theta      \Lambda      \Xi         \Pi         \Sigma      \Upsilon
    '\u{213E}', '\u{0394}', '\u{0398}', '\u{039B}', '\u{039E}', '\u{213F}', '\u{2140}', '\u{03A5}',
    // \Phi       \Psi        \Omega      alpha        beta        gamma       delta       epsilon
    '\u{03A6}', '\u{03A8}', '\u{03A9}', '\u{03B1}', '\u{03B2}', '\u{213D}', '\u{03B4}', '\u{03F5}',
    // zeta       eta         theta       iota         kappa      lambda       mu         nu
    '\u{03B6}', '\u{03B7}', '\u{03B8}', '\u{03B9}', '\u{03BA}', '\u{03BB}', '\u{03BC}', '\u{03BD}',
    // xi         pi          rho         sigma       tau         upsilon     phi         chi
    '\u{03BE}', '\u{213C}', '\u{03C1}', '\u{03C3}', '\u{03C4}', '\u{03C5}', '\u{03D5}', '\u{03C7}',
    // psi
    '\u{03C8}', '!',        '"',        '#',        '$',        '%',        '&',        '\'',
    '\u{27EC}', '\u{27ED}', '*',        '+',        ',',        '-',        '.',        '/',
    // 0          1           2           3             4           5          6           7
    '\u{1D7D8}', '\u{1D7D9}', '\u{1D7DA}', '\u{1D7DB}', '\u{1D7DC}', '\u{1D7DD}', '\u{1D7DE}', '\u{1D7DF}',
    // 8          9           :           ;             <           /          >
    '\u{1D7E0}', '\u{1D7E1}', ':',        ';',        '<',        '\u{22C5}', '>',        '?',
    // partial    A           B           C             D           E          F           G
    '@',        '\u{1D538}', '\u{1D539}', '\u{2102}', '\u{1D53B}', '\u{1D53C}', '\u{1D53D}', '\u{1D53E}',
    // H          I           J           K             L           M          N           O
    '\u{210D}', '\u{1D540}', '\u{1D541}', '\u{1D542}', '\u{1D543}', '\u{1D544}', '\u{2115}', '\u{1D546}',
    // P          Q           R           S             T           U          V           W
    '\u{2119}', '\u{211A}', '\u{211D}', '\u{1D54A}', '\u{1D54B}', '\u{1D54C}', '\u{1D54D}', '\u{1D54E}',
    // X          Y           Z
    '\u{1D54F}', '\u{1D550}', '\u{2124}', '\u{27E6}', '\\',       '\u{27E7}', '{',        '}',
    // ell        a           b           c             d           e          f           g
    '`',        '\u{1D552}', '\u{1D553}', '\u{1D554}', '\u{1D555}', '\u{1D556}', '\u{1D557}', '\u{1D558}',
    // h          i           j           k             l           m          n           o
    '\u{1D559}', '\u{1D55A}', '\u{1D55B}', '\u{1D55C}', '\u{1D55D}', '\u{1D55E}', '\u{1D55F}', '\u{1D560}',
    // p          q           r           s             t           u          v           w
    '\u{1D561}', '\u{1D562}', '\u{1D563}', '\u{1D564}', '\u{1D565}', '\u{1D566}', '\u{1D567}', '\u{1D568}',
    // x          y           z          -             |          --          ``          omega
    '\u{1D569}', '\u{1D56A}', '\u{1D56B}', '-',        '|',       '\u{2013}', '`',        '\u{03C9}'
  ];
  for family in ["blackboard", "bbold"] {
    DeclareFontMap!("U", blackboard.clone(), family);
    // Position 0x7E is two backtick characters in Perl, not U+201C (ligature result)
    DeclareFontMapMultichar!("U", family, {
      0x7Eu8 => "``",
    });
  }
});
