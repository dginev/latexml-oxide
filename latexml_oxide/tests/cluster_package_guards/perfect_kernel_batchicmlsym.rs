//! Red/green guard for ICMLSYM: icml's `\icmlsetsymbol{name}{symbol}` defines only `\@icmlsymbol<name>`
//! (icml2026.sty:465-467, icml2024.sty:489-491), never `\<name>`; a symbol named after a command leaves the command
//! alone (OXIDIZED_DESIGN_DIVERGENCES #1076). Witnesses 2606.08980, 2310.06430.
use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchcr2::convert_clean};

fn row(items: &[&str]) -> Vec<String> { items.iter().map(|s| (*s).to_string()).collect() }

/// `\icmlsetsymbol{dagger}{$\dagger$}` (2606.08980): the label prints its `†` on Ann's name, a mark-only note of her
/// creator, and the body's `$x^\dagger$` is the math dagger. A `\dagger` defined as `$\dagger$` too called itself in
/// the label's trial digestion (6.2 GB, 11.8 s on 2606.08980, 6 cores, `ulimit -v 8912896`: the worker's memory limit
/// made it a Fatal). Repro
/// sectioning-frontmatter/icml_symbol_named_after_a_command_keeps_the_command.
#[test]
fn an_icml_symbol_named_after_a_command_keeps_the_command() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/icml_symbol_named_after_a_command_keeps_the_command.tex"
  ));
  assert_eq!(creators(&xml), [row(&[
    "Ann Author",
    "mark=†",
    "affiliation=Some University",
    "email=ann@uni.edu"
  ])]);
  let body = xml.split("</creator>").last().expect("a creator");
  assert!(
    body.contains(r#"<XMTok fontsize="70%" name="dagger" role="MULOP">†</XMTok>"#),
    "{xml}"
  );
}

/// The command named `dagger` keeps its meaning inside the `\twocolumn[…]` title group after
/// `\icmlsetsymbol{dagger}{$\dagger$}` (a state probe, REVICMLSYM): the old binding's local `\def\dagger` made the
/// body print "redefined", fast, where the guard above only paid its cost. Repro
/// sectioning-frontmatter/icml_symbol_leaves_the_command_alone.
#[test]
fn an_icml_symbol_leaves_the_command_alone() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/icml_symbol_leaves_the_command_alone.tex"
  ));
  let body = xml.split("</creator>").last().expect("a creator");
  assert!(body.contains("The dagger is kept."), "{xml}");
  assert!(!xml.contains("redefined"), "{xml}");
}
