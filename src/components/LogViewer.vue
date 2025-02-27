<template>
  <div class="flex-1">
    <div class="flex h-full flex-col">
      <div
        class="flex items-center justify-between border-b border-gray-200 p-4 dark:border-gray-700"
      >
        <h2 class="text-lg font-semibold text-gray-900 dark:text-white">
          运行日志
        </h2>
        <button
          @click="clearLogs"
          class="rounded-md bg-indigo-600 px-3 py-1.5 text-sm font-medium text-white transition-colors hover:bg-indigo-700"
        >
          清除日志
        </button>
      </div>
      <div
        ref="logContainer"
        class="flex-1 overflow-auto bg-gray-50 p-4 font-mono dark:bg-gray-800"
      >
        <div class="space-y-1">
          <div
            v-for="(log, index) in logs"
            :key="index"
            class="flex items-start py-0.5"
          >
            <div class="mr-2 shrink-0">
              <i-carbon-information-filled
                v-if="log.type === 'info'"
                class="h-4 w-4 text-blue-500"
              />
              <i-carbon-warning-filled
                v-else-if="log.type === 'warning'"
                class="h-4 w-4 text-yellow-500"
              />
              <i-carbon-error-filled
                v-else-if="log.type === 'error'"
                class="h-4 w-4 text-red-500"
              />
            </div>
            <div class="min-w-0 flex-1">
              <div class="flex items-baseline gap-2">
                <span
                  class="text-xs font-medium tracking-wider whitespace-nowrap text-gray-500 dark:text-gray-400"
                >
                  {{ formatTime(log.time) }}
                </span>
                <span
                  :class="[
                    'text-sm break-all',
                    log.type === 'error'
                      ? 'text-red-600 dark:text-red-400'
                      : log.type === 'warning'
                        ? 'text-yellow-600 dark:text-yellow-400'
                        : 'text-gray-700 dark:text-gray-300',
                  ]"
                  >{{ log.message }}</span
                >
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { nextTick, onMounted, ref, watch } from 'vue'

import { useLogState } from '../composables/useLogState'

const logContainer = ref(null)
const { logs, clearLogs, setupLogListener } = useLogState()

const formatTime = (timestamp) => {
  if (!timestamp) return ''
  return timestamp
}

// 自动滚动到底部
const scrollToBottom = async () => {
  await nextTick()
  if (logContainer.value) {
    logContainer.value.scrollTop = logContainer.value.scrollHeight
  }
}

watch(() => logs.value.length, scrollToBottom)

// 初始化日志监听器
onMounted(async () => {
  await setupLogListener()
  await scrollToBottom()
})
</script>
