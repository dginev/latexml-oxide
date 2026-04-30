use latexml_package::prelude::*;

LoadDefinitions!({
  // simple stub for now
  RawTeX!(
    "\\setcounter{DefaultLines}{2}\n\\setcounter{DefaultDepth}{0}\n\\renewcommand*{\\DefaultLoversize}{0}\n\\renewcommand*{\\DefaultLraise}{0}\n\\renewcommand*{\\DefaultLhang}{0}\n\\newlength\\DefaultFindent\n\\newlength\\DefaultNindent\n\\newlength\\DefaultSlope\n\\newlength\\DiscardVskip\n\\setlength{\\DefaultFindent}{0pt}\n\\setlength{\\DefaultNindent}{0.5em}\n\\setlength{\\DefaultSlope}{0pt}\n\\setlength{\\DiscardVskip}{0.2pt}"
  );
  DefMacro!("\\lettrine[]{}{}", "\\textbf{#2}#3");
});
