<template>
  <div class="flex h-screen overflow-hidden bg-white dark:bg-gray-900">
    <LeftBar />
    <div class="flex h-full flex-1 overflow-hidden pl-20">
      <template v-if="currentView === 'main'">
        <Task />
        <TaskConfig />
        <LogViewer />
      </template>
      <Settings v-else-if="currentView === 'settings'" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, provide, ref } from 'vue'

import LeftBar from './components/LeftBar.vue'
import LogViewer from './components/LogViewer.vue'
import Settings from './components/Settings.vue'
import Task from './components/Task.vue'
import TaskConfig from './components/TaskConfig.vue'
import { useTaskState } from './composables/useTaskState'

const { loadConfigs } = useTaskState()
const currentView = ref('main')
const setCurrentView = (view: string) => {
  currentView.value = view
}

provide('currentView', currentView)
provide('setCurrentView', setCurrentView)

onMounted(async () => {
  await loadConfigs()
})
</script>
