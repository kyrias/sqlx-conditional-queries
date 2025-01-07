use std::collections::HashMap;

use itertools::Itertools;
use syn::parse_quote;

use crate::analyze::AnalyzedConditionalQueryAs;

#[derive(Debug)]
pub(crate) struct LoweredConditionalQueryAs {
    pub(crate) output_type: syn::Ident,
    pub(crate) query_string: syn::LitStr,
    /// All expressions that're matched upon.
    /// These expressions are in the same order as the patterns in the `match_arms` field.
    pub(crate) match_expressions: Vec<syn::Expr>,
    pub(crate) match_arms: Vec<MatchArm>,
}

#[derive(Debug)]
pub(crate) struct MatchArm {
    pub(crate) patterns: Vec<syn::Pat>,
    pub(crate) compile_time_bindings: HashMap<String, syn::LitStr>,
}

/// Take all compile time bindings and create the cartesian product between all match statement
/// arms of each binding.
/// This allows us to easily create one gigantic match statement that covers all possible cases in
/// the next step.
pub(crate) fn lower(analyzed: AnalyzedConditionalQueryAs) -> LoweredConditionalQueryAs {
    // Unzip all bindings for easier iteration.
    let (bindings, mut match_expressions): (Vec<_>, Vec<_>) = analyzed
        .compile_time_bindings
        .into_iter()
        .map(|binding| (binding.arms.into_iter(), binding.expression))
        .unzip();

    // This for loop generates all possible permutations of all match arm binding statements.
    // E.g. if there are three match statements:
    // ```
    // let a_t = match a {
    //    a_1 => a_s1
    //    a_2 => a_s2
    // }
    //
    // let b_t = match b {
    //    b_1 => b_s1
    // }
    //
    // let (c_t1, ct2) = match c {
    //    c_1 => (c_s1_1, c_s1_2)
    //    c_2 => (c_s1_2, c_s2_2)
    // }
    // ```
    // This function will generate this cartesian product:
    // `
    //  [
    //      [
    //       ('a_1', [(`a_t`, `a_s1`)]),
    //       ('b_1', [(`b_t`, `b_s1`)]),
    //       ('c_1', [(`c_t1`, `c_s1_1`), (`c_t2`, `c_s1_2`)]),
    //      ],
    //      [
    //       ('a_2', [(`a_t`, `a_s1`)]),
    //       ('b_1', [(`b_t`, `b_s1`)]),
    //       ('c_1', [(`c_t1`, `c_s1_1`), (`c_t2`, `c_s1_2`)]),
    //      ],
    //      [
    //       ('a_1', [(`a_t`, `a_s1`)]),
    //       ('b_1', [(`b_t`, `b_s1`)]),
    //       ('c_2', [(`c_t1`, `c_s2_1`), (`c_t2`, `c_s2_2`)]),
    //      ],
    //      [
    //       ('a_2', [(`a_t`, `a_s1`)]),
    //       ('b_1', [(`b_t`, `b_s1`)]),
    //       ('c_2', [(`c_t1`, `c_s2_1`), (`c_t2`, `c_s2_2`)]),
    //      ],
    //  ]
    //`
    //
    // Note how the order in the product always stays the same!
    // This is an important guarantee we rely on, as this is also the same order the
    // `match_expressions` are in.
    // Due this ordering guarantee, we can lateron assemble the match statements without having to
    // keep track of which match expressions belongs to which part of the match arm's expression.
    let mut match_arms = Vec::new();
    for binding in bindings.into_iter().multi_cartesian_product() {
        // `multi_cartesian_product` returns one empty `Vec` if the iterator was empty.
        if binding.is_empty() {
            continue;
        }

        let mut guards = Vec::new();
        let mut bindings = HashMap::new();
        binding.into_iter().for_each(|(g, b)| {
            guards.push(g);
            bindings.extend(b.into_iter().map(|(name, value)| (name.to_string(), value)));
        });
        match_arms.push(MatchArm {
            patterns: guards,
            compile_time_bindings: bindings,
        });
    }

    // If no compile-time bindings were specified we add an always-true option.
    // TODO: Think about whether we just return earlier in the pipeline and just return the
    // original macro input as a `query_as!` macro.
    if match_expressions.is_empty() {
        match_expressions.push(parse_quote!(()));
        match_arms.push(crate::lower::MatchArm {
            patterns: vec![parse_quote!(())],
            compile_time_bindings: HashMap::new(),
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
