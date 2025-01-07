use proc_macro2::TokenStream;

use crate::DatabaseType;

macro_rules! set_snapshot_suffix {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_suffix(format!($($expr,)*));
        let _guard = settings.bind_to_scope();
    }
}

fn prettyprint(ts: TokenStream) -> String {
    // `prettyplease` operates on full files so we have to wrap the generated output in a dummy
    // function.
    let ts = quote::quote! {
        fn dummy() {
            #ts
        }
    };
    let file = syn::parse2(ts).expect("failed to parse wrapped output as a file");
    prettyplease::unparse(&file)
}

#[rstest::rstest]
#[case::postgres(DatabaseType::PostgreSql)]
#[case::mysql(DatabaseType::MySql)]
#[case::sqlite(DatabaseType::Sqlite)]
fn only_runtime_bound_parameters(#[case] database_type: DatabaseType) {
    set_snapshot_suffix!("{:?}", database_type);
    let input = quote::quote! {
        OutputType,
        r#"
            SELECT column
            FROM table
            WHERE created_at > {created_at}
        "#,
    };
    let output = crate::conditional_query_as(database_type, input).unwrap();
    insta::assert_snapshot!(prettyprint(output));
}

#[rstest::rstest]
#[case::postgres(DatabaseType::PostgreSql)]
#[case::mysql(DatabaseType::MySql)]
#[case::sqlite(DatabaseType::Sqlite)]
fn only_compile_time_bound_parameters(#[case] database_type: DatabaseType) {
    set_snapshot_suffix!("{:?}", database_type);
    let hash = proc_macro2::Punct::new('#', proc_macro2::Spacing::Alone);
    let input = quote::quote! {
        OutputType,
        r#"
            SELECT column
            FROM table
            WHERE value = {#value}
        "#,
        #hash value = match value {
            _ => "value",
        },
    };
    let output = crate::conditional_query_as(database_type, input).unwrap();
    insta::assert_snapshot!(prettyprint(output));
}

#[rstest::rstest]
#[case::postgres(DatabaseType::PostgreSql)]
#[case::mysql(DatabaseType::MySql)]
#[case::sqlite(DatabaseType::Sqlite)]
fn both_parameter_kinds(#[case] database_type: DatabaseType) {
    set_snapshot_suffix!("{:?}", database_type);
    let hash = proc_macro2::Punct::new('#', proc_macro2::Spacing::Alone);
    let input = quote::quote! {
        OutputType,
        r#"
            SELECT column
            FROM table
            WHERE
                created_at > {created_at}
                AND value = {#value}
        "#,
        #hash value = match value {
            _ => "value",
        },
    };
    let output = crate::conditional_query_as(database_type, input).unwrap();
    insta::assert_snapshot!(prettyprint(output));
}
