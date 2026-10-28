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
  let owners = row_address_owners(names, &addresses);
  let tokens = if addresses.len() == 1 {
    vec![line.clone()]
  } else {
    address_tokens(line, addresses)?
  };
  Ok(Some(tokens.into_iter().zip(owners).collect()))
}

/// The place among a row's `names` of the owner of each of `addresses`, the e-mail addresses printed under the row
/// ([`row_email_owners`]): the name it spells; else, in a list of one address per name that every spelled address
/// agrees with, the name at its place; else `None`, the row's names' (FIDELITY_CHARTER §4). An address several names
/// spell agrees with the list only when the name at its place is one of them: `lee@x.edu` at Cid Cole's place under
/// "Ann Lee, Bob Lee and Cid Cole" spells another author than the one there, so the list is in no author order (the
/// 85b review's r01; [`addresses_in_order`] reads such an address as none's). A list of local parts the reader could
/// not part (`{zz|yy}@x.edu`, [`lists_local_parts`]) is several authors' address, so no list holding one is one address
/// per name (user ruling R10: the block's).
pub(in crate::base_utilities) fn row_address_owners(
  names: &[Tokens],
  addresses: &[String],
) -> Vec<Option<usize>> {
  let keys: Vec<NameKeys> = names.iter().map(NameKeys::of).collect();
  let in_order = addresses.len() == names.len()
    && addresses.iter().enumerate().all(|(at, address)| {
      let spelled = spelled_owners(&keys, address);
      !lists_local_parts(address) && (spelled.is_empty() || spelled.contains(&at))
    });
  addresses
    .iter()
    .enumerate()
    .map(|(at, address)| spelled_owner(&keys, address).or(in_order.then_some(at)))
    .collect()
}

/// The owner of each of `addresses` among a row's `names` by its spelling alone ([`spelled_owner`]), when the
/// addresses printed under the row are not all read, so no list of them is known to be one per name.
pub(in crate::base_utilities) fn row_address_spelled_owners(
  names: &[Tokens],
  addresses: &[String],
) -> Vec<Option<usize>> {
  let keys: Vec<NameKeys> = names.iter().map(NameKeys::of).collect();
  addresses
    .iter()
    .map(|address| spelled_owner(&keys, address))
    .collect()
}
