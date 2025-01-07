#![doc = include_str!("../README.md")]

use proc_macro_error::abort;
use sqlx_conditional_queries_core::{AnalyzeError, DatabaseType, Error, ExpandError};

const DATABASE_TYPE: DatabaseType = if cfg!(feature = "postgres") {
    DatabaseType::PostgreSql
} else if cfg!(feature = "mysql") {
    DatabaseType::MySql
} else if cfg!(feature = "sqlite") {
    DatabaseType::Sqlite
} else {
    panic!("No database feature was enabled")
};

// The public docs for this macro live in the sql-conditional-queries crate.
#[proc_macro_error::proc_macro_error]
#[proc_macro]
pub fn conditional_query_as(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: proc_macro2::TokenStream = input.into();

    let ts = match sqlx_conditional_queries_core::conditional_query_as(DATABASE_TYPE, input) {
        Ok(ts) => ts,
        Err(Error::SynError(err)) => {
            return err.to_compile_error().into();
        }
        Err(Error::AnalyzeError(err)) => match err {
            AnalyzeError::ExpectedStringLiteral(span) => abort!(
                span,
                "expected string literal";
                help = "only string literals or tuples of string literals are supported in compile-time bindings";
            ),
            AnalyzeError::BindingNameValueLengthMismatch {
                names,
                names_span,
                values,
                values_span,
            } => abort!(
                names_span,
                "mismatch between number of names and values";
                names = names_span => "number of names: {}", names;
                values = values_span => "number of values: {}", values;
            ),
        },
        Err(Error::ExpandError(err)) => match err {
            // TODO: Make this span point at the binding reference.  Requires https://github.com/rust-lang/rust/issues/54725
            ExpandError::MissingCompileTimeBinding(binding, span) => abort!(
                span,
                "missing compile-time binding";
                help = "found no compile-time binding with the specified name: {}", binding;
            ),
            // TODO: Make this span point at the opening brace.  Requires https://github.com/rust-lang/rust/issues/54725
            ExpandError::MissingBindingClosingBrace(span) => abort!(
                span,
                "missing closing brace for compile-time binding reference"
            ),
            ExpandError::BindingReferenceTypeOverrideParseError(err, span) => abort!(
                span,
                "failed to parse type override in binding reference: {}",
                err
            ),
        },
    };

    let output: proc_macro::TokenStream = ts.into();
    output
}
