// Define ValueWithExpiry here if it’s not already in commands.rs, or import it
#[derive(Clone)]
pub struct ValueWithExpiry {
    pub value: String,
    pub expiry: Option<u128>,
}

