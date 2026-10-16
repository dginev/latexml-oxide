//! beilstein.cls (Beilstein Journal of Nanotechnology / Organic Chemistry): the class is loaded raw, and its author
//! block — `\author` (beilstein.cls:586-697: `\author{name}`, `\author[marks]{name}`, the corresponding author's
//! `\author*[marks]{name}{e-mail}`) and `\affiliation{…}` (:704-737), which number the affiliations in order and give
//! each to the author just named, building accumulators (`\authors`, `\affiliations`, `\emails`) that only its
//! `\maketitle` reads (:739-823) — becomes the frontmatter: each author a creator carrying its marks and the numbers of
//! the affiliations that follow it, the corresponding author's e-mail a contact, each affiliation a contact labelled
//! by its number. Under the raw class (the rawclasses lever) the kernel's locked `\author` refused the class's, so a
//! starred author became an author named `*` with its name and e-mail as stray affiliations, and every affiliation
//! was lost (2203.09951, 2604.06890). An author is handed on when the next one starts or at `\maketitle`, once the
//! affiliations that follow it are known.
use latexml_package::prelude::*;

/// A stored token list (`lx_beilstein_<key>`), empty when unset.
fn tokens_of(key: &str) -> Vec<Token> {
  match lookup_value(&s!("lx_beilstein_{key}")) {
    Some(Stored::Tokens(tokens)) => tokens.unlist(),
    _ => Vec::new(),
  }
}

fn set_tokens(key: &str, tokens: Vec<Token>) {
  assign_value(
    &s!("lx_beilstein_{key}"),
    Stored::Tokens(Tokens::new(tokens)),
    Some(Scope::Global),
  );
}

fn braced(tokens: Vec<Token>) -> Vec<Token> {
  let mut out = vec![T_BEGIN!()];
  out.extend(tokens);
  out.push(T_END!());
  out
}

LoadDefinitions!({
  InputDefinitions!("beilstein", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  // The author just named: its marks, name and e-mail, and the affiliations that followed it.
  DefPrimitive!("\\lx@beilstein@set{}{}{}", sub[(marks, name, email)] {
    set_tokens("marks", marks.unlist());
    set_tokens("name", name.unlist());
    set_tokens("email", email.unlist());
    set_tokens("affiliations", Vec::new());
    assign_value("lx_beilstein_pending", true, Some(Scope::Global));
  });
  DefMacro!("\\lx@beilstein@flush", sub[_args] {
    if !lookup_bool("lx_beilstein_pending") {
      return Ok(Tokens!());
    }
    assign_value("lx_beilstein_pending", false, Some(Scope::Global));
    let marks = tokens_of("marks");
    let mut out = if marks.is_empty() {
      TokenizeInternal!("\\lx@add@creator").unlist()
    } else {
      let mut call = TokenizeInternal!("\\lx@add@authors@append[annotations=").unlist();
      call.extend(braced(marks));
      call.push(T_OTHER!("]"));
      call
    };
    out.extend(braced(tokens_of("name")));
    let email = tokens_of("email");
    if email.iter().any(|t| t.get_catcode() != Catcode::SPACE) {
      out.extend(TokenizeInternal!("\\lx@add@email").unlist());
      out.extend(braced(email));
    }
    out.extend(tokens_of("affiliations"));
    Ok(Tokens::new(out))
  });
  // :704-737: the next affiliation number, given to the author just named, and the affiliation labelled by it.
  DefPrimitive!("\\lx@beilstein@affiliation{}", sub[(text)] {
    let n = lookup_int("lx_beilstein_affiliation_count") + 1;
    assign_value("lx_beilstein_affiliation_count", Stored::Int(n), Some(Scope::Global));
    let number = ExplodeText!(&n.to_string());
    let mut marks = tokens_of("marks");
    if !marks.is_empty() {
      marks.push(T_OTHER!(","));
    }
    marks.extend(number.clone());
    set_tokens("marks", marks);
    let mut affiliations = tokens_of("affiliations");
    affiliations.extend(TokenizeInternal!("\\lx@add@contact[role=affiliation,label=").unlist());
    affiliations.extend(braced(number));
    affiliations.push(T_OTHER!("]"));
    affiliations.extend(braced(text.unlist()));
    set_tokens("affiliations", affiliations);
  });
  RawTeX!(
    r"\def\author{\lx@beilstein@flush\@ifstar\lx@beilstein@star\lx@beilstein@plain}
\def\lx@beilstein@plain{\@ifnextchar[\lx@beilstein@plain@{\lx@beilstein@plain@[]}}
\def\lx@beilstein@plain@[#1]#2{\lx@beilstein@set{#1}{#2}{}}
\def\lx@beilstein@star{\@ifnextchar[\lx@beilstein@star@{\lx@beilstein@star@[]}}
\def\lx@beilstein@star@[#1]#2#3{\lx@beilstein@set{#1}{#2}{#3}}
\renewcommand\affiliation[2][]{\lx@beilstein@affiliation{#2}}"
  );
  AddToMacro!("\\lx@maketitle@begin", "\\lx@beilstein@flush");
});
