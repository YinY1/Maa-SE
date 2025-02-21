<template>
  <LeftBar />
  <Task />
  <MainPage />
</template>

<script setup lang="ts">
import MainPage from './views/MainPage.vue';
import Task from './components/Task.vue';
import LeftBar from './components/LeftBar.vue';
</script>

<style>
html {
  height: 100%;
  background-color: white;
}

body {
  height: 100%;
  margin: 0;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
}
</style>