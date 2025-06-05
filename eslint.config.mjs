import antfu from '@antfu/eslint-config'

export default antfu(
  {
    gitignore: true,
    stylistic: true,
    typescript: true,
    formatters: {
      css: true,
      html: true,
      markdown: 'prettier',
    },
    yaml: true,
    toml: true,
    ignores: [
      '**/dist/**',
      '**/coverage/**',
    ],
  }, {
    rules: {
      'antfu/consistent-list-newline': 'off',
      'style/brace-style': ['error', '1tbs', { allowSingleLine: true }],

      'toml/padding-line-between-pairs': 'off',
      // 'ts/consistent-type-definitions': ['error', 'type'],
      'ts/consistent-type-definitions': 'off',
      'ts/consistent-type-imports': 'off',

      // 'ts/naming-convention': ['error', {
      //     selector: 'variable',
      //     format: ['camelCase', 'UPPER_CASE', 'PascalCase'],
      //     leadingUnderscore: 'forbid',
      //     trailingUnderscore: 'forbid',
      // }],

      'curly': ['error', 'all'],

      'node/prefer-global/process': 'off',
      'node/prefer-global/buffer': 'off',
      'node/no-path-concat': 'off',

      'no-console': 'off',
    },
  },
)
