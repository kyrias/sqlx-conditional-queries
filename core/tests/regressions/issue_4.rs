#[test]
fn regression_test_() {
    let input: proc_macro2::TokenStream = r##"
            SomeType,
            r#"{#a}______{c}"#,
            #a = match _ {
                _ => "b",
            },
        "##
    .parse()
    .unwrap();

    sqlx_conditional_queries_core::conditional_query_as(input).unwrap();
}
