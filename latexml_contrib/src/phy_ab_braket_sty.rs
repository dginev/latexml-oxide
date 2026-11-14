use latexml_package::prelude::*;

// physics2's `ab.braket` module raw-loads unchanged except for its two
// brace-splitting internals. phy-ab.braket.sty:53-58 `\phy@@ab@bk` /
// :49-52 `\phy@@mb@bk` make `|` active inside `\braket<a|b>` as
// `\egroup\phy@abb@bkv\bgroup` (= `\egroup\middle\vert\bgroup`) around a
// `\bgroup#1\egroup` body: real TeX pairs those implicit braces at execution
// time (tex.web §1063), but LaTeXML's `\left` reads its subformula as a
// token-level capture (`\lx@hidden@bgroup`, tex_box.rs), so the split
// `\egroup`/`\bgroup` pairs and `\delclose`'s `\aftergroup\egroup` desync
// and the deferred `\egroup` meets the outer `\begingroup` ("`\egroup`
// Attempt to close boxing group"; physics2/physics2-legacy, ~200 error
// lines, lualatex clean). The split only separates math atoms (spacing);
// the rendered content is `\left\langle a\middle\vert b\right\rangle`, which
// the native fences produce, so the active `|` is just `\phy@abb@bkv` here.
// Every other module (ab, braket, diagmat, doubleprod, xmat, *.legacy)
// raw-loads clean (wave-15 boxes-groups agent, per-module repros). Guard:
// `perfect_kernel_batch54::physics2_braket_active_bar_is_a_middle_fence`.
#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("phy-ab.braket", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  RawTeX!(r#"\begingroup
\catcode`\|=\active
\gdef\phy@@mb@bk#1#2{\begingroup
  \mathcode`\|="8000\def|{#1\vert}%
  \def\<{\mathrel{<}}\def\>{\mathrel{>}}%
  \mathopen#1\langle#2\mathclose#1\rangle\endgroup}
\gdef\phy@@ab@bk#1{\begingroup
  \mathcode`\|="8000\def|{\phy@abb@bkv}%
  \def\<{\mathrel{<}}\def\>{\mathrel{>}}%
  \phy@abopen\langle#1\phy@abclose\rangle\endgroup}
\endgroup"#);
});
