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

  // Perl L62-68: \captionsetup stores key-value pairs as CAPTION_{key} in state
  DefPrimitive!("\\captionsetup[]{}", sub[(_ignore, kv)] {
    // Parse the braced argument as key=value pairs and store each
    let kv_str = kv.to_string();
    for pair in kv_str.split(',') {
      let pair = pair.trim();
      if pair.is_empty() { continue; }
      let (key, value) = if let Some(eq_pos) = pair.find('=') {
        (pair[..eq_pos].trim(), pair[eq_pos+1..].trim())
      } else {
        (pair, "true")
      };
      if !key.is_empty() {
        let state_key = s!("CAPTION_{}", key);
        state::assign_value(
          &state_key,
          Stored::String(arena::pin(value)),
          None,
        );
      }
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
