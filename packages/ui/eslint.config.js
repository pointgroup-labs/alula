import antfu from '@antfu/eslint-config'
import vue from 'eslint-plugin-vue'

export default antfu(
  {
    gitignore: true,
    stylistic: true,
    typescript: true,
    formatters: true,
    prettier: true,
    unicorn: {
      allRecommended: true,
    },
    yaml: true,
    isInEditor: false,
    lessOpinionated: true,
    ignores: [
      '**/dist/**',
      '**/coverage/**',
    ],
  },
  {
    plugins: {
      vue,
    },
    rules: {
      'antfu/consistent-list-newline': 'off',
      'style/brace-style': ['error', '1tbs', { allowSingleLine: true }],
      'regexp/no-unused-capturing-group': 'off',
      'no-console': 'off',

      // TypeScript specific rules
      'ts/consistent-type-definitions': ['error', 'type'],

      // Vue specific rules
      'vue/component-name-in-template-casing': ['error', 'kebab-case'],
      'vue/max-attributes-per-line': ['error', {
        singleline: 1,
        multiline: 1,
      }],
      'vue/html-closing-bracket-newline': ['error', {
        singleline: 'never',
        multiline: 'always',
      }],
      'vue/html-indent': ['error', 2, {
        attribute: 1,
        baseIndent: 1,
        closeBracket: 0,
        alignAttributesVertically: true,
        ignores: [],
      }],
      'vue/singleline-html-element-content-newline': 'off',
      'vue/multiline-html-element-content-newline': 'off',

      // Node.js specific rules
      'node/prefer-global/process': 'off',
      'node/prefer-global/buffer': 'off',

      // Unicorn plugin rules
      'unicorn/unicorn/prefer-global-this': 'off',
      'unicorn/no-typeof-undefined': 'off',
      'unicorn/expiring-todo-comments': 'off',
      'unicorn/no-abusive-eslint-disable': 'off',
      'unicorn/prevent-abbreviations': 'off',
      'unicorn/no-await-expression-member': 'off',
      'unicorn/no-array-reduce': 'off',
      'unicorn/no-null': 'off',
      'unicorn/switch-case-braces': ['error', 'avoid'],
      'unicorn/filename-case': [
        'error',
        {
          cases: { kebabCase: true, pascalCase: true },
          ignore: [/^[A-Z]+\..*$/], // e.g. README.md
        },
      ],
    },
  },
)
