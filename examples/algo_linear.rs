#![allow(dead_code)] // avoid warnings related to the "loggin" feature

use std::convert::TryFrom;
use std::io::Read;
use std::time::Instant;

use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::decomposition::Counter;
use biodivine_lib_param_bn::stats::*;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::BooleanNetwork;

struct ColoredSpineSet {
    spine: StatGraphColoredVertices,
    pivot: StatGraphColoredVertices,
}

fn main() {
    let model_load_start = Instant::now();
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer).unwrap();

    let model = BooleanNetwork::try_from(buffer.as_str()).unwrap();
    println!("Model vars: {}", model.as_graph().num_vars());

    let graph = SymbolicAsyncGraph::new(model).unwrap();
    let graph = StatSymbolicAsyncGraph::new(graph);
    println!(
        "Graph size: {} (Colors {})",
        graph.unit_colored_vertices().approx_cardinality(),
        graph.unit_colors().approx_cardinality()
    );
    let model_load_elapsed = model_load_start.elapsed();

    let algo_start = Instant::now();
    let count = find_sccs(&graph);
    let algo_elapsed = algo_start.elapsed();

    #[cfg(feature = "logging")]
    Stats::print();
    println!("Counted: {}", count);
    println!(
        "Loading model time: {}ms; Algorithm running time: {}ms",
        model_load_elapsed.as_millis(),
        algo_elapsed.as_millis()
    );
}

fn find_sccs(graph: &StatSymbolicAsyncGraph) -> usize {
    let mut counter = Counter::new(graph);
    let vertices = graph.mk_unit_colored_vertices();

    let spine_set = ColoredSpineSet {
        spine: graph.mk_empty_vertices(),
        pivot: graph.mk_empty_vertices(),
    };

    let mut universes = vec![(vertices, spine_set)];

    let _start = Instant::now();
    let mut _trimming = 0;
    let mut _reach = 0;
    while let Some((universe, spine_set)) = universes.pop() {
        #[cfg(feature = "logging")]
        {
            let remaining: f64 = universes.iter().map(|u| u.0.approx_cardinality()).sum();
            println!(
                "Universes: {}; SCCs: {}; Remaining: {}/{}",
                universes.len(),
                counter.len(),
                remaining + universe.approx_cardinality(),
                graph.unit_colored_vertices().approx_cardinality()
            );
            println!(
                "Elapsed: {}; Trim: {}; Reach: {};",
                _start.elapsed().as_millis(),
                _trimming,
                _reach
            );
        }

        let _start_trim = Instant::now();

        let universe = trim(graph, universe);
        if universe.is_empty() {
            #[cfg(feature = "logging")]
            println!("NO SCC");
            continue;
        }

        let spine = spine_set.spine.intersect(&universe);
        let mut pivot = spine_set.pivot.intersect(&universe);
        let diff = spine_set.spine.minus(&spine).minus_colors(&pivot.colors());
        pivot = pivot.union(&graph.pre(&diff).intersect(&spine));

        _trimming += _start_trim.elapsed().as_millis();

        let not_in_skel_pivot = universe.minus_colors(&pivot.colors());
        pivot = pivot.union(&not_in_skel_pivot.pick_vertex());

        let _start_reach = Instant::now();

        let (fw, fw_spine_set) = skel_forward(graph, &universe, &pivot);

        let mut sccs = graph.mk_empty_vertices();
        let mut layer = pivot.clone();
        let mut _i = 0;
        while !layer.is_empty() {
            _i += 1;
            sccs = sccs.union(&layer);

            #[cfg(feature = "logging")]
            if layer.as_bdd().size() > 100_000 {
                println!(
                    "SCCS: {} ({}); L: {} ({}); Iters: {}",
                    sccs.approx_cardinality(),
                    sccs.as_bdd().size(),
                    layer.approx_cardinality(),
                    layer.as_bdd().size(),
                    _i
                );
            }

            layer = graph.pre(&layer).intersect(&fw).minus(&sccs);
        }

        _reach += _start_reach.elapsed().as_millis();

        let non_pivot_states = &sccs.minus(&pivot);
        let non_trivial_colors = non_pivot_states.colors();
        #[cfg(feature = "logging")]
        println!(
            "SCC: {} ({} vertices)",
            sccs.approx_cardinality(),
            sccs.vertices().approx_cardinality()
        );
        if !non_trivial_colors.is_empty() {
            counter.push(&non_trivial_colors);
        } else {
            #[cfg(feature = "logging")]
            println!("TRIVIAL.");
        }

        // first recursive call
        let new_universe = universe.minus(&fw);
        let _universe1_size = new_universe.approx_cardinality();
        let new_spine = spine.minus(&sccs);
        let new_pivot = graph.pre(&sccs.intersect(&spine)).intersect(&new_spine);
        let new_spine_set = ColoredSpineSet {
            spine: new_spine,
            pivot: new_pivot,
        };
        if !new_universe.is_empty() {
            universes.push((new_universe, new_spine_set));
        }

        // second recursive call
        let new_universe = fw.minus(&sccs);
        let _universe2_size = new_universe.approx_cardinality();
        let new_spine_set = ColoredSpineSet {
            spine: fw_spine_set.spine.minus(&sccs),
            pivot: fw_spine_set.pivot.minus(&sccs),
        };
        if !new_universe.is_empty() {
            universes.push((new_universe, new_spine_set));
        }

        #[cfg(feature = "logging")]
        println!("SPLIT: {} - {}", _universe1_size, _universe2_size);

        #[cfg(feature = "logging")]
        Stats::inc_iterations();
    }

    #[cfg(feature = "logging")]
    counter.print();
    counter.len()
}

