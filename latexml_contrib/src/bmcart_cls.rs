//! bmcart.cls (BioMed Central's article class): the papers ship the class (it is not in TeX Live), and it is read raw
//! (OXIDIZED_DESIGN_DIVERGENCES #444). Its `{frontmatter}` lets `\title`, `\author` and `\address` to its printers
//! (:966-984 `\set@frontmatter@cmd`): `\title@fmt` typesets the title in the `{fmbox}` (:1034-1053), `\author@fmt`
//! the name with the marks of its `addressref` keys (:1198-1255), and `\address@fmt` stores the address by its `id`
//! (:1287-1296) for the "Author details" block that `\thebibliography` prints (:1708-1712 `\printaddresses`), which
//! the kernel's bibliography constructor never runs. Read raw, the document had no title, no creator and no address
//! (1711.01177: the title page as boxes, the marks "??"). So, after the read, the three printers are the frontmatter's,
//! as elsart_support_core's `\author[labels]`/`\address[label]` are (elsart_support_core.sty.ltxml:35-42): each
//! `\author` a creator requesting its `addressref`/`presentaddressref` ids, with its `email` addresses; each `\address`
//! an affiliation labelled by its `id` (the class's `aff<n>` when it has none); and the `{fmbox}` the box only holds
//! them, its "Full list of author information …" line (:1002 `\auto@set@thanksbox`) a print flourish. Perl has no
//! binding: OmniBus, whose `\author` read "[" as the name. Guard `perfect_kernel_batch78m::bmcart_frontmatter_is_frontmatter`.
use latexml_package::{
  prelude::*,
  shipped_class::{RawRead, read_raw},
};

const BMCART: RawRead = RawRead {
  shipped: true,
  ..RawRead::new("bmcart", "cls")
};

/// The `key=value` pairs of a keyval list at its top level, each value without the spaces and the one pair of braces
/// around it (`addressref={cmu1,eth}`: `cmu1,eth`).
fn keyval_pairs(list: &Tokens) -> Vec<(String, Vec<Token>)> {
  let mut items: Vec<Vec<Token>> = vec![Vec::new()];
  let mut depth = 0usize;
  for t in list.unlist_ref() {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ if depth == 0 && *t == T_OTHER!(",") => {
        items.push(Vec::new());
        continue;
      },
      _ => {},
    }
    if let Some(item) = items.last_mut() {
      item.push(*t);
    }
  }
  items
    .into_iter()
    .filter_map(|item| {
      let eq = item.iter().position(|t| *t == T_OTHER!("="))?;
      let key = Tokens::new(item[..eq].to_vec())
        .to_string()
        .trim()
        .to_string();
      let mut value: &[Token] = &item[eq + 1..];
      while let [first, rest @ ..] = value
        && first.get_catcode() == Catcode::SPACE
      {
        value = rest;
      }
      while let [rest @ .., last] = value
        && last.get_catcode() == Catcode::SPACE
      {
        value = rest;
      }
      if let [open, inner @ .., close] = value
        && open.get_catcode() == Catcode::BEGIN
        && close.get_catcode() == Catcode::END
      {
        value = inner;
      }
      Some((key, value.to_vec()))
    })
    .collect()
}

/// `tokens` in braces.
fn braced(tokens: &[Token]) -> Vec<Token> {
  let mut out = Vec::with_capacity(tokens.len() + 2);
  out.push(T_BEGIN!());
  out.extend_from_slice(tokens);
  out.push(T_END!());
  out
}

