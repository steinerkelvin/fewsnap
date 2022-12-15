# fewsnap-rs

Keep a few state snapshot layers with the folloing properties:

- `O(log(t))` number of snaphost layers
- `O(log(t))` read
- `O(1)` write
- `O(k)` to recompute history after dropping `k` ticks

where `t` is the time / number of ticks that have passed.
