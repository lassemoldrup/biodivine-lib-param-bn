use std::fs::read_to_string;
use biodivine_lib_param_bn::{BooleanNetwork, symbolic_async_graph::SymbolicAsyncGraph};
use std::convert::TryFrom;

fn main() {
    let filename = "aeon_models/g2a_instantiated.aeon";
    let aeon_string = read_to_string(filename).unwrap();
    let network = BooleanNetwork::try_from(aeon_string.as_str()).unwrap();
    let symbolic_graph = SymbolicAsyncGraph::new(network).unwrap();
}
