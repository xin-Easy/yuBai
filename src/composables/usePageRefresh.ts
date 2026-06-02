import { onBeforeUnmount, onMounted } from 'vue'

export const PAGE_REFRESH_EVENT = 'yubai:page-refresh'

export function requestPageRefresh() {
  window.dispatchEvent(new CustomEvent(PAGE_REFRESH_EVENT))
}

export function usePageRefresh(handler: () => void | Promise<void>) {
  const listener = () => {
    void handler()
  }

  onMounted(() => {
    window.addEventListener(PAGE_REFRESH_EVENT, listener)
  })

  onBeforeUnmount(() => {
    window.removeEventListener(PAGE_REFRESH_EVENT, listener)
  })
}
