use super::{AppConfig, CheckTrait, State};
use std::collections::HashMap;

type Token = String;
struct DefaultFeeThreshold {
    name: String,
    value: f64,
}

pub struct FeeCheck {
    explorer: String,
    thresholds: HashMap<Token, DefaultFeeThreshold>,
}

impl Default for FeeCheck {
    fn default() -> Self {
        Self {
            explorer: "https://explorer75.org/namada/tx/".to_string(),
            thresholds: HashMap::new(),
        }
    }
}

impl FeeCheck {
    fn populate_thresholds(&mut self, chain_id: &str) {
        if chain_id == "campfire-square.ff09671d333707" {
            self.thresholds.insert(
                "tnam1qy4pd2j2wkp34c49epd5wy9ny83qsedekgac6gyr".to_string(),
                DefaultFeeThreshold {
                    name: "apfel".to_string(),
                    value: 0.1,
                },
            );
            self.thresholds.insert(
                "tnam1qy4u69pe54hyssg9g42equq0z2vrj9rlnsrfcu6l".to_string(),
                DefaultFeeThreshold {
                    name: "btc".to_string(),
                    value: 0.1,
                },
            );
            self.thresholds.insert(
                "tnam1qyzv6anc548dyj0nqvezrxxd6679d0a02y78k3xx".to_string(),
                DefaultFeeThreshold {
                    name: "dot".to_string(),
                    value: 0.1,
                },
            );
            self.thresholds.insert(
                "tnam1q9046ls453j29xp0g90vm05dpped9adweyjnplkl".to_string(),
                DefaultFeeThreshold {
                    name: "eth".to_string(),
                    value: 0.1,
                },
            );
            self.thresholds.insert(
                "tnam1q982u50dxneydrlne6nfhrcwxc5mlxtpssjjdp3q".to_string(),
                DefaultFeeThreshold {
                    name: "kartoffel".to_string(),
                    value: 0.1,
                },
            );
            self.thresholds.insert(
                "tnam1qy440ynh9fwrx8aewjvvmu38zxqgukgc259fzp6h".to_string(),
                DefaultFeeThreshold {
                    name: "nam".to_string(),
                    value: 0.1,
                },
            );
            self.thresholds.insert(
                "tnam1qxkdfqv2shgyllcf7dq5qlvf8gt6a2kr0s33ye26".to_string(),
                DefaultFeeThreshold {
                    name: "schnitzel".to_string(),
                    value: 0.1,
                },
            );
            self.thresholds.insert(
                "tnam1phks0geerggjk96ezhxclt6r5tdgu3usa5zteyyc".to_string(),
                DefaultFeeThreshold {
                    name: "transfer/channel-0/uosmo".to_string(),
                    value: 0.05,
                },
            );
            //https://github.com/Luminara-Hub/namada-ecosystem/blob/main/user-and-dev-tools/testnet/housefire/explorers.json
            self.explorer = "https://explorer75.org/namada-campfire/tx".to_string();
        }

        if chain_id == "housefire-alpaca.cc0d3e0c033be" {
            self.thresholds.insert(
                "tnam1q9gr66cvu4hrzm0sd5kmlnjje82gs3xlfg3v6nu7".to_string(),
                DefaultFeeThreshold {
                    name: "nam".to_string(),
                    value: 0.0,
                },
            );
            self.thresholds.insert(
                "tnam1phks0geerggjk96ezhxclt6r5tdgu3usa5zteyyc".to_string(),
                DefaultFeeThreshold {
                    name: "transfer/channel-0/uosmo".to_string(),
                    value: 0.0,
                },
            );
            self.thresholds.insert(
                "tnam1phzvlar06m0rtjjv7n8qx8ny8j8aexayhyq98r7s".to_string(),
                DefaultFeeThreshold {
                    name: "transfer/channel-1/uatom".to_string(),
                    value: 0.0,
                },
            );
            self.thresholds.insert(
                "tnam1phdf4sns3dx653kjfeejgymnehxg2z7xgs4z956n".to_string(),
                DefaultFeeThreshold {
                    name: "transfer/channel-10/utia".to_string(),
                    value: 0.02,
                },
            );
            self.thresholds.insert(
                "tnam1pk22zc02efq85wvgnu6q3zfe07sz828p35xntldz".to_string(),
                DefaultFeeThreshold {
                    name: "transfer/channel-4/stuatom".to_string(),
                    value: 0.0,
                },
            );
            self.thresholds.insert(
                "tnam1p4ak7rgnqatppd0hjnfsvu7dray8twf0sv2rvq3f".to_string(),
                DefaultFeeThreshold {
                    name: "transfer/channel-4/stuosmo".to_string(),
                    value: 0.0,
                },
            );
            self.thresholds.insert(
                "tnam1ph4d4cdwu3tvj8rj6n75lrp3q0pg0yym7gpf75az".to_string(),
                DefaultFeeThreshold {
                    name: "transfer/channel-4/stutia".to_string(),
                    value: 0.0,
                },
            );
            self.thresholds.insert(
                "tnam1p4r2835fw404zme26y88uxex8lnp5rdv4s9yjtu7".to_string(),
                DefaultFeeThreshold {
                    name: "transfer/channel-5/utia".to_string(),
                    value: 0.0,
                },
            );
            self.thresholds.insert(
                "tnam1phavrw42dmxuhzzq3fhwagf663ekmf58lqedrqcv".to_string(),
                DefaultFeeThreshold {
                    name: "transfer/channel-7/uosmo".to_string(),
                    value: 0.02,
                },
            );
            self.thresholds.insert(
                "tnam1pkmcvjcruxul6ncyjfp7j24ady2cda5zzvudakty".to_string(),
                DefaultFeeThreshold {
                    name: "transfer/channel-8/stuatom".to_string(),
                    value: 0.02,
                },
            );
            self.thresholds.insert(
                "tnam1p46jfxscmma7le2lswcuwr9dydxlze83wsjdkygq".to_string(),
                DefaultFeeThreshold {
                    name: "transfer/channel-8/stuosmo".to_string(),
                    value: 0.02,
                },
            );
            self.thresholds.insert(
                "tnam1p4jknczaacetxwwe9p49903nml80e9ex0ufqh3kr".to_string(),
                DefaultFeeThreshold {
                    name: "transfer/channel-8/stutia".to_string(),
                    value: 0.02,
                },
            );
            self.thresholds.insert(
                "tnam1p4zuqqd94csj6zv8n0jxylz9kex4vdsgvg3uglw9".to_string(),
                DefaultFeeThreshold {
                    name: "transfer/channel-9/uatom".to_string(),
                    value: 0.02,
                },
            );
            self.explorer =
                "https://namada-explorer.sproutstake.space/test/transactions".to_string();
        }

        if chain_id == "namada.5f5de2dd1b88cba30586420" {
            self.thresholds.insert(
                "tnam1q9gr66cvu4hrzm0sd5kmlnjje82gs3xlfg3v6nu7".to_string(),
                DefaultFeeThreshold {
                    name: "nam".to_string(),
                    value: 0.05,
                },
            );
            self.thresholds.insert(
                "tnam1p5z5538v3kdk3wdx7r2hpqm4uq9926dz3ughcp7n".to_string(),
                DefaultFeeThreshold {
                    name: "transfer/channel-0/stuatom".to_string(),
                    value: 0.05,
                },
            );
            self.thresholds.insert(
                "tnam1p4px8sw3am4qvetj7eu77gftm4fz4hcw2ulpldc7".to_string(),
                DefaultFeeThreshold {
                    name: "transfer/channel-0/stuosmo".to_string(),
                    value: 0.5,
                },
            );
            self.thresholds.insert(
                "tnam1ph6xhf0defk65hm7l5ursscwqdj8ehrcdv300u4g".to_string(),
                DefaultFeeThreshold {
                    name: "transfer/channel-0/stutia".to_string(),
                    value: 0.05,
                },
            );
            self.thresholds.insert(
                "tnam1p5z8ruwyu7ha8urhq2l0dhpk2f5dv3ts7uyf2n75".to_string(),
                DefaultFeeThreshold {
                    name: "transfer/channel-1/uosmo".to_string(),
                    value: 0.5,
                },
            );
            self.thresholds.insert(
                "tnam1pkg30gnt4q0zn7j00r6hms4ajrxn6f5ysyyl7w9m".to_string(),
                DefaultFeeThreshold {
                    name: "transfer/channel-2/uatom".to_string(),
                    value: 0.05,
                },
            );
            self.thresholds.insert(
                "tnam1pklj3kwp0cpsdvv56584rsajty974527qsp8n0nm".to_string(),
                DefaultFeeThreshold {
                    name: "transfer/channel-3/utia".to_string(),
                    value: 0.05,
                },
            );
            self.explorer = "https://explorer75.org/namada/tx".to_string();
        }
    }

