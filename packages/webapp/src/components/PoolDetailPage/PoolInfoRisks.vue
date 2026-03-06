<script lang="ts" setup>
const data = [
  {
    title: 'How the market works',
    desc: `This market allows users to supply USDC to earn yield or borrow USDC against deposited collateral.

Interest paid by borrowers is distributed to suppliers. Rates adjust automatically depending on the utilization of the pool — as more liquidity is borrowed, borrowing rates increase and supplier yield grows.`,
  },
  {
    title: 'Interest rate model',
    desc: `Interest rates in this market are dynamic and depend on pool utilization.

When liquidity is widely available, borrowing rates remain relatively low. As utilization increases and liquidity becomes scarce, borrowing costs rise to encourage new deposits and restore balance to the market.`,
  },
  {
    title: 'Collateral & borrowing limits',
    desc: `To borrow assets, you must first deposit collateral.

Each asset has defined parameters such as Open LTV and Close LTV. These values determine how much you can borrow relative to the value of your collateral and define the safety threshold of your position.`,
  },
  {
    title: 'Liquidation risk',
    desc: `If the value of your collateral decreases or your borrowed amount grows beyond safe limits, your position may become eligible for liquidation.

During liquidation, a portion of your collateral can be sold to repay the borrowed assets. Maintaining a healthy position by monitoring your health factor can help reduce liquidation risk.`,
  },
  {
    title: 'Liquidity risk',
    desc: `Borrowing availability depends on the amount of liquidity supplied to the pool.

During periods of high demand for borrowing, available liquidity may decrease and borrowing rates may increase significantly.`,
  },
  {
    title: 'Smart contract risk',
    desc: `This protocol operates through smart contracts on the blockchain.

While the contracts are designed to be secure and tested, interacting with decentralized finance protocols always carries inherent technical risks.`,
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
