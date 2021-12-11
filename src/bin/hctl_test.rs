use biodivine_lib_param_bn::{symbolic_async_graph::SymbolicAsyncGraph, BooleanNetwork};
use std::convert::TryFrom;
use std::fs::read_to_string;

fn main() {
    let filename = "aeon_models/g2a_instantiated.aeon";
    let aeon_string = read_to_string(filename).unwrap();
    let network = BooleanNetwork::try_from(aeon_string.as_str()).unwrap();
    let symbolic_graph = SymbolicAsyncGraph::new(network).unwrap();
}
