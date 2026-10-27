//! A class binding's e-mail line under a row of names: the owner of each address among those names.

use super::*;

/// Each address of an e-mail line with the place of its owner among the row's names (`None`: the row's names share it).
pub type AddressOwners = Vec<(Tokens, Option<usize>)>;

/// The addresses of the e-mail `line` a class prints under a row of `names` (eptcs's `\email` under the names of one
/// `\and` cell, 2109.08295's `\{grubenmann, jens.lehmann\}@cs.uni-bonn.de` under "Tobias Grubenmann and Jens Lehmann"),
/// each with the place among `names` it goes to (FIDELITY_CHARTER §4): the name it spells; else, in a list of one
/// address per name that every spelled address agrees with, the name at its place (1101.4424's `g.stavrinos@… \qquad
/// tassiana98@…` under "Yiorgos Stavrinos \qquad Anastasia Veneti"); else none, an address the row's names share.
/// `None` when the line is no list of addresses. A line of one address is the line as written.
pub fn row_email_owners(names: &[Tokens], line: &Tokens) -> Result<Option<AddressOwners>> {
  let Some(addresses) = email_addresses(line) else {
    return Ok(None);
  };
  let keys: Vec<NameKeys> = names.iter().map(NameKeys::of).collect();
  let in_order = addresses.len() == names.len() && addresses_in_order(&keys, &addresses, 0);
  let owners: Vec<Option<usize>> = addresses
    .iter()
    .enumerate()
    .map(|(at, address)| spelled_owner(&keys, address).or(in_order.then_some(at)))
    .collect();
  let tokens = if addresses.len() == 1 {
    vec![line.clone()]
  } else {
    address_tokens(line, addresses)?
  };
  Ok(Some(tokens.into_iter().zip(owners).collect()))
}
