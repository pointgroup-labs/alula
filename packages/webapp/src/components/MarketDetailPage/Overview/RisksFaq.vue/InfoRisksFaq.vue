<script lang="ts" setup>

</script>

<template>
  <div class="info-risks-faq">
    <j-accordion>
      <template #title>
        Liquidation Risk
      </template>
      <div class="risk-content">
        <p><strong>What is liquidation risk?</strong></p>
        <p>When you borrow assets, your collateral can be liquidated if its value drops below the required collateral ratio (Close LTV). This means part of your collateral will be sold to repay your debt.</p>

        <p><strong>How to minimize:</strong></p>
        <ul>
          <li>Maintain a healthy collateral ratio well above the liquidation threshold</li>
          <li>Monitor asset prices regularly, especially volatile tokens</li>
          <li>Consider adding more collateral or repaying debt when approaching liquidation</li>
          <li>Use stable assets as collateral when possible</li>
        </ul>

        <p><strong>Liquidation penalty:</strong> You'll lose a portion of your collateral as a penalty to liquidators (typically 5-10%).</p>
      </div>
    </j-accordion>

    <j-accordion>
      <template #title>
        Smart Contract Risk
      </template>
      <div class="risk-content">
        <p><strong>What is smart contract risk?</strong></p>
        <p>Smart contracts may contain bugs, vulnerabilities, or unexpected behavior that could lead to loss of funds. Even audited contracts can have undiscovered flaws.</p>

        <p><strong>Potential issues:</strong></p>
        <ul>
          <li>Code vulnerabilities allowing unauthorized access</li>
          <li>Oracle manipulation attacks affecting price feeds</li>
          <li>Governance attacks changing protocol parameters</li>
          <li>Flash loan attacks exploiting protocol logic</li>
          <li>Upgrades introducing new bugs or malicious code</li>
        </ul>

        <p><strong>Risk mitigation:</strong></p>
        <ul>
          <li>Protocol has undergone multiple security audits</li>
          <li>Gradual scaling with TVL caps</li>
          <li>Time-locked governance for major changes</li>
          <li>Bug bounty programs for continuous security research</li>
        </ul>
      </div>
    </j-accordion>

    <j-accordion>
      <template #title>
        Market Risk
      </template>
      <div class="risk-content">
        <p><strong>What is market risk?</strong></p>
        <p>Asset prices can be highly volatile, affecting both your collateral value and debt obligations. Market conditions can change rapidly, especially in DeFi.</p>

        <p><strong>Key risks:</strong></p>
        <ul>
          <li><strong>Volatile collateral:</strong> Sharp price drops can trigger liquidations</li>
          <li><strong>Interest rate changes:</strong> Variable rates can increase borrowing costs</li>
          <li><strong>Correlation risk:</strong> Borrowed and collateral assets moving in same direction</li>
          <li><strong>Market manipulation:</strong> Large trades or attacks affecting asset prices</li>
        </ul>

        <p><strong>Market dynamics:</strong></p>
        <ul>
          <li>High utilization periods may cause borrowing rates to spike</li>
          <li>Low liquidity can make it difficult to exit positions</li>
          <li>External market events can cause rapid price movements</li>
        </ul>
      </div>
    </j-accordion>

    <j-accordion>
      <template #title>
        Liquidity Risk
      </template>
      <div class="risk-content">
        <p><strong>What is liquidity risk?</strong></p>
        <p>You might not be able to withdraw your supplied assets or obtain loans when the pool has insufficient liquidity. This can happen during market stress or high demand periods.</p>

        <p><strong>When liquidity issues occur:</strong></p>
        <ul>
          <li>High utilization rates (most assets are borrowed)</li>
          <li>Bank runs during market panic</li>
          <li>Large withdrawals by major suppliers</li>
          <li>Sudden increase in borrowing demand</li>
        </ul>

        <p><strong>Protocol safeguards:</strong></p>
        <ul>
          <li>Interest rate models incentivize repayments during high utilization</li>
          <li>Utilization rate caps prevent complete pool depletion</li>
          <li>Reserve factors maintain protocol liquidity buffers</li>
          <li>Withdrawal limits during scarcity periods</li>
        </ul>

        <p class="warning">⚠️ <strong>Note:</strong> During extreme market conditions, withdrawals may be temporarily limited or delayed.</p>
      </div>
    </j-accordion>

    <j-accordion>
      <template #title>
        Regulatory Risk
      </template>
      <div class="risk-content">
        <p><strong>What is regulatory risk?</strong></p>
        <p>Changing regulations or government policies could affect the protocol's operations, token values, or your ability to access DeFi services.</p>

        <p><strong>Potential regulatory impacts:</strong></p>
        <ul>
          <li>Restrictions on DeFi protocol usage in certain jurisdictions</li>
          <li>Requirements for KYC/AML compliance</li>
          <li>Tax implications on DeFi activities</li>
          <li>Restrictions on specific cryptocurrencies</li>
          <li>Changes in stablecoin regulations affecting collateral assets</li>
        </ul>

        <p><strong>Stay informed:</strong></p>
        <ul>
          <li>Monitor regulatory developments in your jurisdiction</li>
          <li>Consult with legal and tax professionals</li>
          <li>Understand your local laws regarding DeFi participation</li>
          <li>Keep records of all transactions for potential reporting requirements</li>
        </ul>
      </div>
    </j-accordion>

    <j-accordion>
      <template #title>
        Operational Risk
      </template>
      <div class="risk-content">
        <p><strong>What is operational risk?</strong></p>
        <p>Risks related to protocol operations, infrastructure, and external dependencies that could affect normal functioning.</p>

        <p><strong>Key operational risks:</strong></p>
        <ul>
          <li><strong>Oracle failures:</strong> Price feed issues causing incorrect liquidations</li>
          <li><strong>Network congestion:</strong> High gas fees or transaction delays</li>
          <li><strong>Infrastructure downtime:</strong> Frontend or backend service interruptions</li>
          <li><strong>Governance issues:</strong> Poor decision-making by token holders</li>
          <li><strong>Team risk:</strong> Key team members leaving or protocol abandonment</li>
        </ul>

        <p><strong>Dependencies:</strong></p>
        <ul>
          <li>Stellar network stability and performance</li>
          <li>Oracle provider reliability (price feeds)</li>
          <li>Frontend hosting and infrastructure</li>
          <li>Third-party integrations and APIs</li>
        </ul>

        <p class="info">💡 <strong>Tip:</strong> Always have backup plans and diversify across multiple protocols.</p>
      </div>
    </j-accordion>
  </div>
</template>

<style scoped>
.info-risks-faq {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.risk-content {
  padding: 16px 0;
  line-height: 1.6;
}

.risk-content p {
  margin-bottom: 12px;
  color: #333;
}

.risk-content p:last-child {
  margin-bottom: 0;
}

.risk-content strong {
  color: #111;
  font-weight: 600;
}

.risk-content ul {
  margin: 8px 0 16px 0;
  padding-left: 20px;
}

.risk-content li {
  margin-bottom: 6px;
  color: #555;
}

.risk-content li:last-child {
  margin-bottom: 0;
}

.warning {
  background: #fef3cd;
  border-left: 4px solid #ffc107;
  padding: 12px 16px;
  border-radius: 4px;
  margin: 16px 0;
}

.info {
  background: #e7f3ff;
  border-left: 4px solid #0066cc;
  padding: 12px 16px;
  border-radius: 4px;
  margin: 16px 0;
}

.warning strong,
.info strong {
  color: inherit;
}
</style>
