//! xr.sty and xr-hyper.sty — references into another document. Perl ships no binding.
use latexml_core::{
  binding::content::{FindFileOptions, find_file},
  common::error::emit_warn,
};
use latexml_engine::latex_constructs::{brace_groups, record_newlabel};
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
        if let (Some(key), Some(value)) = (groups.next(), groups.next()) {
          record_newlabel(&s!("{prefix}{key}"), value, url);
        }
      } else if let Some(rest) = line.trim_start().strip_prefix("\\@input")
        && let Some(name) = brace_groups(rest).next()
      {
        queue.push(s!("{area}{name}"));
      }
    }
  }
}
