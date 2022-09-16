use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct BankAccount {
    pub accountNumber: Option<String>,
}
