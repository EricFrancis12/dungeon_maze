extern crate proc_macro;

mod world_structures;

use crate::world_structures::parse_world_structures;
use proc_macro::TokenStream;

#[proc_macro]
pub fn proc_parse_world_structures(t: TokenStream) -> TokenStream {
    parse_world_structures(t)
}
