use std::cell::Cell;
use std::ops::Deref;

use biodivine_lib_bdd::Bdd;

use crate::biodivine_std::traits::Set;
use crate::symbolic_async_graph::{GraphColoredVertices, GraphColors, SymbolicAsyncGraph};

#[derive(Default, Clone)]
pub struct Stats {
    iterations: Cell<u64>,
    symbolic_steps: Cell<u64>,
    max_bdd_size: Cell<usize>,
    total_bdds: Cell<u64>,
    avg_bdd_size: Cell<f64>,
}

impl Stats {
    thread_local! {
        static STATS: Stats = Stats::default();
    }

    pub fn inc_iterations() {
        #[cfg(feature = "logging")]
        Self::STATS.with(|s| s.iterations.set(s.iterations.get() + 1));
    }

    pub fn print() {
        #[cfg(feature = "logging")]
        Self::STATS.with(|s| {
            println!("-----------------------------------------");
            println!("|                 STATS                 |");
            println!("-----------------------------------------");
            println!("| Iterations     | {:20} |", s.iterations.get());
            println!("-----------------------------------------");
            println!("| Symbolic steps | {:20} |", s.symbolic_steps.get());
            println!("-----------------------------------------");
            println!("| Max BDD size   | {:20} |", s.max_bdd_size.get());
            println!("-----------------------------------------");
            println!("| Avg BDD size   | {:20.2} |", s.avg_bdd_size.get());
            println!("-----------------------------------------");
        });
    }

    fn inc_symbolic_steps() {
        #[cfg(feature = "logging")]
        Self::STATS.with(|s| s.symbolic_steps.set(s.symbolic_steps.get() + 1));
    }

    fn set_bdd_size(_size: usize) {
        #[cfg(feature = "logging")]
        Self::STATS.with(|s| {
            s.max_bdd_size.set(s.max_bdd_size.get().max(_size));

            // Update the average bdd size
            let total_bdds = s.total_bdds.get();
            s.avg_bdd_size.set(
                (s.avg_bdd_size.get() * total_bdds as f64 + _size as f64)
                    / (total_bdds as f64 + 1f64),
            );
            s.total_bdds.set(total_bdds + 1);
        })
    }
}

#[derive(Clone)]
pub struct StatGraphColoredVertices {
    vertices: GraphColoredVertices,
}

impl StatGraphColoredVertices {
    pub fn new(vertices: GraphColoredVertices) -> Self {
        Self { vertices }
    }

    fn count_unary_op(&self) {
        Stats::inc_symbolic_steps();
        Stats::set_bdd_size(self.as_bdd().size());
    }

    fn count_binary_op(&self, other: &Bdd) {
        self.count_unary_op();
        Stats::set_bdd_size(other.size());
    }

    pub fn union(&self, other: &Self) -> Self {
        self.count_binary_op(other.as_bdd());
        Self::new(self.vertices.union(other))
    }

    pub fn intersect(&self, other: &Self) -> Self {
        self.count_binary_op(other.as_bdd());
        Self::new(self.vertices.intersect(other))
    }

    pub fn minus(&self, other: &Self) -> Self {
        self.count_binary_op(other.as_bdd());
        Self::new(self.vertices.minus(other))
    }

    pub fn minus_colors(&self, other: &GraphColors) -> Self {
        self.count_binary_op(other.as_bdd());
        Self::new(self.vertices.minus_colors(other))
    }

    pub fn pick_vertex(&self) -> Self {
        self.count_unary_op();
        Self::new(self.vertices.pick_vertex())
    }
}

impl Deref for StatGraphColoredVertices {
    type Target = GraphColoredVertices;

    fn deref(&self) -> &Self::Target {
        &self.vertices
    }
}

pub struct StatSymbolicAsyncGraph {
    graph: SymbolicAsyncGraph,
}

impl StatSymbolicAsyncGraph {
    pub fn new(graph: SymbolicAsyncGraph) -> Self {
        Self { graph }
    }

    pub fn mk_empty_vertices(&self) -> StatGraphColoredVertices {
        StatGraphColoredVertices::new(self.graph.mk_empty_vertices())
    }

    pub fn mk_unit_colored_vertices(&self) -> StatGraphColoredVertices {
        StatGraphColoredVertices::new(self.graph.mk_unit_colored_vertices())
    }

    pub fn pre(&self, initial: &StatGraphColoredVertices) -> StatGraphColoredVertices {
        initial.count_unary_op();
        StatGraphColoredVertices::new(self.graph.pre(initial))
    }

    pub fn post(&self, initial: &StatGraphColoredVertices) -> StatGraphColoredVertices {
        initial.count_unary_op();
        StatGraphColoredVertices::new(self.graph.post(initial))
    }
}

impl Deref for StatSymbolicAsyncGraph {
    type Target = SymbolicAsyncGraph;

    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}
