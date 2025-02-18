# Metrics implemented in namada monitoring

| **Metric Name**                 | **Description** |
|---------------------------------|------------------------------------------------------|
| `namada_block_height`           | Tracks the latest block height of the Namada blockchain. |
| `block_time`                    | Tracks the time spent processing a block. |
| `bonds_per_epoch`               | Measures the total amount of tokens bonded per epoch. |
| `unbonds_per_epoch`             | Measures the total amount of tokens unbonded per epoch. |
| `epoch`                         | Tracks the latest epoch recorded on the blockchain. |
| `fees`                          | Total fees paid per block and per token                     |
| `peers`	                        | Number of active peers known to the node, labeled by moniker |
| `total_supply_native_token`     | Monitors the total supply of Namada's native token. |
| `transaction_batch_size`        | Tracks the distribution of transactions within a batch. |
| `transaction_kind`              | Counts the number of transactions by type per epoch. |
| `transfer_amount`               | Tracks the total transfer amount per token and epoch. |
| `one_third_threshold`           | The number of validators needed to reach 1/3 voting power. |
| `two_third_threshold`           | The number of validators needed to reach 2/3 voting power. |


## How to add a new metric

1- First add a new metric file to src/metrics/ for example new_metric.rs. Declare a struct with your metrics and implement MetricTRait & Default trait.
2- Consider adding your metric to the default metric set at src/metrcs/mod.rs::MetricsExporter.default_metrics()
3- Update /Readme.md and  src/metrics/README.md to contain your new metric
4- Go use your new metric in an alert or dashboard
