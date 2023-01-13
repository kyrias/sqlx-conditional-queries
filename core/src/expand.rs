use std::collections::HashMap;

use crate::{lower::LoweredConditionalQueryAs, DatabaseType, DATABASE_TYPE};

#[derive(Debug, thiserror::Error)]
pub enum ExpandError {
    #[error("missing compile-time binding: {0}")]
    MissingCompileTimeBinding(String, proc_macro2::Span),
    #[error("missing binding closing brace")]
    MissingBindingClosingBrace(proc_macro2::Span),
    #[error("failed to parse type override in binding reference: {0}")]
    BindingReferenceTypeOverrideParseError(proc_macro2::LexError, proc_macro2::Span),
}

#[derive(Debug)]
pub(crate) struct ExpandedConditionalQueryAs {
    pub(crate) output_type: syn::Ident,
    pub(crate) match_expressions: Vec<syn::Expr>,
    pub(crate) match_arms: Vec<MatchArm>,
}

#[derive(Debug)]
pub(crate) struct MatchArm {
    pub(crate) patterns: Vec<syn::Pat>,
    pub(crate) query_fragments: Vec<syn::LitStr>,
    pub(crate) run_time_bindings: Vec<(syn::Ident, Option<proc_macro2::TokenStream>)>,
}

/// Corresponds to a single run-time binding name.
#[derive(Debug)]
struct RunTimeBinding {
    /// List of all argument index positions at which this binding needs to be bound.
    ///
    /// - For PostgreSQL only contains one element.
    /// - For MySQL and SQLite it contains one index for each time the binding was referenced.
    indices: Vec<usize>,

    /// Type-override fragment to pass on To SQLx
    type_override: Option<proc_macro2::TokenStream>,
}

#[derive(Debug, Default)]
struct RunTimeBindings {
    counter: usize,
    bindings: HashMap<syn::LitStr, RunTimeBinding>,
}
impl RunTimeBindings {
    /// Returns a database-appropriate run-time binding string for the given binding name.
    ///
    /// Database type selection is done based on the features this crate was built with.
    ///
    /// - PostgreSQL uses 1-indexed references such as `$1`, which means that multiple references
    ///   to the same parameter only need to be bound once.
    /// - MySQL and SQLite always use `?` which means that the arguments need to specified in
    ///   order and be duplicated for as many times as they're used.
    fn get_binding_string(
        &mut self,
        binding_name: syn::LitStr,
        type_override: Option<proc_macro2::TokenStream>,
    ) -> syn::LitStr {
        match DATABASE_TYPE {
            DatabaseType::PostgreSql => {
                let span = binding_name.span();
                let binding = self.bindings.entry(binding_name).or_insert_with(|| {
                    self.counter += 1;
                    RunTimeBinding {
                        indices: vec![self.counter],
                        type_override,
                    }
                });
                syn::LitStr::new(&format!("${}", binding.indices.first().unwrap()), span)
            }
            DatabaseType::MySql | DatabaseType::Sqlite => {
                let span = binding_name.span();
                self.counter += 1;

                // For MySQL and SQLite bindings we need to specify the same argument multiple
                // times if it's reused and so generate a unique index every time.  This ensures
                // that `get_run_time_bindings` will generate the arguments in the correct order.
                self.bindings
                    .entry(binding_name)
                    .and_modify(|binding| binding.indices.push(self.counter))
                    .or_insert_with(|| RunTimeBinding {
                        indices: vec![self.counter],
                        type_override,
                    });
                syn::LitStr::new("?", span)
            }
        }
    }

