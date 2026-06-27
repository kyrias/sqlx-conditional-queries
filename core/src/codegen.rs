use quote::{format_ident, quote};

use crate::expand::ExpandedConditionalQueryAs;

/// This is the final step of the macro generation pipeline.
/// The match arms and the respective query fragments are now used to generate a giant match
/// statement, which covers all variants of the bindings' match statements' cartesian products.
pub(crate) fn codegen(
    expanded: ExpandedConditionalQueryAs,
    checked: bool,
) -> proc_macro2::TokenStream {
    let mut match_arms = Vec::new();
    for (idx, arm) in expanded.match_arms.iter().enumerate() {
        let patterns = &arm.patterns;
        let variant = format_ident!("Variant{}", idx);
        let output_type = &expanded.output_type;
        let query_fragments = &arm.query_fragments;
        let run_time_bindings =
            arm.run_time_bindings
                .iter()
                .map(|(name, type_override)| match type_override {
                    Some(ty) => quote!(#name as #ty),
                    None => quote!(#name),
                });

        let query = if checked {
            format_ident!("query_as")
        } else {
            format_ident!("query_as_unchecked")
        };

        match_arms.push(quote! {
            (#(#patterns,)*) => {
                ConditionalMap::#variant(
                    ::sqlx::#query!(
                        #output_type,
                        #(#query_fragments)+*,
                        #(#run_time_bindings),*
                    )
                )
            },
        });
    }

    let conditional_map = build_conditional_map(expanded.match_arms.len());
    let match_expressions = expanded.match_expressions;

    quote! {
        {
            #conditional_map

            match (#(#match_expressions,)*) {
                #(#match_arms)*
            }
        }
    }
}

