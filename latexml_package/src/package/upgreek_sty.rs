use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: upgreek.sty.ltxml — upright Greek letters
  DefMath!("\\upalpha",   '\u{03B1}', font => { shape => "upright", forceshape => true });
  DefMath!("\\upbeta",    '\u{03B2}', font => { shape => "upright", forceshape => true });
  DefMath!("\\upgamma",   '\u{03B3}', font => { shape => "upright", forceshape => true });
  DefMath!("\\updelta",   '\u{03B4}', font => { shape => "upright", forceshape => true });
  DefMath!("\\upepsilon", '\u{03F5}', font => { shape => "upright", forceshape => true });
  DefMath!("\\upzeta",    '\u{03B6}', font => { shape => "upright", forceshape => true });
  DefMath!("\\upeta",     '\u{03B7}', font => { shape => "upright", forceshape => true });
  DefMath!("\\uptheta",   '\u{03B8}', font => { shape => "upright", forceshape => true });
  DefMath!("\\upiota",    '\u{03B9}', font => { shape => "upright", forceshape => true });
  DefMath!("\\upkappa",   '\u{03BA}', font => { shape => "upright", forceshape => true });
  DefMath!("\\uplambda",  '\u{03BB}', font => { shape => "upright", forceshape => true });
  DefMath!("\\upmu",      '\u{03BC}', font => { shape => "upright", forceshape => true });
  DefMath!("\\upnu",      '\u{03BD}', font => { shape => "upright", forceshape => true });
  DefMath!("\\upxi",      '\u{03BE}', font => { shape => "upright", forceshape => true });
  DefMath!("\\uppi",      '\u{03C0}', font => { shape => "upright", forceshape => true });
  DefMath!("\\uprho",     '\u{03C1}', font => { shape => "upright", forceshape => true });
  DefMath!("\\upsigma",   '\u{03C3}', font => { shape => "upright", forceshape => true });
  DefMath!("\\uptau",     '\u{03C4}', font => { shape => "upright", forceshape => true });
  DefMath!("\\upupsilon", '\u{03C5}', font => { shape => "upright", forceshape => true });
  DefMath!("\\upphi",     '\u{03D5}', font => { shape => "upright", forceshape => true });
  DefMath!("\\upchi",     '\u{03C7}', font => { shape => "upright", forceshape => true });
  DefMath!("\\uppsi",     '\u{03C8}', font => { shape => "upright", forceshape => true });
  DefMath!("\\upomega",   '\u{03C9}', font => { shape => "upright", forceshape => true });

  DefMath!("\\upvarepsilon", '\u{03B5}', font => { shape => "upright", forceshape => true });
  DefMath!("\\upvartheta",   '\u{03D1}', font => { shape => "upright", forceshape => true });
  DefMath!("\\upvarpi",      '\u{03D6}', font => { shape => "upright", forceshape => true });
  Let!("\\upvarrho",   "\\uprho");
  Let!("\\upvarsigma", "\\upsigma");
  DefMath!("\\upvarphi", '\u{03C6}', font => { shape => "upright", forceshape => true });

  DefMath!("\\Upgamma",   '\u{0393}', font => { shape => "upright", forceshape => true });
  DefMath!("\\Updelta",   '\u{0394}', font => { shape => "upright", forceshape => true });
  DefMath!("\\Uptheta",   '\u{0398}', font => { shape => "upright", forceshape => true });
  DefMath!("\\Uplambda",  '\u{039B}', font => { shape => "upright", forceshape => true });
  DefMath!("\\Upxi",      '\u{039E}', font => { shape => "upright", forceshape => true });
  DefMath!("\\Uppi",      '\u{03A0}', font => { shape => "upright", forceshape => true });
  DefMath!("\\Upsigma",   '\u{03A3}', font => { shape => "upright", forceshape => true });
  DefMath!("\\Upupsilon", '\u{03A5}', font => { shape => "upright", forceshape => true });
  DefMath!("\\Upphi",     '\u{03A6}', font => { shape => "upright", forceshape => true });
  DefMath!("\\Uppsi",     '\u{03A8}', font => { shape => "upright", forceshape => true });
  DefMath!("\\Upomega",   '\u{03A9}', font => { shape => "upright", forceshape => true });
});
