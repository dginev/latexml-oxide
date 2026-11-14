use crate::prelude::*;

/// thm-restate.sty:35-49 `\thmt@collect@body`: the tokens up to the first `\end{<env>}` outside braces (the `\end` of
/// another environment is body), the `\end{<env>}` left in the input.
fn collect_body(env: &str) -> Result<Vec<Token>> {
  let end_delim = Tokens!(T_CS!("\\end"));
  let mut body: Vec<Token> = Vec::new();
  loop {
    let Some(text) = read_until(&end_delim)? else {
      break; // end of input: the environment's own runaway
    };
    body.extend(text.unlist());
    let name = read_arg(ExpansionLevel::Off)?;
    let mut end = vec![T_CS!("\\end"), T_BEGIN!()];
    end.extend(name.unlist_ref().iter().copied());
    end.push(T_END!());
    if name.to_string() == env {
      unread_vec(end);
      break;
    }
    body.extend(end);
  }
  Ok(body)
}

/// The state key of a stored theorem's `part` (`env`, `opt`, `body`).
fn stored(part: &str, name: &str) -> String { s!("thm_restate_{part}:{name}") }

fn stored_tokens(part: &str, name: &str) -> Vec<Token> {
  match lookup_value(&stored(part, name)) {
    Some(Stored::Tokens(tokens)) => tokens.unlist(),
    _ => Vec::new(),
  }
}

/// What a theorem stored as `<name>` records right after its head: its number (`\the<counter>`, latex_constructs
/// `\lx@the@@`, frozen), the one a starred restatement prints (thm-restate.sty:130-132 fakes `\the<env>` from the label
/// `thmt@@<name>` that this theorem sets, :158-160)…
fn renumber(name: &str, env: &str) -> Tokens {
  let mut tokens = vec![
    T_CS!("\\expandafter"),
    T_CS!("\\protected@xdef"),
    T_CS!("\\csname"),
  ];
  tokens.extend(Explode!(s!("lx@thm@restate@num@{name}")));
  tokens.extend([
    T_CS!("\\endcsname"),
    T_BEGIN!(),
    T_CS!("\\lx@the@@"),
    T_BEGIN!(),
  ]);
  tokens.extend(Explode!(env));
  tokens.extend([T_END!(), T_END!()]);
  // …and the equation counter before its body, for the equations restated with it (:89-101, :146-152)
  tokens.extend([T_CS!("\\expandafter"), T_CS!("\\xdef"), T_CS!("\\csname")]);
  tokens.extend(Explode!(s!("lx@thm@restate@eq@{name}")));
  tokens.extend([
    T_CS!("\\endcsname"),
    T_BEGIN!(),
    T_CS!("\\the"),
    T_CS!("\\c@equation"),
    T_END!(),
  ]);
  Tokens::new(tokens)
}

/// `\label{restatable:<name>}`, the label of a stored theorem's numbered instance (thm-restate.sty:158-160
/// `\label{thmt@@<name>}`).
fn label(name: &str) -> Vec<Token> {
  let mut tokens = vec![T_CS!("\\label"), T_BEGIN!()];
  tokens.extend(Explode!(s!("restatable:{name}")));
  tokens.push(T_END!());
  tokens
}

/// `\ref{restatable:<name>}`: the number of the stored theorem's numbered instance (thm-restate.sty:133-135
/// `\thmt@trivialref{thmt@@<name>}{??}`).
fn restate_ref(name: &str) -> Vec<Token> {
  let mut tokens = vec![T_CS!("\\ref"), T_BEGIN!()];
  tokens.extend(Explode!(s!("restatable:{name}")));
  tokens.push(T_END!());
  tokens
}

