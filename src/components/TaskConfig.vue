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
              v-if="config.type !== 'checkbox'"
              class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300"
            >
              {{ config.label }}
            </label>
            <div class="flex items-center gap-3">
              <input
                v-if="config.type === 'text'"
                type="text"
                v-model="config.value"
                @blur="handleConfigChange"
                class="block w-full rounded-md border-gray-300 shadow-sm transition-colors duration-200 focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-white"
              />
              <select
                v-if="config.type === 'select'"
                v-model="config.value"
                @change="handleConfigChange"
                class="block w-full rounded-md border-gray-300 shadow-sm transition-colors duration-200 focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-white"
              >
                <option
                  v-for="option in config.options"
                  :key="option"
                  :value="option"
                >
                  {{ config.displayMap ? config.displayMap[option] : option }}
                </option>
              </select>
              <div
                v-if="config.type === 'checkbox'"
                class="flex items-center gap-2 py-1"
              >
                <input
                  type="checkbox"
                  :id="config.label"
                  v-model="config.value"
                  @change="handleConfigChange"
                  class="h-4 w-4 rounded border-gray-300 text-indigo-600 transition-colors duration-200 focus:ring-indigo-600 dark:border-gray-600"
                />
                <label
                  :for="config.label"
                  class="cursor-pointer text-sm text-gray-700 select-none dark:text-gray-300"
                >
                  {{ config.label }}
                </label>
              </div>
              <div v-if="config.type === 'multiSelect'" class="space-y-2">
                <div
                  v-for="option in config.options"
                  :key="option"
                  class="flex items-center"
                >
                  <input
                    type="checkbox"
                    :id="option"
                    :value="option"
                    v-model="selectedValues"
                    @change="updateValue"
                    class="h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-600"
                  />
                  <label :for="option" class="ml-2 text-sm text-gray-600">
                    {{ config.displayMap ? config.displayMap[option] : option }}
                  </label>
                </div>
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
import { computed, ref, watch } from 'vue'

import { useTaskState } from '../composables/useTaskState'

const {
  currentTask,
  taskConfigs,
  isRunning,
  toggleTask,
  updateTaskConfig,
  navigation,
} = useTaskState()

const currentConfigs = computed(() => taskConfigs[currentTask.value])

const selectedValues = ref([])

const updateValue = () => {
  if (currentConfigs.value.some((config) => config.type === 'multiSelect')) {
    currentConfigs.value.forEach((config) => {
      if (config.type === 'multiSelect') {
        config.value = selectedValues.value
      }
    })
    handleConfigChange()
  }
}

const handleConfigChange = async () => {
  const taskItem = navigation.value.find(
    (item) => item.name === currentTask.value,
  )
  if (taskItem) {
    try {
      await updateTaskConfig(taskItem.checked)
    } catch (error) {
      console.error('更新任务配置失败:', error)
    }
  }
}

watch(
  () => currentConfigs.value.map((config) => config.value),
  (newValues) => {
    currentConfigs.value.forEach((config, index) => {
      if (config.type === 'multiSelect') {
        selectedValues.value = Array.isArray(newValues[index])
          ? newValues[index]
          : []
      }
    })
  },
  { immediate: true },
)
</script>
