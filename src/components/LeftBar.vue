<template>
  <head>
    <link href="/src/assets/styles/styles.css" rel="stylesheet" />
  </head>
  <div
    class="fixed inset-y-0 left-0 z-50 block w-20 overflow-y-auto bg-gray-900 pb-4"
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
            :href="item.href"
            :class="[
              item.current
                ? 'bg-gray-800 text-white'
                : 'text-gray-400 hover:bg-gray-800 hover:text-white',
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

const navigation = [
  { name: 'Dashboard', href: '#', icon: HomeIcon, current: true },
  { name: 'Fight', href: '#', icon: PlayIcon, current: false },
  { name: 'Setting', href: '#', icon: Cog8ToothIcon, current: false },
]
</script>