#[rustfmt::skip]
LoadDefinitions!({
  // thm-restate.sty:26 `\RequirePackage{thmtools}`: `\usepackage{thm-restate}` alone brings `\declaretheorem`,
  // `\declaretheoremstyle` and the rest of thmtools (witnesses 2412.02079, 2411.02602). Perl
  // thm-restate.sty.ltxml loads nothing (KNOWN_PERL_ERRORS #629). thmtools requires thm-restate in turn
  // (thmtools.sty:47-49); the cycle ends at the package already in progress, as `\ver@thmtools.sty` ends it in LaTeX.
  RequirePackage!("thmtools");
  // A theorem stored for `\<name>` (thm-restate.sty:103-184 `thmt@restatable`). `{restatable}[<opt>]{<env>}{<name>}`
  // prints its theorem, `\begin{<env>}[<opt>]`, and stores it; so does a theorem's `restate=<name>` key
  // (thm-restate.sty:198-215; latex_constructs `thmtools_use_keys`, which passes the other keys as `<opt>`; 2108.11527,
  // 2602.09462). `\lx@thm@restate@store{<end>}{<env>}{<name>}{<opt>}`, read right after the theorem's head, labels it
  // `restatable:<name>`, freezes its number and stores the body up to `\end{<end>}` (`\thmt@collect@body`), which it
  // leaves in place. `\<name>*` then prints the statement again (thm-restate.sty:122-156): `\begin{<env>}[<opt>]<body>
  // \end{<env>}` in a group where `\label` is gobbled, numbered as the original, no counter stepped, the equation
  // counter put back after it; `\<name>` without the star is the theorem once more as a new one, numbered and labelled,
  // whose number the next `\<name>*` prints (the "this is the one" branch, :116-121, :158-160). Perl
  // (thm-restate.sty.ltxml:28) prints "See <ref>" in place of the statement (KNOWN_PERL_ERRORS #1048). Witnesses
  // 2108.11527, 2602.09462, 2402.14202; guard `perfect_kernel_batch78i::restated_theorem_prints_its_statement`.
  DefMacro!("\\lx@thm@restate@store{}{}{}{}", sub[(end, env, name, opt)] {
    let (end, env, name) = (end.to_string(), env.to_string(), name.to_string());
    let body = collect_body(&end)?;
    assign_value(&stored("env", &name), Stored::Tokens(Tokens::new(Explode!(&env))), Some(Scope::Global));
    assign_value(&stored("opt", &name), Stored::Tokens(opt), Some(Scope::Global));
    assign_value(&stored("body", &name), Stored::Tokens(Tokens::new(body.clone())), Some(Scope::Global));
    let mut tokens = Vec::new();
    if end == "restatable*" {
      // thm-restate.sty:207-213, :130-145: `{restatable*}` is not the one — its number, and a `\<name>*`'s until then,
      // is the later `\<name>`'s that sets the label last (:158-160; each sets it, pdflatex keeps the last), as
      // `continues=` takes its theorem's (the equation counter stored as for the one)
      tokens.extend(renumber(&name, &env).unlist());
      tokens.extend([T_CS!("\\expandafter"), T_CS!("\\gdef"), T_CS!("\\csname")]);
      tokens.extend(Explode!(s!("lx@thm@restate@num@{name}")));
      tokens.extend([T_CS!("\\endcsname"), T_BEGIN!()]);
      tokens.extend(restate_ref(&name));
      tokens.push(T_END!());
    } else {
      tokens.extend(label(&name));
      tokens.extend(renumber(&name, &env).unlist());
    }
    // `\<name>`: `\@ifstar{\lx@thm@restate*{<name>}}{\lx@thm@restate{<name>}}`
    tokens.extend([T_CS!("\\expandafter"), T_CS!("\\gdef"), T_CS!("\\csname")]);
    tokens.extend(Explode!(&name));
    tokens.extend([T_CS!("\\endcsname"), T_BEGIN!(), T_CS!("\\@ifstar"), T_BEGIN!(), T_CS!("\\lx@thm@restate"),
      T_OTHER!("*"), T_BEGIN!()]);
    tokens.extend(Explode!(&name));
    tokens.extend([T_END!(), T_END!(), T_BEGIN!(), T_CS!("\\lx@thm@restate"), T_BEGIN!()]);
    tokens.extend(Explode!(&name));
    tokens.extend([T_END!(), T_END!(), T_END!()]);
    tokens.extend(body);
    Ok(Tokens::new(tokens))
  });
  // `\lx@thm@restate[*]{<name>}`: the stored theorem printed again (above).
  DefMacro!("\\lx@thm@restate OptionalMatch:* {}", sub[(star, name)] {
    let name = name.to_string();
    let env = stored_tokens("env", &name);
    let env_name: String = env.iter().map(|t| t.to_string()).collect();
    let opt = stored_tokens("opt", &name);
    let begin = |tokens: &mut Vec<Token>| {
      tokens.extend([T_CS!("\\begin"), T_BEGIN!()]);
      tokens.extend(env.iter().copied());
      tokens.push(T_END!());
      // thm-restate.sty:153 `\ifx\@nx#1\@nx\else[{#1}]\fi`
      if !opt.is_empty() {
        tokens.push(T_OTHER!("["));
        tokens.extend(opt.iter().copied());
        tokens.push(T_OTHER!("]"));
      }
    };
    let mut tokens = vec![T_CS!("\\begingroup")];
    if star.is_some() {
      // :133-145: the number faked, the counter not stepped, labels gobbled; :146-152, :173: the inner counter put back
      tokens.extend(TokenizeInternal!(r"\let\label\thmt@gobble@label\let\ltx@label\@gobble\edef\lx@thm@restate@eq{\the\c@equation}").unlist());
      tokens.extend([T_CS!("\\lx@thm@restate@number"), T_BEGIN!(), T_CS!("\\csname")]);
      tokens.extend(Explode!(s!("lx@thm@restate@num@{name}")));
      tokens.extend([T_CS!("\\endcsname"), T_END!()]);
      tokens.extend(TokenizeInternal!(r"\setcounter{equation}").unlist());
      tokens.extend([T_BEGIN!(), T_CS!("\\csname")]);
      tokens.extend(Explode!(s!("lx@thm@restate@eq@{name}")));
      tokens.extend([T_CS!("\\endcsname"), T_END!()]);
      begin(&mut tokens);
    } else {
      begin(&mut tokens);
      // each sets the label `thmt@@<name>` (:158-160), the last one the label the earlier numbers refer to (pdflatex
      // reads the last `\newlabel`, "Label `thmt@@<name>' multiply defined"): a `{restatable*}` previews the last
      // `\<name>`'s number, not the first's
      tokens.extend(label(&name));
      tokens.extend(renumber(&name, &env_name).unlist());
    }
    tokens.extend(stored_tokens("body", &name));
    tokens.extend([T_CS!("\\end"), T_BEGIN!()]);
    tokens.extend(env.iter().copied());
    tokens.push(T_END!());
    if star.is_some() {
      tokens.extend(TokenizeInternal!(r"\setcounter{equation}{\lx@thm@restate@eq}").unlist());
    }
    tokens.push(T_CS!("\\endgroup"));
    Ok(Tokens::new(tokens))
  });
  // The number the next theorem takes without stepping its counter (latex_constructs, the theorem's properties).
  DefPrimitive!("\\lx@thm@restate@number{}", sub[(number)] {
    assign_value("thmt_restate_number", Stored::Tokens(number), None);
  });
  // thm-restate.sty:186-191: a label in a restated theorem is gobbled, cleveref's `\label[<type>]{…}` too.
  RawTeX!(r"\let\thmt@gobble@label\@gobble
\AtBeginDocument{\@ifpackageloaded{cleveref}{\renewcommand*\thmt@gobble@label[2][]{}}{}}");
  RawTeX!("\\newenvironment{restatable}[3][]{\\begin{#2}[#1]\\lx@thm@restate@store{restatable}{#2}{#3}{#1}}{\\end{\\@currenvir}}");
  // thm-restate.sty:207-213 `{restatable*}` is `\thmt@thisistheonefalse`: its theorem printed as a starred restatement
  // is (:130-152) — numbered as the later `\<name>` that is the one (a reference to that instance's label, resolved
  // with the document's references, as `continues=`; pdflatex reads it from the .aux), no counter stepped, its labels
  // gobbled (the later `\<name>` sets them), the equation counter put back after it. Witness 1702.04225 (the
  // introduction's "Theorem 6.21", "Proposition 6.13"; the restate census's other `{restatable*}` papers 1407.7827,
  // 1910.14249, 1911.00646, 2004.07319, 2402.00791, 2404.17677, 2508.18759). Guard
  // `perfect_kernel_batch78i::{restatable_star_takes_the_later_number, restatable_star_takes_the_last_number}`.
  RawTeX!(r"\newenvironment{restatable*}[3][]{\let\label\thmt@gobble@label\let\ltx@label\@gobble
\edef\lx@thm@restate@eq{\the\c@equation}\lx@thm@restate@number{\ref{restatable:#3}}%
\begin{#2}[#1]\lx@thm@restate@store{restatable*}{#2}{#3}{#1}}{\end{\@currenvir}\setcounter{equation}{\lx@thm@restate@eq}}");
});
