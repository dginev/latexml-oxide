//! Binding for apa7.cls (APA 7th edition manuscripts, TeX Live apa7 2022/07/25).
//!
//! The class itself is read for its modes (man, stu, jou, doc), the packages it loads and its commands; the bibliography
//! package is the one its option names (none by default, :179-216). Its author block — names, numbered affiliations,
//! author note, note — is kept in etoolbox lists for its own title page (:780-890), which the frontmatter replaces, so
//! those setters are mapped onto the frontmatter API here. With the `mask` option the class prints no author identity
//! (:131, 1257-1310), and neither does the binding.
use latexml_package::prelude::*;

/// The items of an etoolbox comma list (`\forcsvlist`, etoolbox.sty:1373): split at top-level commas, each item's
/// surrounding spaces and one enclosing brace group removed (`{1,3}` → `1,3`); empty items are dropped.
fn csv_items(list: Tokens) -> Vec<Tokens> {
  split_tokens(list, vec![T_OTHER!(",").into()])
    .into_iter()
    .filter_map(|item| {
      let mut toks = item.unlist();
      while toks.first() == Some(&T_SPACE!()) {
        toks.remove(0);
      }
      while toks.last() == Some(&T_SPACE!()) {
        toks.pop();
      }
      if toks.len() >= 2
        && toks[0].get_catcode() == Catcode::BEGIN
        && toks[toks.len() - 1].get_catcode() == Catcode::END
      {
        let mut depth = 0usize;
        let closes_last = toks.iter().enumerate().all(|(i, t)| {
          match t.get_catcode() {
            Catcode::BEGIN => depth += 1,
            Catcode::END => depth -= 1,
            _ => {},
          }
          depth > 0 || i == toks.len() - 1
        });
        if closes_last {
          toks = toks[1..toks.len() - 1].to_vec();
        }
      }
      (!toks.is_empty()).then(|| Tokens::new(toks))
    })
    .collect()
}

