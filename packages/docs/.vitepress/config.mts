import {defineConfig} from '@lando/vitepress-theme-default-plus/config'
import {withMermaid} from 'vitepress-plugin-mermaid'

// Lando's defineConfig is async, so we must await it before wrapping with withMermaid.
// withMermaid expects a resolved config object (not a Promise) so it can properly
// wrap the markdown.config callback and inject the Mermaid Vite plugin.
const baseConfig = await defineConfig({
  title: 'Alula Docs',
  description: 'User guides and tech docs for Alula',

  themeConfig: {
    contributors: false,
    label: false,
    logo: '/logo.svg',
    docFooter: true,
    lastUpdated: false,

    // Disable Theme+ "guide" collection behavior so /guides/ uses normal doc defaults
    collections: {
      guide: {
        patterns: ['__disabled__/**/*.md'], // matches nothing
      },
    },

    search: {
      provider: 'local',
    },
    nav: [
      { text: 'Home', link: '/' },
      { text: 'User Guides & Explainers', link: '/guides/' },
      { text: 'Business & Tech Docs', link: '/tech-docs/' },
      { text: 'API Reference', link: '/api/' },
    ],

    sidebar: {
      '/guides/': [
        { text: 'Start Here', link: '/guides/' },

        {
          text: 'Supply (Earn)',
          link: '/guides/supply-earn/',
          collapsed: true,
          items: [
            { text: 'Key Supply Metrics', link: '/guides/supply-earn/key-supply-metrics' },
            { text: 'How to Supply Assets', link: '/guides/supply-earn/how-to-supply-assets' },
            { text: 'How to Withdraw Supplied Assets', link: '/guides/supply-earn/how-to-withdraw-supplied-assets' },
            { text: 'Supply and Withdrawal Fees', link: '/guides/supply-earn/supply-and-withdrawal-fees' },
            { text: 'Supply-Related Risks', link: '/guides/supply-earn/supply-related-risks' },
          ],
        },

        {
          text: 'Borrow',
          link: '/guides/borrow/',
          collapsed: true,
          items: [
            { text: 'Key Borrow Metrics', link: '/guides/borrow/key-borrow-metrics' },
            { text: 'How to Borrow Assets', link: '/guides/borrow/how-to-borrow-assets' },
            { text: 'How to Repay Borrows', link: '/guides/borrow/how-to-repay-borrows' },
            { text: 'Borrow Fees', link: '/guides/borrow/borrow-fees' },
            { text: 'Borrow-Related Risks', link: '/guides/borrow/borrow-related-risks' },
          ],
        },

        {
          text: 'Multiply',
          link: '/guides/multiply/',
          collapsed: true,
          items: [
            { text: 'Key Multiply Metrics', link: '/guides/multiply/key-multiply-metrics' },
            { text: 'How to Multiply Assets', link: '/guides/multiply/how-to-multiply-assets' },
            { text: 'How to Reduce or Close a Multiply Position', link: '/guides/multiply/how-to-reduce-or-close-a-multiply-position' },
            { text: 'Multiply Fees', link: '/guides/multiply/multiply-fees' },
            { text: 'Multiply-Related Risks', link: '/guides/multiply/multiply-related-risks' },
          ],
        },

        { text: 'FAQ', link: '/guides/faq' },
      ],

      '/tech-docs/': [
        {
          text: 'General Information',
          collapsed: false,
          items: [
            { text: 'Protocol Features', link: '/tech-docs/' },
            { text: 'Business Logic', link: '/tech-docs/general-information/business-logic' },
            { text: 'Use Cases', link: '/tech-docs/general-information/use-cases' },
            { text: 'Upcoming Capabilities', link: '/tech-docs/general-information/upcoming-capabilities' },
          ],
        },

        {
          text: 'Technical Details',
          collapsed: false,
          items: [
            { text: 'Architecture Overview', link: '/tech-docs/technical-details/architecture-overview' },

            {
              text: 'Risk Management',
              link: '/tech-docs/technical-details/risk-management/',
              collapsed: true,
              items: [
                { text: 'Asset Pool', link: '/tech-docs/technical-details/risk-management/asset-pool' },
                { text: 'Insurance Fund', link: '/tech-docs/technical-details/risk-management/insurance-fund' },
                { text: 'Health Factor', link: '/tech-docs/technical-details/risk-management/health-factor' },
                { text: 'Aggregated Price Oracle', link: '/tech-docs/technical-details/risk-management/aggregated-price-oracle' },
                { text: 'Liquidations', link: '/tech-docs/technical-details/risk-management/liquidations' },
                { text: 'Utilization-Based Interest Rates', link: '/tech-docs/technical-details/risk-management/utilization-based-interest-rates' },
                { text: 'Data, Pricing, and Cross-Pool Evaluation', link: '/tech-docs/technical-details/risk-management/data-pricing-and-cross-pool-evaluation' },
                { text: 'Withdrawal Throttles', link: '/tech-docs/technical-details/risk-management/withdrawal-throttles' },
              ],
            },
          ],
        },

        {
          text: 'Deep Dive',
          collapsed: false,
          items: [
            { text: 'Architecture Components', link: '/tech-docs/deep-dive/architecture-components' },
            { text: 'Market Methods', link: '/tech-docs/deep-dive/market-methods' },
            { text: 'Configurable Parameters', link: '/tech-docs/deep-dive/configurable-parameters' },
            { text: 'User Roles and Authorizations', link: '/tech-docs/deep-dive/user-roles-and-authorizations' },
          ],
        },
      ],

      '/api/': [
        { text: 'User Operations', link: '/api/' },
        { text: 'Query Operations', link: '/api/query-operations' },
        { text: 'Admin Operations', link: '/api/admin-operations' },
        { text: 'Miscellaneous', link: '/api/miscellaneous' },
      ],
    },
  },
})

export default withMermaid(baseConfig)
