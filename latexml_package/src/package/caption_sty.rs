use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: caption.sty.ltxml
  // Basically all of this is ignorable (other than needing the macros defined).
  // In principle, we could make use of some of the fonts...

  // Perl L24-59: DefKeyVal declarations for caption package
  DefKeyVal!("caption", "format", "", "");
  DefKeyVal!("caption", "indentation", "Dimension", "0pt");
  DefKeyVal!("caption", "labelformat", "", "default");
  DefKeyVal!("caption", "labelsep", "", "");
  DefKeyVal!("caption", "textformat", "", "");
  DefKeyVal!("caption", "justification", "", "");
  DefKeyVal!("caption", "singlelinecheck", "", "");
  DefKeyVal!("caption", "font", "", "");
  DefKeyVal!("caption", "labelfont", "", "");
  DefKeyVal!("caption", "textfont", "", "");
  DefKeyVal!("caption", "font+", "", "");
  DefKeyVal!("caption", "labelfont+", "", "");
  DefKeyVal!("caption", "textfont+", "", "");
  DefKeyVal!("caption", "margin", "Dimension", "0pt");
  DefKeyVal!("caption", "margin*", "Dimension", "0pt");
  DefKeyVal!("caption", "minmargin", "Dimension", "0pt");
  DefKeyVal!("caption", "maxmargin", "Dimension", "0pt");
  DefKeyVal!("caption", "parskip", "Dimension", "0pt");
  DefKeyVal!("caption", "width", "Dimension", "0pt");
  DefKeyVal!("caption", "oneside", "", "");
  DefKeyVal!("caption", "twoside", "", "");
  DefKeyVal!("caption", "hangindent", "Dimension", "0pt");
  DefKeyVal!("caption", "style", "", "");
  DefKeyVal!("caption", "skip", "Dimension", "0pt");
  DefKeyVal!("caption", "position", "", "");
  DefKeyVal!("caption", "figureposition", "", "");
  DefKeyVal!("caption", "tableposition", "", "");
  DefKeyVal!("caption", "list", "", "");
  DefKeyVal!("caption", "listformat", "", "");
  DefKeyVal!("caption", "name", "", "");
  DefKeyVal!("caption", "type", "", "");
  // Additional caption.sty options not in Perl's pre-registration list.
  // Rust-only divergence paired with `21e730e71e` Info→Warn promotion.
  for key in [
    "compatibility", "calcmargin", "ignoreLTcapwidth",
    "captionlinewidth", "subrefformat",
    "subskip", "belowskip", "aboveskip",
    "rule", "tableposition", "labelseparator",
    "options", "ruled", "boxed",
    "above", "below", "outside", "inside",
    "centerlast", "centering", "raggedright", "raggedleft",
  ] {
    DefKeyVal!("caption", key, "");
  }

  // Perl L62-68: \captionsetup stores key-value pairs as CAPTION_{key}
  // in state. Perl uses `RequiredKeyVals:caption` so brace-nested and
  // quoted values parse correctly; the prior Rust version accepted
  // `{}` and manually split on `,`, which mis-parsed values containing
  // commas inside braces (e.g. `font={normal,bold}`).
  DefPrimitive!("\\captionsetup[] RequiredKeyVals:caption", sub[(_ignore, kv)] {
    for (key, value) in kv.get_pairs() {
      let state_key = s!("CAPTION_{key}");
      state::assign_value(
        &state_key,
        Stored::String(arena::pin(value.to_string())),
        None,
      );
    }
  });
  DefMacro!("\\DeclareCaptionStyle{}[]{}", "");
  DefMacro!("\\DeclareCaptionLabelFormat{}{}", "");
  DefMacro!("\\DeclareCaptionLabelSeparator{}{}", "");
  DefMacro!("\\DeclareCaptionFont{}{}", "");
  DefMacro!("\\DeclareCaptionFormat{}{}", "");
  DefMacro!("\\DeclareCaptionJustification{}{}", "");
  DefMacro!("\\DeclareCaptionOption{}[]{}", "");
  DefMacro!("\\DeclareCaptionPackage{}", "");

  // caption3 internals used by raw-loaded sibling packages like
  // floatrow.sty. Real `\caption@setkeys [opt] {family} {kvs}` calls
  // `\setkeys{family}{kvs}` with caption-specific error handling
  // (caption3_2020-10-26.sty L337-360). Stub to a plain `\setkeys`
  // — drops the optional error-handler context but preserves
  // keyval-processing semantics. Witness cluster: papers using
  // `\usepackage{floatrow}` which raw-loads its body containing
  // `\caption@setkeys{...}{...}` calls.
  DefMacro!("\\caption@setkeys[]{}{}", "\\setkeys{#2}{#3}");
  // `\undefine@key` removes a keyval. Real keyval.sty defines it
  // post-2018; xkeyval too. Both Perl LaTeXML's keyval.sty.ltxml
  // hand-port and our Rust binding pre-date that and don't include
  // it. Stub as a no-op — keyval removal is mostly an authoring
  // hygiene issue; missing it means stale keys linger but no
  // tokenization breakage. Witness: same floatrow chain.
  DefMacro!("\\undefine@key{}{}", "");

  DefMacro!("\\bothIfFirst{}{}", sub[(first, second)] {
    if first.is_empty() { Ok(Tokens!()) } else {
      let mut result = first.unlist();
      result.extend(second.unlist());
      Ok(Tokens::new(result))
    }
  });

  DefMacro!("\\bothIfSecond{}{}", sub[(first, second)] {
    if second.is_empty() { Ok(Tokens!()) } else {
      let mut result = first.unlist();
      result.extend(second.unlist());
      Ok(Tokens::new(result))
    }
  });

  DefMacro!("\\AtBeginCaption{}", "");
  DefMacro!("\\AtEndCaption{}", "");
  DefMacro!("\\ContinuedFloat", "");
  DefMacro!("\\ProcessOptionsWithKV{}", "");

  DefMacro!("\\captionfont", "");
  DefMacro!("\\captionsize", "");

  DefRegister!("\\captionparindent"  => Dimension::new(0));
  DefRegister!("\\captionindent"     => Dimension::new(0));
  DefRegister!("\\captionhangindent" => Dimension::new(0));
  DefRegister!("\\captionmargin"     => Dimension::new(0));
  DefRegister!("\\captionwidth"      => Dimension::new(0));

  // Override \caption to support \caption* (starred form)
  DefMacro!("\\caption",
    r"\lx@donecaptiontrue\@ifundefined{@captype}{\maybe@@generic@caption}{\@ifstar{\@scaption}{\expandafter\@caption\expandafter{\@captype}}}"
  );
  DefMacro!("\\@scaption{}", "\\@@caption{#1}");

  // \captionof — fake a caption in any context.
  //
  // Perl caption.sty.ltxml L110-115 routes through the `CAPTION_type` state
  // value set by `\captionsetup{type=…}`: when the author has declared a
  // float type, `\maybe@@generic@caption` expands to `\@captionof{type}`
  // so the caption digests inside the proper environment; otherwise it
  // falls through to `\@@generic@caption`. Rust previously hardcoded the
  // fallback, silently dropping the captionsetup type.
  DefMacro!("\\maybe@@generic@caption", sub[_args] {
    if let Some(Stored::String(t)) = state::lookup_value("CAPTION_type") {
      let ty = arena::with(t, |s| s.to_string());
      if !ty.is_empty() {
        let mut out = vec![T_CS!("\\@captionof"), T_BEGIN!()];
        out.extend(ExplodeText!(&ty));
        out.push(T_END!());
        return Ok(Tokens::new(out));
      }
    }
    Ok(Tokens!(T_CS!("\\@@generic@caption")))
  });
  DefMacro!("\\captionof", "\\@ifstar{\\@scaptionof}{\\@captionof}");
  DefMacro!("\\@captionof{}[]{}", r"\@ifnext\label{\@captionof@postlabel{#1}{#2}{#3}}{\@captionof@{#1}{#2}{#3}}");
  DefMacro!("\\@captionof@postlabel{}{}{} SkipMatch:\\label Semiverbatim", r"\@captionof@{#1}{#2}{#3\label{#4}}");
  DefMacro!("\\@captionof@{}{}{}", r"\begin{#1}\@caption@{#1}{#2}{#3}\end{#1}");
  DefMacro!("\\@scaptionof{}{}", r"\begin{#1*}\@scaption{#2}\end{#1*}");

  DefMacro!("\\clearcaptionsetup", "");
  DefMacro!("\\rotcaption", "");
  DefMacro!("\\showcaptionsetup[]{}", "");
});
