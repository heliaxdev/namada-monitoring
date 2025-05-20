#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Supply {
    pub total: u64,
    pub effective: u64,
    pub token: String,
}
