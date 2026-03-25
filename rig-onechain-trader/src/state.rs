pub struct State<'a> {
    pub account_id: &'a str,
    // Add other state fields
}

impl<'a> State<'a> {
    pub fn new(account_id: &'a str) -> Self {
        Self { account_id }
    }
} 