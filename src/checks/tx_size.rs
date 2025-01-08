use anyhow::anyhow;

#[derive(Clone, Debug, Default)]
pub struct TxSizeCheck {}

impl TxSizeCheck {
    pub async fn run(
        &self,
        pre_state: &crate::state::State,
        post_state: &crate::state::State,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
