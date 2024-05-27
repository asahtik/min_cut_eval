use clap::Parser;
use rand::{seq::SliceRandom, thread_rng};
use rayon::prelude::*;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    num::ParseIntError,
    path::PathBuf,
};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, value_delimiter = ',', num_args = 1..)]
    files: Vec<PathBuf>,
    #[arg(short, long)]
    iters: u32,
}

#[derive(Debug, Clone)]
struct Node {
    comps: Vec<usize>,
    edges: Vec<usize>,
}

fn read_input(input: &PathBuf) -> Result<(Vec<(usize, usize)>, usize), ParseIntError> {
    let mut edge_list = Vec::new();
    let file = File::open(input).expect("Failed to open input file");
    let mut max_idx = 0;
    for line in BufReader::new(file).lines() {
        let line = line.expect("Failed to read line");
        let parts = line.split_whitespace().collect::<Vec<_>>();
        assert_eq!(parts.len(), 2, "Expected two numbers per line");
        let edge = (parts[0].parse::<usize>()?, parts[1].parse::<usize>()?);
        edge_list.push(edge);
        max_idx = usize::max(max_idx, usize::max(edge.0, edge.1));
    }
    Ok((edge_list, max_idx + 1))
}

fn simple_cut(edges: &[(usize, usize)], n: usize) -> [Node; 2] {
    let mut nodes = vec![
        Node {
            comps: Vec::new(), // Contains nodes that were merged together
            edges: Vec::new()  // Contains edges connected to this node
        };
        n
    ];
    // Fill comps of each node with its index
    nodes.iter_mut().enumerate().for_each(|(i, node)| {
        node.comps = vec![i];
    });
    // Fill edges of each node with the index of the edges connected to it
    let mut edges = edges.to_vec();
    for (i, (u, v)) in edges.iter().enumerate() {
        nodes[*u].edges.push(i);
        nodes[*v].edges.push(i);
    }
    // Clear isolated nodes
    for node in &mut nodes {
        if node.edges.is_empty() {
            node.comps.clear();
        }
    }

    // Shuffle the edges to avoid having to use random at each iteration
    let mut shuffled = (0..edges.len()).collect::<Vec<usize>>();
    shuffled.shuffle(&mut thread_rng());
    // remaining[i] is true if the edge i is still in the graph
    let mut remaining = vec![true; edges.len()];

    // nodes vector also contains isolated nodes, we need to count only the nodes with edges
    let mut remaining_nodes = nodes.iter().filter(|n| !n.comps.is_empty()).count();
    let mut index = 0;
    // While there are more than 2 nodes
    while remaining_nodes > 2 {
        let edge_index = shuffled[index];
        index += 1;

        if !remaining[edge_index] {
            continue;
        }

        // Get "random" edge
        let (mut u, mut v) = edges[edge_index];

        // To optimize edge remapping we merge the node with less edges into the one with more edges
        if nodes[u].edges.len() < nodes[v].edges.len() {
            std::mem::swap(&mut u, &mut v);
        }
        assert!(u != v, "Self-loop detected");
        let comps_v = nodes[v].comps.clone();
        let edges_v = nodes[v].edges.clone();
        // Add the components and edges of v to u
        nodes[u].comps.extend(&comps_v);
        nodes[u].edges.extend(&edges_v);
        // Remove those edges from the "new" node that connect between the old two nodes (to avoid
        // self-loops)
        nodes[u].edges.retain(|&x| {
            !((edges[x].0 == u && edges[x].1 == v) || (edges[x].0 == v && edges[x].1 == u))
        });
        remaining_nodes -= 1;

        // Update the edges of v to point to u instead of v
        for &i in &edges_v {
            if !((edges[i].0 == u && edges[i].1 == v) || (edges[i].0 == v && edges[i].1 == u)) {
                // If the edge is not between the two nodes we are merging, we need to update it
                // (change v to u)
                if edges[i].0 == v {
                    edges[i].0 = u;
                } else {
                    edges[i].1 = u;
                }
                // This should never happen but I'm paranoid
                debug_assert!(edges[i].0 != edges[i].1, "Self-loop detected");
            } else {
                // This is the same condition as with nodes.edges.retain - here we mark the edge
                // between the two nodes as removed
                remaining[i] = false;
            }
        }
        // Clear the components and edges of v to save memory
        // This also marks the node as removed
        nodes[v].comps.clear();
        nodes[v].edges.clear();
    }

    // Remove all isolated (removed) nodes from list
    nodes.retain(|n| !n.comps.is_empty());

    // This should always be true
    assert_eq!(nodes.len(), 2);

    [nodes[0].clone(), nodes[1].clone()]
}

fn get_cut_size(cut: &[Node; 2], edges: &[(usize, usize)], n: usize) -> usize {
    // Each node gets assigned 1 if it is in the first partition or 2 if it is in the second
    let mut partition = vec![0; n];
    for (i, node) in cut.iter().enumerate() {
        for &comp in &node.comps {
            // No node should be in both partitions - this should always be true
            debug_assert_eq!(partition[comp], 0, "Partition is not a partition");
            partition[comp] = i + 1;
        }
    }

    let mut cut_size = 0;
    for (u, v) in edges {
        // Increment cut size if the nodes are in different partitions
        if partition[*u] != partition[*v] {
            cut_size += 1;
        }
    }

    cut_size
}

fn main() {
    let args = Args::parse();
    println!("|            name |          (n, m) |       opt | avg. runs |");
    println!("|-----------------|-----------------|-----------|-----------|");
    for input in &args.files {
        let (edges, n) = read_input(input).expect("Failed to read input file");

        // Get cut size for each iteration
        let cuts = (0..args.iters as usize)
            .into_par_iter()
            .map(|_| {
                let cut = simple_cut(&edges, n);
                get_cut_size(&cut, &edges, n)
            })
            .collect::<Vec<usize>>();

        // Get minimum
        let min_cut_size = *cuts.iter().min().unwrap();

        // Simulate sequential runs to find out how many iterations it takes from the last min cut
        // to the next one
        let mut runs = Vec::with_capacity(args.iters as usize);
        let mut last_run = 0;
        for (i, cut) in cuts.iter().enumerate() {
            if *cut == min_cut_size {
                runs.push(i + 1 - last_run);
                last_run = i + 1;
            }
        }
        let avg_minimum = runs.iter().sum::<usize>() as f64 / runs.len() as f64;

        println!(
            "|{:>16} | {:>15} |{:10} |{:10.2} |",
            input.file_name().unwrap().to_str().unwrap(),
            format!("({},{})", n, edges.len()),
            min_cut_size,
            avg_minimum
        );
    }
}
