import { listen } from '@tauri-apps/api/event'

import { ref } from 'vue'

interface Log {
  message: string
  type: 'info' | 'warning' | 'error'
}

const logs = ref<Log[]>([])

async function setupLogListener() {
  await listen('callback-log', (event: { payload: string }) => {
    logs.value.push({
      message: event.payload,
      type: 'info',
    })
  })
}

function clearLogs() {
  logs.value = []
}

export const useLogState = () => {
  return {
    logs,
    clearLogs,
    setupLogListener,
  }
}
