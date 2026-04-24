export const VAULT_INFO = {
  title: 'Vault Info',
  shortDesciption: `Multiply uses one-click looping with a flash loan to boost your yield. Choose a multiplier to set leverage.
            Higher multiplier means higher APY and higher liquidation risk. You can reduce or close the position at any
            time.`,
  description: `
            The Multiply vault enables leveraged exposure to the selected collateral asset through an automated looping strategy. 
            When a position is opened, the vault supplies your collateral, borrows the debt asset against it, and automatically 
            reinvests the borrowed funds to increase your total exposure.
            <br>
            <br>
            Leverage is achieved using a flash loan to efficiently establish the position in a single transaction, eliminating the 
            need for manual borrowing and redepositing. This allows users to access amplified staking or lending yields while 
            maintaining a streamlined experience.
            <br>
            <br>
            Your position remains fully non-custodial. You retain control and can reduce or close the position at any time, subject 
            to network conditions and available liquidity.
            <br>
            <br>
            Higher leverage increases both potential yield and liquidation risk. If the value of your collateral declines or the 
            borrowed position grows relative to it, your loan-to-value (LTV) ratio will rise. Positions approaching the liquidation 
            threshold may be partially or fully liquidated to repay outstanding debt and protect the protocol.
            <br>
            <br>
            Risk levels depend on market volatility, interest rates, and oracle pricing. It is strongly recommended to maintain a 
            healthy buffer below the liquidation threshold and actively monitor your position when using higher multipliers.
            <br>
            <br>
            This vault is designed for users seeking capital-efficient strategies while understanding the risks associated with 
            leveraged positions.
            `,
}
