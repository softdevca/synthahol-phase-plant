#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Metadata {
    pub name: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,

    // TODO: Should it be categories: Vec<String>?
    /// Category such as "bass" or "keys"
    pub category: Option<String>,
}
