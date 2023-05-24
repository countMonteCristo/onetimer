# onetimer
Simple one-time-link service written in Rust.

## Description
The service is availiable to provide one-time access for any secret data. This can be useful for transfering sensitive information (for example, passwords) to users.

Suppose you've been hired a new employee and you need to grant him the access to your internal services, i.e. you have to give him his login and password. The way you could do it is to upload this data (login+password) to `onetimer` and send the link given to your employee. He can follow this link only once, so you can be sure that noone will get this sensitive information.

## Internal stucture
`onetimer` itself is a simple HTTP web server with database. It accepts only two methods: **/add** for adding new data and **/get** for providing data to user.

Supported database engines:
* `sqlite` - SQLite3 (database is stored in a local file)
* `mysql` - MySQL database
* `postgresql` - PostgreSQL database
* `memory` - data is stored in service process memory (you better do not want to use this engine in production!)
* `file` - data is stored in files into the directory, specified in `path` argument

## Dependencies
* [clap](https://docs.rs/clap/latest/clap/)
* [config](https://docs.rs/config/latest/config/)
* [log](https://docs.rs/log/latest/log/)
* [mysql](https://docs.rs/mysql/latest/mysql/)
* [postgres](https://docs.rs/postgres/latest/postgres/)
* [r2d2](https://docs.rs/r2d2/latest/r2d2/)
* [r2d2_postgres](https://docs.rs/r2d2_postgres/latest/r2d2_postgres/)
* [rand](https://docs.rs/config/latest/config/)
* [serde](https://docs.rs/serde/latest/serde/)
* [serde_derive](https://docs.rs/serde_derive/latest/serde_derive/)
* [serde_json](https://docs.rs/serde_json/latest/serde_json/)
* [simplelog](https://docs.rs/simplelog/latest/simplelog/)
* [sqlite](https://docs.rs/sqlite/latest/sqlite/)
* [threadpool](https://docs.rs/threadpool/latest/threadpool/)
* [time](https://docs.rs/time/latest/time/)
* [tiny_http](https://docs.rs/tiny_http/latest/tiny_http/)

## Quick start

### Build and run the service:
```console
$ cargo build --release
...
$ ./target/release/onetimer ./conf/config.toml
```

### Send your secret data:
```console
$ curl -d '{"data": "my secret data", "max_clicks": 3, "lifetime": 60}' http://127.0.0.1:8080/add
{"msg":"http://127.0.0.1:8080/get/3cfd3cd9b4913bbc571435314a63d011d2a51a8c9790c4dbbb7331932719d93e","status":"OK","created":1684490894,"expired":1684490954}
```

where
* `max_clicks` - number of clicks allowed to get your secret data (by deafult is 1)
* `lifetime` - maximum time in seconds your secret data will be availiable (has higher priority than `max_clicks`). By default is 1 week

Here `lifetime` == `expired` - `created`, and `expired` is the timestamp after which secret data will be deleted.

### Get secret data using one-time link:
```console
$ curl http://127.0.0.1:8080/get/3cfd3cd9b4913bbc571435314a63d011d2a51a8c9790c4dbbb7331932719d93e
{"msg":"my secret data","status":"OK"}
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
{"msg":"","status":"Link was not found or has been deleted"}
```

### Config file format
You can specify your own config file for `onetimer` service. Configurational files are written in TOML format. Here is an example ([config.toml](conf/config.toml)):
```toml
[database]
kind = "memory"                     # engine type, supported engines are "memory", "sqlite" and "file"
url = "./db.sqlite"                 # for SQLite3: path to db file or ":memory:"; for MySQL and PostgreSQL - connection string, for "file" type - path to directory where data files will be located

[server]
host = "127.0.0.1"                  # host for tiny-http to start the server
port = 8080                         # port for tiny-http to start the server
workers = 4                         # number of threads for threadpool
address = "http://127.0.0.1:8080"   # address being sent to user to one-time access his secret data

[log]
kind = "console"                    # logging type; supported types are "file" and "console"
file = "./onetimer.log"             # log file for "file" logging type
level = "info"                      # logging level
```

### Tests
You can run all tests at once:
```console
$ cargo build --release
$ ./tests/runtest.sh
[T00.sh] Check lifetime [memory]:
OK
[T00.sh] Check lifetime [sqlite]:
OK
[T00.sh] Check lifetime [file]:
OK
[T00.sh] Check lifetime [mysql]:
OK
[T00.sh] Check lifetime [postgresql]:
OK
[T01.sh] Check max_clicks [memory]:
OK
[T01.sh] Check max_clicks [sqlite]:
OK
[T01.sh] Check max_clicks [file]:
OK
[T01.sh] Check max_clicks [mysql]:
OK
[T01.sh] Check max_clicks [postgresql]:
OK
```
or run single test:
```console
$ cargo build --release
$ cd tests
$ ./T01.sh sqlite
[T01.sh] Check max_clicks [sqlite]:
OK

Attention! For `mysql` and `postgresql` tests to pass you need to setup these databases by hand.
```

## TODO
* add support for other databases
* prepare databases before tests
* log input requests to database
* clear data instead of deleting records in db table
* proper collisions handling
* notify when someone follow one-time link (add `notify` input parameter for /add)
