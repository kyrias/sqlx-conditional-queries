#![doc = include_str!("../README.md")]

pub use analyze::AnalyzeError;
pub use expand::ExpandError;

mod analyze;
mod codegen;
mod expand;
mod lower;
mod parse;

#[cfg(test)]
mod snapshot_tests;

#[derive(Clone, Copy, Debug)]
pub enum DatabaseType {
    PostgreSql,
    MySql,
    Sqlite,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("syn error: {0}")]
    SynError(#[from] syn::Error),
    #[error("analyze error: {0}")]
    AnalyzeError(#[from] analyze::AnalyzeError),
    #[error("expand error: {0}")]
    ExpandError(#[from] expand::ExpandError),
}

pub fn conditional_query_as(
    database_type: DatabaseType,
    input: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, Error> {
    let parsed = syn::parse2::<parse::ParsedConditionalQueryAs>(input)?;
    let analyzed = analyze::analyze(parsed)?;
    let lowered = lower::lower(analyzed);
    let expanded = expand::expand(database_type, lowered)?;
    let codegened = codegen::codegen(expanded);

    Ok(codegened)
}