    pub fn new(config: &AppConfig) -> Self {
        let mut instance = Self::default();
        instance.populate_thresholds(&config.chain_id.clone());
        instance
    }
}

impl CheckTrait for FeeCheck {
    fn check(&self, states: &[&State]) -> Vec<String> {
        // get lastest state
        let state = states.last().unwrap();
        let block = state.get_last_block();
        let mut alerts = vec![];
        for tx in &block.transactions {
            let amount_per_gas = tx
                .fee
                .amount_per_gas_unit
                .parse::<f64>()
                .unwrap_or_default();
            let gas_limit = tx.fee.gas.parse::<f64>().unwrap_or_default();
            let gas_used = tx.fee.gas_used as f64;

            // If no gas used continue
            if gas_used == 0.0 {
                let summary = format!(
                    "💸 {}  <{}/{}|WrapperTx> with {} inners used ZERO gas units of {}.",
                    if tx.atomic { "Atomic" } else { "" },
                    self.explorer,
                    tx.id,
                    tx.inners.len(),
                    tx.fee.gas_token,
                );
                alerts.push(summary);
                continue;
            }
            // Amount per gas is zero
            if amount_per_gas == 0.0 {
                let summary = format!(
                    "💸 {}  <{}/{}|WrapperTx> with {} inners used ZERO amount per gas units of {}.",
                    if tx.atomic { "Atomic" } else { "" },
                    self.explorer,
                    tx.id,
                    tx.inners.len(),
                    tx.fee.gas_token,
                );
                alerts.push(summary);
                continue;
            }

            if gas_limit < gas_used {
                let summary = format!("💸 {}  <{}/{}|WrapperTx> with {} inners used more gas units than the limit {} > {}.",
                    if tx.atomic { "Atomic" } else { "" },
                    self.explorer, tx.id,
                    tx.inners.len(),
                    gas_used,
                    gas_limit,
                );
                alerts.push(summary);
            }

            // Using the thresholds in self check if any tx paid more than the threshold considering the token matches
            let fee_threshold = self.thresholds.get(&tx.fee.gas_token);
            if fee_threshold.is_none() {
                // not threshold configured for token
                let summary = format!("💸 {}  <{}/{}|WrapperTx> with {} inners paid a total fee of {} {} which is not configured in the alert thresholds.",
                    if tx.atomic { "Atomic" } else { "" },
                    self.explorer, tx.id,
                    tx.inners.len(),
                    gas_used * amount_per_gas,
                    tx.fee.gas_token,
                );
                alerts.push(summary);
                continue;
            }

            let fee_threshold = fee_threshold.unwrap();
            let gas_token_name = fee_threshold.name.clone();
            // total fee / num_inners > 10x default_gas
            // total fee > 60x default_gas
            // num_inners > 10
            let fee = gas_used * amount_per_gas;

            if tx.inners.len() == 1 && fee > 10.0 * fee_threshold.value {
                let summary = format!("💸 {}  <{}/{}|WrapperTx> with a sinle inner tx paid a total fee of {} {} which is more than the alert threshold {} {}.",
                    if tx.atomic { "Atomic" } else { "" },
                    self.explorer, tx.id,
                    fee,
                    gas_token_name,
                    10.0 * fee_threshold.value,
                    gas_token_name
                );
                alerts.push(summary);
            } else if fee / tx.inners.len() as f64 > 10.0 * fee_threshold.value
                && fee > 60.0 * fee_threshold.value
            {
                let summary = format!("💸💸 {}  <{}/{}|WrapperTx> with {} inners paid a total fee of {} {} and average fee of {} {} which is more than both the total threshold {} {} the average threshold {} {}.",
                    if tx.atomic { "Atomic" } else { "" },
                    self.explorer, tx.id,
                    tx.inners.len(),
                    fee,
                    gas_token_name,
                    fee / tx.inners.len() as f64,
                    gas_token_name,
                    10.0 * fee_threshold.value,
                    gas_token_name,
                    60.0 * fee_threshold.value,
                    gas_token_name
                );
                alerts.push(summary);
            } else if fee / tx.inners.len() as f64 > 10.0 * fee_threshold.value {
                let summary = format!("💸 {}  <{}/{}|WrapperTx> with {} inners paid a total fee of {} {} and an average fee of {} {} which is more than the average threshold {} {}",
                    if tx.atomic { "Atomic" } else { "" },
                    self.explorer, tx.id,
                    tx.inners.len(),
                    fee,
                    gas_token_name,
                    fee / tx.inners.len() as f64,
                    gas_token_name,
                    10.0 * fee_threshold.value,
                    gas_token_name
                );
                alerts.push(summary);
            } else if fee > 60.0 * fee_threshold.value {
                let summary = format!("💸 {}  <{}/{}|WrapperTx> with {} inners paid a total fee of {} {} which is more than the total threshold {} {}",
                    if tx.atomic { "Atomic" } else { "" },
                    self.explorer, tx.id,
                    tx.inners.len(),
                    fee,
                    gas_token_name,
                    60.0 * fee_threshold.value,
                    gas_token_name
                );
                alerts.push(summary);
            }
        }
        alerts
    }
}
