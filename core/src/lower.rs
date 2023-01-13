use std::collections::HashMap;

use itertools::Itertools;
use syn::parse_quote;

use crate::analyze::AnalyzedConditionalQueryAs;

#[derive(Debug)]
pub(crate) struct LoweredConditionalQueryAs {
    pub(crate) output_type: syn::Ident,
    pub(crate) query_string: syn::LitStr,
    pub(crate) match_expressions: Vec<syn::Expr>,
    pub(crate) match_arms: Vec<MatchArm>,
}

#[derive(Debug)]
pub(crate) struct MatchArm {
    pub(crate) patterns: Vec<syn::Pat>,
    pub(crate) comp_time_bindings: HashMap<String, syn::LitStr>,
}

pub(crate) fn lower(analyzed: AnalyzedConditionalQueryAs) -> LoweredConditionalQueryAs {
    let (bindings, mut match_expressions): (Vec<_>, Vec<_>) = analyzed
        .comp_time_bindings
        .into_iter()
        .map(|binding| (binding.variants.into_iter(), binding.expression))
        .unzip();

    let mut match_arms = Vec::new();
    for binding in bindings.into_iter().multi_cartesian_product() {
        let mut guards = Vec::new();
        let mut bindings = HashMap::new();
        binding.into_iter().for_each(|(g, b)| {
            guards.push(g);
            bindings.extend(b.into_iter().map(|(name, value)| (name.to_string(), value)));
        });
        match_arms.push(MatchArm {
            patterns: guards,
            comp_time_bindings: bindings,
        });
    }

    // If no compile-time bindings were specified we have add an always-true option.
    if match_expressions.is_empty() {
        match_expressions.push(parse_quote!(true));
        match_arms.push(crate::lower::MatchArm {
            patterns: vec![parse_quote!(true)],
            comp_time_bindings: HashMap::new(),
        });
    }

    LoweredConditionalQueryAs {
        output_type: analyzed.output_type,
        query_string: analyzed.query_string,
        match_expressions,
        match_arms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_syntax() {
        let parsed = syn::parse_str::<crate::parse::ParsedConditionalQueryAs>(
            r#"
                SomeType,
                "some SQL query",
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
        let _lowered = lower(analyzed);
    }
}
