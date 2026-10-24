//! xr.sty and xr-hyper.sty — references into another document. Perl ships no binding.
use latexml_core::{
  binding::content::{FindFileOptions, find_file},
  common::error::emit_warn,
};
use latexml_engine::latex_constructs::{record_external_cref, record_external_label};
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // xr.sty:44-47 (v6; xr-hyper.sty:38-41 alike): `\externaldocument[prefix][nocite]{file}[url]` reads `file.aux` and
  // every `.aux` it `\@input`s, each `\newlabel{l}{{text}{page}…}` defining `\r@<prefix>l` (`\XR@test`), so `\ref{l}`
  // prints the other document's number. The raw package's `\r@…` never reached our `\ref`, a constructor post resolves:
  // each such reference was a missing label where pdflatex prints "Fig. S3" (1307.3388, 1608.05314; 21 of the census's 125
  // `\externaldocument` papers ship the external `.aux`, the rest print "??" in pdflatex too). The labels are recorded
  // for `\ref` and `\eqref`, their text typeset as TeX (sect11.rs `record_external_label`), and cleveref's `<label>@cref`
  // records and hyperref's anchors for `\cref` and `\autoref`, which name the type (2412.14256, 2508.18808, 2402.05535);
  // the `\bibcite`s v6 imports too (unless the second argument says `nocite`) are not. Where the URL is given its anchors are linked; the default, `file.pdf`, names no
  // page of ours. The file is Semiverbatim so a name with `_` (`ex_supplement`) does not leak the underscore into text
  // mode (witness 2402.12241); xr v5 read `[prefix]{file}` only (xr-2023-07-04.sty:46-48), which this reads too.
  DefPrimitive!("\\externaldocument[][] Semiverbatim OptionalSemiverbatim", sub[(prefix, _nocite, file, url)] {
    let prefix = prefix.map(|tokens| do_expand(tokens).map(|t| t.to_string())).transpose()?.unwrap_or_default();
    let url = url.map(|tokens| tokens.to_string());
    import_external_document(prefix.trim(), file.to_string().trim(), url.as_deref().map(str::trim));
  });
  Let!("\\externalcitedocument", "\\externaldocument");
});

/// `\XR@@`/`\XR@loop` (xr.sty:57-80): the labels of `file.aux` and of the `.aux` files it `\@input`s (each named from
/// the directory of the file that inputs it, `\filename@area`), recorded under `prefix`; a missing file warns, as xr's
/// "No file …  LABELS NOT IMPORTED" does.
fn import_external_document(prefix: &str, file: &str, url: Option<&str>) {
  // `\set@curr@file` unquotes the name (latex.ltx:9654-9666); `\XR@next\@curr@file.aux` appends `.aux` to all of it
  let mut queue = vec![s!("{}.aux", file.replace('"', ""))];
  let mut seen = Vec::new();
  while let Some(aux) = queue.pop() {
    if seen.contains(&aux) {
      continue;
    }
    seen.push(aux.clone());
    let found = find_file(
      &aux,
      Some(FindFileOptions {
        ext_type: Some("aux".into()),
        ..FindFileOptions::default()
      }),
    );
    let Some(text) = found.and_then(|path| std::fs::read_to_string(path).ok()) else {
      emit_warn(
        "missing_file",
        &aux,
        &s!("No file {aux}; labels not imported (xr)"),
      );
      continue;
    };
    let area = aux.rfind('/').map_or("", |slash| &aux[..=slash]);
    for line in text.lines() {
      if let Some(rest) = line.trim_start().strip_prefix("\\newlabel") {
        let mut groups = brace_groups(rest);
        let (Some(key), Some(value)) = (groups.next(), groups.next()) else {
          continue;
        };
        let fields: Vec<&str> = brace_groups(value).collect();
        let Some(text) = fields.first().map(|text| text.trim()) else {
          continue;
        };
        // cleveref's own record of the label, `\newlabel{<label>@cref}{{[<type>][<n>][<parents>]<number>}…}`
        // (cleveref.sty `\cref@label`), is what `\cref` reads (`\r@<label>@cref`)
        if let Some(key) = key.strip_suffix("@cref") {
          if let Some((ty, number)) = cref_record(text) {
            let name = s!("{prefix}{key}");
            record_external_cref(&clean_label(&name, None), ty, number);
          }
          continue;
        }
        // hyperref's `\newlabel` holds the anchor fourth (`{text}{page}{title}{anchor}{}`)
        let anchor = fields
          .get(3)
          .map(|anchor| anchor.trim())
          .filter(|anchor| !anchor.is_empty());
        let href = url.filter(|url| !url.is_empty()).map(|url| match anchor {
          Some(anchor) => s!("{url}#{anchor}"),
          None => url.to_string(),
        });
        let name = s!("{prefix}{key}");
        let label = clean_label(&name, None);
        record_external_label(&label, text, anchor, href.as_deref());
      } else if let Some(rest) = line.trim_start().strip_prefix("\\@input")
        && let Some(name) = brace_groups(rest).next()
      {
        queue.push(s!("{area}{name}"));
      }
    }
  }
}

/// The balanced `{…}` groups at the start of `text`, their contents without the outer braces; a `\{` or `\}` does not
/// count.
fn brace_groups(text: &str) -> impl Iterator<Item = &str> {
  let mut rest = text;
  std::iter::from_fn(move || {
    let start = rest.find(|c: char| !c.is_whitespace())?;
    if !rest[start..].starts_with('{') {
      return None;
    }
    let (mut depth, mut escaped) = (0usize, false);
    for (offset, c) in rest[start..].char_indices() {
      match c {
        _ if escaped => escaped = false,
        '\\' => escaped = true,
        '{' => depth += 1,
        '}' => {
          depth -= 1;
          if depth == 0 {
            let group = &rest[start + 1..start + offset];
            rest = &rest[start + offset + 1..];
            return Some(group);
          }
        },
        _ => {},
      }
    }
    None
  })
}

/// The type and the number of a cleveref record, `[<type>][<n>][<parents>]<number>` (cleveref.sty `\cref@label`).
fn cref_record(text: &str) -> Option<(&str, &str)> {
  let mut rest = text.trim_start();
  let mut groups = Vec::new();
  for _ in 0..3 {
    let inner = rest.strip_prefix('[')?;
    let close = inner.find(']')?;
    groups.push(&inner[..close]);
    rest = &inner[close + 1..];
  }
  Some((groups[0].trim(), rest.trim()))
}
