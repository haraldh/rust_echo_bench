# rust_echo_bench
A simple rust echo server benchmark.

```
$ cargo run --release -- --help
   Compiling echo_bench v0.1.0 (…)
    Finished release [optimized] target(s) in 1.40 secs
     Running `target/release/echo_bench --help`
Echo benchmark.

Usage:
  echo_bench [ -a <address> ] [ -l <lenght> ] [ -c <number> ] [ -t <duration> ]
  echo_bench (-h | --help)
  echo_bench --version

Options:
  -h, --help                 Show this screen.
  -a, --address <address>    Target echo server address.
  -l, --lenght <lenght>      Test message length.
  -t, --duration <duration>  Test duration in seconds.
  -c, --number <number>      Test connection number.
```

Run it against a server:
```
$ cargo run --release -- --address "127.0.0.1:12345" --number 1000 --duration 60 --length 512
    Finished release [optimized] target(s) in 0.0 secs
     Running `target/release/echo_bench --address 127.0.0.1:12345 --number 1000 --duration 60 --length 512`
Benchmarking: 127.0.0.1:12345
1000 clients, running 512 bytes, 60 sec.

Speed: 670864 request/sec, 670864 response/sec
Requests: 40251881
Responses: 40251872
```

A simple rust echo server can be found at: https://github.com/haraldh/rust_echo_server

This benchmark can also be used to run it against various other echo servers, e.g. those found at https://gist.github.com/idada/9342414
