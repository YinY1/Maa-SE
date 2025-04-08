<template>
  <div
    class="fixed inset-y-0 left-0 z-50 block w-20 overflow-y-auto border-r border-gray-200 bg-white pb-4 text-gray-600 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-300"
  >
    <div class="flex h-16 shrink-0 items-center justify-center">
      <img
        class="h-8 w-auto"
        src="https://tailwindui.starxg.com/plus-assets/img/logos/mark.svg?color=indigo&shade=500"
      />
    </div>
    <nav class="mt-8">
      <ul role="list" class="flex flex-col items-center space-y-1">
        <li v-for="item in navigation" :key="item.name">
          <a
            href="#"
            @click.prevent="handleNavClick(item.name.toLowerCase())"
            :class="[
              item.current
                ? 'bg-indigo-100 text-indigo-600 dark:bg-gray-700 dark:text-white'
                : 'text-gray-600 hover:bg-gray-100 hover:text-indigo-600 dark:text-gray-300 dark:hover:bg-gray-700',
              'group flex gap-x-3 rounded-md p-3 text-sm/6 font-semibold',
            ]"
          >
            <component
              :is="item.icon"
              class="size-6 shrink-0"
              aria-hidden="true"
            />
            <span class="sr-only">{{ item.name }}</span>
          </a>
        </li>
      </ul>
    </nav>
  </div>
</template>

<script setup lang="ts">
import { Cog8ToothIcon, HomeIcon, PlayIcon } from '@heroicons/vue/24/outline'

import { type Ref, inject, ref, watch } from 'vue'

const currentView = inject<Ref<string>>('currentView')
const setCurrentView = inject<(view: string) => void>('setCurrentView')

const navigation = ref([
  { name: 'Dashboard', href: '#', icon: HomeIcon, current: true },
  { name: 'Fight', href: '#', icon: PlayIcon, current: false },
  { name: 'Setting', href: '#', icon: Cog8ToothIcon, current: false },
])

if (currentView) {
  watch(currentView, (newView) => {
    navigation.value.forEach((item) => {
      item.current =
        (item.name.toLowerCase() === 'dashboard' && newView === 'main') ||
        (item.name.toLowerCase() === 'setting' && newView === 'settings') ||
        (item.name.toLowerCase() === 'fight' && newView === 'fight')
    })
  })
}

const handleNavClick = (view: string) => {
  if (!setCurrentView) return

  if (view === 'dashboard') {
    setCurrentView('main')
  } else if (view === 'setting') {
    setCurrentView('settings')
  } else if (view === 'fight') {
    setCurrentView('fight')
  }

  navigation.value.forEach((item) => {
    item.current = item.name.toLowerCase() === view
  })
}
</script>
