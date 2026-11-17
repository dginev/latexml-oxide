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
//! them. Its end prints the thanksbox (:1002 `\auto@set@thanksbox`), whose marks the binding settles there: each
//! article note under the glyph the class prints, the corresponding author's e-mail named "Correspondence: "
//! (2112.10068; OD #842), the "Full list of author information …" line a print flourish. Perl has no binding: OmniBus,
//! whose `\author` read "[" as the name. Guards `perfect_kernel_batch78m::bmcart_frontmatter_is_frontmatter`,
//! `perfect_kernel_batchbmcart`.
use latexml_package::{
  prelude::*,
  shipped_class::{RawRead, read_raw},
};

const BMCART: RawRead = RawRead {
  shipped: true,
  ..RawRead::new("bmcart", "cls")
};

/// The `key=value` pairs of a keyval list at its top level, each value without the spaces and the one pair of braces
/// around it (`addressref={cmu1,eth}`: `cmu1,eth`); a key given no value (`corref`, :1153 its default) an empty one.
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
      let eq = item
        .iter()
        .position(|t| *t == T_OTHER!("="))
        .unwrap_or(item.len());
      let key = Tokens::new(item[..eq].to_vec())
        .to_string()
        .trim()
        .to_string();
      if key.is_empty() {
        return None;
      }
      let mut value: &[Token] = item.get(eq + 1..).unwrap_or_default();
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

/// The authors read (the class's `author` counter, :1199).
const AUTHOR_COUNT: &str = "lx_bmcart_author_count";
/// The authors with a `corref` key, as `<n>,` each.
const CORREF_AUTHORS: &str = "lx_bmcart_corref_authors";
/// The first author with an `email` key.
const FIRST_EMAILED: &str = "lx_bmcart_first_emailed";
/// The corresponding authors the thanksbox settled, as `,<n>,` each.
const CORRESPONDING: &str = "lx_bmcart_corresponding";
/// The class makes every `corref` author a corresponding author (the 2017/07/31 class, :1225), not the last alone.
const CORREF_LISTS: &str = "lx_bmcart_corref_lists";
/// The class makes the first author with an `email` the corresponding author where no `corref` key works (:944).
const CORREF_DEFAULT: &str = "lx_bmcart_corref_default";
/// The article notes' ids, as `<id>,` each, in the order `\printartnotes` prints them (`\bmcartnote@id@list`, :1384).
const NOTE_IDS: &str = "lx_bmcart_note_ids";
/// The `symbol` key of note `<id>`, its id appended.
const NOTE_SYMBOL: &str = "lx_bmcart_note_symbol_";
/// The mark the thanksbox prints for note `<id>`, its id appended.
const NOTE_MARK: &str = "lx_bmcart_note_mark_";

/// What a mark prints (`*`, `\textdagger`, `\^{}`; a `symbol` key's `$\star$`, its math symbol's glyph), as digested:
/// its glyph, when it is one character.
fn printed_mark(mark: Tokens) -> Result<String> {
  let ink: Vec<&Token> = mark
    .unlist_ref()
    .iter()
    .filter(|t| {
      !matches!(
        t.get_catcode(),
        Catcode::SPACE | Catcode::BEGIN | Catcode::END
      )
    })
    .collect();
  if let [open, symbol, close] = ink.as_slice()
    && open.get_catcode() == Catcode::MATH
    && close.get_catcode() == Catcode::MATH
    && let Some(glyph) = math_symbol_glyph(symbol)
  {
    return Ok(glyph);
  }
  Ok(digest_text(mark)?.to_string().trim().to_string())
}

/// The glyph of the class's `thanks` counter at `n` (:1095-1097 `\thethanks`, `\getitemvalue{bmcsymbol}{n}`: *, †,
/// ^), none past its list, where pdflatex prints none (and errs on the label it writes).
fn thanks_mark(n: usize) -> Result<String> {
  Ok(
    match lookup_definition(&T_CS!(s!("\\bmcsymbol@item@{n}")))? {
      Some(defn) => match defn.get_expansion() {
        Some(ExpansionBody::Tokens(body)) => printed_mark(body.clone())?,
        _ => String::new(),
      },
      None => String::new(),
    },
  )
}

