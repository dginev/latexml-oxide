//! The Foundation Compositio Mathematica class family — `compositio`, `alggeom` and `moduli` (`CPM@` internals) and the
//! older `hha`, `jhrs` and `jcm` built from the same code (`HHA@`…): shipped with each paper, not in TeX Live. One
//! binding serves them all, the union of their frontmatter commands.
use latexml_package::prelude::*;

/// Interpret the shipped class `name` raw (as the document asked for it, a path-prefixed copy included:
/// [`crate::shipped_class::requested_shipped_class`]), then route the frontmatter it stores for its own title page and closing
/// address block to the frontmatter:
/// - `\address`, `\curraddr`, `\email`, `\homepage` (compositio.cls:141-165, hha.cls:104-137) append to the current
///   author's `\@curr@author` and the `\AtEndDocument` `\@addresses` block, and error without an `\author` the kernel
///   lock had refused (182 compositio, 34 alggeom, 3 hha, 2 jhrs, 2 jcm, 1 moduli of run 336's papers: 2606.19241,
///   1109.5540, math0409234, 0805.3030, quant-ph0412067, 2111.06381); here each is the last author's contact, labelled
///   as the class prints it (`\textit{Current address:}`, compositio.cls:149-150);
/// - `\keywords` and `\classification` (`\subjclass`, compositio.cls:181-189, hha.cls:146-149) store for `\maketitle`
///   only; here they are the frontmatter's keywords and MSC classification, the label the class's own definition prints
///   (its MSC year: 2000 in hha, 2010 in compositio and alggeom, 2020 in moduli);
/// - `\thanks` (compositio.cls:174-176) stores a `\footnotetext` for `\maketitle` only; here it is the kernel's;
/// - `\received`, `\revised`, `\online` (compositio.cls:192-206), `\published` (hha.cls:161) and `\dedication`
///   (compositio.cls:179) store for `\maketitle` only; here they are the frontmatter's dates and dedication;
/// - the class's own `\maketitle` (compositio.cls:209-271) lays out only those fields, so it is not deposited
///   (OXIDIZED_DESIGN #265): with `\@authors` filled by its replayed `\author` (#482) it typeset the names again in
///   the body, its `\address\relax` an empty contact, and its emptied stores' warnings ("No MSC classification").
///
/// A paper whose source lacks the class falls to OmniBus, as before the binding (OXIDIZED_DESIGN #444).
pub(crate) fn compositio_family_class(name: &str) -> Result<()> {
  let Some(class) = crate::shipped_class::requested_shipped_class(name)? else {
    LoadClass!("OmniBus");
    return Ok(());
  };
  Let!("\\lx@compositio@kernel@thanks", "\\thanks");
  InputDefinitions!(&class, noltxml => true, extension => Some(Cow::Borrowed("cls")));
  // a class only named like the family (the prefix fallback's `jcmfoo`, `hhafoo`) is left as it is: without the
  // family's current-author store (`\@curr@author`, compositio.cls:126, hha.cls, jcm.cls:7), it has none of its setters
  if !IsDefined!(&T_CS!("\\@curr@author")) {
    return Ok(());
  }
  let msc = msc_label();
  // hha.cls:142-144, jhrs: "Key words and phrases:"; compositio.cls:181-183: "Keywords:"
  let keywords = class_body_text("\\keywords");
  let keywords = if keywords.contains("Key words and phrases") {
    "Key words and phrases:"
  } else {
    "Keywords:"
  };
  let homepage_args = class_parameter_count("\\homepage");
  Let!("\\thanks", "\\lx@compositio@kernel@thanks");
  DefMacro!("\\address[]{}", "\\lx@add@address{#2}");
  DefMacro!(
    "\\curraddr[]{}",
    "\\lx@add@contact[role=current_address,name={\\textit{Current address:}\\ }]{#2}"
  );
  DefMacro!("\\email[] Semiverbatim", "\\lx@add@email{#2}");
  // jcm.cls:157 `\homepage{url}{text}` (`\href{#1}{\tt #2}`); hha.cls:133 `\homepage{url}`
  if homepage_args == 2 {
    DefMacro!(
      "\\homepage Semiverbatim {}",
      "\\lx@add@contact[role=url]{\\href{#1}{\\texttt{#2}}}"
    );
  } else {
    DefMacro!("\\homepage Semiverbatim", "\\lx@add@url{#1}");
  }
  RawTeX!(&s!("\\def\\lx@compositio@keywords{{{keywords}}}"));
  DefMacro!(
    "\\keywords{}",
    "\\lx@add@keywords[name={\\lx@compositio@keywords\\ }]{#1}"
  );
  RawTeX!(&s!("\\def\\lx@compositio@msc{{{msc}}}"));
  DefMacro!(
    "\\classification[]{}",
    "\\lx@add@classification[scheme={Mathematics Subject Classification},name={\\lx@compositio@msc\\ }]{#2}"
  );
  Let!("\\subjclass", "\\classification");
  DefMacro!(
    "\\received{}",
    "\\lx@add@date[role=received,name={Received }]{#1}"
  );
  // compositio.cls:195/:205 `\revised[2][revised]`, `\online[2][published online]`: the label is the author's
  DefMacro!(
    "\\revised[Default:revised]{}",
    "\\lx@add@date[role=revised,name={#1 }]{#2}"
  );
  DefMacro!(
    "\\online[Default:published online]{}",
    "\\lx@add@date[role=published,name={#1 }]{#2}"
  );
  DefMacro!(
    "\\published{}",
    "\\lx@add@date[role=published,name={Published }]{#1}"
  );
  // hha.cls:164-166, jhrs: `({\itshape communicated by #1})` in the title block
  DefMacro!(
    "\\submitted{}",
    "\\lx@add@pubnote[role=communicated,name={communicated by }]{#1}"
  );
  DefMacro!("\\dedication{}", "\\lx@add@pubnote[role=dedication]{#1}");
  Let!("\\lx@dropped@maketitle", "\\relax");
  Let!("\\@maketitle", "\\@empty");
  Ok(())
}

