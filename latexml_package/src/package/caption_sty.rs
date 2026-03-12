use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: caption.sty.ltxml
  // Basically all of this is ignorable (other than needing the macros defined).

  // Key-value pairs are all ignorable for now
  // DefKeyVal not yet supported in Rust, so we just define the user-facing macros

  DefMacro!("\\captionsetup[]{}", "");
  DefMacro!("\\DeclareCaptionStyle{}[]{}", "");
  DefMacro!("\\DeclareCaptionLabelFormat{}{}", "");
  DefMacro!("\\DeclareCaptionLabelSeparator{}{}", "");
  DefMacro!("\\DeclareCaptionFont{}{}", "");
  DefMacro!("\\DeclareCaptionFormat{}{}", "");
  DefMacro!("\\DeclareCaptionJustification{}{}", "");
  DefMacro!("\\DeclareCaptionOption{}[]{}", "");
  DefMacro!("\\DeclareCaptionPackage{}", "");

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

  // \captionof — fake a caption in any context
  DefMacro!("\\maybe@@generic@caption", "\\@@generic@caption");
  DefMacro!("\\captionof", "\\@ifstar{\\@scaptionof}{\\@captionof}");
  DefMacro!("\\@captionof{}[]{}", r"\@ifnext\label{\@captionof@postlabel{#1}{#2}{#3}}{\@captionof@{#1}{#2}{#3}}");
  DefMacro!("\\@captionof@postlabel{}{}{} SkipMatch:\\label Semiverbatim", r"\@captionof@{#1}{#2}{#3\label{#4}}");
  DefMacro!("\\@captionof@{}{}{}", r"\begin{#1}\@caption@{#1}{#2}{#3}\end{#1}");
  DefMacro!("\\@scaptionof{}{}", r"\begin{#1*}\@scaption{#2}\end{#1*}");

  DefMacro!("\\clearcaptionsetup", "");
  DefMacro!("\\rotcaption", "");
  DefMacro!("\\showcaptionsetup[]{}", "");
});
