use latexml_core::util::image::{image_candidates, read_pdf_page_count};

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

/// The pages an `\includepdf` page list names, in order (pdfpages.sty's `pages` key: a comma list of `N`, `N-M`
/// (descending when `M < N`), `N-`, `-M`, `-`, `last`, and `{}`, a blank page, which places nothing here). `None` for a
/// list not understood, or one that needs the page count when `count` cannot read it.
fn include_pdf_pages(spec: &str, count: impl FnOnce() -> Option<u32>) -> Option<Vec<u32>> {
  let spec = spec.trim();
  // (one group around the whole list is the list: `{1,3}`, not `{},1,{}`'s first and last blank pages)
  let spec = match outer_group(spec) {
    Some(inner) if !inner.trim().is_empty() => inner,
    _ => spec,
  };
  let mut count = Some(count);
  let mut known: Option<u32> = None;
  let mut last = || -> Option<u32> {
    if let Some(count) = count.take() {
      known = count();
    }
    known
  };
  let mut bound = |text: &str, open: u32| -> Option<u32> {
    match text.trim() {
      "" if open == 0 => last(),
      "" => Some(open),
      "last" => last(),
      number => number.parse().ok().filter(|n| *n > 0),
    }
  };
  let mut pages = Vec::new();
  for item in spec.split(',').map(str::trim) {
    if item.is_empty() || item == "{}" {
      continue;
    }
    match item.split_once('-') {
      Some((from, to)) => {
        let from = bound(from, 1)?;
        let to = bound(to, 0)?;
        if from <= to {
          pages.extend(from..=to);
        } else {
          pages.extend((to..=from).rev());
        }
      },
      None => pages.push(bound(item, 0)?),
    }
  }
  Some(pages)
}

/// The pages of `pages` the file has (all, when its page `count` is not known), and those past its last.
fn split_existing_pages(pages: Vec<u32>, count: Option<u32>) -> (Vec<u32>, Vec<u32>) {
  match count {
    Some(count) => pages.into_iter().partition(|page| *page <= count),
    None => (pages, Vec::new()),
  }
}

/// The text inside `text`'s braces when one group encloses it all.
fn outer_group(text: &str) -> Option<&str> {
  let inner = text.strip_prefix('{')?.strip_suffix('}')?;
  let mut depth = 0i32;
  for ch in inner.chars() {
    match ch {
      '{' => depth += 1,
      '}' if depth == 0 => return None,
      '}' => depth -= 1,
      _ => {},
    }
  }
  (depth == 0).then_some(inner)
}

