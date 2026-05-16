use latexml_package::prelude::*;

LoadDefinitions!({
  // Perl `ar5iv-bindings/bindings/stix.sty.ltxml` is an empty stub —
  // it intercepts the FindFile lookup so the heavy raw-load of
  // TL stix.sty doesn't fire, but doesn't provide any symbols. Papers
  // that import `\usepackage[notextcomp]{stix}` and use the package's
  // math symbols (e.g. `\blacktriangle`, `\mdblksquare`) without
  // separately loading `amssymb` then hit undefined-CS errors.
  //
  // Extend the stub with the most-witnessed STIX-only math symbols.
  // The full TL `stix.sty` defines 1478 symbols via `\stix@MathSymbol`;
  // we cover just the ones papers in the arxmliv corpus actually use,
  // mapped to their canonical Unicode codepoints from the STIX font
  // tables (cf. TL `stix.sty` L1213, L1593-1647).
  // Witness: arXiv:2509.13186 — `\blacktriangle` + `\mdblksquare`.
  DefMath!("\\blacktriangle",          "\u{25B2}", role => "ID"); // ▲
  DefMath!("\\blacktriangledown",      "\u{25BC}", role => "ID"); // ▼
  DefMath!("\\blacktriangleleft",      "\u{25C0}", role => "RELOP"); // ◀
  DefMath!("\\blacktriangleright",     "\u{25B6}", role => "RELOP"); // ▶
  DefMath!("\\bigblacktriangleup",     "\u{25B2}", role => "ID"); // ▲ (large)
  DefMath!("\\bigblacktriangledown",   "\u{25BC}", role => "ID"); // ▼ (large)
  DefMath!("\\smallblacktriangleright","\u{25B8}", role => "RELOP"); // ▸
  DefMath!("\\smallblacktriangleleft", "\u{25C2}", role => "RELOP"); // ◂
  DefMath!("\\lrblacktriangle",        "\u{25E2}", role => "ID"); // ◢
  DefMath!("\\llblacktriangle",        "\u{25E3}", role => "ID"); // ◣
  DefMath!("\\ulblacktriangle",        "\u{25E4}", role => "ID"); // ◤
  DefMath!("\\urblacktriangle",        "\u{25E5}", role => "ID"); // ◥
  DefMath!("\\mdblksquare",            "\u{25FC}", role => "ID"); // ◼ medium black square
});
