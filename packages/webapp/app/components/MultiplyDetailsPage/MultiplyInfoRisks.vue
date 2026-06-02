<script lang="ts" setup>
const data = [
  {
    title: 'How the market works',
    desc: `Alula Protocol functions as an institution-ready lending market deployed on the Stellar network, 
    utilizing Soroban smart contracts to manage decentralized liquidity. The architecture is built upon segregated, 
    risk-controlled lending pools, meaning each asset operates within its own dedicated environment and strict limits 
    rather than a monolithic structure. Users can supply assets — ranging from native XLM and stablecoins like USDC 
    and EURC to tokenized Real-World Assets (RWAs) such as sovereign bonds and trade invoices — to earn yield. Supplied 
    assets accrue interest automatically and simultaneously double as collateral to support borrowing operations. 
    The protocol also supports composable atomic batch operations, allowing users to combine multiple actions into a 
    single transaction for complex position management.`,
  },
  {
    title: 'Interest rate model',
    desc: `The cost of borrowing is determined by a Double Kink interest rate model. Rates adjust dynamically based on the 
    pool's utilization ratio. The model establishes an optimal target utilization level for each specific asset. When actual 
    utilization operates below this threshold, borrowing rates remain relatively low to incentivize capital deployment. As 
    utilization approaches or exceeds the stress kinks, the borrowing rate increases sharply — up to a maximum prohibitive APR. 
    This aggressive rate adjustment creates a necessary feedback loop designed to encourage debt repayment and attract new deposits, 
    ensuring liquidity remains available for withdrawals.`,
  },
  {
    title: 'Collateral & borrowing limits',
    desc: `Borrowing capacity is not evaluated in isolation but through Cross-Pool Evaluation. This mechanism allows collateral 
    deposited in one pool to support borrowing in another, enabling multi-asset strategies from a single position. Each supplied 
    asset is governed by an Open LTV parameter, which defines its maximum borrowing power upon opening a position. Conversely, 
    borrowed assets are evaluated using a Liability Factor, a specific risk weight applied to the debt to reflect the asset's 
    volatility and scale its risk. Operations that would result in exceeding these dynamically calculated thresholds are automatically 
    blocked to maintain the safety of the protocol.`,
  },
  {
    title: 'Liquidation risk',
    desc: `A position becomes eligible for liquidation when its health drops below a safe threshold, strictly defined by the Close LTV 
    and the Liquidation Threshold. To minimize market price impact and avoid unnecessary total wipeouts of user positions, liquidations 
    execute in slices governed by the Close Factor. During a liquidation event, third parties can repay a portion of the unhealthy debt 
    and, in return, seize a proportional amount of collateral plus a Liquidation Bonus. This parameterized premium compensates the liquidator, 
    such as a 10% bonus for volatile assets like XLM or a 5% bonus for stablecoins.`,
  },
  {
    title: 'Liquidity risk',
    desc: `High borrowing demand can lead to liquidity scarcity, temporarily restricting suppliers from withdrawing their assets. To protect 
    the protocol from rapid liquidity drains under stress, Alula enforces Withdrawal throttles. Furthermore, the protocol incorporates a Scarcity 
    Multiplier, which acts as a penalty accelerator when a pool approaches depletion. This exponential constraint ensures that remaining liquidity 
    is preserved and properly priced during extreme market conditions, discouraging massive exits while the pool is under stress.`,
  },
  {
    title: 'Protocol & Smart contract risks',
    desc: `Interacting with decentralized finance involves inherent technical risks tied to smart contracts and blockchain infrastructure. Alula 
    employs defense-in-depth mechanisms to mitigate these vectors. An Oracle Circuit Breaker continuously monitors median prices from multiple SEP-40 sources. 
    If anomalous price movements occur within a short window, the circuit breaker automatically pauses borrows, collateral withdrawals, and liquidations 
    until prices stabilize. Additionally, the protocol incorporates an Insolvency Threshold to monitor for severe undercollateralization. If bad debt accrues, 
    an Insurance Fund is designed to absorb the losses before they impact the liquidity providers. All parameter changes must also follow a time-locked governance 
    flow to ensure transparency and predictability.`,
  },
]
</script>

<template>
  <section id="pool-info-risks">
    <j-accordion
      v-for="value in data"
      :key="value.title"
      :title="value.title"
      class="mb-3"
    >
      {{ value.desc }}
    </j-accordion>
  </section>
</template>
