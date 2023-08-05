# Readme

Rust version of [bustools](https://github.com/BUStools/bustools) command line interface.
At this point, it's **far from complete and correct**, but rather a project to learn rust.

This is heavly built on [rustbustools](https://github.com/redst4r/rustbustools), which handles all the basic interactions with busfiles.
## Example
```sh
# sorting
rustbustools --output /tmp/sorted.bus sort --ifile /tmp/unsorted.bus

# correcting CBs
rustbustools --output /tmp/corrected.bus sort --ifile /tmp/sorted.bus --whitelist /tmp/10x_whitelist.txt

# inspecting
rustbustools --output /dev/null --ifile /tmp/sorted.bus

# count
rustbustools --output /tmp/count_folder --ifile /tmp/sorted.bus --t2g /tmp/transcripts_to_gene.txt

# compression
# -N is the chunksize of a busz block
rustbustools --output /tmp/compressed.busz -i /tmp/sorted.bus -N 10000

# decompression
rustbustools --output /tmp/plain.bus -i /tmp/compressed.busz
```

## Todo 08/05

- [ ] performance checks
  - [ ] sort: seems to be slow
  - [ ] correct: some performance issues due to BKTree
  - [x] count: slightly slower than original bustools, but its in the ballpark
  - [x] inspect: pretty quick already
  - [x] butterfly amplfication: pretty quick already
- [x] handle compressed busfiles
- [ ] make CLI args compatible/consistent with original bustools
