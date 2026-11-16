use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: enumerate.sty.ltxml
  // Redefines LaTeX's enumerate to take an additional style argument
  DefEnvironment!("{enumerate} OptionalUndigested",
    "<ltx:enumerate xml:id='#id'>#body</ltx:enumerate>",
    properties => sub[_args] { BeginItemize!("enumerate", "enum") },
    after_digest_begin => sub[whatsit] {
      if let Some(arg) = whatsit.get_arg(1) {
        set_enumeration_style(arg.raw_tokens(), None)?;
      }
    },
    mode => "internal_vertical"
  );
  // enumerate.sty:66 `\providecommand\@enhook{}`, the hook its label parser calls on a character it does not know
  // (:63), which this binding's parser does not call; a package extends it (misccorr.sty:22 `\g@addto@macro`,
  // 2606.12178, 2606.17008; OD #1036): code it runs, no text it prints.
  RawTeX!(r"\providecommand\@enhook{}");
});
