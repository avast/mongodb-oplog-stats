# mongodb-oplog-stats

Obtaining of statistics about a [replica-set
oplog](https://docs.mongodb.com/manual/core/replica-set-oplog/) of a
[MongoDB](https://www.mongodb.com/) instance.

If you have ever wondered why is the duration of your oplog so short, look no
further. The tool queries the oplog of a MongoDB instance and prints an
overview about which collections and operation types have the greatest impact
on its duration. For each collection and operation type, it provides the number
of operations, their total size, and their share among all the operations in
the oplog.

In a greater detail, each MongoDB operation (e.g. insert or updated) is
represented by a [BSON](https://docs.mongodb.com/manual/reference/bson-types/)
document. Whenever you run a MongoDB operation, it is recorded in the oplog so
that all the [secondary
nodes](https://docs.mongodb.com/manual/core/replica-set-secondary/) in your
replicas set can reflect the operation in their storage. The bigger the size of
the oplog, the longer your replicas can be down without a need for a [complete
resync](https://docs.mongodb.com/manual/core/replica-set-sync/#initial-sync).
Sometimes, you may have a large oplog, but its duration is not as long as you
would expect. In such cases, you can utilize the fact that the oplog is a
MongoDB collection. The `mongodb-oplog-stats` tool queries the documents in the
oplog and gives you its overview that you can use to see which collections and
operation types are shortening your oplog duration the most.

For very basic info about the oplog, you can also use the
[`rs.printReplicationInfo()`](https://docs.mongodb.com/manual/reference/method/rs.printReplicationInfo/#rs.printReplicationInfo)
function from the [mongo shell](https://docs.mongodb.com/manual/mongo/):
```
> rs.printReplicationInfo()
configured oplog size:   1024MB
log length start to end: 65422secs (18.22hrs)
oplog first event time:  Mon Jun 23 2019 17:47:18 GMT+0200 (CEST)
oplog last event time:   Tue Jun 24 2019 11:57:40 GMT+0200 (CEST)
now:                     Thu Jun 26 2019 14:24:39 GMT+0200 (CEST)
```

## Requirements

A recent-enough version of [Rust](https://www.rust-lang.org/). The tool is
written in the [2018 edition of
Rust](https://blog.rust-lang.org/2018/12/06/Rust-1.31-and-rust-2018.html).

## Build and run

```
git clone https://github.com/avast/mongodb-oplog-stats.git
cd mongodb-oplog-stats
cargo run --release --host myhost.com --username myuser
```
For more information about the available parameters, run
```
cargo run --release -- --help
```
Connection and authentication parameters are identical to those used by
standard MongoDB tools.

Notes:
* The user that you use must have [sufficient
  privileges](https://docs.mongodb.com/manual/reference/privilege-actions/) to
  be able to query the oplog.
* If you are missing a parameter, please [create an
  issue](https://github.com/avast/mongodb-oplog-stats/issues) or submit a [pull
  request](https://github.com/avast/mongodb-oplog-stats/pulls).

## Example output

```
+--------------------------------------+-----------+------------+-----------+
| Entry                                | Documents | Total size | Share (%) |
+--------------------------------------+-----------+------------+-----------+
| moviedb.movies:i                     |      2634 |  675.12 MB |     71.04 |
| store.books:i                        |       196 |  146.23 MB |     17.68 |
| moviedb.actors:u                     |        35 |   10.22 MB |      3.64 |
| blog.comments:i                      |     10598 |    9.07 MB |      2.34 |
| moviedb.movies:i                     |        18 |    7.87 MB |      2.03 |
| store.books:u                        |        81 |    7.59 MB |      1.96 |
| store.reviews:i                      |     14459 |    5.07 MB |      1.31 |
| moviedb.trailers:d                   |       205 |    3.17 MB |      0.82 |
| store.employees:d                    |         1 |      170 B |    < 0.01 |
| blog.articles:i                      |         1 |      158 B |    < 0.01 |
+--------------------------------------+-----------+------------+-----------+
```

## Output format

Each entry has the following format:
```
database.collection:op
```
where
* `database` is the name of the database,
* `collection` is the name of the collection,
* `op` is one of the following abbreviations:
    * `i` for _inserts_,
    * `u` for _updates_,
    * `d` for _deletes_,
    * `c` for _commands_ that affect databases at a high level,
    * `db` which announces the presence of a _database_,
    * `n` for _no-ops_, used for operations which do not result in a change in
      the stored data. If you have a lot of these operations in your oplog,
      take a look [here](https://jira.mongodb.org/browse/PHPC-1523) and
      [here](https://jira.mongodb.org/browse/SERVER-45442).

As for the other columns, `Documents` is the number of documents corresponding
to the operations, `Total size` is the total size of the BSON representation of
those documents, and `Share` gives you the percentage of the entry with regards
to other entries in the oplog.

## Development

Here are few useful commands to use during development:
* Check for errors without code generation (faster than `cargo run`):
    ```
    $ cargo check
    ```
* Format sources files:
    ```
    $ rustup component add rustfmt
    $ cargo fmt
    ```
* Run lints to catch common mistakes and improve the code:
    ```
    $ rustup component add clippy
    $ cargo clippy
    ```
* Add a new dependency:
    ```
    $ cargo install cargo-edit
    $ cargo add PACKAGE_NAME
    ```
* List outdated dependencies (`-R` is for the direct ones):
    ```
    $ cargo install cargo-outdated
    $ cargo outdated -R
    ```

## License

Copyright (c) 2020 Avast Software, licensed under the MIT license. See the
[`LICENSE`](https://github.com/avast/mongodb-oplog-stats/blob/master/LICENSE)
file for more details.
