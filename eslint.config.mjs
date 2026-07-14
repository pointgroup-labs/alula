import antfu from '@antfu/eslint-config'

export default antfu(
  {
    gitignore: true,
    stylistic: true,
    typescript: true,
    yaml: true,
    ignores: [
      '**/dist/**',
      '**/coverage/**',
      '**/Cargo.toml', // sorted + formatted by cargo-sort (see tomlfmt.toml)
    ],
  },
  {
    rules: {
      'antfu/consistent-list-newline': 'off',
      'style/brace-style': ['error', '1tbs', { allowSingleLine: true }],

      'ts/consistent-type-definitions': 'off',
      'ts/consistent-type-imports': 'off',

      'curly': ['error', 'all'],

      'node/prefer-global/process': 'off',
      'node/prefer-global/buffer': 'off',
      'node/no-path-concat': 'off',

      'no-console': 'off',
    },
  },
)