fn skel_forward(
    graph: &StatSymbolicAsyncGraph,
    universe: &StatGraphColoredVertices,
    pivot: &StatGraphColoredVertices,
) -> (StatGraphColoredVertices, ColoredSpineSet) {
    let mut fw = graph.mk_empty_vertices();
    let mut stack = vec![];
    let mut layer = pivot.clone();
    while !layer.is_empty() {
        fw = fw.union(&layer);
        stack.push(layer);

        let last_layer = stack.last().unwrap();

        #[cfg(feature = "logging")]
        if last_layer.as_bdd().size() > 100_000 {
            println!(
                "FW: {} ({}); L: {} ({}); stack: {}",
                fw.approx_cardinality(),
                fw.as_bdd().size(),
                last_layer.approx_cardinality(),
                last_layer.as_bdd().size(),
                stack.len()
            );
        }

        layer = graph.post(last_layer).intersect(&universe).minus(&fw);
    }

    let mut new_spine = graph.mk_empty_vertices();
    let mut new_pivot = graph.mk_empty_vertices();
    while let Some(layer) = stack.pop() {
        let not_in_new_pivot = layer.minus_colors(&new_pivot.colors());
        new_pivot = new_pivot.union(&not_in_new_pivot.pick_vertex()); // todo: look at pick vertex

        let pre_spine = graph.pre(&new_spine).intersect(&layer);
        new_spine = new_spine.union(&new_pivot).union(&pre_spine.pick_vertex());

        #[cfg(feature = "logging")]
        if new_spine.as_bdd().size() > 100_000 {
            println!(
                "S': {} ({}); stack: {}",
                new_spine.approx_cardinality(),
                new_spine.as_bdd().size(),
                stack.len() + 1
            );
        }
    }

    let new_spine_set = ColoredSpineSet {
        spine: new_spine,
        pivot: new_pivot,
    };
    (fw, new_spine_set)
}

// Equivalent to the trim function in algo_lockstep.rs
fn trim(
    graph: &StatSymbolicAsyncGraph,
    mut set: StatGraphColoredVertices,
) -> StatGraphColoredVertices {
    //let initial = set.as_bdd().size();
    //println!("Start trim: {}", initial);
    loop {
        // Predecessors of set inside set
        let pre = graph.pre(&set).intersect(&set);
        // Successors of these predecessors inside set.
        let post = graph.post(&pre).intersect(&set);
        // Everything in set \ post has no predecessor in this set.
        if set.is_subset(&post) {
            // set == post
            break;
        }
        #[cfg(feature = "logging")]
        if set.as_bdd().size() > 10_000 {
            println!(
                "TRIM: {}; {}",
                set.as_bdd().size(),
                set.approx_cardinality()
            );
        }
        set = post;
        //if set.as_bdd().size() > 2 * initial {
        //    return set;
        //}
    }
    loop {
        // Successors of set inside set
        let post = graph.post(&set).intersect(&set);
        // Predecessors of these successors inside set.
        let pre = graph.pre(&post).intersect(&set);
        // Everything in set \ pre has no successor in set.
        if set.is_subset(&pre) {
            // set == pre
            break;
        }
        #[cfg(feature = "logging")]
        if set.as_bdd().size() > 10_000 {
            println!(
                "TRIM: {}; {}",
                set.as_bdd().size(),
                set.approx_cardinality()
            );
        }
        set = pre;
        //if set.as_bdd().size() > 2 * initial {
        //    return set;
        //}
    }

    set
}
