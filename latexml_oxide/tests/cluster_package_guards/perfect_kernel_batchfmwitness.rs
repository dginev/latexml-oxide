//! FMWITNESS: an author-segmentation witness bank. Each row is a minimal pdflatex-checked repro of one real arXiv shape
//! (`tools/perfect_kernel/repros/sectioning-frontmatter/fmw_*.tex`, witness ids in each header): how an author block
//! splits into creators and contacts — separators (`,`, `and`, `&`, `\and`, `\And`, `\AND`, `\\`, `\thanks`), marks
//! (`$^1$`, `\inst`, `\IEEEauthorrefmark`, `\footnotemark`), shared and per-author affiliations, e-mail lists, block
//! authors and department groups, over article, authblk, amsart, revtex4-2, aastex631, elsarticle, IEEEtran, llncs,
//! svjour3, iopart, acmart, neurips, icml and an unknown class (OmniBus). Every row is asserted whole: each creator's
//! role, `before=`, name, every contact (role and text) and every note (role, mark and text), in order; the text left
//! outside the creators; exact Error/Warning counts. The shapes rc6 gets wrong are `fmw_*_red.tex` repros, not rows.
use latexml::util::test::convert_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// One witness: its repro, its Warning count, its creators (as [`strict_creators`] reads them) and the text outside them.
struct Row {
  repro:    &'static str,
  warnings: usize,
  creators: &'static [&'static [&'static str]],
  rest:     &'static str,
}

/// `fragment`'s text, markup aside (each tag a space), its entities decoded and its whitespace collapsed.
fn text(fragment: &str) -> String {
  let mut out = String::new();
  let mut in_tag = false;
  for c in fragment.chars() {
    match c {
      '<' => in_tag = true,
      '>' => {
        in_tag = false;
        out.push(' ');
      },
      _ if !in_tag => out.push(c),
      _ => {},
    }
  }
  decode(&out)
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
}

/// `escaped` with XML's entities decoded.
fn decode(escaped: &str) -> String {
  escaped
    .replace("&lt;", "<")
    .replace("&gt;", ">")
    .replace("&quot;", "\"")
    .replace("&amp;", "&")
}

/// The value of attribute `name` in the open tag `tag`.
fn attribute(tag: &str, name: &str) -> Option<String> {
  let key = format!(" {name}=\"");
  let at = tag.find(&key)? + key.len();
  let value = &tag[at..at + tag[at..].find('"')?];
  Some(decode(value))
}

/// The element children of `inner` at its top level: (name, open tag, content).
fn children(inner: &str) -> Vec<(&str, &str, &str)> {
  let mut out = Vec::new();
  let mut pos = 0;
  while let Some(lt) = inner[pos..].find('<').map(|at| pos + at) {
    let open_end = lt + inner[lt..].find('>').expect("an open tag");
    let tag = &inner[lt..=open_end];
    if tag.starts_with("</") {
      break; // the parent's own close tag
    }
    if tag.starts_with("<!") || tag.starts_with("<?") {
      pos = open_end + 1;
      continue;
    }
    let name = tag[1..]
      .split(|c: char| c.is_whitespace() || c == '>' || c == '/')
      .next()
      .unwrap_or("");
    if tag.ends_with("/>") {
      out.push((name, tag, ""));
      pos = open_end + 1;
      continue;
    }
    // the matching close tag: count nested elements of the same name
    let (open, close) = (format!("<{name}"), format!("</{name}>"));
    let (mut depth, mut at) = (1, open_end + 1);
    while depth > 0 {
      let next_close = at + inner[at..].find(&close).expect("a closed element");
      let next_open = inner[at..next_close]
        .match_indices(&open)
        .map(|(i, _)| at + i)
        .find(|&i| {
          let after = inner[i + open.len()..].chars().next();
          matches!(after, Some(c) if c.is_whitespace() || c == '>')
            && !inner[i..].split('>').next().unwrap_or("").ends_with('/')
        });
      match next_open {
        Some(i) => {
          depth += 1;
          at = i + open.len();
        },
        None => {
          depth -= 1;
          at = next_close + close.len();
        },
      }
    }
    out.push((name, tag, &inner[open_end + 1..at - close.len()]));
    pos = at;
  }
  out
}

/// Each creator of `xml`, whole and in order: `@role`, `before=…` when present, the personname's text, then each
/// contact as `role=text` (an empty one by its `href`) and each note as `note:role[mark]=text`.
fn strict_creators(xml: &str) -> Vec<Vec<String>> {
  let document = &xml[xml.find("<document").expect("a document")..];
  let body = &document[document.find('>').expect("its open tag") + 1..];
  children(body)
    .into_iter()
    .filter(|(name, ..)| *name == "creator")
    .map(|(_, tag, inner)| {
      let mut out = vec![format!("@{}", attribute(tag, "role").unwrap_or_default())];
      if let Some(before) = attribute(tag, "before") {
        out.push(format!("before={before}"));
      }
      for (name, tag, content) in children(inner) {
        match name {
          "personname" => out.push(text(content)),
          "contact" => {
            let mut value = text(content);
            if value.is_empty() {
              value = attribute(content, "href").unwrap_or_default();
            }
            out.push(format!(
              "{}={value}",
              attribute(tag, "role").unwrap_or_default()
            ));
          },
          "note" => out.push(format!(
            "note:{}{}={}",
            attribute(tag, "role").unwrap_or_default(),
            attribute(tag, "mark").map_or(String::new(), |mark| format!("[{mark}]")),
            text(content)
          )),
          other => out.push(format!("<{other}>={}", text(content))),
        }
      }
      out
    })
    .collect()
}

/// The text of `xml`'s document outside its creators (a `<date>` and a section's `<tags>` aside): the title, abstract
/// and body, and any author-block word that leaked out of a creator.
fn rest(xml: &str) -> String {
  let mut out = xml[xml.find("<document").expect("a document")..].to_string();
  for name in ["creator", "date", "tags"] {
    let (open, close) = (format!("<{name} "), format!("</{name}>"));
    let open_bare = format!("<{name}>");
    while let Some(start) = out.find(&open).or_else(|| out.find(&open_bare)) {
      let end = start + out[start..].find(&close).expect("a closed element") + close.len();
      out.replace_range(start..end, " ");
    }
  }
  text(&out)
}

/// Every row's mismatches against its conversion (none when it matches).
fn mismatches(row: &Row) -> Vec<String> {
  let (log, xml) = convert_with(row.repro, Some("ar5iv.sty"));
  let name = row
    .repro
    .lines()
    .find(|l| l.starts_with("% shape:"))
    .unwrap_or("?");
  let mut out = Vec::new();
  let counts = (error_count(&log), warning_count(&log));
  if counts != (0, row.warnings) {
    out.push(format!(
      "{name}\n  E/W {counts:?}, want (0, {})\n{log}",
      row.warnings
    ));
  }
  let found = strict_creators(&xml);
  let want: Vec<Vec<String>> = row
    .creators
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect();
  if found != want {
    out.push(format!("{name}\n  creators {found:#?}\n  want {want:#?}"));
  }
  let left = rest(&xml);
  if left != row.rest {
    out.push(format!("{name}\n  rest {left:?}, want {:?}", row.rest));
  }
  out
}

/// The witness bank: every row converts with its exact counts, creators and outside text. A failure lists every
/// mismatching row, not only the first.
#[test]
fn author_segmentation_witnesses() {
  let failures: Vec<String> = ROWS.iter().flat_map(mismatches).collect();
  assert!(
    failures.is_empty(),
    "{} mismatches:\n{}",
    failures.len(),
    failures.join("\n\n")
  );
}

#[rustfmt::skip]
const ROWS: &[Row] = &[
  Row {
    repro: repro!("fmw_aas_shared_affiliations_by_position"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Nobeyama Radio Observatory, City A", "affiliation=The Graduate University for Advanced Studies, City B"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Nobeyama Radio Observatory, City A", "affiliation=The Graduate University for Advanced Studies, City B"],
      &["@author", "before=\u{2003}\u{2003}", "Cat Cole", "affiliation=Department of Physics, Univ C, City C"],
    ],
    rest: "T A. 1 Intro Text.",
  },
  Row {
    repro: repro!("fmw_acm_institution_city_country_email"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "email=ann@a.edu", "affiliation=Univ A , City A , State A , USA"],
      &["@author", "before= and ", "Bob Baker", "email=bob@b.edu", "affiliation=Univ B , City B , Italy"],
    ],
    rest: "T A. Text.",
  },
  Row {
    repro: repro!("fmw_acm_one_call_names_shared_email_list"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Department of Computer Science, Univ A , USA", "email=ann@a.edu"],
      &["@author", "before=, ", "Bob Baker", "note:footnotemark[*]=", "affiliation=Department of Computer Science, Univ A , USA", "email=bob@b.com"],
      &["@author", "before= and ", "Cat Cole", "affiliation=Department of Computer Science, Univ A , USA", "email=cat@a.edu"],
    ],
    rest: "T A. Text.",
  },
  Row {
    repro: repro!("fmw_ams_address_lines_with_name"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "address=Ann Able School of Mathematical Sciences Univ A City A", "email=ann@a.edu"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_ams_address_name_prefix"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "note:footnotemark[1]=", "address=Ann Able: Institute of Mathematics, Univ A, City A", "email=ann@a.edu"],
      &["@author", "before= and ", "Bob Baker", "note:footnotemark[2]=", "address=Bob Baker: Mathematical Science Center, Univ B, City B", "email=bob@b.edu"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_ams_address_optional_name"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "address=Department of Mathematical Sciences, Univ A, City A", "email=ann@a.edu"],
      &["@author", "before= and ", "Bob Baker", "address=Graduate School of Mathematics, Univ B, City B", "email=bob@b.edu"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_ams_and_in_one_call"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able"],
      &["@author", "before= and ", "Bob Baker"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_ams_author_address_email_each"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "address=Institute of Mathematics, Univ A, City A", "email=ann@a.edu"],
      &["@author", "before= and ", "Bob Baker", "address=Department of Mathematics, Univ B, City B", "email=bob@b.edu"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_ams_authors_then_addresses"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "address=Department of Mathematics, Univ A, City A", "email=aable@a.edu"],
      &["@author", "before= and ", "Bob Baker", "address=Department of Mathematics, Univ B, City B", "email=bbaker@b.edu"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_art_amp_names"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Univ X"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Univ X"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_art_and_column_group_alone"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Machine Learning Group"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Machine Learning Group"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_art_and_columns_lines_email"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Univ A", "email=ann@a.edu"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Univ B", "email=bob@b.edu"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_art_and_names_shared_affiliation_line"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Department of Mathematics, Univ X"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Department of Mathematics, Univ X"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_art_authblk_email_in_affil"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Univ A, City A", "email=ann@a.edu"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Univ B, City B", "email=bob@b.edu"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_art_authblk_labelled_affils"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Univ A, City A"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Univ B, City B"],
      &["@author", "before=\u{2003}\u{2003}", "Cat Cole", "affiliation=Univ A, City A", "affiliation=Univ B, City B"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_art_comma_company_group"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Alibaba Group"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_art_department_group_name_with_and"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Databases and Information Systems Group", "affiliation=Univ X"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Databases and Information Systems Group", "affiliation=Univ X"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_art_footnote_and_footnotemark_share"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "note:thanks[*]=Institute of Mathematics, Univ X."],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "note:thanks[*]=Institute of Mathematics, Univ X."],
      &["@author", "before=\u{2003}\u{2003}", "Cat Cole", "note:thanks[†]=Department of Physics, Univ Y."],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_art_group_own_line_then_address"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Theory Group", "affiliation=Department of Physics, Univ X"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Theory Group", "affiliation=Department of Physics, Univ X"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_art_group_then_address_next_line"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Department of Physics, Univ X"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Department of Physics, Univ X"],
      &["@author", "before=\u{2003}\u{2003}", "Cat Cole", "affiliation=Theory Group", "affiliation=Department of Physics, Univ X"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_art_marked_names_newline_affils_one_line"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Univ A"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Robotics Group"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_art_spacing_marked_names_affils_line"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Univ B"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Univ A"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_art_tabular_blocks"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Univ A", "email=ann@a.edu"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Univ B", "email=bob@b.edu"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_art_thanks_affil_url_email_each"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "note:thanks[*]=School of Mathematics, Univ X, City ( ann@x.edu )"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "note:thanks[†]=Graduate School of Engineering, Univ Y, Town ( bob@y.edu )"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_art_thanks_on_last_names_all"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker"],
      &["@author", "before=\u{2003}\u{2003}", "Cat Cole", "note:thanks[*]=A. Able and C. Cole are with Univ A (ann@a.edu, cat@a.edu). B. Baker is with Univ B."],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_els_affiliation_keyval"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Applied Science Group, Univ A, City A, Country A", "affiliation=Applied Numerical Algorithms Group, Lab B, City B, Country B", "correspondent=Corresponding author"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Applied Numerical Algorithms Group, Lab B, City B, Country B"],
    ],
    rest: "T A. Text.",
  },
  Row {
    repro: repro!("fmw_els_corref_ead_url_address"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "email=ann@a.edu", "url=http://a.edu/ann", "address=LMAH, Univ A, City A", "correspondent=Corresponding author"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "email=bob@a.edu", "address=LITIS, Univ A, City A"],
    ],
    rest: "T A. Text.",
  },
  Row {
    repro: repro!("fmw_els_labelled_addresses"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "address=Department of Physics, Univ A, City A", "address=Institute of Physics, Univ B, City B"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "address=Department of Physics, Univ A, City A"],
    ],
    rest: "T A. Text.",
  },
  Row {
    repro: repro!("fmw_icml_authorlist_affil_corresponding"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Department of Computer Science, Univ A, City A", "email=ann@a.edu"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Department of Computer Science, Univ A, City A", "affiliation=Lab B, City B"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_ieee_and_columns_blocks"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Machine Learning Department Univ A City A", "email=ann@a.edu"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Statistics Department Univ A City A", "email=bob@a.edu"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_ieee_blockN_spaced_marks_affil_blockN"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "note:footnotemark[♠]=", "affiliation=Univ B"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "note:footnotemark[♣]=", "affiliation=Univ A"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_ieee_membership_thanks_emails"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "membership=Member, IEEE", "affiliation=the Department of Engineering, Univ A, City A", "email=ann@a.edu"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "membership=Fellow, IEEE", "affiliation=the Department of Engineering, Univ A, City A", "email=bob@a.edu"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_ieee_one_block_names_shared_brace_email"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Centre for Security, Univ A, City A", "email=ann.able@a.edu"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Centre for Security, Univ A, City A", "email=bob.baker@a.edu"],
      &["@author", "before=\u{2003}\u{2003}", "Cat Cole", "affiliation=Centre for Security, Univ A, City A", "email=cat.cole@a.edu"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_ieee_plain_lines_one_author"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Computer Science Department", "affiliation=Univ A", "email=ann@a.edu"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_ieee_refmarks_blocks_email_list"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=* Computer Science Department Univ A, City A", "email=ann@a.edu"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=* Computer Science Department Univ A, City A", "email=bob@a.edu"],
      &["@author", "before=\u{2003}\u{2003}", "Cat Cole", "affiliation=† Lab B, City B", "email=cat@b.com"],
      &["@author", "before=\u{2003}\u{2003}", "Dan Dale", "affiliation=* Computer Science Department Univ A, City A", "email=dan@a.edu"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_iop_ead_with_name"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=1 Department of Physics, Univ A, City A", "email=ann@a.edu (Ann Able)"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=2 Area de Fisica, Univ B, City B", "email=bob@b.edu (Bob Baker)"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_iop_marked_addresses_ead"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=1 Institute of Physics, Univ A, City A", "affiliation=2 ICFO, City B", "email=ann.able@a.edu"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=2 ICFO, City B"],
      &["@author", "before=\u{2003}\u{2003}", "Cat Cole", "affiliation=1 Institute of Physics, Univ A, City A"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_lncs_amp_names_thanks"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Mathematical Institute (MI), Country", "email=ann@mi.ac.in"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "note:thanks[⋆]=Part of the work was done on a visit to Institute X", "affiliation=Mathematical Institute (MI), Country", "email=bob@mi.ac.in"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_lncs_inst_institutes_emails"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Department of Mathematics, Univ A, City A", "email=ann@a.edu"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Department of Engineering, Univ B, City B", "email=bob@b.edu"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_lncs_inst_list_shared_email_line"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Department of Computer Science, Univ A, City A", "affiliation=Department of Mathematics, Univ B, City B", "email=ann@a.edu"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Department of Mathematics, Univ B, City B", "email=bob@b.edu"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_lncs_institute_dollar_marks_one_line"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Department of Computer Science, Univ A, City A."],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Department of Computer Science, Univ A, City A."],
      &["@author", "before=\u{2003}\u{2003}", "Cat Cole", "affiliation=Department of Computer Science, Univ A, City A.", "affiliation=Department of Mathematics, Univ B, City B."],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_lncs_shared_institute_brace_email"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=State University, City, Country", "email=ann_able@su.edu"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=State University, City, Country", "email=bbaker@su.edu"],
      &["@author", "before=\u{2003}\u{2003}", "Cat Cole", "affiliation=State University, City, Country", "email=ccole@su.edu"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_neurips_AND_columns"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Lab A", "email=ann@a.edu"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Univ B", "email=bob@b.edu"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_neurips_And_lines_texttt_thanks"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Lab A", "email=ann@a.edu"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "note:thanks[*]=Work done as a visiting student at Lab A.", "affiliation=Univ B", "email=bob@b.edu"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_rev_altaffiliation_thanks"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "altaffiliation=Also at Univ C, City C", "thanks=Corresponding author", "affiliation=Univ A, City A"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Univ B, City B"],
    ],
    rest: "T A. Text.",
  },
  Row {
    repro: repro!("fmw_rev_author_and_in_one_call"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Department of Physics, Univ A, City A"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Department of Physics, Univ A, City A"],
    ],
    rest: "T A. Text.",
  },
  Row {
    repro: repro!("fmw_rev_consecutive_authors_share"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Department of Physics, Univ A, City A", "affiliation=Center for Quantum Physics, Univ A, City A"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Department of Physics, Univ A, City A", "affiliation=Center for Quantum Physics, Univ A, City A"],
      &["@author", "before=\u{2003}\u{2003}", "Cat Cole", "affiliation=Department of Physics, Univ A, City A"],
    ],
    rest: "T A. Text.",
  },
  Row {
    repro: repro!("fmw_rev_email_then_two_affiliations"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "email=ann@a.edu", "affiliation=Quantum Research Group, School of Physics, Univ A, City A"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Quantum Research Group, School of Physics, Univ A, City A", "affiliation=National Institute for Sciences (NIS), Country B"],
    ],
    rest: "T A. Text.",
  },
  Row {
    repro: repro!("fmw_svj_institute_at_per_author"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Univ A, City A", "email=ann@a.edu"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=Univ B, City B", "email=bob@b.edu"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_svj_institute_tel_fax"),
    warnings: 0,
    creators: &[
      &["@author", "Ann Able", "affiliation=Univ A and INFN, City A Tel.: +39-06-1234567 Fax: +39-06-7654321", "email=ann@a.edu"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "affiliation=INFN Sezione B, City B"],
    ],
    rest: "T Text.",
  },
  Row {
    repro: repro!("fmw_unknown_class_marked_names_affil_lines"),
    warnings: 1,
    creators: &[
      &["@author", "Ann Able", "note:footnotemark[1]=", "affiliation=Univ A, City A"],
      &["@author", "before=\u{2003}\u{2003}", "Bob Baker", "note:footnotemark[2]=", "affiliation=Univ B, City B"],
    ],
    rest: "T Text.",
  },
];
