use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct BlockExplorer {
    pub base_url: String,
    pub tx_endpoint: String,
    pub block_endpoint: String,
}

impl BlockExplorer {
    pub fn get_tx_url(&self, tx_hash: &str) -> String {
        format!(
            "{}{}",
            self.base_url,
            self.tx_endpoint.replace("{tx_hash}", tx_hash)
        )
    }

    pub fn get_block_url(&self, block_height: u32) -> String {
        format!(
            "{}{}",
            self.base_url,
            self.block_endpoint
                .replace("{block_height}", &block_height.to_string())
        )
    }
}
