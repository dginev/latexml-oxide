use crate::prelude::*;

/// Perl: absorbedString — digests tokens and extracts text content.
/// Mirrors the Perl function in dcolumn.sty.ltxml that creates a temporary
/// document to get the display form of math tokens (e.g. \cdot → ⋅).
fn absorbed_string(todelim: &Tokens) -> String {
  // An EMPTY todelim (e.g. the column spec `D{.}{}{3}` — no output separator)
  // has an empty alignment character: Perl's `absorbedString(\ensuremath{})`
  // digests to the same empty `textContent`. We return "" directly rather than
  // digest `\ensuremath{}`, because the empty body expands to `$$` (display-math
  // start) — a degenerate token sequence that leaks a display-math mode frame
  // onto the stomach at column-parse time. Both engines emit `$$` here, but
  // wrapped in a float/center the leaked frame makes a later `\end{...}`/`\endgroup`
  // error in Rust while Perl recovers (witness 1805.00875: Perl 0 / Rust was 5).
  // Replicating the empty RESULT without the buggy side effect is faithful.
  if todelim.unlist_ref().is_empty() {
    return String::new();
  }
  // Build \ensuremath{todelim} tokens
  let mut toks = vec![T_CS!("\\ensuremath"), T_BEGIN!()];
  toks.extend_from_slice(todelim.unlist_ref());
  toks.push(T_END!());
  // Digest and extract text from resulting boxes
  match digest(Tokens::new(toks)) {
    Ok(digested) => collect_text(&digested),
    Err(_) => todelim.to_string(),
  }
}

/// Recursively extract leaf text content from a Digested tree.
fn collect_text(digested: &Digested) -> String {
  let mut result = String::new();
  match digested.data() {
    DigestedData::TBox(b) => {
      let tbox = b.borrow();
      with(tbox.text, |text| result.push_str(text));
    },
    DigestedData::List(l) => {
      let list = l.borrow();
      for item in list.boxes.iter() {
        result.push_str(&collect_text(item));
      }
    },
    DigestedData::Whatsit(w) => {
      let whatsit = w.borrow();
      if let Some(Stored::Digested(body)) = whatsit.properties.get("body") {
        result.push_str(&collect_text(body));
      }
    },
    _ => {},
  }
  result
}

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: dcolumn.sty.ltxml — decimal-aligned columns
  RequirePackage!("array");

  // Perl: \lx@unactivate DefToken — resets mathcode of a character.
  // Perl kind is DefMacro with an imperative sub body (no token return);
  // Rust DefPrimitive runs the side effect at stomach time. WISDOM #44:
  // the two kinds differ under expansion (`\edef` etc.); safe here because
  // `\lx@unactivate` is only emitted inside `\DC@` expansions that execute
  // at math-mode stomach time, never captured by `\edef`.
  // WISDOM #44 verified 2026-04-23: zero `\edef`/`\ifx`/`\expandafter`
  // uses of `\lx@unactivate` across LaTeXML/lib + ar5iv-bindings.
  DefPrimitive!("\\lx@unactivate DefToken", sub[(delim_tok)] {
    let delim_str = delim_tok.to_string();
    if let Some(ch) = delim_str.chars().next() {
      assign_mathcode(ch, 0u16, None);
    }
  });

  // Perl: \DC@{}{}{} — activates the decimal delimiter in math mode
  DefMacro!("\\DC@{}{}{}", sub[(delim, todelim, _ndec)] {
    let delim_str = delim.to_string();
    let todelim_str = todelim.to_string();
    if delim_str != todelim_str {
      if let Some(ch) = delim_str.chars().next() {
        // Make the delimiter math-active (code 0x8000)
        assign_mathcode(ch, 0x8000u16, None);
      }
      // Define the active character's expansion
      let expansion_body = s!(
        "\\lx@hidden@bgroup\\lx@unactivate{{{}}}\\lx@wrap[role=PERIOD]{{{}}}\\lx@hidden@egroup",
        delim_str, todelim_str
      );
      let expansion = mouth::tokenize_internal(&expansion_body);
      def_macro(T_CS!(delim_str), None, expansion, None)?;
    }
    // Save and deactivate $
    Let!("\\DC@saved@dollar", "$");
    let_i(&T_MATH!(), &T_CS!("\\relax"), None);
    // Perl: return Tokens(LookupValue('IN_MATH') ? () : T_CS('\lx@begin@inline@math'));
    // FAITHFUL: no `DC_started_math` flag. Perl's \DC@ / \DC@end are symmetric by
    // construction — \DC@ begins inline math unless already IN_MATH, and \DC@end
    // ALWAYS ends inline math. The earlier Rust guard flag desynced across
    // consecutive D columns (a global value read back in a different cell scope),
    // leaking the math-mode frame so a later \endgroup/\end{...} failed with
    // "close a group that switched to mode display_math" (witness 1805.00875:
    // `D{.}{}{3}` followed by `D{.}{.}{-1}` — Perl 0, Rust was 5).
    if lookup_bool_sym(pin!("IN_MATH")) {
      Ok(Tokens::default())
    } else {
      Ok(Tokens!(T_CS!("\\lx@begin@inline@math")))
    }
  });

  // Perl: \DC@end — restores $ and ALWAYS ends inline math (unconditional).
  DefMacro!("\\DC@end", sub[_args] {
    let_i(&T_MATH!(), &T_CS!("\\DC@saved@dollar"), None);
    // Perl: return (T_CS('\lx@end@inline@math'));
    Ok(Tokens!(T_CS!("\\lx@end@inline@math")))
  });

  // Perl: DefColumnType('D{}{}{}', ...) — decimal alignment column
  // Perl: align => 'char:' . absorbedString(Tokens(T_CS('\ensuremath'), T_BEGIN, $todelim, T_END))
  DefColumnType!("D{}{}{}", sub[(delim, todelim, ndec)] {
    // Perl: absorbedString — digest \ensuremath{todelim} to get display character
    let alignment = absorbed_string(&todelim);
    // Build before tokens: \DC@{delim}{todelim}{ndec}
    let mut before = vec![T_CS!("\\DC@"), T_BEGIN!()];
    before.extend(delim.unlist());
    before.push(T_END!());
    before.push(T_BEGIN!());
    before.extend(todelim.unlist());
    before.push(T_END!());
    before.push(T_BEGIN!());
    before.extend(ndec.unlist());
    before.push(T_END!());
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(Cell {
        before: Some(Tokens::new(before)),
        after: Some(Tokens!(T_CS!("\\DC@end"))),
        align: Some(Align::Char(alignment)),
        ..Cell::default()
      });
    });
  });
});
