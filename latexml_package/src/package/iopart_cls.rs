use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: iopart.cls.ltxml

  // foreach my $option (qw()) { DeclareOption($option, undef); }
  // (empty list — no ignorable options)

  // Anything else gets passed to article.
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });

  LoadClass!("article");
  RequirePackage!("iopart_support");
  // The page a float's panels are set in rows against (the panel grid rule, `declare_panel_page`): iopart's one
  // column, as its size files set it (iopart10.clo:72 `\textwidth{31pc}`, iopart12.clo:73 `{37.2pc}`, `11pt` reading
  // iopart12.clo, iopart.cls:41-43) — 372pt, or 446.4pt at 11pt and 12pt. The registers' 345pt read as the page broke
  // two `height=6cm` graphics with a `\qquad` into two rows where the PDF sets one (2109.08525 Fig 3).
  if is_document_class("iopart") {
    let text = match last_declared_class_option(&["10pt", "11pt", "12pt"]) {
      Some("11pt" | "12pt") => 446.39996,
      _ => 372.0,
    };
    declare_panel_page(text, text, false);
  }
});
