<template>
  <LeftBar></LeftBar>
  <Task></Task>
  <MainPage></MainPage>
</template>

<script setup lang="ts">
import LeftBar from './components/LeftBar.vue'
import Task from './components/Task.vue'
import MainPage from './views/MainPage.vue'
</script>

<style>
html {
  height: 100%;
  background-color: white;
}

body {
  height: 100%;
  margin: 0;
  font-family:
    -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue',
    Arial, sans-serif;
}
</style>
