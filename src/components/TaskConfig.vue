<template>
  <div class="w-96 border-r border-gray-200 dark:border-gray-700">
    <div class="p-6">
      <div class="flex items-center justify-between">
        <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
          {{ currentTask }}
        </h2>
        <button
          @click="toggleTask"
          :class="[
            isRunning
              ? 'bg-red-600 hover:bg-red-700'
              : 'bg-indigo-600 hover:bg-indigo-700',
            'rounded-md px-3 py-1.5 text-sm font-medium text-white transition-colors',
          ]"
        >
          {{ isRunning ? '停止任务' : '开始任务' }}
        </button>
      </div>
      <div class="mt-6 rounded-lg bg-gray-50 p-4 dark:bg-gray-800">
        <div v-if="currentConfigs">
          <div
            v-for="(config, index) in currentConfigs"
            :key="index"
            class="mb-6 last:mb-0"
          >
            <label
              class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300"
            >
              {{ config.label }}
            </label>
            <div class="flex items-center gap-3">
              <input
                v-if="config.type === 'text'"
                type="text"
                v-model="config.value"
                class="block w-full rounded-md border-gray-300 shadow-sm transition-colors duration-200 focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-white"
              />
              <select
                v-if="config.type === 'select'"
                v-model="config.value"
                class="block w-full rounded-md border-gray-300 shadow-sm transition-colors duration-200 focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-white"
              >
                <option
                  v-for="option in config.options"
                  :key="option"
                  :value="option"
                >
                  {{ option }}
                </option>
              </select>
              <div
                v-if="config.type === 'checkbox'"
                class="flex items-center gap-2"
              >
                <input
                  type="checkbox"
                  v-model="config.value"
                  class="h-4 w-4 rounded border-gray-300 text-indigo-600 transition-colors duration-200 focus:ring-indigo-600 dark:border-gray-600"
                />
                <span class="text-sm text-gray-600 dark:text-gray-400"
                  >启用</span
                >
              </div>
              <div
                v-if="config.type === 'checkbox-number'"
                class="flex items-center gap-3"
              >
                <input
                  type="checkbox"
                  v-model="config.value.enabled"
                  class="h-4 w-4 rounded border-gray-300 text-indigo-600 transition-colors duration-200 focus:ring-indigo-600 dark:border-gray-600"
                />
                <input
                  type="number"
                  v-model="config.value.count"
                  min="0"
                  class="block w-20 rounded-md border-gray-300 shadow-sm transition-colors duration-200 focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                />
              </div>
            </div>
          </div>
        </div>
        <div v-else class="text-gray-500 dark:text-gray-400">
          该任务暂无配置项
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, watch } from 'vue'

import { useTaskState } from '../composables/useTaskState'

const { currentTask, taskConfigs, isRunning, toggleTask, updateTaskConfig } =
  useTaskState()

const currentConfigs = computed(() => {
  return taskConfigs[currentTask.value]
})

// 监听配置变化
watch(
  currentConfigs,
  async (newConfigs) => {
    if (newConfigs) {
      await updateTaskConfig(true)
    }
  },
  { deep: true },
)
</script>
