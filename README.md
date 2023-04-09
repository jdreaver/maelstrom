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

# Echo
$ ./maelstrom/maelstrom test -w echo --bin target/debug/main --node-count 1 --time-limit 10

# Unique IDs
$ ./maelstrom/maelstrom test -w unique-ids --bin target/debug/main --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition

# Broadcast
$ ./maelstrom/maelstrom test -w broadcast --bin target/debug/main --node-count 1 --time-limit 20 --rate 10
$ ./maelstrom/maelstrom test -w broadcast --bin target/debug/main --node-count 5 --time-limit 20 --rate 10
```

Start maelstrom server for debugging:

```
$ ./maelstrom/maelstrom serve

# Then go go localhost:8080 in a web browser
```
