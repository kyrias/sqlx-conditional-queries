#![doc = include_str!("../README.md")]

pub use analyze::AnalyzeError;
pub use expand::ExpandError;

mod analyze;
mod codegen;
mod expand;
mod lower;
mod parse;

const DATABASE_TYPE: DatabaseType = if cfg!(feature = "postgres") {
    DatabaseType::PostgreSql
} else if cfg!(feature = "mysql") {
    DatabaseType::MySql
} else if cfg!(feature = "sqlite") {
    DatabaseType::Sqlite
} else {
    panic!("No database feature was enabled")
};

enum DatabaseType {
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
    input: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, Error> {
    let parsed = syn::parse2::<parse::ParsedConditionalQueryAs>(input)?;
    let analyzed = analyze::analyze(parsed)?;
    let lowered = lower::lower(analyzed);
    let expanded = expand::expand(lowered)?;
    let codegened = codegen::codegen(expanded);

    Ok(codegened)
}
