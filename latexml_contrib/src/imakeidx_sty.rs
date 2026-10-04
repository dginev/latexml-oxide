//! imakeidx.sty — several indexes, each with its own title. Raw, the package writes each index's entries to
//! `<name>.idx` and `\printindex[name]` inputs `<name>.ind`, the output of an external makeindex run (imakeidx.sty
//! `\imki@makeindex`, `\imki@printindex`), which never runs here: every named index was lost (biblatex-sbl's author
//! index, sbl-paper's subject, scripture and author indexes; Perl has no binding, alike). The binding keeps the model
//! and hands it to the kernel's: `\makeindex[name=<name>,title=<title>]` records the index's title (other keys —
//! `intoc`, `columns`, `program`, `options`, `noautomatic`, `columnsep`, `columnseprule` — are layout or the external
//! run); `\index[<name>]{<entry>}` files the entry in list `<name>` (`\lx@index@inlist`, latex_constructs sect11), the
//! default index (`\jobname`, or no name) being the kernel's `\index`; `\printindex[<name>]` is an `ltx:index` listing
//! `<name>` under its title, which MakeIndex fills, and the default one the kernel's `{theindex}` under its title.
//! The raw package is loaded first, so its switches, options and required packages stay. User ruling 2026-10-03 (not a
//! bibliography-rendering residual). Residual: `\indexprologue` text is not typeset.
//! Witnesses biblatex-sbl/biblatex-sbl, biblatex-sbl/sbl-paper. Guard `perfect_kernel_batch61::imakeidx_named_indexes`.
use latexml_package::{
  engine::latex_constructs::{adjust_backmatter_element, note_backmatter_element},
  prelude::*,
};

/// The `key=value` pairs of a `\makeindex[…]` option list, split at top-level commas.
fn keyval_pairs(options: &Tokens) -> Vec<(String, Tokens)> {
  let mut pairs = Vec::new();
  let mut depth = 0usize;
  let mut item: Vec<Token> = Vec::new();
  let flush = |item: &mut Vec<Token>, pairs: &mut Vec<(String, Tokens)>| {
    let eq = item
      .iter()
      .position(|t| t.get_catcode() == Catcode::OTHER && t.to_string() == "=");
    let (key, value) = match eq {
      Some(i) => (
        Tokens::new(item[..i].to_vec()).to_string(),
        item[i + 1..].to_vec(),
      ),
      None => (Tokens::new(item.clone()).to_string(), Vec::new()),
    };
    let key = key.trim().to_string();
    if !key.is_empty() {
      let mut value = value;
      while value
        .first()
        .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
      {
        value.remove(0);
      }
      while value
        .last()
        .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
      {
        value.pop();
      }
      if value.len() >= 2
        && value[0].get_catcode() == Catcode::BEGIN
        && value[value.len() - 1].get_catcode() == Catcode::END
      {
        value = value[1..value.len() - 1].to_vec();
      }
      pairs.push((key, Tokens::new(value)));
    }
    item.clear();
  };
  for t in options.unlist_ref().iter().cloned() {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      Catcode::OTHER if depth == 0 && t.to_string() == "," => {
        flush(&mut item, &mut pairs);
        continue;
      },
      _ => {},
    }
    item.push(t);
  }
  flush(&mut item, &mut pairs);
  pairs
}

/// The index a `[name]` names: `None` for the default index (no name, or `\jobname`, spelled or expanded).
fn named_index(name: Tokens) -> Result<Option<String>> {
  let name = Expand!(name).to_string();
  let name = name.trim();
  let jobname = Expand!(T_CS!("\\jobname")).to_string();
  Ok((!name.is_empty() && name != jobname).then(|| name.to_string()))
}