/// Whether the class's macro `cs` holds `pattern` in its body.
fn body_holds(cs: &str, pattern: &[Token]) -> Result<bool> {
  Ok(match lookup_definition(&T_CS!(cs))? {
    Some(defn) => matches!(defn.get_expansion(), Some(ExpansionBody::Tokens(body))
      if body.unlist_ref().windows(pattern.len()).any(|at| at == pattern)),
    None => false,
  })
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
    // (the copies differ in who corresponds: 2112.10068's 2014/01/24 class the last `corref` author, :1219, 2012.08156's
    // and 2101.05946's 2017/07/31 one each, :1225 `\xdef\corr@author@id{\corr@author@id,\author@id}`; 2409.06429's
    // 2017 copy never makes the first e-mailed author one, its `\endfrontmatter` no `\set@authorcorref@notset`)
    let lists = body_holds("\\author@fmt", &[T_CS!("\\corr@author@id"), T_OTHER!(",")])?;
    assign_value(CORREF_LISTS, lists, Some(Scope::Global));
    let default = body_holds("\\endfrontmatter", &[T_CS!("\\set@authorcorref@notset")])?;
    assign_value(CORREF_DEFAULT, default, Some(Scope::Global));
    // :1034-1053: the title, its optional keys (`\setkeys{title}`) the page's
    DefMacro!("\\title@fmt[]{}", "\\lx@add@title{#2}");
    // :1198-1255: the author, its name parts as the printed name reads them (:1247-1249: `\inits` dropped, the others
    // as written), requesting the addresses its `addressref` and `presentaddressref` keys name; its `email` addresses
    // (`email={a@x, b@y}`, 1711.01177) its contacts, a possible corresponding author's named where the thanksbox decides
    DefMacro!("\\author@fmt[]{}", sub[(keys, name)] {
      let pairs = keys.as_ref().map(keyval_pairs).unwrap_or_default();
      let mut labels: Vec<Token> = Vec::new();
      // (:1153-1158, :1212-1222: the corresponding author, whom the thanksbox settles, `\lx@bmcart@thanksbox`, is one
      // with a `corref` key or the first with an `email`: its e-mail's name the class's "Correspondence: ")
      let n = lookup_int(AUTHOR_COUNT) + 1;
      assign_value(AUTHOR_COUNT, Stored::Int(n), Some(Scope::Global));
      let corref = pairs.iter().any(|(key, _)| key == "corref");
      if corref {
        let mut list = lookup_string(CORREF_AUTHORS);
        list.push_str(&s!("{n},"));
        assign_value(CORREF_AUTHORS, Stored::String(pin(&list)), Some(Scope::Global));
      }
      let first_emailed =
        lookup_int(FIRST_EMAILED) == 0 && pairs.iter().any(|(key, _)| key == "email");
      if first_emailed {
        assign_value(FIRST_EMAILED, Stored::Int(n), Some(Scope::Global));
      }
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
            if corref || first_emailed {
              let name = s!("[name={{\\lx@bmcart@emailname{{{n}}}}}]");
              out.extend(TokenizeInternal!(TeXString::assembled(name)).unlist());
            }
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
    // "Equal contributor"), under the mark the thanksbox prints for it (`\lx@bmcart@notemark`)
    DefMacro!("\\@bmcartnote[]{}", sub[(keys, text)] {
      let pairs = keys.as_ref().map(keyval_pairs).unwrap_or_default();
      let id = pairs
        .iter()
        .find(|(key, _)| key == "id")
        .map(|(_, value)| value.clone())
        .filter(|value| !value.is_empty());
      let mut out = TokenizeInternal!("\\lx@add@contact[role=note").unlist();
      let mut shown = Vec::new();
      if let Some(id) = id {
        out.extend(TokenizeInternal!(",label=").unlist());
        let mut label = ExplodeText!("note:");
        label.extend(id.iter().copied());
        out.extend(braced(&label));
        let id = Tokens::new(id).to_string();
        let mut ids = lookup_string(NOTE_IDS);
        ids.push_str(&s!("{id},"));
        assign_value(NOTE_IDS, Stored::String(pin(&ids)), Some(Scope::Global));
        let symbol = pairs.iter().find(|(key, _)| key == "symbol").map(|(_, value)| value);
        if let Some(symbol) = symbol {
          let symbol = Stored::Tokens(Tokens::new(symbol.clone()));
          assign_value(&s!("{NOTE_SYMBOL}{id}"), symbol, Some(Scope::Global));
        }
        shown = TokenizeInternal!("\\lx@bmcart@notemark").unlist();
        shown.extend(braced(&ExplodeText!(&id)));
      }
      out.push(T_OTHER!("]"));
      shown.extend(text.unlist());
      out.extend(braced(&shown));
      Ok(Tokens::new(out))
    });
    // The mark of note `#1`, read where the frontmatter digests the note: the one the thanksbox gave it, the class's
    // footnote mark (`_fnmark`) the frontmatter shows on every name carrying the note (OD #1054).
    DefMacro!("\\lx@bmcart@notemark{}", sub[(id)] {
      let glyph = lookup_string(&s!("{NOTE_MARK}{}", id.to_string().trim()));
      Ok(if glyph.is_empty() {
        Tokens::default()
      } else {
        TokenizeInternal!(TeXString::assembled(s!("\\lx@fm@entry@attr{{_fnmark}}{{{glyph}}}")))
      })
    });
    // :993-1012: the `{fmbox}` only holds the frontmatter calls; its end prints the thanksbox
    DefMacro!("\\bmc@fmbox", "");
    DefMacro!("\\bmc@endfmbox", "\\lx@bmcart@thanksbox");
    // :1435-1463 `\auto@set@thanksbox`, unless the paper set its own `{thanksbox}` (:1414): `\printcorrtext` prints the
    // corresponding author's mark, `\corref@text` and e-mail, then `\printartnotes` each note under its mark. The
    // author is the last with a `corref` key (the 2017 class each one), else (and with one address, whose class ignores
    // the key, :1066-1074) the first with an `email` (the class's `\authorcorref@notset`, written to the aux, :944,
    // :1166). Each `\thanksmark` steps `thanks` (:376-394; * † ^, :1095-1097), save the corresponding author's of a sole
    // author (:1076-1087) and a note's own `symbol` (:1393-1398). pdflatex: 2112.10068's "Sharma1†, Jain2†, … Chaterji1*", "*Correspondence:
    // schaterji@purdue.edu", "† Equal contributor".
    DefPrimitive!("\\lx@bmcart@thanksbox", sub[()] {
      if !matches!(lookup_definition(&T_CS!("\\auto@set@thanksbox"))?,
        Some(defn) if matches!(defn.get_expansion(), Some(ExpansionBody::Tokens(_))))
      {
        return Ok(Vec::new());
      }
      let corref: Vec<i64> =
        lookup_string(CORREF_AUTHORS).split(',').filter_map(|n| n.parse().ok()).collect();
      let (authors, addresses) = (lookup_int(AUTHOR_COUNT), lookup_int("lx_bmcart_address_count"));
      // (one address makes `corref` a key that does nothing, :1066-1074; the 2017 class only with one author too)
      let lists = lookup_bool(CORREF_LISTS);
      let keyed = !corref.is_empty() && (addresses != 1 || lists && authors != 1);
      let first_emailed = lookup_int(FIRST_EMAILED);
      let corresponding: Vec<i64> = match corref.last() {
        Some(last) if keyed => if lists { corref.clone() } else { vec![*last] },
        _ if lookup_bool(CORREF_DEFAULT) && first_emailed > 0 => vec![first_emailed],
        _ => Vec::new(),
      };
      let mut step = 0;
      if !corresponding.is_empty() {
        let listed: String = corresponding.iter().map(|n| s!(",{n},")).collect();
        assign_value(CORRESPONDING, Stored::String(pin(&listed)), Some(Scope::Global));
        // (its mark, a single author's, is shown on no name, FIDELITY_CHARTER §4: the step alone)
        if authors != 1 {
          step = 1;
        }
      }
      let ids = lookup_string(NOTE_IDS);
      let mut marked: Vec<&str> = Vec::new();
      for id in ids.split(',').filter(|id| !id.is_empty()) {
        let printed = match lookup_tokens(&s!("{NOTE_SYMBOL}{id}")) {
          Some(symbol) => printed_mark(symbol)?,
          None => {
            step += 1;
            thanks_mark(step)?
          },
        };
        // (a repeated id keeps the label its first mark wrote, :334-338 `\safelabel`)
        if marked.contains(&id) {
          continue;
        }
        marked.push(id);
        if printed.chars().count() == 1 {
          assign_value(&s!("{NOTE_MARK}{id}"), Stored::String(pin(&printed)), Some(Scope::Global));
        } else {
          Info!(
            "frontmatter",
            "mark",
            s!("bmcart: article note {id}'s mark prints {printed:?}, no one glyph; kept unmarked")
          );
        }
      }
      Ok(Vec::new())
    });
    // The name of author `#1`'s e-mail, read where the frontmatter digests it: the corresponding author's the class's
    // `\corref@text` (:843 "Correspondence: "), as `\printcorrtext` prints it before the address (:1458-1463); one a
    // paper empties (1902.08707's `\setattribute{corref}{text}{}`) names nothing, the e-mail keeps its own name.
    DefMacro!("\\lx@bmcart@emailname{}", sub[(n)] {
      let corresponding = lookup_string(CORRESPONDING).contains(&s!(",{},", n.to_string().trim()));
      let printed = match lookup_definition(&T_CS!("\\corref@text"))? {
        Some(defn) => matches!(defn.get_expansion(), Some(ExpansionBody::Tokens(text))
          if text.unlist_ref().iter().any(|t| t.get_catcode() != Catcode::SPACE)),
        None => false,
      };
      Ok(if corresponding && printed {
        TokenizeInternal!("\\corref@text")
      } else {
        TokenizeInternal!("\\lx@contact@email@name")
      })
    });
  }
});
