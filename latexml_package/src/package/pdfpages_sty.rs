use crate::prelude::*;

fn is_page_spec(s: &str) -> bool {
  let s = s.trim();
  if s.is_empty() {
    return false;
  }
  let inner = if s.starts_with('{') && s.ends_with('}') && s.len() >= 2 {
    &s[1..s.len() - 1]
  } else {
    s
  };
  let inner = inner.trim();
  if inner == "-" || inner == "last" {
    return true;
  }
  let without_last = inner.replace("last", "");
  !without_last.is_empty()
    && without_last
      .chars()
      .all(|c| c.is_ascii_digit() || c == '-' || c == ',' || c.is_ascii_whitespace())
}

fn parse_file_page_list(input: &str) -> Vec<(String, Option<String>)> {
  let mut entries = Vec::new();
  let mut current = String::new();
  let mut depth = 0;
  for ch in input.chars() {
    match ch {
      '{' => {
        depth += 1;
        current.push(ch);
      },
      '}' => {
        if depth > 0 {
          depth -= 1;
        }
        current.push(ch);
      },
      ',' if depth == 0 => {
        let trimmed = current.trim();
        if !trimmed.is_empty() {
          entries.push(trimmed.to_string());
        }
        current.clear();
      },
      _ => {
        current.push(ch);
      },
    }
  }
  let trimmed = current.trim();
  if !trimmed.is_empty() {
    entries.push(trimmed.to_string());
  }

  let mut pairs = Vec::new();
  let mut current_file: Option<String> = None;

  for entry in entries {
    if let Some(file) = current_file.take() {
      if is_page_spec(&entry) {
        let ps = entry.trim();
        let ps = if ps.starts_with('{') && ps.ends_with('}') && ps.len() >= 2 {
          ps[1..ps.len() - 1].trim().to_string()
        } else {
          ps.to_string()
        };
        pairs.push((file, Some(ps)));
      } else {
        pairs.push((file, None));
        current_file = Some(entry);
      }
    } else {
      current_file = Some(entry);
    }
  }
  if let Some(file) = current_file {
    pairs.push((file, None));
  }

  pairs
}

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: pdfpages.sty.ltxml
  RequirePackage!("ifthen");
  RequirePackage!("calc");
  RequirePackage!("eso-pic");
  RequirePackage!("graphicx");

  // Perl pdfpages.sty.ltxml L30-36: `\includepdf OptionalKeyVals{}` —
  // optional keyvals with a `pages` key, then the file path. Constructor
  // emits a `<ltx:resource>` and follows with "See [pages X of ]<ref>".
  // Prior Rust stub used `[]` instead of `OptionalKeyVals` and dropped
  // the `pages` key entirely, so `\includepdf[pages=1-3]{foo}` never got
  // the "pages 1-3 of " prefix.
  // pdfpages.sty:205 `\includepdfset{keyvals}` accumulates default options
  // for later `\includepdf`s; ours ignores everything but `pages`, so the
  // defaults are absorbed (tutodoc-en/fr :1339; Perl pdfpages.sty.ltxml lacks
  // it too). Guard: `perfect_kernel_batch56::includepdfset_is_absorbed`.
  DefMacro!("\\includepdfset{}", "");
  DefConstructor!("\\includepdf OptionalKeyVals {}",
    "<ltx:resource src='#src' type='application/pdf'/>See #pages<ltx:ref href='#src'>#src</ltx:ref>",
    properties => sub[args] {
      let pages = args[0].as_ref().and_then(|d| {
        if let DigestedData::KeyVals(kvs) = d.data() {
          kvs.get_value("pages").map(|v| v.to_string())
        } else { None }
      });
      let src = args[1].as_ref().map(|d| d.to_string()).unwrap_or_default();
      Ok(stored_map!(
        "src"   => src,
        "pages" => pages.map(|p| format!("pages {} of ", p)).unwrap_or_default()
      ))
    });

  // pdfpages.sty:262 `\includepdfmerge[opts]{file-page-list}` iterates over a
  // comma-separated list of filenames and optional page specs (e.g. `doc.pdf, 1-9, other.pdf`),
  // delegating each file to `\includepdf` with its specific or default page range.
  DefMacro!("\\includepdfmerge OptionalKeyVals {}", sub[args] {
    let mut it = args.into_iter();
    let kv_arg = it.next().unwrap();
    let file_list_tokens: Tokens = it.next().unwrap().into();
    let file_list_str = file_list_tokens.to_string();

    let pairs = parse_file_page_list(&file_list_str);
    if pairs.is_empty() {
      return Ok(Tokens!());
    }

    // Extract any existing options from kv_arg (other than pages).
    let mut base_opts: Vec<(String, String)> = Vec::new();
    let default_pages = if let ArgWrap::KV(ref kv) = kv_arg {
      for (k, v) in kv.get_pairs() {
        if k != "pages" {
          base_opts.push((k.clone(), v.to_string()));
        }
      }
      kv.get_value("pages").map(|v| v.to_string())
    } else {
      None
    };

    let mut tex_buf = String::new();
    for (file, page_spec) in pairs {
      tex_buf.push_str("\\includepdf");
      let active_pages = page_spec.or_else(|| default_pages.clone());
      let mut file_opts = base_opts.clone();
      if let Some(ps) = active_pages {
        file_opts.push(("pages".to_string(), ps));
      }
      if !file_opts.is_empty() {
        tex_buf.push('[');
        let opt_parts: Vec<String> = file_opts
          .into_iter()
          .map(|(k, v)| format!("{k}={{{v}}}"))
          .collect();
        tex_buf.push_str(&opt_parts.join(","));
        tex_buf.push(']');
      }
      tex_buf.push('{');
      tex_buf.push_str(&file);
      tex_buf.push('}');
    }

    Ok(mouth::tokenize(TeXString::assembled(tex_buf)))
  });
});
