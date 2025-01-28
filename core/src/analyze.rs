use std::collections::HashSet;

use syn::spanned::Spanned;

use crate::parse::ParsedConditionalQueryAs;

#[derive(Debug, thiserror::Error)]
pub enum AnalyzeError {
    #[error("expected string literal")]
    ExpectedStringLiteral(proc_macro2::Span),
    #[error("mismatch between number of names ({names}) and values ({values})")]
    BindingNameValueLengthMismatch {
        names: usize,
        names_span: proc_macro2::Span,
        values: usize,
        values_span: proc_macro2::Span,
    },
    #[error("found two compile-time bindings with the same binding: {first}")]
    DuplicatedCompileTimeBindingsFound {
        first: proc_macro2::Ident,
        second: proc_macro2::Ident,
    },
}

/// This represents the finished second step in the processing pipeline.
/// The compile time bindings have been further processed to a form that allows us to easily create
/// the cartesian product and thereby all query variations in the next step.
#[derive(Debug)]
pub(crate) struct AnalyzedConditionalQueryAs {
    pub(crate) output_type: syn::Ident,
    pub(crate) query_string: syn::LitStr,
    pub(crate) compile_time_bindings: Vec<CompileTimeBinding>,
}

/// This represents a single combination of a single compiletime binding of a query.
#[derive(Debug)]
pub(crate) struct CompileTimeBinding {
    /// The actual expression used in the match statement.
    /// E.g. for `match something`, this would be `something`.
    pub(crate) expression: syn::Expr,
    /// Each entry in this Vec represents a single expanded `match` and the
    /// binding names with the binding values from that specific arm.
    /// (`match arm pattern`, Vec(binding_name, binding_value)`
    pub(crate) arms: Vec<(syn::Pat, Vec<(syn::Ident, syn::LitStr)>)>,
}

/// Further parse and analyze all compiletime binding statements.
/// Each binding is split into individual entries of this form:
/// (`match arm pattern`, Vec(binding_name, binding_value)`
pub(crate) fn analyze(
    parsed: ParsedConditionalQueryAs,
) -> Result<AnalyzedConditionalQueryAs, AnalyzeError> {
    let mut compile_time_bindings = Vec::new();

    let mut known_binding_names = HashSet::new();

    for (names, match_expr) in parsed.compile_time_bindings {
        let binding_names_span = names.span();
        // Convert the OneOrPunctuated enum in a list of `Ident`s.
        // `One(T)` will be converted into a Vec with a single entry.
        let binding_names: Vec<_> = names.into_iter().collect();

        // Find duplicate compile-time bindings.
        for name in &binding_names {
            let Some(first) = known_binding_names.get(name) else {
                known_binding_names.insert(name.clone());
                continue;
            };
            return Err(AnalyzeError::DuplicatedCompileTimeBindingsFound {
                first: first.clone(),
                second: name.clone(),
            });
        }

        let mut bindings = Vec::new();
        for arm in match_expr.arms {
            let arm_span = arm.body.span();

            let binding_values = match *arm.body {
                // If the match arm expression just contains a literal, use that.
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(literal),
                    ..
                }) => vec![literal],

                // If there's a tuple, treat each literal inside that tuple as a binding value.
                syn::Expr::Tuple(tuple) => {
                    let mut values = Vec::new();
                    for elem in tuple.elems {
                        match elem {
                            syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(literal),
                                ..
                            }) => values.push(literal),

                            _ => return Err(AnalyzeError::ExpectedStringLiteral(elem.span())),
                        }
                    }
                    values
                }

                body => return Err(AnalyzeError::ExpectedStringLiteral(body.span())),
            };

            // There must always be a matching amount of binding values in each match arm.
            // Error if there are more or fewer values than binding names.
            if binding_names.len() != binding_values.len() {
                return Err(AnalyzeError::BindingNameValueLengthMismatch {
                    names: binding_names.len(),
                    names_span: binding_names_span,
                    values: binding_values.len(),
                    values_span: arm_span,
                });
            }

            bindings.push((
                arm.pat,
                binding_names
                    .iter()
                    .cloned()
                    .zip(binding_values)
                    .collect::<Vec<_>>(),
            ));
        }

        compile_time_bindings.push(CompileTimeBinding {
            expression: *match_expr.expr,
            arms: bindings,
        });
    }

    Ok(AnalyzedConditionalQueryAs {
        output_type: parsed.output_type,
        query_string: parsed.query_string,
        compile_time_bindings,
    })
}

#[cfg(test)]
mod tests {
    use quote::ToTokens;

    use super::*;

    #[test]
    fn valid_syntax() {
        let parsed = syn::parse_str::<ParsedConditionalQueryAs>(
            r#"
                SomeType,
                "some SQL query",
                #binding = match foo {
                    bar => "baz",
                },
                #(a, b) = match c {
                    d => ("e", "f"),
                },
            "#,
        )
        .unwrap();
        let mut analyzed = analyze(parsed.clone()).unwrap();

        assert_eq!(parsed.output_type, analyzed.output_type);
        assert_eq!(parsed.query_string, analyzed.query_string);

        assert_eq!(analyzed.compile_time_bindings.len(), 2);

        {
            let compile_time_binding = dbg!(analyzed.compile_time_bindings.remove(0));
            assert_eq!(
                compile_time_binding
                    .expression
                    .to_token_stream()
                    .to_string(),
                "foo",
            );

            assert_eq!(compile_time_binding.arms.len(), 1);
            {
                let arm = &compile_time_binding.arms[0];
                assert_eq!(arm.0.to_token_stream().to_string(), "bar");
                assert_eq!(
                    arm.1
                        .iter()
                        .map(|v| (
                            v.0.to_token_stream().to_string(),
                            v.1.to_token_stream().to_string(),
                        ))
                        .collect::<Vec<_>>(),
                    &[("binding".to_string(), "\"baz\"".to_string())],
                );
            }
        }

        {
            let compile_time_binding = dbg!(analyzed.compile_time_bindings.remove(0));
            assert_eq!(
                compile_time_binding
                    .expression
                    .to_token_stream()
                    .to_string(),
                "c",
            );

            assert_eq!(
                compile_time_binding
                    .arms
                    .iter()
                    .map(|v| v.0.to_token_stream().to_string())
                    .collect::<Vec<_>>(),
                &["d"],
            );

            assert_eq!(compile_time_binding.arms.len(), 1);
            {
                let arm = &compile_time_binding.arms[0];
                assert_eq!(arm.0.to_token_stream().to_string(), "d");
                assert_eq!(
                    arm.1
                        .iter()
                        .map(|v| (
                            v.0.to_token_stream().to_string(),
                            v.1.to_token_stream().to_string(),
                        ))
                        .collect::<Vec<_>>(),
                    &[
                        ("a".to_string(), "\"e\"".to_string()),
                        ("b".to_string(), "\"f\"".to_string())
                    ],
                );
            }
        }
    }

    #[test]
    fn duplicate_compile_time_bindings() {
        let parsed = syn::parse_str::<ParsedConditionalQueryAs>(
            r##"
                SomeType,
                r#"{#a}"#,
                #a = match _ {
                    _ => "1",
                },
                #a = match _ {
                    _ => "2",
                },
            "##,
        )
        .unwrap();
        let analyzed = analyze(parsed.clone()).unwrap_err();

        assert!(matches!(
            analyzed,
            AnalyzeError::DuplicatedCompileTimeBindingsFound { .. }
        ));
    }
}
