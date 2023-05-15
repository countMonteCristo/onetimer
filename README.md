# onetimer
Simple one-time-link service written in Rust.

## Description
The service is availiable to provide one-time access for any secret data. This can be useful for transfering sensitive information (for example, passwords) to users.

Suppose you've been hired a new employee and you need to grant him the access to your internal services, i.e. you have to give him his login and password. The way you could do it is to upload this data (login+password) to `onetimer` and send the link given to your employee. He can follow this link only once, so you can be sure that noone will get this sensitive information.

## Internal stucture
`onetimer` itself is a simple HTTP web server with database. It accepts only two methods: **add** for adding new data and **get** for providing data to user. For now only SQLite database is supported.

## Dependencies
* config: "0.13.3"
* log: "0.4.17"
* rand: "0.8.5"
* simplelog: "0.12.1"
* sqlite: "0.30.4"
* time: "0.3.21"
* tiny_http: "0.12.0"

## Quick start

### Build and run the service:
```console
$ cargo build --release
...
$ ./target/release/onetimer ./conf/config.toml
```

### Send your secret data:
```console
$ curl -d 'my secret data' http://127.0.0.1:8080/add
http://127.0.0.1:8080/get/3cfd3cd9b4913bbc571435314a63d011d2a51a8c9790c4dbbb7331932719d93e
```

### Get secret data using one-time link:
```console
$ curl http://127.0.0.1:8080/get/3cfd3cd9b4913bbc571435314a63d011d2a51a8c9790c4dbbb7331932719d93e
my secret data
```

### Try to get secret data one more time:
```console
$ curl -v http://127.0.0.1:8080/get/3cfd3cd9b4913bbc571435314a63d011d2a51a8c9790c4dbbb7331932719d93e
*   Trying 127.0.0.1:8080...
* Connected to 127.0.0.1 (127.0.0.1) port 8080 (#0)
> GET /get/3cfd3cd9b4913bbc571435314a63d011d2a51a8c9790c4dbbb7331932719d93e HTTP/1.1
> Host: 127.0.0.1:8080
> User-Agent: curl/7.68.0
> Accept: */*
>
* Mark bundle as not supporting multiuse
< HTTP/1.1 404 Not Found
< Server: tiny-http (Rust)
< Date: Fri, 12 May 2023 09:02:11 GMT
< Content-Type: text/plain; charset=UTF-8
< Content-Length: 14
<
* Connection #0 to host 127.0.0.1 left intact
404: Not Found
```

### Config file format
You can specify your own config file for `onetimer` service. Configurational files are written in TOML format. Here is an example ([config.toml](conf/config.toml)):
```toml
[database]
path = "db.sqlite"                  # path to SQLite3 database file or ":memory:"

[server]
host = "127.0.0.1"                  # host for tiny-http to start the server
port = 8080                         # port for tiny-http to start the server
address = "http://127.0.0.1:8080"   # address being sent to user to one-time access his secret data

[log]
type = "file"                       # logging type; supported types are "file" or "console"
file = "onetimer.log"               # log file for "file" logging type
level = "info"                      # logging level
```

## TODO
* add support for other databases (PostgreSQL, MySQL, etc)
* log input requests to database
* clear data instead if deleting records in db table
* proper collisions handling
* specify secret data lifetime as input parameter for /add
* notify when someone follow one-time link (add `notify` input parameter for /add)
* use some crate (`clap`?) for parsing command-line arguments
