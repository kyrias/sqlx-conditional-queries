use sqlx_conditional_queries_core::DatabaseType;

#[rstest::rstest]
#[case(DatabaseType::PostgreSql)]
#[case(DatabaseType::MySql)]
#[case(DatabaseType::Sqlite)]
fn regression_test_(#[case] database_type: DatabaseType) {
    let input: proc_macro2::TokenStream = r##"
            SomeType,
            r#"{#a}______{c}"#,
            #a = match _ {
                _ => "b",
            },
        "##
    .parse()
    .unwrap();

    sqlx_conditional_queries_core::conditional_query_as(database_type, input).unwrap();
}
