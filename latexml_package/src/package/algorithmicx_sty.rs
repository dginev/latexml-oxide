use crate::prelude::*;

LoadDefinitions!({
  // Perl: algorithmicx.sty.ltxml
  // Was algorithmic.sty loaded? If so: BAIL immediately. (deeply incompatible)
  // NOTE: must use `is_defined_token` (Perl `IsDefined`) — not `has_meaning` —
  // because users routinely do `\let\algorithmic\relax` before loading
  // algpseudocode (which loads us via the algorithmicx chain) to opt out
  // of algorithmic.sty. `\let X \relax` is *defined* in the state machine
  // but is "LaTeX-y undefined" — Perl's `IsDefined` treats it as undefined,
  // so the bail does not fire and the algorithmicx setup proceeds.
  // Witness: arXiv:2603.09221 (`\let\algorithmic\relax` + algpseudocode).
  if is_defined_token(&T_CS!("\\algorithmic")) {
    Warn!(
      "unexpected",
      "\\algorithmic",
      "Another package has already defined \\algorithmic, will not load algorithmicx.sty"
    );
    // Defensive stubs for algorithmicx top-level commands so user
    // preambles that call \algdef / \algnewcommand / \algnewlanguage
    // / \alglanguage after the bail don't crash. The actual line-
    // formatting machinery is gone, but the preamble setup commands
    // need to gobble cleanly. Witness 2410.03000 (3 papers using
    // algorithmic + algpseudocode together).
    def_macro_noop("\\algdef OptionalKeyVals:algdef SkipSpaces {} [] [] {}")?;
    def_macro_noop("\\algblock [] {}{}")?;
    def_macro_noop("\\algcblock [] {}{}")?;
    def_macro_noop("\\algblockx [] {}{}")?;
    def_macro_noop("\\algcblockx [] {}{}")?;
    def_macro_noop("\\algnewlanguage{}")?;
    def_macro_noop("\\algdeflanguage{}")?;
    def_macro_noop("\\alglanguage{}")?;
    DefMacro!("\\algnewcommand", "\\newcommand");
    DefMacro!("\\algrenewcommand", "\\renewcommand");
    def_macro_noop("\\algdefaulttext[]{}")?;
    return Ok(());
  }

  // Load core, make a few redefinitions
  InputDefinitions!("algorithmicx", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  let_i(
    &T_CS!("\\lx@orig@algorithmic"),
    &T_CS!("\\algorithmic"),
    None,
  );
  DefMacro!(
    "\\algorithmic",
    "\\lx@setup@algorithmicx\\lx@orig@algorithmic"
  );

  DefPrimitive!("\\lx@setup@algorithmicx", sub [_args] {
    ResetCounter!("ALG@line");
    // If we are not within an algorithm environment, step the counter for its id's
    let in_algorithm = with_stacked_values_sym(pin!("current_environment"), |vals| {
      vals.iter().any(|v| {
        matches!(v, Stored::String(s) if with(*s, |v| v == "algorithm"))
      })
    });
    if !in_algorithm {
      ref_step_id("algorithm")?;
    }
    let_i(&T_CS!("\\list"), &T_CS!("\\lx@algorithmicx@beginlist"), None);
    let_i(&T_CS!("\\endlist"), &T_CS!("\\lx@algorithmicx@endlist"), None);
    let_i(&T_CS!("\\item"), &T_CS!("\\lx@algorithmicx@item"), None);
    let_i(&T_CS!("\\hfill"), &T_CS!("\\lx@algorithmicx@hfill"), None);
  });

  // IGNORE \list 1st arg (we'll handle counter stepping in \item)
  DefMacro!(
    "\\lx@algorithmicx@beginlist{}{}",
    "\\lx@algorithmicx@beginlist@{#2}"
  );
  DefConstructor!("\\lx@algorithmicx@beginlist@{}", "<ltx:listing>");

  DefConstructor!("\\lx@algorithmicx@endlist", "</ltx:listing>",
    before_construct => sub[document] {
      document.maybe_close_element("ltx:listingline")?;
    }
  );

  // Empty lines still get an \item, but they're followed by \nointerlineskip!
  // We do NOT want to generate a listingline in those cases.
  // An `\item[label]` line shows its label, unnumbered: the line number is only the list's default label (`\ALG@step`,
  // algorithmicx.sty:88), which a given one replaces. algpseudocode.sty:78 `\Require` is `\item[\algorithmicrequire]`
  // (Perl and earlier Rust numbered "Require:" lines 1, 2, …), and algorithmicx.sty:632 `\Statex` `\item[]`, a line
  // with no number at all (63j). The label goes braced to its own constructor, so a `]` in it stays and the numbered
  // `\lx@algorithmicx@@item` reads no argument from the line's text (`\Statex [Phase one] begins`).
  DefMacro!("\\lx@algorithmicx@item[]", sub[(label)] {
    let mut out = mouth::tokenize_internal("\\@ifnextchar\\nointerlineskip{}").unlist();
    out.push(T_BEGIN!());
    match label {
      Some(label) => {
        out.push(T_CS!("\\lx@algorithmicx@@item@label"));
        out.push(T_BEGIN!());
        out.extend(label.unlist());
        out.push(T_END!());
      },
      None => out.push(T_CS!("\\lx@algorithmicx@@item")),
    }
    out.push(T_END!());
    Ok(Tokens::new(out))
  });
  DefConstructor!(
    "\\lx@algorithmicx@labeltags{}",
    "<ltx:tags><ltx:tag>#1</ltx:tag></ltx:tags>"
  );

  // algpseudocodex.sty:185 wraps each code line in a `varwidth` box that only
  // the NEXT `\State`/`\If`… ends (`\algpx@endCodeCommand`, :781-797);
  // `\Statex` = `\item[]` (algorithmicx.sty:632) has no such hook, so in TeX
  // its text is set INSIDE the previous line's box (pdflatex clean). When the
  // line cannot be closed because that box is still open, the item is a line
  // break within the open `ltx:listingline` rather than a nested one
  // (`<ltx:listingline>` isn't allowed in `<ltx:p>`; algpseudocodex manual,
  // coloredtheorem). Guard: `perfect_kernel_batch54::statex_continues_the_open_line_box`.
  DefConstructor!("\\lx@algorithmicx@@item", sub[document, _args, props] {
    open_algorithmicx_line(document, props)?;
  },
    properties => sub[_args] {
      let step = Digest!(Tokens::new(vec![
        T_BEGIN!(), T_CS!("\\ALG@step"), T_END!()
      ]))?;
      let id = Digest!(Tokens::new(vec![T_CS!("\\theALG@line@ID")]))?;
      let step_revert = step.revert()?;
      let invocation = Invocation!("\\lx@make@tags", vec![Some(Tokens::new(vec![T_OTHER!("ALG@line")]))]);
      let mut tag_tokens = vec![T_BEGIN!(), T_CS!("\\def"), T_CS!("\\fnum@ALG@line"), T_BEGIN!()];
      tag_tokens.extend(step_revert.unlist());
      tag_tokens.push(T_END!());
      tag_tokens.extend(invocation.unlist());
      tag_tokens.push(T_END!());
      let tags = Digest!(Tokens::new(tag_tokens))?;
      Ok(stored_map!("id" => id, "tags" => tags))
    }
  );
  // The line an `\item[label]` opens: tagged by its label, or untagged for an empty one, the counter unstepped.
  DefConstructor!("\\lx@algorithmicx@@item@label Undigested", sub[document, _args, props] {
    open_algorithmicx_line(document, props)?;
  },
    properties => sub[args] {
      let id = Digest!(Tokens::new(vec![T_CS!("\\theALG@line@ID")]))?;
      let label = match args.first() {
        Some(Some(label)) => Some(label.revert()?).filter(|label| !label.unlist_ref().is_empty()),
        _ => None,
      };
      Ok(match label {
        Some(label) => stored_map!("id" => id,
          "tags" => Digest!(Invocation!("\\lx@algorithmicx@labeltags", vec![Some(label)]))?),
        None => stored_map!("id" => id),
      })
    }
  );

  // Ideally, these appear within an algorithm environment, and we'd like to number lines within it.
  // BUT algorithm package isn't required, so define it here!
  NewCounter!("algorithm", "", idprefix => "alg");
  NewCounter!("ALG@line", "algorithm", idprefix => "l");

  // Hopefully this will only get used for right justifying a comment;
  // the ltx:text should autoclose at end of line?
  DefConstructor!(
    "\\lx@algorithmicx@hfill",
    "<ltx:text cssstyle='float:right'>"
  );

  // Protect against obsolete versions of algorithmicx source
  def_macro_noop("\\ALG@g{}")?;
  def_macro_noop("\\endALG@g")?;
});

/// Open the `ltx:listingline` of an algorithmicx `\item`, with its id and tags, or — inside a line still open
/// (algpseudocodex's `varwidth` box, see `\lx@algorithmicx@@item`) — break that line.
///
/// The nearest listing bounds the walk, as algorithm2e's `line_reach` does: in a listing nested in a line (an
/// `{algorithmic}` inside an algorithm2e float's line or inside another `{algorithmic}`'s `\State`) a line opens in
/// that listing; walking on to the outer line put breaks and bare text in the inner listing (112 TooManyErrors papers
/// of run 336: 1203.4481, 2410.01553, 2510.14887, 1604.06452).
fn open_algorithmicx_line(document: &mut Document, props: &SymHashMap<Stored>) -> Result<()> {
  let open_line = document.maybe_close_element("ltx:listingline")?.is_none() && {
    let mut n = Some(document.get_node().clone());
    let mut found = false;
    while let Some(node) = n {
      let qname = document::get_node_qname(&node);
      if qname == pin!("ltx:listingline") {
        found = true;
        break;
      }
      if qname == pin!("ltx:listing") {
        break;
      }
      n = node.get_parent();
    }
    found
  };
  if open_line {
    document.insert_element("ltx:break", Vec::new(), None)?;
  } else {
    let mut attrs: HashMap<String, String> = HashMap::default();
    if let Some(id) = props.get("id") {
      attrs.insert("xml:id".into(), id.to_string());
    }
    document.open_element("ltx:listingline", Some(attrs), None)?;
    if let Some(tags) = props.get("tags") {
      let digested: Option<Digested> = tags.into();
      if let Some(ref d) = digested {
        document.absorb(d, None)?;
      }
    }
  }
  Ok(())
}