/// The text of the class's own definition of `cs` (empty where it has none).
fn class_body_text(cs: &str) -> String {
  match lookup_meaning(&T_CS!(cs)) {
    Some(Stored::Expandable(ref d)) => match d.get_expansion() {
      Some(ExpansionBody::Tokens(body)) => body.clone().untex(),
      _ => String::new(),
    },
    _ => String::new(),
  }
}

/// How many parameters the class's own `cs` takes (0 where it has none).
fn class_parameter_count(cs: &str) -> usize {
  match lookup_meaning(&T_CS!(cs)) {
    Some(Stored::Expandable(ref d)) => d.get_parameters().map_or(0, |p| p.get_parameters().len()),
    _ => 0,
  }
}

/// The MSC label the class's own `\classification` prints — "<year> Mathematics Subject Classification" (the year
/// before the words, in its body), or the words alone.
fn msc_label() -> String {
  let words = "Mathematics Subject Classification";
  let body = class_body_text("\\classification");
  let Some(at) = body.find(words) else {
    return words.to_string();
  };
  let year: String = body[..at]
    .trim_end()
    .chars()
    .rev()
    .take(4)
    .collect::<Vec<_>>()
    .into_iter()
    .rev()
    .collect();
  // hha.cls:148 prints its colon ("2000 Mathematics Subject Classification:"), compositio.cls:187 none
  let colon = if body[at + words.len()..].starts_with(':') {
    ":"
  } else {
    ""
  };
  if year.len() == 4 && year.chars().all(|c| c.is_ascii_digit()) {
    s!("{year} {words}{colon}")
  } else {
    s!("{words}{colon}")
  }
}

LoadDefinitions!({
  compositio_family_class("compositio")?;
});