/// The PDF `name` names, found as a graphic is (`\graphicspath`, the source directory).
fn pdf_file(name: &str) -> Option<std::path::PathBuf> {
  let found = image_candidates(name)
    .split(',')
    .find(|found| found.to_ascii_lowercase().ends_with(".pdf"))
    .map(str::to_string)?;
  let path = std::path::Path::new(&found);
  let path = if path.is_relative() {
    std::path::Path::new(&lookup_string("SOURCEDIRECTORY")).join(path)
  } else {
    path.to_path_buf()
  };
  path.is_file().then_some(path)
}

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: pdfpages.sty.ltxml
  RequirePackage!("ifthen");
  RequirePackage!("calc");
  RequirePackage!("eso-pic");
  RequirePackage!("graphicx");

  // pdfpages.sty:205 `\includepdfset{keyvals}` accumulates default options
  // for later `\includepdf`s; ours ignores them, so the
  // defaults are absorbed (tutodoc-en/fr :1339; Perl pdfpages.sty.ltxml lacks
  // it too). Guard: `perfect_kernel_batch56::includepdfset_is_absorbed`.
  DefMacro!("\\includepdfset{}", "");
  // `\includepdf[pages=<list>]{file}` places the file's pages, one a page (pdfpages.sty: `pages` defaults to 1, `-`
  // is all of them, `{}` a blank page). Each page is the file's graphic at that page, `\includegraphics[page=N]`, as
  // pdfpages itself includes it (pdfpages.sty `\AM@includegraphics`), with the graphicx keys given (`scale`, `angle`,
  // `width`, `height`, `trim`, `clip`, `viewport`, `keepaspectratio`); an open range (`-`, `3-`, `last`) reads the
  // page count from the file. Perl (pdfpages.sty.ltxml:30-36) and the former binding printed "See pages - of
  // ⟨link⟩" with the PDF as a resource, which remains for a PDF not found (pdflatex's "Cannot find file") or a
  // list not understood:
  // OXIDIZED_DESIGN_DIVERGENCES #846. Guard `perfect_kernel_batch78j::includepdf_places_its_pages`.
  DefMacro!("\\includepdf OptionalKeyVals Semiverbatim", sub[args] {
    let mut it = args.into_iter();
    let kv_arg = it.next().unwrap();
    let file: Tokens = it.next().unwrap().into();
    // (the name and the page list expanded, as pdfpages reads them: `\foreach \x {…}` loops a page `pages={\x}`,
    // a name kept in a macro, 2605.05847, 2406.13026)
    let name = Expand!(file).to_string().trim().to_string();
    let mut spec = None;
    let mut graphic_keys = Vec::new();
    if let ArgWrap::KV(ref kv) = kv_arg {
      for (key, value) in kv.get_pairs() {
        match key.as_str() {
          "pages" => {
            spec = Some(match value.as_tokens() {
              Ok(Some(tokens)) => Expand!(tokens.into_owned()).to_string(),
              _ => value.to_string(),
            })
          },
          "scale" | "angle" | "width" | "height" | "trim" | "clip" | "viewport" | "keepaspectratio" => {
            graphic_keys.push(s!("{key}={{{}}}", value.to_string()))
          },
          _ => {},
        }
      }
    }
    let mut out = String::new();
    let (mut count, mut file_name) = (None, name.clone());
    let pages = pdf_file(&name).and_then(|path| {
      count = read_pdf_page_count(&path);
      if let Some(found) = path.file_name() {
        file_name = found.to_string_lossy().into_owned();
      }
      include_pdf_pages(spec.as_deref().unwrap_or("1"), || count)
    });
    match pages {
      Some(pages) => {
        // A page past the file's last is an error and places nothing (pdfpages.sty:909-920 `\AM@checkpagenumber`, "Page 6
        // of `five.pdf' does not exist"; pdflatex then fails to include it); a count not read keeps every page.
        let (pages, missing) = split_existing_pages(pages, count);
        for page in missing {
          Error!("latex", "(pdfpages)", s!("Package pdfpages Error: Page {page} of `{file_name}' does not exist."));
        }
        for page in pages {
          let mut keys = graphic_keys.clone();
          keys.push(s!("page={page}"));
          out.push_str(&s!("\\par\\includegraphics[{}]{{{name}}}\\par", keys.join(",")));
        }
      },
      None => {
        out.push_str("\\lx@includepdf@link");
        if let Some(spec) = spec {
          out.push_str(&s!("[{spec}]"));
        }
        out.push_str(&s!("{{{name}}}"));
      },
    }
    Ok(mouth::tokenize_internal(TeXString::assembled(out)))
  });
  // Perl pdfpages.sty.ltxml L30-36: a link to the PDF, copied as a resource, "See [pages X of ]<ref>". The file name
  // is Semiverbatim, as Perl graphicx.sty.ltxml:52 reads `\@includegraphicx`'s: Perl's `{}` tokenized
  // `\includepdf{a_b.pdf}` under document catcodes and errored "Script _ can only appear in math mode" (latex4wp,
  // `pandoc_template.pdf`; pdflatex reads the name as a file name). Guard:
  // `binding_singletons_56::includepdf_file_name_is_semiverbatim`.
  DefConstructor!("\\lx@includepdf@link[] Semiverbatim",
    "<ltx:resource src='#src' type='application/pdf'/>See #pages<ltx:ref href='#src'>#src</ltx:ref>",
    properties => sub[args] {
      let pages = args[0].as_ref().map(|d| format!("pages {d} of ")).unwrap_or_default();
      let src = args[1].as_ref().map(|d| d.to_string()).unwrap_or_default();
      Ok(stored_map!("src" => src, "pages" => pages))
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn include_pdf_pages_reads_pdfpages_lists() {
    let pages = |spec: &str| include_pdf_pages(spec, || Some(5));
    assert_eq!(pages("1"), Some(vec![1]));
    assert_eq!(pages("-"), Some(vec![1, 2, 3, 4, 5]));
    assert_eq!(pages("{2-4}"), Some(vec![2, 3, 4]));
    assert_eq!(pages("3-"), Some(vec![3, 4, 5]));
    assert_eq!(pages("-2"), Some(vec![1, 2]));
    assert_eq!(pages("last"), Some(vec![5]));
    assert_eq!(pages("last-3"), Some(vec![5, 4, 3]));
    assert_eq!(pages("{},1,{},2,{}"), Some(vec![1, 2]));
    assert_eq!(pages("{{},1,{},3}"), Some(vec![1, 3]));
    assert_eq!(pages("x"), None);
    assert_eq!(include_pdf_pages("-", || None), None);
    assert_eq!(include_pdf_pages("2,4", || None), Some(vec![2, 4]));
    // (`pages=4-7` of a 5-page file: pages 6 and 7 do not exist, an error each, pdfpages.sty:909-920)
    assert_eq!(
      split_existing_pages(pages("4-7").unwrap(), Some(5)),
      (vec![4, 5], vec![6, 7])
    );
    assert_eq!(split_existing_pages(vec![2, 9], None), (vec![2, 9], vec![]));
  }
}
