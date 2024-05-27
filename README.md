# Graph min cut approximation

This is a Rust implementation of the Karger's algorithm for finding the minimum cut of a graph.

The implementation takes a list of files as input and runs the algorithm on each of them. The output is the number of edges in the minimum cut. The input file should contain a single edge per line, with the two vertices separated by a space. The vertices are 0-indexed.

## How to run

```
cargo run --release -- -f $(ls tests | awk '{print "tests/" $0}' | paste -sd ",") -i 1000
```
