# Conditional compile-time verified queries with SQLx

[<img alt="github" src="https://img.shields.io/badge/github-kyrias/sqlx--conditional--queries-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/kyrias/sqlx-conditional-queries)
[<img alt="crates.io" src="https://img.shields.io/crates/v/sqlx-conditional-queries.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/sqlx-conditional-queries)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-sqlx--conditional--queries-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/sqlx-conditional-queries)

This crate provides a macro for generating conditional compile-time verified
queries while using the SQLx `query_as!` macro.  This allows you to have parts
of the query conditional in ways in which your chosen database doesn't allow by
emitting multiple `query_as!` invocations that are chosen over by a match
statement.


## Variant growth

Note that this means that we end up emitting as many `query_as!` invocations as
there are elements in the [Cartesian product] of all of the different
conditionals.  This means that the number of variants increase very rapidly!

[Cartesian product]: https://en.wikipedia.org/wiki/Cartesian_product


## Features

Which database type should be supported is specified by activating one of the
following features.  If more than one feature is activated then the first one
in the list takes precedence.

- `postgres`
- `mysql`
- `sqlite`


#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
