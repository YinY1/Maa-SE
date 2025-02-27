<template>
  <div class="flex h-screen overflow-hidden bg-white dark:bg-gray-900">
    <LeftBar />
    <div class="flex h-full flex-1 overflow-hidden pl-20">
      <Task />
      <TaskConfig />
      <LogViewer />
    </div>
  </div>
</template>

<script setup lang="ts">
import LeftBar from './components/LeftBar.vue'
import LogViewer from './components/LogViewer.vue'
import Task from './components/Task.vue'
import TaskConfig from './components/TaskConfig.vue'
</script>
