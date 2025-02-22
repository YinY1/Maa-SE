<template>
  <head>
    <link href="/src/assets/styles/styles.css" rel="stylesheet" />
  </head>
  <head>
    <link href="/src/assets/styles/styles.css" rel="stylesheet" />
  </head>
  <div class="pl-20">
    <div class="p-4">
      <nav class="flex flex-1 flex-col" aria-label="Sidebar">
        <ul role="list" class="-mx-2 space-y-1">
          <li v-for="item in navigation" :key="item.name">
            <a
              :href="item.href"
              :class="[
                item.current
                  ? 'bg-gray-50 text-indigo-600'
                  : 'text-gray-700 hover:bg-gray-50 hover:text-indigo-600',
                'group flex items-center gap-x-3 rounded-md p-2 pl-3 text-sm/6 font-semibold',
              ]"
            >
              <input
                type="checkbox"
                v-model="item.checked"
                class="h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-600"
                @click.stop
              />
              {{ item.name }}
            </a>
          </li>
        </ul>
      </nav>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const navigation = ref([
  { name: '开始唤醒', href: '#', current: true, checked: false },
  { name: '自动公招', href: '#', current: false, checked: false },
  { name: '基建换班', href: '#', current: false, checked: false },
  { name: '获取信用', href: '#', current: false, checked: false },
  { name: '刷理智', href: '#', current: false, checked: false },
  { name: '领取奖励', href: '#', current: false, checked: false },
  { name: '集成战略', href: '#', current: false, checked: false },
  { name: '生息演算', href: '#', current: false, checked: false },
])
</script>
