use syn::{parenthesized, parse::Parse};

#[derive(Clone, Debug)]
pub(crate) struct ParsedConditionalQueryAs {
    /// This is the equivalent of sqlx's output type in a `query_as!` macro.
    pub(crate) output_type: syn::Ident,
    /// The actual string of the query.
    pub(crate) query_string: syn::LitStr,
    /// All compile time bindings, each with its variables and associated `match` statement.
    pub(crate) compile_time_bindings: Vec<(
        OneOrPunctuated<syn::Ident, syn::token::Comma>,
        syn::ExprMatch,
    )>,
}

/// This enum represents the identifier (`#foo`, `#(foo, bar)`) of single binding expression
/// inside a query.
///
/// Normal statements such as `#foo = match something {...}` are represented by the `One(T)`
/// variant.
///
/// It's also possible to match tuples such as:
/// `#(order_dir, order_dir_rev) = match order_dir {...}`
/// These are represented by the `Punctuated(...)` variant.
#[derive(Clone, Debug)]
pub(crate) enum OneOrPunctuated<T, P> {
    One(T),
    Punctuated(syn::punctuated::Punctuated<T, P>, proc_macro2::Span),
}

impl<T: syn::spanned::Spanned, P> OneOrPunctuated<T, P> {
    pub(crate) fn span(&self) -> proc_macro2::Span {
        match self {
            OneOrPunctuated::One(t) => t.span(),
            OneOrPunctuated::Punctuated(_, span) => *span,
        }
    }
}

impl<T, P> IntoIterator for OneOrPunctuated<T, P> {
    type Item = T;

    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            OneOrPunctuated::One(item) => vec![item].into_iter(),
            OneOrPunctuated::Punctuated(punctuated, _) => {
                punctuated.into_iter().collect::<Vec<_>>().into_iter()
            }
        }
    }
}

impl Parse for ParsedConditionalQueryAs {
    /// Take a given raw token stream from a macro invocation and parse it into our own
    /// representation for further processing.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse the ident of the output type that we're going to pass to `query_as!`.
        let output_type = input.parse::<syn::Ident>()?;
        input.parse::<syn::token::Comma>()?;

        // Parse the actual query string literal.
        let query_string = input.parse::<syn::LitStr>()?;

        // The rest of the input has to be an optional sequence of compile-time binding
        // expressions.
        let mut compile_time_bindings = Vec::new();
        while !input.is_empty() {
            // Every binding expression has to be preceeded by a comma, and we also allow the final
            // comma to be optional.
            input.parse::<syn::token::Comma>()?;
            if input.is_empty() {
                break;
            }

            // Every binding expression starts with a #.
            input.parse::<syn::token::Pound>()?;

            // Then we parse the binding names.
            let binding_names = if input.peek(syn::token::Paren) {
                // If the binding names start with parens we're parsing a tuple of binding names.
                let content;
                let paren_token = parenthesized!(content in input);
                OneOrPunctuated::Punctuated(
                    content.parse_terminated(syn::Ident::parse)?,
                    paren_token.span,
                )
            } else {
                // Otherwise we only parse a single ident.
                let name = input.parse::<syn::Ident>()?;
                OneOrPunctuated::One(name)
            };

            // Binding names and match is delimited by equals sign.
            input.parse::<syn::token::Eq>()?;

            // And finally we parse a match expression.
            let match_expression = input.parse::<syn::ExprMatch>()?;

            compile_time_bindings.push((binding_names, match_expression));
        }

        Ok(ParsedConditionalQueryAs {
            output_type,
            query_string,
            compile_time_bindings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_syntax() {
        let mut parsed = syn::parse_str::<ParsedConditionalQueryAs>(
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

        assert_eq!(
            parsed.output_type,
            syn::Ident::new("SomeType", proc_macro2::Span::call_site()),
        );

        assert_eq!(
            parsed.query_string,
            syn::LitStr::new("some SQL query", proc_macro2::Span::call_site()),
        );

        assert_eq!(parsed.compile_time_bindings.len(), 2);

        {
            let (names, _) = parsed.compile_time_bindings.remove(0);

            assert_eq!(
                names.into_iter().collect::<Vec<_>>(),
                [syn::Ident::new("binding", proc_macro2::Span::call_site())]
            );
        }

        {
            let (names, _) = parsed.compile_time_bindings.remove(0);

            assert_eq!(
                names.into_iter().collect::<Vec<_>>(),
                [
                    syn::Ident::new("a", proc_macro2::Span::call_site()),
                    syn::Ident::new("b", proc_macro2::Span::call_site()),
                ]
            );
        }
    }
}
