use crate::prelude::*;

LoadDefinitions!({
  // ulem.sty:46 `\global\let\UL@protected\csname protected\endcsname`: packages
  // that build on ulem read it (oblivoir-misc.sty:51-54 defines its own only
  // when ulem is absent). Perl's ulem.sty.ltxml omits it (SHARED).
  // Guard: `perfect_kernel_batch56::sweep46_single_name_gaps`.
  Let!("\\UL@protected", "\\protected");
  RequireResource!("ltx-ulem.css");

  DefConstructor!("\\uline{}",
    "?#isMath(<ltx:XMWrap class='ltx_ulem_uline'>#1</ltx:XMWrap>)(<ltx:text class='ltx_ulem_uline'>#1</ltx:text>)",
    enter_horizontal => true);
  DefConstructor!("\\uuline{}",
    "?#isMath(<ltx:XMWrap class='ltx_ulem_uuline'>#1</ltx:XMWrap>)(<ltx:text class='ltx_ulem_uuline'>#1</ltx:text>)",
    enter_horizontal => true);
  DefConstructor!("\\uwave{}",
    "?#isMath(<ltx:XMWrap class='ltx_ulem_uwave'>#1</ltx:XMWrap>)(<ltx:text class='ltx_ulem_uwave'>#1</ltx:text>)",
    enter_horizontal => true);
  DefConstructor!("\\sout{}",
    "?#isMath(<ltx:XMWrap class='ltx_ulem_sout'>#1</ltx:XMWrap>)(<ltx:text class='ltx_ulem_sout'>#1</ltx:text>)",
    enter_horizontal => true);
  DefConstructor!("\\xout{}",
    "?#isMath(<ltx:XMWrap class='ltx_ulem_xout'>#1</ltx:XMWrap>)(<ltx:text class='ltx_ulem_xout'>#1</ltx:text>)",
    enter_horizontal => true);
  DefConstructor!("\\dashuline{}",
    "?#isMath(<ltx:XMWrap class='ltx_ulem_dashuline'>#1</ltx:XMWrap>)(<ltx:text class='ltx_ulem_dashuline'>#1</ltx:text>)",
    enter_horizontal => true);
  DefConstructor!("\\dotuline{}",
    "?#isMath(<ltx:XMWrap class='ltx_ulem_dotuline'>#1</ltx:XMWrap>)(<ltx:text class='ltx_ulem_dotuline'>#1</ltx:text>)",
    enter_horizontal => true);
  // ulem's own lines, as loaded: a declaration made from one of them digests the rest of its group (`\useunder` below).
  RawTeX!(
    r"\let\lx@ulem@uline\uline \let\lx@ulem@uuline\uuline \let\lx@ulem@uwave\uwave \let\lx@ulem@sout\sout
    \let\lx@ulem@xout\xout \let\lx@ulem@dashuline\dashuline \let\lx@ulem@dotuline\dotuline"
  );

  // ulem.sty:310-317 `\ULforem` (`\useunder{\uline}{\em}{\emph}`: `\em`/`\emph` underline) and
  // `\normalem` (back to italics, from the copies ulem takes at load). As Perl's binding, `\em` keeps
  // its italics at load (ulem's own default is `\ULforem`, :317, unless `[normalem]`); a document's
  // `\ULforem` switches to underlining, its `\normalem` back (run 340: 109 first errors, witnesses
  // 2103.16561, 2004.14103, 1811.00488).
  RawTeX!(r"\let\LA@em\em \let\LA@emph\emph");
  DefMacro!("\\normalem", None, "\\let\\em\\LA@em \\let\\emph\\LA@emph");
  DefMacro!("\\ULforem", None, "\\useunder{\\uline}{\\em}{\\emph}");

  // ulem L343: \newdimen\ULdepth (initialized to \maxdimen for auto-mode).
  // Witnesses 2406.02021, 2406.18999.
  DefRegister!("\\ULdepth" => Dimension!("0pt"));
  DefRegister!("\\ULthickness" => Dimension!("0.4pt"));
  DefRegister!("\\UL@height" => Dimension!("0pt"));

  // Real-ulem word-machinery internals: our high-level constructors never
  // call them, but RAW add-on code wires into them by name — xeCJKfntef.sty
  // L102-134 installs `\xeCJK_ulem_word:nw` over `\UL@word` and probes
  // `\UL@stop`/`\UL@hook`/`\UL@end`/`\UL@start` (the
  // `wisdom_latexml_reimpl_internal_name_mismatch` shape; fixdif-zh-cn's
  // digestion loop). Provide the surface inertly: identity word processor,
  // no-op start/stop, the real `\UL@end *` delimited shape (ulem.sty L59).
  DefRegister!("\\UL@hook", Tokens!());
  def_macro_noop("\\UL@start")?;
  def_macro_noop("\\UL@stop")?;
  def_macro_noop("\\UL@putbox")?;
  def_macro_noop("\\UL@ender")?;
  def_macro_identity("\\UL@word{}")?;
  // ulem's documented extension contract (ulem.sty:232-233): a special
  // underline is `\def\command{\bgroup \markoverwith{<leader>} \ULon}`, and
  // the `\bgroup` is closed by the word machinery — `\UL@on#1` ends in
  // `\UL@end *` whose real body (:59) closes the group, `\UL@onin`/`\UL@onmath`
  // (:90-102) end in `\egroup`. Keep that contract with the identity word
  // processor: CJKfntef.sty:258-283 `\CJK@UL` (`\CJKunderline`, raw) left the
  // group open and unbalanced the enclosing list (jnuexam exam sheets, 8
  // CJKfntef manuals). `\markoverwith` only builds the leader box.
  def_macro_noop("\\markoverwith{}")?;
  DefMacro!(
    "\\ULon",
    r"\ifmmode\expandafter\UL@onmath\else\expandafter\UL@on\fi"
  );
  DefMacro!("\\UL@on{}", r"#1\UL@end *");
  DefMacro!("\\UL@onin{}", r"#1\egroup");
  DefMacro!("\\UL@onmath{}", r"#1\egroup");
  // ulem.sty:59 `\def\UL@end *{\relax\relax}`: `*` is a delimiter, nothing after it is read. An
  // `OptionalMatch:*` then skips spaces (Perl Parameter.pm:98-100), so a custom underline built on
  // `\ULon` (`\bgroup\markoverwith{…}\ULon`, ulem.sty:232-233; witnesses 2605.15048 `\redout`,
  // 2605.16437 `\Erase`) ate the space after its argument. The mark itself is not rendered yet (RED
  // `fonts-nfss/ulem_custom_mark_is_rendered`).
  DefMacro!("\\UL@end Match:*", r"\egroup");
  DefMacro!("\\UL@spfactor", None, "1000");

  // ulem.sty:286-293 `\useunder{ucmd}{decl}{argcmd}`: `argcmd` is `ucmd`, and `decl` a declaration that applies `ucmd`
  // to the rest of its group — `{\ul some words}` underlines "some words" (ulem.sty:288-289 opens `ucmd`'s argument with
  // an unmatched brace, the group's own `}` closing it, and `\UL@swender`, :107, puts the group's `}` back after). An
  // empty slot (papers write `\useunder{\uline}{\ul}{}`) defines nothing. Witnesses 2406.08270 (`\useunder` with
  // `\uline{…}`), 2405.20343 and 2406.03441 (`{\ul 18.8262}`, `{\ul …}`: one character underlined while `\ul` was let
  // to `\uline`). A `\ul{x}` outside any group, a fatal error in pdflatex, underlines up to the end of its paragraph.
  DefMacro!("\\useunder{}{}{}", sub[(ucmd, decl, argcmd)] {
    let mut out: Vec<Token> = Vec::new();
    let ucmd_v = ucmd.unlist();
    if !ucmd_v.is_empty() {
      let target = ucmd_v[0];
      let decl_v = decl.unlist();
      if !decl_v.is_empty() {
        out.extend([T_CS!("\\def"), decl_v[0], T_BEGIN!(), T_CS!("\\lx@ulem@declaration"), target, T_END!()]);
      }
      let arg_v = argcmd.unlist();
      if !arg_v.is_empty() {
        out.extend([T_CS!("\\let"), arg_v[0], target]);
      }
    }
    Ok(Tokens::new(out))
  });
  // `\lx@ulem@declaration\uline` underlines the rest of the current group. One of ulem's own lines digests it up to the
  // group's end: its `}` (put back after the line for the group), a cell's end (an Error), or a `\multicolumn` body's
  // end, where latex.ltx:16664-16676 closes the template's `{…}` around the body (2312.12023
  // `\multicolumn{1}{c|}{\ul 29.9970}`: a balanced read ran out of input there). Any other `ucmd` takes the rest of the
  // group read as its argument.
  DefMacro!("\\lx@ulem@declaration{}", sub[(ucmd)] {
    let ucmd_v = ucmd.unlist();
    if let [target] = ucmd_v.as_slice() {
      for line in ["uline", "uuline", "uwave", "sout", "xout", "dashuline", "dotuline"] {
        if x_equals(target, &T_CS!(&format!("\\lx@ulem@{line}"))) {
          return Ok(Tokens::new(vec![
            T_CS!("\\lx@ulem@declared@line"), *target, T_BEGIN!(), T_OTHER!(line), T_END!()]));
        }
      }
    }
    let mut out = vec![T_CS!("\\lx@ulem@declared")];
    out.extend(ucmd_v);
    out.push(T_BEGIN!());
    Ok(Tokens::new(out))
  });
  DefConstructor!("\\lx@ulem@declared@line Undigested Semiverbatim DigestedBody",
  "?#isMath(<ltx:XMWrap class='ltx_ulem_#2'>#3</ltx:XMWrap>)(<ltx:text class='ltx_ulem_#2'>#3</ltx:text>)",
  enter_horizontal => true, reversion => "#1{#3}", sizer => "#3",
  // `lx@ulem@body` is the line's nesting depth on its own frame, so a line inside another tells its own frame from
  // the outer line's (`{\ul a \so b}`, `\multicolumn{1}{c}{a \ul b \so c}`).
  before_digest => {
    let depth = lookup_int("lx@ulem@depth") + 1;
    assign_value("lx@ulem@depth", Stored::Int(depth), Some(Scope::Global));
    let opened = get_locator().to_string();
    assign_value(&format!("lx@ulem@opened@{depth}"), Stored::String(pin(&opened)), Some(Scope::Local));
    // Made directly in another line's text (`{a \ul b \so c}`), it shares that line's closer, which ends the other's
    // argument: nothing closes this one, pdflatex's fatal "File ended while scanning use of \UL@on".
    let shared = is_value_bound("lx@ulem@body", Some(0));
    assign_value(&format!("lx@ulem@shared@{depth}"), Stored::Bool(shared), Some(Scope::Local));
    bgroup();
    assign_value("lx@ulem@body", Stored::Int(depth), Some(Scope::Local));
    Let!("\\lx@alignment@multicolumn@end", "\\lx@ulem@declared@end");
  },
  after_digest => sub[whatsit] {
    let depth = lookup_int("lx@ulem@depth");
    assign_value("lx@ulem@depth", Stored::Int(depth - 1), Some(Scope::Global));
    let opened = lookup_string(&format!("lx@ulem@opened@{depth}"));
    let shared = lookup_bool(&format!("lx@ulem@shared@{depth}"));
    if matches!(lookup_value("lx@ulem@body"), Some(Stored::Int(own)) if own == depth) {
      // A cell's end (`a & \ul 6.7 \\`) closes no group TeX's reading of the argument counts: pdflatex's fatal "File
      // ended while scanning use of \UL@on". The cell's text stays underlined, the next cell intact.
      egroup()?;
      Error!(
        "expected",
        "}",
        "Missing } to end the underline declaration made by \\useunder",
        s!("The declaration at {opened} underlines the text up to its cell's end.")
      );
    } else {
      // the group's `}` closed the line's: it leaves the body and is read again for the group (ulem.sty:107)
      let mut closed = lookup_value("lx@ulem@ended").is_some();
      if let Some(body) = whatsit.get_arg_mut(3)
        && let DigestedData::List(list) = body.data()
      {
        let mut list = list.borrow_mut();
        let closer = list.boxes.last().and_then(|last| last.revert().ok());
        let brace = |tokens: Tokens| {
          matches!(tokens.unlist_ref().as_slice(), [t] if is_right_brace(t))
        };
        if closer.is_some_and(brace) {
          list.boxes.pop();
          unread_one(T_END!());
          closed = true;
        }
      }
      // Neither its group's `}` nor a `\multicolumn` body's end closed it: the input ended first (`Text \ul{x} y.`
      // outside any group), pdflatex's fatal "File ended while scanning use of \UL@on". The text stays underlined.
      if !closed || shared {
        let why = if shared {
          "is made in another declaration's text, whose end does not end it"
        } else {
          "underlines the text after it"
        };
        Error!(
          "expected",
          "}",
          "Missing } to end the underline declaration made by \\useunder",
          s!("The declaration at {opened} {why}.")
        );
      }
      assign_value("lx@ulem@ended", Stored::None, Some(Scope::Global));
    }
  });
  // A `\multicolumn` body's end closes the line's group as the template's `}` does, before the column's after-code,
  // and is read again for a declaration still open outside it.
  DefPrimitive!("\\lx@ulem@declared@end", {
    if is_value_bound("lx@ulem@body", Some(0)) {
      egroup()?;
      assign_value("lx@ulem@ended", Stored::Bool(true), Some(Scope::Global));
      unread_one(T_CS!("\\lx@alignment@multicolumn@end"));
    }
  });
  DefMacro!("\\lx@ulem@declared{}{}", sub[(ucmd, content)] {
    let mut out = ucmd.unlist();
    out.push(T_BEGIN!());
    out.extend(content.unlist());
    out.extend([T_END!(), T_END!()]);
    Ok(Tokens::new(out))
  });
  // ulem.sty:221 `\newbox\ULC@box`, the box the `\sout`-family builders measure in;
  // xeCJKfntef.sty sets it directly (xdupgthesis, xduugthesis, xduugtp; class
  // census 2026-09-24).
  RawTeX!(r"\newbox\ULC@box");
  // ulem.sty:63-66 `\ULset`, the underline set-up `\uline` runs before `\ULon` (its leaders at
  // `\ULdepth`), and :69-73 `\UL@setULdepth`, which sets `\ULdepth` from the font when it is unset
  // (`\maxdimen`): documents build their own lines from them (`\renewcommand\sout{\bgroup
  // \color{red}\ULdepth=-.5ex \ULset}`, a red double underline `\UL@setULdepth …\markoverwith{…}\ULon`).
  // The rule is drawing (as `\markoverwith`): the text is `\ULon`'s. Run 340: `\UL@setULdepth` 16,
  // `\ULset` 6 first errors (witnesses 2411.10835, 2204.07159, 2109.10013, 0709.1684, 2001.10689, 1706.09467).
  def_macro_noop("\\UL@setULdepth")?;
  DefMacro!("\\ULset", "\\UL@setULdepth\\ULon");
});
