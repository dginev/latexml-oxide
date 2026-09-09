//! animate.sty — PDF/SVG animation from graphics files
//!
//! Replaces the unbounded/many-frame loops of animate.sty with a single
//! representative frame (the first frame), avoiding memory exhaustion on
//! multi-frame animations (e.g. tikz-among-us with 180 frames).
//! Exposes `frame-count` as an attribute on `<ltx:block class="ltx_animate">`.

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use latexml_core::{
  gullet::reading_from_mouth,
  keyvals::{KeyVals, KeyvalsConfig},
  mouth::{self, Mouth},
  tokens::TeXString,
};
use latexml_package::prelude::*;

use crate::discard_env::read_env_body_tokens;

struct AnimateContext {
  frame_count: usize,
  /// animate.sty:3197-3201 `end` key: code run at the END of every frame
  /// (`\@anim@endframe`, animate.sty:2314-2340); the `begin` code is emitted
  /// straight into the stream by `\begin{animateinline}`.
  end:         Vec<Token>,
  /// The representative frame has been closed (its `end` code emitted).
  ended:       bool,
}

/// Discard the rest of the `{animateinline}` body up to its literal
/// `\end{animateinline}` (unexpanded read, bounded by the end of input) and
/// count the `\newframe` separators it contained.
fn discard_remaining_frames() -> Result<usize> {
  let body = read_env_body_tokens("animateinline")?;
  Ok(
    body
      .iter()
      .filter(|t| t.get_catcode() == Catcode::CS && t.to_string() == "\\newframe")
      .count(),
  )
}

fn env_tokens(cs: &str, env: &str) -> Vec<Token> {
  let mut toks = vec![T_CS!(cs), T_BEGIN!()];
  toks.extend(tok_str(env).unlist());
  toks.push(T_END!());
  toks
}

/// The open `{animateinline}` contexts, innermost last. Each entry is SHARED
/// (`Rc`) with the environment whatsit's `anim_ctx` property
/// (`Stored::Opaque`), so the macros (`\newframe`, `\multiframe`,
/// `\end{animateinline}`) mutate the innermost one and the whatsit reads its
/// own at construction — in whatever order deferred whatsits construct. The
/// stack is popped when the environment ends (`after_digest`), not at
/// construction.
type AnimCtx = Rc<RefCell<AnimateContext>>;

thread_local! {
  static ANIM_STACK: RefCell<Vec<AnimCtx>> = const { RefCell::new(Vec::new()) };
}

/// Run `f` on the innermost open context, if any.
fn with_top<R>(f: impl FnOnce(&mut AnimateContext) -> R) -> Option<R> {
  let top: Option<AnimCtx> = ANIM_STACK.with(|s| s.borrow().last().cloned());
  top.map(|ctx| f(&mut ctx.borrow_mut()))
}

fn tok_str(s: &str) -> Tokens { mouth::tokenize_internal(TeXString::assembled(s.to_string())) }

/// Parse a single variable declaration like `var = init + inc` or `var = init - inc` or `var = init`.
fn parse_var_decl(decl: &str) -> Option<(String, String)> {
  let (name, rest) = decl.split_once('=')?;
  let var_name = name.trim().trim_start_matches('\\');
  let rest = rest.trim();
  if var_name.is_empty() || rest.is_empty() {
    return None;
  }
  // Find '+' or '-' separating initial value and increment,
  // skipping any leading sign of the initial value.
  let start = if rest.starts_with('+') || rest.starts_with('-') {
    1
  } else {
    0
  };
  let sep_idx = rest[start..].find(['+', '-']).map(|i| start + i);
  let mut init_val = match sep_idx {
    Some(idx) => rest[..idx].trim(),
    None => rest,
  };
  if init_val.starts_with('{') && init_val.ends_with('}') && init_val.len() >= 2 {
    init_val = init_val[1..init_val.len() - 1].trim();
  }
  Some((var_name.to_string(), init_val.to_string()))
}