    /// Returns the `query_as!` arguments for all referenced run-time bindings.
    fn get_arguments(self) -> Vec<(syn::Ident, Option<proc_macro2::TokenStream>)> {
        let mut run_time_bindings: Vec<_> = self
            .bindings
            .into_iter()
            .flat_map(|(name, binding)| {
                binding
                    .indices
                    .into_iter()
                    .map(|index| {
                        (
                            syn::Ident::new(&name.value(), name.span()),
                            binding.type_override.clone(),
                            index,
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        run_time_bindings.sort_by_key(|(_, _, index)| *index);

        run_time_bindings
            .into_iter()
            .map(|(ident, type_override, _)| (ident, type_override))
            .collect()
    }
}

pub(crate) fn expand(
    lowered: LoweredConditionalQueryAs,
) -> Result<ExpandedConditionalQueryAs, ExpandError> {
    let mut match_arms = Vec::new();

    for arm in lowered.match_arms {
        let mut fragments = vec![lowered.query_string.clone()];
        while fragments
            .iter()
            .any(|fragment| fragment.value().contains("{#"))
        {
            fragments = expand_compile_time_bindings(fragments, &arm.comp_time_bindings)?;
        }

        let mut run_time_bindings = RunTimeBindings::default();
        let expanded = expand_run_time_bindings(fragments, &mut run_time_bindings)?;

        match_arms.push(MatchArm {
            patterns: arm.patterns,
            query_fragments: expanded,
            run_time_bindings: run_time_bindings.get_arguments(),
        });
    }

    Ok(ExpandedConditionalQueryAs {
        output_type: lowered.output_type,
        match_expressions: lowered.match_expressions,
        match_arms,
    })
}

fn expand_compile_time_bindings(
    unexpanded_fragments: Vec<syn::LitStr>,
    comp_time_bindings: &HashMap<String, syn::LitStr>,
) -> Result<Vec<syn::LitStr>, ExpandError> {
    let mut expanded_fragments = Vec::new();

    for fragment in unexpanded_fragments {
        let fragment_string = fragment.value();
        let mut fragment_str = fragment_string.as_str();

        while let Some(start_of_binding) = fragment_str.find('{') {
            let next_char = fragment_str[start_of_binding..].chars().nth(1);
            if next_char == Some('{') {
                // If we find `{{` that means that we've hit an escaped brace which will be
                // unescaped by the run-time binding pass.
                expanded_fragments.push(syn::LitStr::new(
                    &fragment_str[..start_of_binding + 2],
                    fragment.span(),
                ));
                fragment_str = &fragment_str[start_of_binding + 2..];
                continue;
            }

            // Otherwise we've hit either a compile-time or a run-time binding, so first we
            // push any prefix before the binding.
            if !fragment_str[..start_of_binding].is_empty() {
                expanded_fragments.push(syn::LitStr::new(
                    &fragment_str[..start_of_binding],
                    fragment.span(),
                ));
                fragment_str = &fragment_str[start_of_binding..];
            }

            // Then we find the matching closing brace.
            let end_of_binding = if let Some(end_of_binding) = fragment_str.find('}') {
                end_of_binding
            } else {
                return Err(ExpandError::MissingBindingClosingBrace(fragment.span()));
            };

            if next_char == Some('#') {
                // If the binding is a compile-time binding, expand it.
                let binding_name = &fragment_str[2..end_of_binding];
                if let Some(binding) = comp_time_bindings.get(binding_name) {
                    expanded_fragments.push(binding.clone());
                } else {
                    return Err(ExpandError::MissingCompileTimeBinding(
                        binding_name.to_string(),
                        fragment.span(),
                    ));
                }
            } else {
                // Otherwise push it as-is for the next pass.
                expanded_fragments.push(syn::LitStr::new(
                    &fragment_str[..end_of_binding + 1],
                    fragment.span(),
                ));
            }

            fragment_str = &fragment_str[end_of_binding + 1..];
        }

        // Push trailing query fragment.
        if !fragment_str.is_empty() {
            expanded_fragments.push(syn::LitStr::new(fragment_str, fragment.span()));
        }
    }

    Ok(expanded_fragments)
}

fn expand_run_time_bindings(
    unexpanded_fragments: Vec<syn::LitStr>,
    run_time_bindings: &mut RunTimeBindings,
) -> Result<Vec<syn::LitStr>, ExpandError> {
    let mut expanded_query = Vec::new();

    for fragment in unexpanded_fragments {
        let fragment_string = fragment.value();
        let mut fragment_str = fragment_string.as_str();

        while let Some(start_of_binding) = fragment_str.find('{') {
            let next_char = fragment_str[start_of_binding..].chars().nth(1);
            if next_char == Some('{') {
                // If we find `{{` that means that we've hit an escaped brace which we will
                // unescape by skipping the second one.
                expanded_query.push(syn::LitStr::new(
                    &fragment_str[..start_of_binding + 1],
                    fragment.span(),
                ));
                fragment_str = &fragment_str[start_of_binding + 2..];
                continue;
            }

            // Otherwise we've hit a run-time binding, so first we push any prefix before the
            // binding.
            expanded_query.push(syn::LitStr::new(
                &fragment_str[..start_of_binding],
                fragment.span(),
            ));

            // Then we find the matching closing brace.
            fragment_str = &fragment_str[start_of_binding + 1..];
            let end_of_binding = if let Some(end_of_binding) = fragment_str.find('}') {
                end_of_binding
            } else {
                return Err(ExpandError::MissingBindingClosingBrace(fragment.span()));
            };

            let binding_name = &fragment_str[..end_of_binding];
            let (binding_name, type_override) = if let Some(offset) = binding_name.find(':') {
                let (binding_name, type_override) = binding_name.split_at(offset);
                let type_override =
                    type_override
                        .parse::<proc_macro2::TokenStream>()
                        .map_err(|err| {
                            ExpandError::BindingReferenceTypeOverrideParseError(
                                err,
                                fragment.span(),
                            )
                        })?;
                (binding_name.trim(), Some(type_override))
            } else {
                (binding_name, None)
            };

            // And finally we push a bound parameter argument
            let binding = run_time_bindings.get_binding_string(
                syn::LitStr::new(binding_name, fragment.span()),
                type_override,
            );
            expanded_query.push(binding);

            fragment_str = &fragment_str[end_of_binding + 1..];
        }

        // Push trailing query fragment.
        if !fragment_str.is_empty() {
            expanded_query.push(syn::LitStr::new(fragment_str, fragment.span()));
        }
    }

    Ok(expanded_query)
}

#[cfg(test)]
mod tests {
    use quote::ToTokens;

    use super::*;

    #[test]
    fn expands_compile_time_bindings() {
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
        let expanded = expand(lowered).unwrap();

        assert_eq!(
            expanded.match_arms[0]
                .query_fragments
                .iter()
                .map(|qs| qs.to_token_stream().to_string())
                .collect::<Vec<_>>(),
            &[
                "\"some \"",
                "\"e\"",
                "\" \"",
                "\"f\"",
                "\" \"",
                "\"l\"",
                "\" query\""
            ],
        );
    }

    #[test]
    fn expands_run_time_bindings() {
        let parsed = syn::parse_str::<crate::parse::ParsedConditionalQueryAs>(
            r#"
                SomeType,
                "some {foo} {bar} {foo} query",
            "#,
        )
        .unwrap();
        let analyzed = crate::analyze::analyze(parsed.clone()).unwrap();
        let lowered = crate::lower::lower(analyzed);
        let expanded = expand(lowered).unwrap();

        assert_eq!(
            expanded.match_arms[0]
                .query_fragments
                .iter()
                .map(|qs| qs.to_token_stream().to_string())
                .collect::<Vec<_>>(),
            match DATABASE_TYPE {
                DatabaseType::PostgreSql => &[
                    "\"some \"",
                    "\"$1\"",
                    "\" \"",
                    "\"$2\"",
                    "\" \"",
                    "\"$1\"",
                    "\" query\""
                ],
                DatabaseType::MySql | DatabaseType::Sqlite => &[
                    "\"some \"",
                    "\"?\"",
                    "\" \"",
                    "\"?\"",
                    "\" \"",
                    "\"?\"",
                    "\" query\""
                ],
            }
        );
    }
}
