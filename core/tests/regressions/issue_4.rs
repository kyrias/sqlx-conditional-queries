use sqlx_conditional_queries_core::DatabaseType;

#[rstest::rstest]
#[case(DatabaseType::PostgreSql, true)]
#[case(DatabaseType::PostgreSql, false)]
#[case(DatabaseType::MySql, true)]
#[case(DatabaseType::MySql, false)]
#[case(DatabaseType::Sqlite, true)]
#[case(DatabaseType::Sqlite, false)]
fn regression_test_(#[case] database_type: DatabaseType, #[case] checked: bool) {
    let input: proc_macro2::TokenStream = r##"
            SomeType,
            r#"{#a}______{c}"#,
            #a = match _ {
                _ => "b",
            },
        "##
    .parse()
    .unwrap();

    sqlx_conditional_queries_core::conditional_query_as(database_type, input, checked).unwrap();
}
