//! animate.sty — PDF/SVG animation from graphics files
//!
//! Replaces the unbounded/many-frame loops of animate.sty with a single
//! representative frame (the first frame), avoiding memory exhaustion on
//! multi-frame animations (e.g. tikz-among-us with 180 frames).
//! Exposes `frame-count` as an attribute on `<ltx:block class="ltx_animate">`.

use std::{cell::RefCell, collections::HashMap};

use latexml_core::{mouth, tokens::TeXString};
use latexml_package::prelude::*;

struct AnimateContext {
  frame_count: usize,
}

thread_local! {
  static ANIM_STACK: RefCell<Vec<AnimateContext>> = const { RefCell::new(Vec::new()) };
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
  RequirePackage!("graphicx");

  model::add_tag_attribute("ltx:block", vec!["frame-count"]);

  DefEnvironment!(
    "{animateinline} [] {}",
    sub[document, _args, props] {
      model::add_tag_attribute("ltx:block", vec!["frame-count"]);
      document.maybe_close_element("ltx:p")?;
      let frame_count = ANIM_STACK
        .with(|s| s.borrow_mut().pop())
        .map(|c| c.frame_count)
        .unwrap_or(1);
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
    before_digest => {
      ANIM_STACK.with(|s| s.borrow_mut().push(AnimateContext { frame_count: 1 }));
    },
    mode => "internal_vertical"
  );

  DefMacro!("\\anim@set@framecount {Number}", sub[(count)] {
    let count_num = count.value_of().max(1) as usize;
    ANIM_STACK.with(|s| {
      if let Some(ctx) = s.borrow_mut().last_mut() {
        ctx.frame_count = count_num;
      }
    });
    Ok(Tokens::new(vec![]))
  });

  DefMacro!("\\multiframe {Number} {} {}", sub[(count, vars, body)] {
    let count_num = count.value_of().max(1) as usize;
    let is_inside_animate = ANIM_STACK.with(|s| !s.borrow().is_empty());
    if is_inside_animate {
      ANIM_STACK.with(|s| {
        if let Some(ctx) = s.borrow_mut().last_mut() {
          ctx.frame_count = count_num;
        }
      });
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

  DefMacro!("\\newframe OptionalMatch:* []", sub[(_star, _fps)] {
    ANIM_STACK.with(|s| {
      if let Some(ctx) = s.borrow_mut().last_mut() {
        ctx.frame_count += 1;
      }
    });
    Ok(Tokens::new(vec![]))
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
