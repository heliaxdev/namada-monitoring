# Metrics implemented in namada monitoring

| **Metric Name**                 | **Description** |
|---------------------------------|------------------------------------------------------|
| `namada_block_height`           | Tracks the latest block height of the Namada blockchain. |
| `epoch`                         | Tracks the latest epoch recorded on the blockchain. |
| `total_supply_native_token`     | Monitors the total supply of Namada's native token. |
| `bonds_per_epoch`               | Measures the total amount of tokens bonded per epoch. |
| `unbonds_per_epoch`             | Measures the total amount of tokens unbonded per epoch. |
| `transaction_batch_size`        | Tracks the distribution of transactions within a batch. |
| `transaction_kind`              | Counts the number of transactions by type per epoch. |
| `transfer_amount`               | Tracks the total transfer amount per token and epoch. |
| `one_third_threshold`           | The number of validators needed to reach 1/3 voting power. |
| `two_third_threshold`           | The number of validators needed to reach 2/3 voting power. |


## Block Height Counter (namada_block_height)

This metric tracks the latest block height of the Namada blockchain. It provides a real-time view of block progression, and helps monitor chain liveness and ensure continuous block production.

* The metric is a monotonic counter that increments as new blocks are added to the chain.
* It is updated at each block by fetching the latest block height from the blockchain state.

### Example
```
# HELP namada_block_height the latest block height recorded
# TYPE namada_block_height counter
namada_block_height{chain_id="$CHAINID"} 12960
```


## Bonding Activity Metrics (bonds_per_epoch, unbonds_per_epoch)

These metrics track the number of bonds and unbonds per epoch in the Namada blockchains. They help monitor staking activity, validator participation, and network security dynamics. These metrics are gauges, updated at the start of each epoch based on the blockchain state.


* bonds_per_epoch: Measures the total amount of tokens bonded in a given epoch.
* unbonds_per_epoch: Measures the total amount of tokens unbonded in a given epoch.


### Example
```
# HELP namada_bonds_per_epoch Total bonds per epoch
# TYPE namada_bonds_per_epoch gauge
namada_bonds_per_epoch{epoch="2160",chain_id="local.300e84e1e16080e34547d538"} 120000000000
# HELP namada_unbonds_per_epoch Total unbonds per epoch
# TYPE namada_unbonds_per_epoch gauge
namada_unbonds_per_epoch{epoch="2160",chain_id="local.300e84e1e16080e34547d538"} 0
```


## Epoch Counter (epoch)

This metric tracks the latest epoch recorded on the Namada blockchain, providing visibility into epoch progression and chain activity over time.

### Usage and Interpretation
	•	A steadily increasing counter indicates normal epoch progression.
	•	Stagnation or irregular jumps may signal network disruptions, delayed finalization, or protocol changes.
	•	Analysts can use this metric to detect anomalies in epoch transitions and ensure expected blockchain activity.

* epoch: A monotonic counter that increments as new epochs are finalized.

### Example
```
# HELP epoch The latest epoch recorded  
# TYPE epoch counter  
epoch 256 
```

## Total Supply of Native Token (total_supply_native_token)

This metric tracks the total supply of the native token on the Namada blockchain. Namada blockchain mints NAM tokens as rewards for validators and delegators who secure the network through Cubic Proof-of-Stake. Users also earn NAM tokens for shielding assets and contributing to shared data protection14. The total supply of NAM tokens will be 1 billion

### Usage and Interpretation
	•	An increasing supply may indicate token minting due to staking rewards or protocol inflation.
	•	A stable or decreasing supply could suggest burning mechanisms, slashing events, or governance changes.
	•	Monitoring this metric helps track the economic policies of the blockchain.

* total_supply_native_token: A monotonic counter that records the latest total supply of the native token.


### Example
```
# HELP total_supply_native_token The latest total supply of the native token recorded  
# TYPE total_supply_native_token counter  
total_supply_native_token 1000000000  
```


## Transaction Metrics

This set of metrics tracks transaction activity in the Namada blockchain, capturing both batch sizes and transaction kinds per epoch. These metrics help monitor network load, transaction diversity, and failure rates.

### transaction_batch_size (Histogram)

Measures the number of inner transactions within a batch. Provides insights into how transactions are grouped and processed. Uses predefined buckets (1, 2, 4, 8, ..., 256) 

#### Usage and Interpretation
TODO! No idea what should be normal here ???
•	A skewed distribution in transaction_batch_size may suggest inefficient batch processing. ?

### Example
```
# HELP transaction_batch_size The number of inner transactions in the batch
# TYPE transaction_batch_size histogram
transaction_batch_size_bucket{le="1"} 5
transaction_batch_size_bucket{le="2"} 10
transaction_batch_size_bucket{le="4"} 20
transaction_batch_size_bucket{le="8"} 30
transaction_batch_size_bucket{le="16"} 40
transaction_batch_size_bucket{le="32"} 50
transaction_batch_size_bucket{le="64"} 60
transaction_batch_size_bucket{le="128"} 70
transaction_batch_size_bucket{le="256"} 80
transaction_batch_size_count 80
transaction_batch_size_sum 3200
```

### transaction_kind (CounterVec)

Tracks the count of different transaction types per epoch.

#### Labels:
	- kind: The specific type of transaction:
        - transfer: Standard token transfer.
        - ibc_transfer: Cross-chain transfer via IBC.
        - bond/unbond/redelegate: Staking-related actions.
        - claim_rewards/withdraw: Reward and withdrawal operations.
        - vote_proposal/init_proposal: Governance voting and proposal creation.
        - metadata_change/commission_change”: Validator updates.
        - reveal_public_key: Public key revelation.
        - become_validator/deactivate_validator/reactivate_validator/unjail_validator: Validator lifecycle actions.
	- epoch: The epoch in which the transaction was included.
	- failed: A boolean (true/false) indicating if the transaction failed.
	
### Usage and Interpretation
	•	A steady count in transaction_kind indicates normal blockchain activity.
	•	A high failure rate (failed = "true") should be considered abnormal.

### Example
```
# HELP transaction_kind Transaction kind count per epoch
# TYPE transaction_kind counter
transaction_kind{kind="transfer", epoch="256", failed="false"} 120
transaction_kind{kind="bond", epoch="256", failed="false"} 15
transaction_kind{kind="vote_proposal", epoch="256", failed="true"} 3
```

## Voting Power Metrics (one_third_threshold, two_third_threshold)

These metrics track the number of validators required to reach 1/3 and 2/3 of the total voting power. They provide insight into consensus formation and validator distribution in the Namada blockchain.

### Labels:
  - one_third_threshold: The number of validators needed to reach 1/3 of the voting power.
  - two_third_threshold: The number of validators needed to reach 2/3 of the voting power.

### Usage and Interpretation
  - If the `one_third_threshold` is low, a small number of validators hold significant influence, potentially increasing centralization risks.
  - If the `two_third_threshold` is low, it may indicate that a small group can finalize blocks quickly, but also raises concerns about validator distribution.
  - These metrics help assess network decentralization and validator power concentration.

### Example
```
# HELP one_third_threshold Number of validators to reach 1/3 of the voting power
# TYPE one_third_threshold gauge
one_third_threshold 5

# HELP two_third_threshold Number of validators to reach 2/3 of the voting power
# TYPE two_third_threshold gauge
two_third_threshold 12
```