#[rustfmt::skip]
LoadDefinitions!({
  // Per-conversion reset: a mid-body fatal leaves an unbalanced push behind
  // on a long-running worker thread (chemnum resets the same way).
  ANIM_STACK.with(|s| s.borrow_mut().clear());
  RequirePackage!("graphicx");

  model::add_tag_attribute("ltx:block", vec!["frame-count"]);
  // animate.sty:3197-3201: the per-frame `begin`/`end` code keys (the ones
  // the single-frame model consumes; the rest are read and skipped).
  DefKeyVal!("animate", "begin", "UndigestedKey");
  DefKeyVal!("animate", "end", "UndigestedKey");

  // `\begin{animateinline}[opts]{fps}` is a macro layer over the block
  // constructor `{lx@animateinline}`: the option list's `begin`/`end` code is
  // run at the start/end of the ONE representative frame, exactly where
  // animate.sty:2314-2340 runs `\@anim@begin`/`\@anim@end` for every frame,
  // and the first `\newframe` closes that frame and discards the rest of the
  // body (liftarm.tex:626-641 `\liftarmanimate` wraps every frame in a
  // tikzpicture through `begin=`/`end=`: without them 72 frames ran outside
  // any picture, 813 errors, sweep 63).
  DefMacro!(T_CS!("\\begin{animateinline}"), "[] {}", sub[(opts, _fps)] {
    // The option list is a keyval list (animate.sty:3197-3201 `begin`/`end`
    // are `.tl_gset:N` keys): the core `KeyVals` reader (`read_from`, the
    // `=`/`,` grammar with brace handling; `Tokens::to_keyvals` is only a
    // token-pairing), silenced for animate's many undeclared keys.
    let mut kv = KeyVals::new(KeyvalsConfig { keysets: vec!["animate".into()], ..Default::default() });
    if let Some(list) = opts.owned_tokens() {
      // `read_from` consumes the opening delimiter itself.
      let mut toks = vec![T_OTHER!("[")];
      toks.extend(list.unlist());
      toks.push(T_OTHER!("]"));
      reading_from_mouth(Mouth::default(), || {
        unread(Tokens::new(toks));
        kv.read_from(T_OTHER!("]"), true)
      })?;
    }
    let key_tokens = |key: &str| -> Vec<Token> {
      kv.get_value(key)
        .and_then(|a| a.clone().owned_tokens())
        .map(|t| t.unlist())
        .unwrap_or_default()
    };
    let begin = key_tokens("begin");
    let end = key_tokens("end");
    ANIM_STACK.with(|s| {
      s.borrow_mut()
        .push(Rc::new(RefCell::new(AnimateContext { frame_count: 1, end, ended: false })))
    });
    let mut toks = env_tokens("\\begin", "lx@animateinline");
    toks.extend(begin);
    Ok(Tokens::new(toks))
  });
  DefMacro!(T_CS!("\\end{animateinline}"), None, sub[_args] {
    let mut toks = with_top(|ctx| {
      if ctx.ended {
        Vec::new()
      } else {
        ctx.ended = true;
        ctx.end.clone()
      }
    })
    .unwrap_or_default();
    toks.extend(env_tokens("\\end", "lx@animateinline"));
    Ok(Tokens::new(toks))
  });

  DefEnvironment!(
    "{lx@animateinline}",
    sub[document, _args, props] {
      document.maybe_close_element("ltx:p")?;
      let frame_count = match props.get("anim_ctx") {
        Some(Stored::Opaque(payload)) => payload
          .downcast_ref::<AnimCtx>()
          .map(|ctx| ctx.borrow().frame_count)
          .unwrap_or(1),
        _ => 1,
      };
      let mut attrs = HashMap::default();
      attrs.insert("class".to_string(), "ltx_animate".to_string());
      attrs.insert("frame-count".to_string(), frame_count.to_string());
      document.open_element("ltx:block", Some(attrs), None)?;
      if let Some(Stored::Digested(body)) = props.get("body") {
        document.absorb(body, None)?;
      }
      document.close_element("ltx:block")?;
      Ok(())
    },
    mode => "internal_vertical",
    properties => {
      let mut props = stored_map!();
      if let Some(ctx) = ANIM_STACK.with(|s| s.borrow().last().cloned()) {
        props.insert("anim_ctx", Stored::Opaque(Rc::new(ctx)));
      }
      Ok(props)
    },
    after_digest => sub[_whatsit] {
      ANIM_STACK.with(|s| s.borrow_mut().pop());
    }
  );

  DefMacro!("\\anim@set@framecount {Number}", sub[(count)] {
    let count_num = count.value_of().max(1) as usize;
    with_top(|ctx| ctx.frame_count = count_num);
    Ok(Tokens::new(vec![]))
  });

  DefMacro!("\\multiframe {Number} {} {}", sub[(count, vars, body)] {
    let count_num = count.value_of().max(1) as usize;
    let is_inside_animate = ANIM_STACK.with(|s| !s.borrow().is_empty());
    if is_inside_animate {
      // `\multiframe{n}` inside a frame sequence stands for n frames.
      with_top(|ctx| ctx.frame_count += count_num - 1);
    }

    let vars_str = vars.to_string();
    let mut toks = Vec::new();
    for decl in vars_str.split(',') {
      if let Some((name, init_val)) = parse_var_decl(decl) {
        toks.push(T_CS!("\\def"));
        toks.push(T_CS!(format!("\\{}", name)));
        toks.push(T_BEGIN!());
        toks.extend(tok_str(&init_val).unlist());
        toks.push(T_END!());
      }
    }

    toks.extend(body.unlist());

    if !is_inside_animate {
      let mut wrapped = Vec::new();
      wrapped.push(T_CS!("\\begin"));
      wrapped.push(T_BEGIN!());
      wrapped.extend(tok_str("animateinline").unlist());
      wrapped.push(T_END!());
      wrapped.push(T_BEGIN!());
      wrapped.extend(tok_str("1").unlist());
      wrapped.push(T_END!());

      wrapped.push(T_CS!("\\anim@set@framecount"));
      wrapped.push(T_BEGIN!());
      wrapped.extend(tok_str(&count_num.to_string()).unlist());
      wrapped.push(T_END!());

      wrapped.extend(toks);

      wrapped.push(T_CS!("\\end"));
      wrapped.push(T_BEGIN!());
      wrapped.extend(tok_str("animateinline").unlist());
      wrapped.push(T_END!());
      Ok(Tokens::new(wrapped))
    } else {
      Ok(Tokens::new(toks))
    }
  });

  // The first `\newframe` ends the representative frame: it runs the `end`
  // code, discards the remaining frames (counting their separators for
  // `frame-count`) and re-emits the environment end.
  DefMacro!("\\newframe OptionalMatch:* []", sub[(_star, _fps)] {
    let end = with_top(|ctx| {
      ctx.frame_count += 1;
      if ctx.ended {
        None
      } else {
        ctx.ended = true;
        Some(ctx.end.clone())
      }
    })
    .flatten();
    let Some(mut toks) = end else {
      return Ok(Tokens::new(vec![]));
    };
    let rest = discard_remaining_frames()?;
    with_top(|ctx| ctx.frame_count += rest);
    toks.extend(env_tokens("\\end", "animateinline"));
    Ok(Tokens::new(toks))
  });

  def_macro_noop("\\multiframebreak")?;

  DefMacro!(
    "\\animategraphics [] {} {} {} {}",
    sub[(opts, fps, basename, first, last)] {
      let f_str = first.to_string().trim().to_string();
      let l_str = last.to_string().trim().to_string();
      let base_str = basename.to_string().trim().to_string();
      let f_num = f_str.parse::<i64>();
      let l_num = l_str.parse::<i64>();
      let frame_count = match (f_num, l_num) {
        (Ok(f), Ok(l)) => ((l - f).abs() + 1) as usize,
        _ => 1,
      };
      let file = if f_str.is_empty() {
        base_str
      } else {
        format!("{base_str}{f_str}")
      };
      let mut toks = Vec::new();
      // \begin{animateinline}[opts]{fps}
      toks.push(T_CS!("\\begin"));
      toks.push(T_BEGIN!());
      toks.extend(tok_str("animateinline").unlist());
      toks.push(T_END!());
      if let Some(ref opts) = opts {
        toks.push(T_OTHER!("["));
        toks.extend(opts.clone().unlist());
        toks.push(T_OTHER!("]"));
      }
      toks.push(T_BEGIN!());
      toks.extend(fps.unlist());
      toks.push(T_END!());

      // \anim@set@framecount{frame_count}
      toks.push(T_CS!("\\anim@set@framecount"));
      toks.push(T_BEGIN!());
      toks.extend(tok_str(&frame_count.to_string()).unlist());
      toks.push(T_END!());

      // \includegraphics[opts]{file}
      toks.push(T_CS!("\\includegraphics"));
      if let Some(opts) = opts {
        toks.push(T_OTHER!("["));
        toks.extend(opts.unlist());
        toks.push(T_OTHER!("]"));
      }
      toks.push(T_BEGIN!());
      toks.extend(tok_str(&file).unlist());
      toks.push(T_END!());

      // \end{animateinline}
      toks.push(T_CS!("\\end"));
      toks.push(T_BEGIN!());
      toks.extend(tok_str("animateinline").unlist());
      toks.push(T_END!());

      Ok(Tokens::new(toks))
    }
  );
});
