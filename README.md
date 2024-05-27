# How to run

```
cargo run --release -- -f $(ls tests | awk '{print "tests/" $0}' | paste -sd ",") -i 1000
```
