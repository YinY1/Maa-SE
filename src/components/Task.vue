<template>
  <div class="w-60 border-r border-gray-200 dark:border-gray-700">
    <div class="p-4">
      <nav class="flex flex-1 flex-col" aria-label="Sidebar">
        <ul role="list" class="-mx-2 space-y-1">
          <li v-for="item in navigation" :key="item.name">
            <a
              href="#"
              @click.prevent="selectTask(item.name)"
              :class="[
                item.name === currentTask
                  ? 'bg-gray-50 text-indigo-600 dark:bg-gray-800 dark:text-white'
                  : 'text-gray-700 hover:bg-gray-50 hover:text-indigo-600 dark:text-gray-300 dark:hover:bg-gray-800',
                'group flex items-center gap-x-3 rounded-md p-2 pl-3 text-sm/6 font-semibold',
              ]"
            >
              <input
                type="checkbox"
                v-model="item.checked"
                class="h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-600 dark:border-gray-600 dark:text-white"
                @click.stop
                @change="handleTaskToggle(item)"
              />
              {{ item.name }}
            </a>
          </li>
        </ul>
      </nav>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useTaskState } from '../composables/useTaskState'

const { currentTask, navigation, updateTaskEnable } = useTaskState()

const selectTask = (taskName: string) => {
  currentTask.value = taskName
}

const handleTaskToggle = async (item: { name: string; checked: boolean }) => {
  try {
    // 不改变当前显示的任务
    await updateTaskEnable(item.name, item.checked)
  } catch (error) {
    item.checked = !item.checked
    console.error('切换任务状态失败:', error)
  }
}
</script>
