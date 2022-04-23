use std::cell::Cell;
use std::ops::Deref;

use crate::biodivine_std::traits::Set;
use crate::symbolic_async_graph::{GraphColoredVertices, GraphColors, SymbolicAsyncGraph};

#[allow(dead_code)]
#[derive(Default, Clone)]
pub struct Stats {
    iterations: Cell<u64>,
    union_count: Cell<u64>,
    intersect_count: Cell<u64>,
    minus_count: Cell<u64>,
    pick_vertex_count: Cell<u64>,
    post_count: Cell<u64>,
    pre_count: Cell<u64>,
    total_symbolic_steps: Cell<u64>,
    trim_mode: Cell<bool>,
    trim_intersect_count: Cell<u64>,
    trim_post_count: Cell<u64>,
    trim_pre_count: Cell<u64>,
    max_bdd_size: Cell<usize>,
    total_bdds: Cell<u64>,
    avg_bdd_size: Cell<f64>,
}

#[cfg(feature = "logging")]
impl Stats {
    thread_local! {
        static STATS: Stats = Stats::default();
    }

    pub fn inc_iterations() {
        Self::STATS.with(|s| s.iterations.set(s.iterations.get() + 1));
    }

    pub fn toggle_trim_mode() {
        Self::STATS.with(|s| s.trim_mode.set(!s.trim_mode.get()));
    }

    pub fn print() {
        Self::STATS.with(|s| {
            println!("------------------------------------------");
            println!("|               STATISTICS               |");
            println!("------------------------------------------");
            println!("| Iterations      | {:20} |", s.iterations.get());
            println!("-----------------------------------------");
            println!("| Max BDD size    | {:20} |", s.max_bdd_size.get());
            println!("-----------------------------------------");
            println!("| Avg BDD size    | {:20.2} |", s.avg_bdd_size.get());
            println!("------------------------------------------");
            println!("|             SYMBOLIC STEPS             |");
            println!("------------------------------------------");
            println!("| Union steps     | {:20} |", s.union_count.get());
            println!("------------------------------------------");
            println!("| Intersect steps | {:20} |", s.intersect_count.get());
            println!("------------------------------------------");
            println!("| Minus steps     | {:20} |", s.minus_count.get());
            println!("------------------------------------------");
            println!("| Pick steps      | {:20} |", s.pick_vertex_count.get());
            println!("------------------------------------------");
            println!("| Post steps      | {:20} |", s.post_count.get());
            println!("------------------------------------------");
            println!("| Pre steps       | {:20} |", s.pre_count.get());
            println!("------------------------------------------");
            println!("| Total steps     | {:20} |", s.total_symbolic_steps.get());
            println!("------------------------------------------");
            println!("|             TRIMMING STEPS             |");
            println!("------------------------------------------");
            println!("| Intersect steps | {:20} |", s.trim_intersect_count.get());
            println!("------------------------------------------");
            println!("| Post steps      | {:20} |", s.trim_post_count.get());
            println!("------------------------------------------");
            println!("| Pre steps       | {:20} |", s.trim_pre_count.get());
            println!("------------------------------------------");
            println!("| Total trimming  | {:20} |", s.total_trimming_steps());
            println!("------------------------------------------");
        });
    }

    fn inc_symbolic_steps() {
        Self::STATS.with(|s| s.total_symbolic_steps.set(s.total_symbolic_steps.get() + 1));
    }

    fn inc_union_count() {
        Self::STATS.with(|s| s.union_count.set(s.union_count.get() + 1));
    }

    fn inc_intersect_count() {
        Self::STATS.with(|s| {
            if s.trim_mode.get() {
                s.trim_intersect_count.set(s.trim_intersect_count.get() + 1)
            }
            s.intersect_count.set(s.intersect_count.get() + 1)
        });
    }

    fn inc_minus_count() {
        Self::STATS.with(|s| s.minus_count.set(s.minus_count.get() + 1));
    }

    fn inc_pick_vertex_count() {
        Self::STATS.with(|s| s.pick_vertex_count.set(s.pick_vertex_count.get() + 1));
    }

    fn inc_post_count() {
        Self::STATS.with(|s| {
            if s.trim_mode.get() {
                s.trim_post_count.set(s.trim_post_count.get() + 1)
            }
            s.post_count.set(s.post_count.get() + 1)
        });
    }

    fn inc_pre_count() {
        Self::STATS.with(|s| {
            if s.trim_mode.get() {
                s.trim_pre_count.set(s.trim_pre_count.get() + 1)
            }
            s.pre_count.set(s.pre_count.get() + 1)
        });
    }

    fn total_trimming_steps(&self) -> u64 {
        self.trim_intersect_count.get() + self.trim_post_count.get() + self.trim_pre_count.get()
    }

    fn update_bdd_size(_size: usize) {
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

macro_rules! ops {
    ($( fn $name:ident ( $inc:ident, other: $other_ty:ty ); )+) => {
        #[inline]
        $(pub fn $name(&self, other: $other_ty) -> Self {
            #[cfg(feature = "logging")]
            {
                self.count_binary_op(other.as_bdd());
                Stats::$inc();
            }
            Self::new(self.vertices.$name(other))
        })+
    };
}

#[derive(Clone)]
pub struct StatGraphColoredVertices {
    vertices: GraphColoredVertices,
}

impl StatGraphColoredVertices {
    pub fn new(vertices: GraphColoredVertices) -> Self {
        Self { vertices }
    }

    #[cfg(feature = "logging")]
    fn count_unary_op(&self) {
        Stats::inc_symbolic_steps();
        Stats::update_bdd_size(self.as_bdd().size());
    }

    #[cfg(feature = "logging")]
    fn count_binary_op(&self, other: &biodivine_lib_bdd::Bdd) {
        self.count_unary_op();
        Stats::update_bdd_size(other.size());
    }

    ops! {
        fn union(inc_union_count, other: &Self);
        fn intersect(inc_intersect_count, other: &Self);
        fn intersect_colors(inc_intersect_count, other: &GraphColors);
        fn minus(inc_minus_count, other: &Self);
        fn minus_colors(inc_minus_count, other: &GraphColors);
    }

    #[inline]
    pub fn pick_vertex(&self) -> Self {
        #[cfg(feature = "logging")]
        {
            self.count_unary_op();
            Stats::inc_pick_vertex_count();
        }
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
    #[inline]
    pub fn new(graph: SymbolicAsyncGraph) -> Self {
        Self { graph }
    }

    #[inline]
    pub fn mk_empty_vertices(&self) -> StatGraphColoredVertices {
        StatGraphColoredVertices::new(self.graph.mk_empty_vertices())
    }

    #[inline]
    pub fn mk_unit_colored_vertices(&self) -> StatGraphColoredVertices {
        StatGraphColoredVertices::new(self.graph.mk_unit_colored_vertices())
    }

    #[inline]
    pub fn pre(&self, initial: &StatGraphColoredVertices) -> StatGraphColoredVertices {
        #[cfg(feature = "logging")]
        {
            initial.count_unary_op();
            Stats::inc_pre_count();
        }
        StatGraphColoredVertices::new(self.graph.pre(initial))
    }

    #[inline]
    pub fn post(&self, initial: &StatGraphColoredVertices) -> StatGraphColoredVertices {
        #[cfg(feature = "logging")]
        {
            initial.count_unary_op();
            Stats::inc_post_count();
        }
        StatGraphColoredVertices::new(self.graph.post(initial))
    }
}

impl Deref for StatSymbolicAsyncGraph {
    type Target = SymbolicAsyncGraph;

    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}
