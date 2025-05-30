# Metrics implemented in namada monitoring

| **Metric Name**              | **Description**                                            |
| ---------------------------- | ---------------------------------------------------------- |
| `block_height`               | Tracks the latest block height of the Namada blockchain.   |
| `block_time`                 | Tracks the time spent processing a block.                  |
| `bonds`                      | Measures the total amount of tokens bonded per epoch.      |
| `unbonds`                    | Measures the total amount of tokens unbonded per epoch.    |
| `epoch`                      | Tracks the latest epoch recorded on the blockchain.        |
| `fees`                       | Total fees paid per block and per token                    |
| `token_total_supply`         | Monitors the total supply of Namada's native token.        |
| `transaction_kind`           | Counts the number of transactions by type per epoch.       |
| `transfer_amount`            | Tracks the total transfer amount per token and epoch.      |
| `one_third_threshold`        | The number of validators needed to reach 1/3 voting power. |
| `two_third_threshold`        | The number of validators needed to reach 2/3 voting power. |
| `slashes`                    | Count occurring slashes.                                   |
| `block_signatures`           | Count how many signatures there are in a block             |
| `consensus_validators`       | The numnber of validator with state consensus              |
| `jailed_validators`          | The numnber of validator with state jailed                 |
| `inactive_validators`        | The numnber of validator with state inactive               |
| `below_threshold_validators` | The numnber of validator with state below threshold        |
| `below_capacity_validators`  | The numnber of validator with state below capacity         |

## How to add a new metric

1. Add a new metric file to src/metrics/, declare a struct with your metric and implement MetricTRait & Default trait.
2. Consider adding your metric to the default metric set at src/metrcs/mod.rs::MetricsExporter.default_metrics()
3. Update Readme.md to contain your new metric
4. Go use your new metric in an alert or dashboard