LoadDefinitions!({
  // The raw package first — its switches, options and the packages it loads (xkeyval, ifxetex/ifluatex, multicol,
  // imakeidx.sty:23-47, :219), on which documents rely (atableau, tikz-ext-manual, genealogy-profiles) — then the
  // three commands that reach the external run are replaced. The kernel's `\index` and its `\@index` constructor are
  // kept across the load: imakeidx.sty:159-160 redefine both for its `.idx` writer.
  RawTeX!(r"\let\lx@imakeidx@kernel@index\index\let\lx@imakeidx@kernel@@index\@index");
  InputDefinitions!("imakeidx", noltxml => true, extension => Some(Cow::Borrowed("sty")),
    handleoptions => true);
  RawTeX!(r"\let\@index\lx@imakeidx@kernel@@index");
  DefMacro!("\\makeindex[]", sub[(options)] {
    let options = options.unwrap_or_default();
    let pairs = keyval_pairs(&options);
    let name = pairs.iter().find(|(k, _)| k == "name").map(|(_, v)| v.clone()).unwrap_or_default();
    let key = named_index(name)?.unwrap_or_default();
    if let Some((_, title)) = pairs.iter().find(|(k, _)| k == "title") {
      assign_value(&s!("imakeidx_title:{key}"), Stored::Tokens(title.clone()), Some(Scope::Global));
    }
    Ok(TokenizeInternal!(r"\ifdefined\@indexfile\else\csname newwrite\endcsname\@indexfile\fi"))
  });
  DefMacro!(
    "\\index",
    "\\@ifnextchar[{\\lx@imakeidx@index}{\\lx@imakeidx@kernel@index}"
  );
  DefMacro!("\\lx@imakeidx@index[]", sub[(name)] {
    Ok(match named_index(name.unwrap_or_default())? {
      Some(list) => {
        let mut out = vec![T_CS!("\\lx@index@inlist"), T_BEGIN!()];
        out.extend(ExplodeText!(list));
        out.push(T_END!());
        Tokens::new(out)
      },
      None => Tokens!(T_CS!("\\lx@imakeidx@kernel@index")),
    })
  });
  DefMacro!("\\printindex[]", sub[(name)] {
    let list = named_index(name.unwrap_or_default())?;
    let key = list.clone().unwrap_or_default();
    let mut out = vec![T_BEGIN!()];
    if let Some(Stored::Tokens(title)) = lookup_value(&s!("imakeidx_title:{key}")) {
      out.extend([T_CS!("\\def"), T_CS!("\\indexname"), T_BEGIN!()]);
      out.extend(title.unlist());
      out.push(T_END!());
    }
    match list {
      Some(list) => {
        out.extend([T_CS!("\\lx@imakeidx@print"), T_BEGIN!()]);
        out.extend(ExplodeText!(list));
        out.push(T_END!());
      },
      None => out.extend(TokenizeInternal!(r"\begin{theindex}\end{theindex}").unlist()),
    }
    out.push(T_END!());
    Ok(Tokens::new(out))
  });
  // The list name is read Semiverbatim, so it is its characters under any encoding (an LGR document's `subject` would
  // read back as Greek and miss the marks' `inlist`, which `\@index` takes from the source tokens).
  DefConstructor!("\\lx@imakeidx@print Semiverbatim",
  "<ltx:index xml:id='#id' lists='#list'><ltx:title font='#titlefont' _force_font='true'>#title</ltx:title></ltx:index>",
  after_digest => sub[whatsit] {
    note_backmatter_element(whatsit, "ltx:index");
    let list = whatsit.get_arg(1).map(|a| a.to_string()).unwrap_or_default();
    let docid: String = Expand!(T_CS!("\\thedocument@ID")).to_string();
    let name = clean_id(&list);
    let id = if docid.is_empty() { s!("idx.{name}") } else { s!("{docid}.idx.{name}") };
    whatsit.set_property("id", id);
    whatsit.set_property("list", list);
    if let Some(title) = DigestIf!("\\indexname")? {
      if let Some(titlefont) = title.get_font()? {
        whatsit.set_property("titlefont", titlefont);
      }
      whatsit.set_property("title", title);
    }
  },
  before_construct => sub[doc, whatsit] {
    adjust_backmatter_element(doc, whatsit)?;
  });
});
