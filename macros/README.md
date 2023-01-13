**Note**: This is an internal library only meant to be used by the sqlx-conditional-queries crate.

The reason for this crate being separate from the main sqlx-conditional-queries
crate is to be able to export additional types through that crate which are
necessary for the generated code.  The main proc-macro crate cannot expose them
because they're only allowed to export macros.


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