fn build_conditional_map(variant_count: usize) -> proc_macro2::TokenStream {
    let function_params: Vec<_> = (0..variant_count)
        .map(|index| format_ident!("F{}", index))
        .collect();
    let variants: Vec<_> = (0..variant_count)
        .map(|index| format_ident!("Variant{}", index))
        .collect();

    quote! {
        enum ConditionalMap<'q, DB: ::sqlx::Database, A, #(#function_params),*> {
            #(
                #variants(
                    ::sqlx::query::Map<'q, DB, #function_params, A>
                ),
            )*
        }

        impl<'q, DB, A, O, #(#function_params),*> ConditionalMap<'q, DB, A, #(#function_params),*>
        where
            DB: ::sqlx::Database,
            A: 'q + ::sqlx::IntoArguments<DB> + ::std::marker::Send,
            O: ::std::marker::Unpin + ::std::marker::Send,
            #(
                #function_params: ::std::ops::FnMut(DB::Row) -> ::sqlx::Result<O> + ::std::marker::Send,
            )*
        {
            /// See [`sqlx::query::Map::fetch`]
            pub fn fetch<'e, 'c: 'e, E>(
                self,
                executor: E,
            ) -> ::sqlx_conditional_queries::exports::BoxStream<'e, ::sqlx::Result<O>>
            where
                'q: 'e,
                E: 'e + ::sqlx::Executor<'c, Database = DB>,
                DB: 'e,
                O: 'e,
                #(
                    #function_params: 'e,
                )*
            {
                match self {
                    #(
                        Self::#variants(map) => map.fetch(executor),
                    )*
                }
            }

            /// See [`sqlx::query::Map::fetch_many`]
            #[deprecated = "Only the SQLite driver supports multiple statements in one prepared statement and that behavior is deprecated. Use `sqlx::raw_sql()` instead. See https://github.com/launchbadge/sqlx/issues/3108 for discussion."]
            pub fn fetch_many<'e, 'c: 'e, E>(
                mut self,
                executor: E,
            ) -> ::sqlx_conditional_queries::exports::BoxStream<'e, ::sqlx::Result<::sqlx::Either<DB::QueryResult, O>>>
            where
                'q: 'e,
                E: 'e + ::sqlx::Executor<'c, Database = DB>,
                DB: 'e,
                O: 'e,
                #(
                    #function_params: 'e,
                )*
            {
                match self {
                    #(
                        Self::#variants(map) => {
                            #[allow(deprecated)]
                            map.fetch_many(executor)
                        }
                    )*
                }
            }

            /// See [`sqlx::query::Map::fetch_all`]
            pub async fn fetch_all<'e, 'c: 'e, E>(
                self,
                executor: E,
            ) -> ::sqlx::Result<::std::vec::Vec<O>>
            where
                'q: 'e,
                E: 'e + ::sqlx::Executor<'c, Database = DB>,
                DB: 'e,
                O: 'e,
                #(
                    #function_params: 'e,
                )*
            {
                match self {
                    #(
                        Self::#variants(map) => map.fetch_all(executor).await,
                    )*
                }
            }

            /// See [`sqlx::query::Map::fetch_one`]
            pub async fn fetch_one<'e, 'c: 'e, E>(
                self,
                executor: E,
            ) -> ::sqlx::Result<O>
            where
                'q: 'e,
                E: 'e + ::sqlx::Executor<'c, Database = DB>,
                DB: 'e,
                O: 'e,
                #(
                    #function_params: 'e,
                )*
            {
                match self {
                    #(
                        Self::#variants(map) => map.fetch_one(executor).await,
                    )*
                }
            }

            /// See [`sqlx::query::Map::fetch_optional`]
            pub async fn fetch_optional<'e, 'c: 'e, E>(
                self,
                executor: E,
            ) -> ::sqlx::Result<::std::option::Option<O>>
            where
                'q: 'e,
                E: 'e + ::sqlx::Executor<'c, Database = DB>,
                DB: 'e,
                O: 'e,
                #(
                    #function_params: 'e,
                )*
            {
                match self {
                    #(
                        Self::#variants(map) => map.fetch_optional(executor).await,
                    )*
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::DatabaseType;

    use super::*;

    #[rstest::rstest]
    #[case(DatabaseType::PostgreSql, true)]
    #[case(DatabaseType::PostgreSql, false)]
    #[case(DatabaseType::MySql, true)]
    #[case(DatabaseType::MySql, false)]
    #[case(DatabaseType::Sqlite, true)]
    #[case(DatabaseType::Sqlite, false)]
    fn valid_syntax(#[case] database_type: DatabaseType, #[case] checked: bool) {
        let parsed = syn::parse_str::<crate::parse::ParsedConditionalQueryAs>(
            r#"
                SomeType,
                "some {#a} {#b} {#j} query",
                #(a, b) = match c {
                    d => ("e", "f"),
                    g => ("h", "i"),
                },
                #j = match i {
                    k => "l",
                    m => "n",
                },
            "#,
        )
        .unwrap();
        let analyzed = crate::analyze::analyze(parsed.clone()).unwrap();
        let lowered = crate::lower::lower(analyzed);
        let expanded = crate::expand::expand(database_type, lowered).unwrap();
        let _codegened = codegen(expanded, checked);
    }

    #[rstest::rstest]
    #[case(DatabaseType::PostgreSql, true)]
    #[case(DatabaseType::PostgreSql, false)]
    #[case(DatabaseType::MySql, true)]
    #[case(DatabaseType::MySql, false)]
    #[case(DatabaseType::Sqlite, true)]
    #[case(DatabaseType::Sqlite, false)]
    fn type_override(#[case] database_type: DatabaseType, #[case] checked: bool) {
        let parsed = syn::parse_str::<crate::parse::ParsedConditionalQueryAs>(
            r#"
                SomeType,
                "{some_binding:ty}",
            "#,
        )
        .unwrap();
        let analyzed = crate::analyze::analyze(parsed.clone()).unwrap();
        let lowered = crate::lower::lower(analyzed);
        let expanded = crate::expand::expand(database_type, lowered).unwrap();
        let codegened = codegen(expanded, checked);

        let stringified = codegened.to_string();
        assert!(
            stringified.contains(" some_binding as ty"),
            "binding type override was not correctly generated: {stringified}"
        );
    }
}
