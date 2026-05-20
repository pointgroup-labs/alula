import type { ToastOrchestratorParam } from 'bootstrap-vue-next'
import { useToastController } from 'bootstrap-vue-next'

import AlertToast from '~/features/toast/components/AlertToast.vue'
import DefaultToast from '~/features/toast/components/DefaultToast.vue'
import './assets/styles/toast.scss'

export type ToastAction = {
  label: string
  color?: string
  target?: string
  class?: string
  href?: string
  onClick?: () => void
}

export type AlertToastVariant = 'success' | 'error'

export type ToastProps = {
  body?: string
  title?: string
  alertProps: {
    variant: AlertToastVariant
  }
  actions?: Array<ToastAction>
  noCloseButton?: boolean
} & ToastOrchestratorParam

export function useToast() {
  const { create: createToast } = useToastController()

  /**
   * value @type {number}
   * - `Infinity` — not auto close
   * - `1000` —  close after 1000 ms
   * progressProps
   * @type {Partial<ProgressProps>}
   * { variant: 'success' | 'info' | 'warning' | 'danger' }
   */
  async function create({
    position = 'bottom-start',
    variant = 'success',
    modelValue = 5000,
    toastClass = '',
    noHoverPause = false,
    noCloseButton = false,
    noProgress = true,
    alertProps,
    actions = [],
    ...args
  }: Partial<ToastProps> = {}) {
    const classMap: Record<string, string> = {
      success: 'toast--success',
      info: 'toast--info',
      warning: 'toast--warning',
      danger: 'toast--danger',
    }

    const props = {
      position,
      modelValue: Number(modelValue) > 0 ? modelValue : Infinity,
      toastClass: `j-toast ${toastClass} ${alertProps?.variant ? 'toast-alert' : ''} ${classMap[variant!] || ''}`,
      noHoverPause,
      noCloseButton,
      noProgress,
      variant,
      title: undefined,
      alertProps,
      actions,
      ...args,
    }

    let instance: any

    const onClose = async () => {
      await nextTick()
      instance?.destroy()
    }

    const toastComponent = computed(() => {
      return alertProps && alertProps?.variant ? AlertToast : DefaultToast
    })

    instance = createToast({
      props,
      'component': toastComponent.value,
      'onUpdate:modelValue': onClose,
    })

    return {
      onClose,
      dismiss: async () => {
        instance.set?.({ modelValue: false })
      },
    }
  }

  return {
    create,
  }
}
