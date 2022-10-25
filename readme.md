# Sponk

High performance http server based on `io_uring` for http client performance testing.  
It only response `Hello world\n` to maximize its performance.  
**Note: Linux 5.10+ only!**  
**Note: This project use unsafe!**  
**Note: Currently broken with go's http client keep-alive**

<small>Sponk let you squeeze all performance from your machine</small>

# Goal

+ [x] HTTP 1 (1.1)
+ [ ] HTTP 2

# Performance

> About 200k req/s as http 1  
> on `Intel(R) Core(TM) i5-1135G7`

```shell
# Run sponk in 1 thread with default configuration
$ cargo run --package sponk --bin sponk --release -- -t 1 &
$ oha http://localhost:8080 -z 10s &
$ fg & <ctrl+c>
Summary:
  Success rate: 1.0000
  Total:        10.0004 secs
  Slowest:      0.0166 secs
  Fastest:      0.0000 secs
  Average:      0.0000 secs
  Requests/sec: 210773.2689

  Total data:   26.13 MiB
  Size/request: 13 B
  Size/sec:     2.61 MiB

Response time histogram:
  0.000 [42010]  |■
  0.000 [879022] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  0.000 [736322] |■■■■■■■■■■■■■■■■■■■■■■■■■■
  0.000 [266469] |■■■■■■■■■
  0.000 [104300] |■■■
  0.000 [49584]  |■
  ...

Latency distribution:
  ...
  95% in 0.0000 secs
  99% in 0.0001 secs

...
Status code distribution:
  [200] 2107814 responses
```

> Thanks to [Oha](https://github.com/hatoo/oha) for http load generator

# Running

```shell
cargo run --package sponk --bin sponk --release
```

## Not planned (But you can request for it)

+ Support another os (generic implementation)
+ Custom response
+ Testing :)
