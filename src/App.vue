<template>
  <div class="flex min-h-screen bg-white dark:bg-gray-900">
    <LeftBar />
    <div class="flex flex-1 pl-20">
      <!-- 使内容不被 LeftBar 覆盖 -->
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
