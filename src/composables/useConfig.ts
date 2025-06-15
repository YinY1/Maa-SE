import { invoke } from '@tauri-apps/api/core'

import { ref } from 'vue'

import type { TaskConfig } from './useTaskState'

const CLIENT_TYPE_MAP: Record<string, string> = {
  Official: '官服',
  Bilibili: 'B服',
}

interface Config {
  custom: Record<string, any>
  daily: Record<string, any>
  extraTask: Record<string, any>
  settings: Record<string, any>
  toolStorage: Record<string, any>
}

const configs = ref<Config>({
  custom: {},
  daily: {},
  extraTask: {},
  settings: {},
  toolStorage: {},
})

async function loadConfigs(
  navigation: { name: string; checked: boolean }[],
  taskTypeMap: Record<string, string>,
  taskConfigs: Record<string, TaskConfig[]>,
  configKeyMap: Record<string, Record<string, string>>,
): Promise<void> {
  try {
    configs.value = await invoke<Config>('get_config')
    const dailyConfig = configs.value.daily

    navigation.forEach((item) => {
      const taskType = taskTypeMap[item.name]
      const taskConfig = dailyConfig[taskType]
      if (!taskType || !taskConfig) return

      item.checked = taskConfig.enable

      const taskConfigList = taskConfigs[item.name]
      if (!taskConfigList) return

      taskConfigList.forEach((config) => {
        const key = configKeyMap[item.name]?.[config.label]

        // 特殊处理星级
        if (taskType === 'Recruit' && config.label.startsWith('自动确认')) {
          const star = parseInt(
            config.label.replace('自动确认', '').replace('星', ''),
          )
          config.value =
            Array.isArray(taskConfig.confirm) &&
            taskConfig.confirm.includes(star)
        } else if (key && taskConfig[key] !== undefined) {
          config.value = updateConfigValue(config, taskConfig[key], taskType)
        }
      })
    })
  } catch (error) {
    console.error('读取配置失败:', error)
    throw error
  }
}

function updateConfigValue(
  config: TaskConfig,
  value: any,
  taskType: string,
): string | boolean | string[] {
  if (config.type === 'multiSelect' && Array.isArray(value)) {
    return value
  }
  if (config.type === 'checkbox') {
    return !!value
  }
  if (taskType === 'StartUp' && config.label === '客户端版本') {
    return CLIENT_TYPE_MAP[value] || value
  }
  return value.toString()
}

export const useConfig = () => {
  return {
    configs,
    loadConfigs,
  }
}
