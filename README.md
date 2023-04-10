# onetimer
Simple one-time-link service written in Rust

## Dependencies
* config: "0.13.3"
* rand: "0.8.5"
* sqlite: "0.30.4"

## Quick start

### Build and run the service:
```bash
$ cargo build --release
...
$ ./target/release/onetimer
```

### Send your secret data:
```bash
$ curl -d 'my secret data' http://127.0.0.1:8080/add
http://127.0.0.1:8080/get/3cfd3cd9b4913bbc571435314a63d011d2a51a8c9790c4dbbb7331932719d93e
```

### Get secret data using one-time link:
```bash
$ curl http://127.0.0.1:8080/get/3cfd3cd9b4913bbc571435314a63d011d2a51a8c9790c4dbbb7331932719d93e
my secret data
```

### Try to get secret data one more time:
```bash
$ curl -v http://127.0.0.1:8080/get/3cfd3cd9b4913bbc571435314a63d011d2a51a8c9790c4dbbb7331932719d93e
*   Trying 127.0.0.1:8080...
* TCP_NODELAY set
* Connected to 127.0.0.1 (127.0.0.1) port 8080 (#0)
> GET /get/3cfd3cd9b4913bbc571435314a63d011d2a51a8c9790c4dbbb7331932719d93e HTTP/1.1
> Host: 127.0.0.1:8080
> User-Agent: curl/7.68.0
> Accept: */*
>
* Mark bundle as not supporting multiuse
< HTTP/1.1 404 Not Found
< Content-Type: text/plain
< Content-Length: 0
<
* Connection #0 to host 127.0.0.1 left intact
```