LoadDefinitions!({
  if read_raw(&BMCART)? {
    // :1034-1053: the title, its optional keys (`\setkeys{title}`) the page's
    DefMacro!("\\title@fmt[]{}", "\\lx@add@title{#2}");
    // :1198-1255: the author, its name parts as the printed name reads them (:1247-1249: `\inits` dropped, the others
    // as written), requesting the addresses its `addressref` and `presentaddressref` keys name; its `email` addresses
    // (`email={a@x, b@y}`, 1711.01177) its contacts
    DefMacro!("\\author@fmt[]{}", sub[(keys, name)] {
      let pairs = keys.as_ref().map(keyval_pairs).unwrap_or_default();
      let mut labels: Vec<Token> = Vec::new();
      for (key, value) in &pairs {
        if (key == "addressref" || key == "presentaddressref") && !value.is_empty() {
          if !labels.is_empty() {
            labels.push(T_OTHER!(","));
          }
          labels.extend_from_slice(value);
        }
        // (:1131-1143: the article notes its `noteref` key, and `thanksref`, whose alias it is, names; each id the
        // `note:`-labelled contact `\@bmcartnote` makes)
        if key == "noteref" || key == "thanksref" {
          for id in value.split(|t| *t == T_OTHER!(",")) {
            let id: Vec<Token> =
              id.iter().copied().filter(|t| t.get_catcode() != Catcode::SPACE).collect();
            if id.is_empty() {
              continue;
            }
            if !labels.is_empty() {
              labels.push(T_OTHER!(","));
            }
            labels.extend(ExplodeText!("note:"));
            labels.extend(id);
          }
        }
      }
      let mut out = TokenizeInternal!("\\lx@add@creator[role=author,annotations=").unlist();
      out.extend(braced(&labels));
      out.push(T_OTHER!("]"));
      let mut shown = TokenizeInternal!(
        "\\def\\inits#1{}\\def\\fnm#1{#1}\\def\\snm#1{#1}\\def\\particle#1{#1}\\def\\suffix#1{#1}"
      )
      .unlist();
      shown.extend(name.unlist());
      out.extend(braced(&braced(&shown)));
      for (key, value) in &pairs {
        if key != "email" {
          continue;
        }
        for address in value.split(|t| *t == T_OTHER!(",") || *t == T_OTHER!(";")) {
          let address: Vec<Token> =
            address.iter().copied().filter(|t| t.get_catcode() != Catcode::SPACE).collect();
          if !address.is_empty() {
            out.extend(TokenizeInternal!("\\lx@add@email").unlist());
            out.extend(braced(&address));
          }
        }
      }
      Ok(Tokens::new(out))
    });
    // :1287-1296: the address, labelled by its `id` key, else the class's `aff<n>`; its parts as the class prints them
    // (:1298-1304 `\address@definitions`, without the space its body opens with; a `\\` the kernel's line break)
    DefMacro!("\\address@fmt[]{}", sub[(keys, text)] {
      let n = lookup_int("lx_bmcart_address_count") + 1;
      assign_value("lx_bmcart_address_count", Stored::Int(n), Some(Scope::Global));
      let id = keys
        .as_ref()
        .map(keyval_pairs)
        .unwrap_or_default()
        .into_iter()
        .find(|(key, _)| key == "id")
        .map(|(_, value)| value)
        .unwrap_or_else(|| ExplodeText!(&s!("aff{n}")));
      let mut out = TokenizeInternal!("\\lx@add@contact[role=affiliation,label=").unlist();
      out.extend(braced(&id));
      out.push(T_OTHER!("]"));
      let mut shown = TokenizeInternal!(
        "\\def\\orgdiv#1{#1}\\def\\orgname#1{#1}\\def\\street#1{#1}\\def\\postcode#1{#1}\\def\\postbox#1{#1}\\def\\city#1{#1}\\def\\state#1{#1}\\def\\cny#1{#1}\\let\\interref\\surl"
      )
      .unlist();
      shown.extend(text.unlist());
      out.extend(braced(&braced(&shown)));
      Ok(Tokens::new(out))
    });
    // :1391-1395: an article note (`\note[id=n1]{Corresponding author}` in `{artnotes}`, :1371-1378), which the class
    // stores by its `id` for `\printartnotes` (:1406-1409) to print under the title block's rule (:999-1002
    // `\auto@set@thanksbox`, :1440-1447), a note of the authors whose `noteref` names it, as elsart_support_core's
    // `\fntext[label]` is (1507.06515 "Corresponding author", 2007.00437 "Second corresponding author: …", 2112.10068
    // "Equal contributor"); its `symbol` key the mark the class prints
    DefMacro!("\\@bmcartnote[]{}", sub[(keys, text)] {
      let id = keys
        .as_ref()
        .map(keyval_pairs)
        .unwrap_or_default()
        .into_iter()
        .find(|(key, _)| key == "id")
        .map(|(_, value)| value)
        .filter(|value| !value.is_empty());
      let mut out = TokenizeInternal!("\\lx@add@contact[role=note").unlist();
      if let Some(id) = id {
        out.extend(TokenizeInternal!(",label=").unlist());
        let mut label = ExplodeText!("note:");
        label.extend(id);
        out.extend(braced(&label));
      }
      out.push(T_OTHER!("]"));
      out.extend(braced(&text.unlist()));
      Ok(Tokens::new(out))
    });
    // :993-1012: the `{fmbox}` only holds the frontmatter calls
    DefMacro!("\\bmc@fmbox", "");
    DefMacro!("\\bmc@endfmbox", "");
  }
});
