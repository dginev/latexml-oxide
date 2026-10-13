//! Shared helper: the shipped class file a class-family binding reads or inspects (`compositio_cls`, `jinst_cls`,
//! `mn2e_cls`).
use crate::prelude::*;

/// The class file in the paper's source that the family binding `binding` interprets: the class the document asked for
/// when the loader answered it with this binding, else the binding's own name; `None` when the source does not have it.
/// The request is the fallback's (`fallback_request`, when its `fallback_target` is this binding: a renamed `JINST_mod`
/// or `myJINST`, 1307.5525, 1407.3938), or the load's own `\@currname` when the dispatch matched the binding by the
/// basename of a path-prefixed request (`JINST-Sample-files/JINST`, whose class is in a subdirectory: 1504.01965,
/// 1410.4420, 1805.09245; the raw load of a basename-keyed package binding resolves its directory the same way,
/// `input_definitions`).
pub fn requested_shipped_class(binding: &str) -> Result<Option<String>> {
  let is_binding = |name: &str| {
    let name = name.rsplit(['/', '\\']).next().unwrap_or_default();
    let name = strip_cls(name);
    name.eq_ignore_ascii_case(binding)
  };
  let fallback_request = lookup_string("fallback_request");
  let currname = Expand!(Tokens!(T_CS!("\\@currname"))).to_string();
  let class = if !fallback_request.is_empty() && is_binding(&lookup_string("fallback_target")) {
    strip_cls(&fallback_request)
  } else if is_binding(&currname) {
    strip_cls(&currname)
  } else {
    binding
  };
  Ok(
    find_file(
      class,
      Some(FindFileOptions {
        ext_type: Some(Cow::Borrowed("cls")),
        forbid_ltxml: true,
        ..Default::default()
      }),
    )
    .map(|_| class.to_string()),
  )
}

/// The text of the class file [`requested_shipped_class`] finds, its bytes read as Latin-1 characters (empty when the
/// file cannot be read); `None` when the source does not have it. A family binding reads the copy the paper ships to
/// follow its version where the versions conflict (`mn2e_cls`, `mdpi_cls`, `jfm_cls`).
pub fn shipped_class_text(binding: &str) -> Result<Option<String>> {
  Ok(requested_shipped_class(binding)?.map(|class| {
    find_file(
      &class,
      Some(FindFileOptions {
        ext_type: Some(Cow::Borrowed("cls")),
        forbid_ltxml: true,
        ..Default::default()
      }),
    )
    .and_then(|path| std::fs::read(path).ok())
    .map(|bytes| bytes.iter().map(|&b| b as char).collect::<String>())
    .unwrap_or_default()
  }))
}

/// Does `text` (a class file's) load `package` with `\RequirePackage`/`\usepackage` outside a comment, alone or in a
/// list (`\RequirePackage{graphicx,epstopdf}`)?
pub fn class_text_loads(text: &str, package: &str) -> bool {
  text.lines().any(|line| {
    let line = line.split('%').next().unwrap_or_default();
    ["\\RequirePackage", "\\usepackage"].iter().any(|command| {
      line.match_indices(command).any(|(at, _)| {
        let rest = &line[at + command.len()..];
        let rest = match rest.trim_start().strip_prefix('[') {
          Some(options) => options
            .split_once(']')
            .map(|(_, after)| after)
            .unwrap_or_default(),
          None => rest,
        };
        rest
          .trim_start()
          .strip_prefix('{')
          .and_then(|list| list.split_once('}'))
          .is_some_and(|(list, _)| list.split(',').any(|name| name.trim() == package))
      })
    })
  })
}

/// `name` without a `.cls` extension, in any case.
fn strip_cls(name: &str) -> &str {
  match name.len().checked_sub(4) {
    Some(cut) if name.is_char_boundary(cut) && name[cut..].eq_ignore_ascii_case(".cls") => {
      &name[..cut]
    },
    _ => name,
  }
}
