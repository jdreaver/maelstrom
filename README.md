# Maelstrom

Doing this challenge <https://fly.io/dist-sys/> based on <https://github.com/jepsen-io/maelstrom>

## Installing maelstrom

<https://github.com/jepsen-io/maelstrom/blob/main/doc/01-getting-ready/index.md>

```
$ wget https://github.com/jepsen-io/maelstrom/releases/download/v0.2.3/maelstrom.tar.bz2
$ tar xvf maelstrom.tar.bz2
$ rm maelstrom.tar.bz2
```

## Running

```
$ cargo build
$ ./maelstrom/maelstrom test -w echo --bin target/debug/main --node-count 1 --time-limit 10
```
