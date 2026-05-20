// types/bootstrap-vue-next.d.ts
import 'bootstrap-vue-next'

declare module 'bootstrap-vue-next' {
  // eslint-disable-next-line ts/consistent-type-definitions
  export interface BaseColorVariant {
    'dark': unknown
    'outline-purple': unknown

    'gray': unknown
    'accent': unknown
    'ghost': unknown
    'blue': unknown
    'purple': unknown
    'cyan': unknown
    'brand': unknown
    'positive': unknown
    'negative': unknown
    'brand-secondary': unknown

    'outlined-brand': unknown
    'outlined-brand-secondary': unknown
    'outlined-accent': unknown
    'outlined-positive': unknown
    'outlined-negative': unknown

    'brand-outlined': unknown
    'brand-secondary-outlined': unknown
    'accent-outlined': unknown
    'positive-outlined': unknown
    'negative-outlined': unknown
  }

  // eslint-disable-next-line ts/consistent-type-definitions
  export interface BaseSize {
    xs: unknown
  }
}
