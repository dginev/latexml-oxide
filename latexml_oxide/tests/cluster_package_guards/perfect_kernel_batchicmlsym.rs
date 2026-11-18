//! Red/green guard for ICMLSYM: icml's `\icmlsetsymbol{name}{symbol}` defines only `\@icmlsymbol<name>`
//! (icml2026.sty:465-467, icml2024.sty:489-491), never `\<name>`; a symbol named after a command leaves the command
//! alone (OXIDIZED_DESIGN_DIVERGENCES #1076). Witnesses 2606.08980, 2310.06430.
use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchcr2::convert_clean};

fn row(items: &[&str]) -> Vec<String> { items.iter().map(|s| (*s).to_string()).collect() }

/// `\icmlsetsymbol{dagger}{$\dagger$}` (2606.08980): the label prints its `†` on Ann's name, a mark-only note of her
/// creator, and the body's `$x^\dagger$` is the math dagger. A `\dagger` defined as `$\dagger$` too called itself in
/// the label's trial digestion (6 GB, 5.6 s: the worker's memory limit made it a Fatal). Repro
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
  assert!(
    xml.contains(r#"<XMTok fontsize="70%" name="dagger" role="MULOP">†</XMTok>"#),
    "{xml}"
  );
}
