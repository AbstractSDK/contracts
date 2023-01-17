extern crate proc_macro2;

use proc_macro::TokenStream;

const DELIMITER: &str = ",";

#[proc_macro]
pub fn abstract_response(input: TokenStream) -> TokenStream {
    let input = input.to_string();
    let mut input = input.split(DELIMITER);
    let contract_name = input.next().unwrap().trim();
    let action = input.next().unwrap().trim();
    // Collect the remaining
    let attrs = input.collect::<Vec<&str>>().join(DELIMITER);

    let attribute_addition = if attrs.is_empty() {
        "".to_string()
    } else {
        format!(".add_attributes(vec!{})", attrs)
    };
    let output = format!(
        "cosmwasm_std::Response::new()
        .add_event(
            cosmwasm_std::Event::new(\"abstract\")
                .add_attributes(vec![(\"contract\", {}), (\"action\", {})])
                {}
        )",
        contract_name, action, attribute_addition
    );

    output.parse().unwrap()
}
