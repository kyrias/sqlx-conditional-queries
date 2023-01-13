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
}

#[derive(Debug)]
pub(crate) struct AnalyzedConditionalQueryAs {
    pub(crate) output_type: syn::Ident,
    pub(crate) query_string: syn::LitStr,
    pub(crate) comp_time_bindings: Vec<CompileTimeBinding>,
}

#[derive(Debug)]
pub(crate) struct CompileTimeBinding {
    pub(crate) expression: syn::Expr,
    pub(crate) variants: Vec<(syn::Pat, Vec<(syn::Ident, syn::LitStr)>)>,
}

pub(crate) fn analyze(
    parsed: ParsedConditionalQueryAs,
) -> Result<AnalyzedConditionalQueryAs, AnalyzeError> {
    let mut comp_time_bindings = Vec::new();

    for (names, match_expr) in parsed.comp_time_bindings {
        let binding_names_span = names.span();
        let binding_names: Vec<_> = names.into_iter().collect();

        let mut bindings = Vec::new();
        for arm in match_expr.arms {
            let arm_span = arm.body.span();

            let binding_values = match *arm.body {
                // If the match arm expression just contains a literal, use that.
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(literal),
                    ..
                }) => vec![literal],

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

        comp_time_bindings.push(CompileTimeBinding {
            expression: *match_expr.expr,
            variants: bindings,
        });
    }

    Ok(AnalyzedConditionalQueryAs {
        output_type: parsed.output_type,
        query_string: parsed.query_string,
        comp_time_bindings,
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

        assert_eq!(analyzed.comp_time_bindings.len(), 2);

        {
            let comp_time_binding = dbg!(analyzed.comp_time_bindings.remove(0));
            assert_eq!(
                comp_time_binding.expression.to_token_stream().to_string(),
                "foo",
            );

            assert_eq!(comp_time_binding.variants.len(), 1);
            {
                let variant = &comp_time_binding.variants[0];
                assert_eq!(variant.0.to_token_stream().to_string(), "bar");
                assert_eq!(
                    variant
                        .1
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
            let comp_time_binding = dbg!(analyzed.comp_time_bindings.remove(0));
            assert_eq!(
                comp_time_binding.expression.to_token_stream().to_string(),
                "c",
            );

            assert_eq!(
                comp_time_binding
                    .variants
                    .iter()
                    .map(|v| v.0.to_token_stream().to_string())
                    .collect::<Vec<_>>(),
                &["d"],
            );

            assert_eq!(comp_time_binding.variants.len(), 1);
            {
                let variant = &comp_time_binding.variants[0];
                assert_eq!(variant.0.to_token_stream().to_string(), "d");
                assert_eq!(
                    variant
                        .1
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
}
