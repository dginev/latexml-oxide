use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
  // TODO: Change to f64 to keep precision
  // raw_map!("emwidth"=>65536.1875, "exheight" => 28216.875),
  pub static ref STDMETRICS: HashMap<&'static str, HashMap<&'static str, f32>> = raw_map!("cmr" => raw_map!("emwidth"=>65536.19, "exheight" => 28216.875),
    "cmm"=>raw_map!("emwidth"=>65536.19));
}
