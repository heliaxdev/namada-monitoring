use crate::state::State;
use anyhow::anyhow;

#[derive(Clone, Debug)]
pub struct TxSizeCheck {
    max_tx_size: u64,
    max_inner_len: u64,
}

impl TxSizeCheck {
    pub fn new(max_tx_size: u64, max_inner_len: u64) -> Self {
        Self {
            max_tx_size,
            max_inner_len,
        }
    }

    pub async fn run(&self, _pre_state: &State, post_state: &State) -> anyhow::Result<()> {
        for tx in &post_state.get_last_block().transactions {
            if tx.inners.len() > self.max_inner_len as usize {
                return Err(anyhow!(
                    "Transaction inner length is too large: {}",
                    tx.inners.len()
                ));
            }
            for inner in &tx.inners {
                if inner.kind.size() > self.max_tx_size as usize {
                    return Err(anyhow!(
                        "Transaction size is too large: {}",
                        inner.kind.size()
                    ));
                }
            }
        }
        Ok(())
    }
}