LoadDefinitions!({
  // apa7.cls:1041-1075 redefines `\appendix` to set each appendix `\section` as a centred heading under its own
  // `appendix` counter; the kernel's `\section` refuses the redefinition, so the class's `\appendix` left the
  // sections numbered on in arabic ("Appendix 9" for pdflatex's "Appendix A", 2609.00670). The kernel's appendix, its
  // sections lettered, stands (semantic markup before PDF fidelity, user 2026-10-06).
  // article (apa7.cls:159-175 `\LoadClass`, loaded here first so its `\appendix` can be kept).
  LoadClass!("article");
  Let!("\\lx@apa@kernel@appendix", "\\appendix");
  InputDefinitions!("apa7", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  Let!("\\appendix", "\\lx@apa@kernel@appendix");

  // apa7.cls:798-806: `\authorsnames[marks]{names}`, both comma lists, the n-th mark list for the n-th name, printed
  // as `name\textsuperscript{marks}` (:840-863) over the affiliations numbered in order (:868-881). The names, with
  // their marks, go to the kernel's author-line parser, which links each mark to the affiliation it numbers
  // (2609.12869 `[{1,3},{2},{3,5},4]`, 00670, 02899); without marks, as written (2609.19448 `David M. Abel$^1$`).
  // `\author` is the class's one-name form (:766), which the kernel's `\author` already is.
  DefMacro!("\\authorsnames[]{}", sub[(marks, names)] {
    let marked = marks.as_ref().is_some_and(|m| !m.is_empty());
    let mut line: Vec<Token> = Vec::new();
    if marked {
      let marks = csv_items(marks.unwrap());
      for (i, name) in csv_items(names).into_iter().enumerate() {
        if i > 0 {
          line.extend([T_OTHER!(","), T_SPACE!()]);
        }
        line.extend(name.unlist());
        if let Some(mark) = marks.get(i) {
          line.extend([T_CS!("\\textsuperscript"), T_BEGIN!()]);
          line.extend(mark.clone().unlist());
          line.push(T_END!());
        }
      }
    } else {
      line = names.unlist();
    }
    let mut out = TokenizeInternal!(r"\def\lx@apa@marked").unlist();
    out.push(T_BEGIN!());
    if marked {
      out.push(T_OTHER!("1"));
    }
    out.push(T_END!());
    out.extend(TokenizeInternal!(r"\@ifundefined{apaSeven@maskauthoridentity}").unlist());
    out.push(T_BEGIN!());
    out.push(T_CS!("\\lx@add@authors"));
    out.push(T_BEGIN!());
    out.extend(line);
    out.push(T_END!());
    out.push(T_END!());
    out.extend([T_BEGIN!(), T_END!()]);
    // the affiliations given before the names, linked now
    out.extend(TokenizeInternal!(r"\gdef\lx@apa@named{1}\lx@apa@pending\gdef\lx@apa@pending{}").unlist());
    Ok(Tokens::new(out))
  });
  RawTeX!(r"\def\lx@apa@marked{}\def\lx@apa@named{}\def\lx@apa@pending{}");
  // apa7.cls:808-812: `\authorsaffiliations{{first},{second}}`, numbered in order when the names carry marks (:875-878),
  // each linked to the names marked with its number; else listed under the names, each for every author.
  // `\affiliation` is its deprecated one-item form (:767).
  // apa7.cls builds its title block at `\maketitle` from the stored lists (:780-890), so the affiliations may come
  // before the names: they are kept until the names are given, which say whether they are numbered.
  DefMacro!("\\authorsaffiliations{}", sub[(list)] {
    if do_expand(T_CS!("\\lx@apa@named"))?.is_empty() {
      let mut kept = TokenizeInternal!(r"\g@addto@macro\lx@apa@pending").unlist();
      kept.extend([T_BEGIN!(), T_CS!("\\authorsaffiliations"), T_BEGIN!()]);
      kept.extend(list.unlist());
      kept.extend([T_END!(), T_END!()]);
      return Ok(Tokens::new(kept));
    }
    let marked = !do_expand(T_CS!("\\lx@apa@marked"))?.is_empty();
    let mut out: Vec<Token> = Vec::new();
    for (j, item) in csv_items(list).into_iter().enumerate() {
      if marked {
        let mut labelled = vec![T_CS!("\\textsuperscript"), T_BEGIN!()];
        labelled.extend(mouth::tokenize(TeXString::assembled((j + 1).to_string())).unlist());
        labelled.extend([T_END!(), T_BEGIN!()]);
        labelled.extend(item.unlist());
        labelled.push(T_END!());
        out.extend(Invocation!(T_CS!("\\lx@add@affiliation@marked"), vec![None, Some(Tokens::new(labelled))]).unlist());
      } else {
        // Unnumbered: linked by marks of its own (2609.19448 `$^1$Department …\\…`) when the names carry marks,
        // else listed under the names, each for every author.
        out.extend(
          Invocation!(T_CS!("\\lx@add@affiliation@marked"), vec![Some(TokenizeInternal!("annotate=all")), Some(item)])
            .unlist(),
        );
      }
    }
    let mut masked = TokenizeInternal!(r"\@ifundefined{apaSeven@maskauthoridentity}").unlist();
    masked.push(T_BEGIN!());
    masked.extend(out);
    masked.push(T_END!());
    masked.extend([T_BEGIN!(), T_END!()]);
    Ok(Tokens::new(masked))
  });
  DefMacro!("\\affiliation{}", "\\authorsaffiliations{{#1}}");
  // apa7.cls:766: `\author` is the one-name `\authorsnames`, so `mask` hides it too.
  DefMacro!("\\author{}", "\\authorsnames{#1}");

  // apa7.cls:784-787, 1257-1310: the author note, printed under the heading "Author Note" (:375), and the note under
  // the affiliations; neither under `mask`.
  DefMacro!(
    "\\authornote{}",
    "\\@ifundefined{apaSeven@maskauthoridentity}{\\lx@add@frontmatter{ltx:note}[role=authornote]{#1}}{}"
  );
  DefMacro!(
    "\\note{}",
    "\\@ifundefined{apaSeven@maskauthoridentity}{\\lx@add@frontmatter{ltx:note}[role=note]{#1}}{}"
  );
  // apa7.cls:785-786: the abstract and keywords are setters, printed on the abstract page (:1320-1330).
  DefMacro!("\\abstract{}", "\\lx@add@abstract{#1}");
  DefMacro!("\\keywords{}", "\\lx@add@keywords{#1}");
  // apa7.cls:780-791, 1292-1294, 1450-1460, 1593-1598: the student title page's course, instructor and due date (`stu`
  // mode), and the journal masthead's journal, volume, copyright line and copyright number (`jou` and `doc` modes);
  // printed whether or not `mask` is set.
  DefMacro!(
    "\\course{}",
    "\\@ifundefined{def@stu}{}{\\lx@add@frontmatter{ltx:note}[role=course]{#1}}"
  );
  DefMacro!(
    "\\professor{}",
    "\\@ifundefined{def@stu}{}{\\lx@add@frontmatter{ltx:note}[role=professor]{#1}}"
  );
  DefMacro!(
    "\\duedate{}",
    "\\@ifundefined{def@stu}{}{\\lx@add@frontmatter{ltx:note}[role=duedate]{#1}}"
  );
  DefMacro!(
    "\\journal{}",
    "\\ifapamodeman{}{\\lx@add@pubnote[role=journal]{#1}}"
  );
  DefMacro!(
    "\\volume{}",
    "\\ifapamodeman{}{\\lx@add@pubnote[role=volume]{#1}}"
  );
  DefMacro!(
    "\\ccoppy{}",
    "\\ifapamodeman{}{\\lx@add@pubnote[role=copyright]{#1}}"
  );
  DefMacro!(
    "\\copnum{}",
    "\\ifapamodeman{}{\\lx@add@pubnote[role=pubid]{#1}}"
  );
  // A DOI stays a link to its record (user 2026-10-06: semantic markup before PDF fidelity), as OmniBus's `\doi` gave
  // these papers before the binding: apacite's own (`\providecommand{\doi}`, apacite.sty:1812-1815, which this then
  // leaves) sets it as url-styled text only. Read as a url is, so `_`, `~` and `%` are its own characters — from a
  // brace only, as OmniBus's and apacite's `\doi` (a bare `\doi` takes its one token, not the text up to the next
  // brace).
  DefMacro!(
    "\\doi",
    "\\@ifnextchar\\bgroup\\lx@apa@doi@verbatim\\lx@apa@doi@token"
  );
  DefMacro!(
    "\\lx@apa@doi@verbatim HyperVerbatim",
    "\\href{https://doi.org/#1}{#1}"
  );
  DefMacro!("\\lx@apa@doi@token{}", "\\href{https://doi.org/#1}{#1}");
  // apa7.cls:322-326: the name followed by the ORCID iD logo linking to the record (2609.00670, 02899); the
  // kernel's ORCID link (`\lx@orcidlink`, as orcidlink.sty's).
  DefMacro!(
    "\\addORCIDlink{}{}",
    "#1\\ \\lx@orcidlink{#2}{\\lx@orcidlogo}"
  );
});
