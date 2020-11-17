/*
   HOW DOES THIS ALL WORK?

   Warning: This is a prototype. We should definitely work on architectural updates to BDD lib
   which will make this much nicer. Also something something FUTURES! However, that being said...

   We consider the same encoding as in normal AsyncGraph, but we add extra BDD variables
   to represent the variables of the network (we call these state variables). These are added
   AFTER the parameter variables.

   This means that if there are no constraints on the state variables, our BDDs look exactly as
   normal `BddParams`, except their cardinality is higher (since the cardinality algorithm also
   counts state variable valuations). However, we can normalize this...
*/

mod _impl_graph_colors;

use biodivine_lib_bdd::Bdd;

/*
   BDDs representing the graph colors. These still contain both state and parameter variables,
   but are only constrained on parameter variables. We thus need a normalization factor to
   account for this.
*/
#[derive(Clone)]
pub struct GraphColors {
    bdd: Bdd,
    p_var_count: u16,
}

/*
   BDD representing the $V \times C$ relation (colored vertex set) of a graph. Essentially
   behaves like a relation/set.
*/
#[derive(Clone)]
pub struct GraphColoredVertices {
    bdd: Bdd,
    p_var_count: u16,
}
