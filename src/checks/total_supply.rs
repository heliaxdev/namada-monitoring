use super::{CheckTrait, State};

#[derive(Default)]
pub struct TotalSupplyCheck {}

impl CheckTrait for TotalSupplyCheck {
    fn check(&self, states: &[&State]) -> Vec<String> {
        let (pre_state, pos_state) = if let [.., a, b] = states {
            (a, b)
        } else {
            return vec![];
        };

        if pre_state.get_total_supply_native_token() > pos_state.get_total_supply_native_token() {
            return vec![format!(
                "⚠️ Total supply of native token decreased from {} to {}",
                pre_state.get_total_supply_native_token(),
                pos_state.get_total_supply_native_token()
            )];
        }
        vec![]
    }
}
