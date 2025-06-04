// src/composables/useNotification.js
import { useNotification, NotificationOptions } from 'naive-ui'

type NotifyOptions = Omit<NotificationOptions, 'type'>

export function useNotify() {
  const notification = useNotification()

  const notify = {
    success(options: NotifyOptions) {
      notification.success({
        duration: 2000,
        keepAliveOnHover: true,
        ...options
      })
    },
    error(options: NotifyOptions) {
      notification.error({
        duration: 2000,
        keepAliveOnHover: true,
        ...options
      })
    },
    info(options: NotifyOptions) {
      notification.info({
        duration: 2000,
        keepAliveOnHover: true,
        ...options
      })
    },
    warning(options: NotifyOptions) {
      notification.warning({
        duration: 2000,
        keepAliveOnHover: true,
        ...options
      })
    }
  }

  return notify
}
