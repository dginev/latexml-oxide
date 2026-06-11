use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: overpic.sty.ltxml
  RequirePackage!("graphicx");
  RequirePackage!("epic");

  // Perl: creates an EMPTY picture environment with tex attribute for LaTeX image generation.
  // afterDigestBody digests \@includegraphicx to get dimensions, sets width/height/tex.
  // The picture element gets the graphic's dimensions and a tex attribute for fallback rendering.
  DefEnvironment!("{overpic} OptionalKeyVals:Gin Semiverbatim",
    "<ltx:picture fill='none' stroke='none' tex='#tex'></ltx:picture>",
    after_digest_body => sub[whatsit] {
      // Set tex attribute from reversion of the full overpic content
      let tex_str = whatsit.revert().map(|t| t.to_string()).unwrap_or_default();
      whatsit.set_property("tex", Stored::String(pin(tex_str)));
    }
  );

  // Perl: {Overpic} (capital O) takes random TeX instead of image — not ported.
  // Used in only ~3 papers on arXiv.
});
