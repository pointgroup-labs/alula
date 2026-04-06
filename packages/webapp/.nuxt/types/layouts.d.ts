import _default from "/Users/garniy/pc/work/alula/packages/webapp/src/layouts/default.vue";
import empty from "/Users/garniy/pc/work/alula/packages/webapp/src/layouts/empty.vue";
import type { ComputedRef, MaybeRef } from 'vue'
declare module 'nuxt/app' {
  interface NuxtLayouts {
    'default': InstanceType<typeof _default>['$props'],
    'empty': InstanceType<typeof empty>['$props'],
}
  export type LayoutKey = keyof NuxtLayouts extends never ? string : keyof NuxtLayouts
  interface PageMeta {
    layout?: MaybeRef<LayoutKey | false> | ComputedRef<LayoutKey | false>
  }
}