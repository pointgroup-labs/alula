import 'bootstrap-vue-next'

declare module 'node_modules/bootstrap-vue-next/dist/src/types/ColorTypes' {
  // eslint-disable-next-line ts/consistent-type-definitions
  export interface BaseColorVariant {
    ['dark']: unknown
    ['outline-purple']: unknown
    gray: unknown
    accent: unknown
    ghost: unknown
    blue: unknown
    purple: unknown
    cyan: unknown
  }
}

declare module 'node_modules/bootstrap-vue-next/dist/src/types/Size' {
  // eslint-disable-next-line ts/consistent-type-definitions
  export interface BaseSize {
    xs: unknown
  }
}
