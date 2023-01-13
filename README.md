# Conditional compile-time verified queries with SQLx

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
