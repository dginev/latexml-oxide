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
});
