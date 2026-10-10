//! Shared helper: the shipped class file a class-family binding reads raw (`compositio_cls`, `jinst_cls`).
use latexml_package::prelude::*;

/// The class file in the paper's source that the family binding `binding` interprets: the class the document asked for
/// when the loader answered it with this binding, else the binding's own name; `None` when the source does not have it.
/// The request is the fallback's (`fallback_request`, when its `fallback_target` is this binding: a renamed `JINST_mod`
/// or `myJINST`, 1307.5525, 1407.3938), or the load's own `\@currname` when the dispatch matched the binding by the
/// basename of a path-prefixed request (`JINST-Sample-files/JINST`, whose class is in a subdirectory: 1504.01965,
/// 1410.4420, 1805.09245; the raw load of a basename-keyed package binding resolves its directory the same way,
/// `input_definitions`).
pub(crate) fn requested_shipped_class(binding: &str) -> Result<Option<String>> {
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

/// `name` without a `.cls` extension, in any case.
fn strip_cls(name: &str) -> &str {
  match name.len().checked_sub(4) {
    Some(cut) if name.is_char_boundary(cut) && name[cut..].eq_ignore_ascii_case(".cls") => {
      &name[..cut]
    },
    _ => name,
  }
}
