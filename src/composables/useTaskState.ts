import { invoke } from '@tauri-apps/api/core'

import { ref } from 'vue'

interface TaskConfig {
  label: string
  type: 'text' | 'select' | 'checkbox'
  value: string | boolean
  options?: string[]
}

interface StartUpParams {
  client_type: string
  // start_game_enabled: boolean
}

interface FightParams {
  enable: boolean
  stage: string
}

const currentTask = ref<string>('开始唤醒')
const isRunning = ref<boolean>(false)

const taskTypeMap: Record<string, string> = {
  开始唤醒: 'StartUp',
  刷理智: 'Fight',
}

const taskConfigs: Record<string, TaskConfig[]> = {
  开始唤醒: [
    {
      label: '客户端版本',
      type: 'select',
      value: '官服',
      options: ['官服', 'B服'],
    },
    // {
    //   label: '自动启动客户端',
    //   type: 'checkbox',
    //   value: false,
    // },
  ],
  刷理智: [
    {
      label: '关卡名称',
      type: 'text',
      value: '',
    },
  ],
}

function convertStartUpConfig(configs: TaskConfig[]): StartUpParams {
  const clientTypeMap: Record<string, string> = {
    官服: 'Official',
    B服: 'Bilibili',
  }

  return {
    client_type:
      clientTypeMap[
        configs.find((c) => c.label === '客户端版本')?.value as string
      ] || '',
    // start_game_enabled:
    //   (configs.find((c) => c.label === '自动启动客户端')?.value as boolean) ||
    //   false,
  }
}

function convertFightConfig(configs: TaskConfig[]): FightParams {
  return {
    enable: true,
    stage: (configs.find((c) => c.label === '关卡名称')?.value as string) || '',
  }
}

async function updateTaskConfig(enable: boolean): Promise<void> {
  const taskType = taskTypeMap[currentTask.value]
  if (!taskType) return

  try {
    let params = {}
    if (taskType === 'StartUp') {
      params = convertStartUpConfig(taskConfigs[currentTask.value])
    } else if (taskType === 'Fight') {
      params = convertFightConfig(taskConfigs[currentTask.value])
    }
    console.log('更新任务配置:', params)
    await invoke('update_task', {
      enable,
      name: taskType,
      params: JSON.stringify(params),
    })
  } catch (error) {
    console.error('更新任务配置失败:', error)
    throw error
  }
}

// async function disableTask(name: string): Promise<void> {
//   try {
//     await invoke('disable_task', { name })
//     console.log('禁用任务成功:', name)
//   } catch (error) {
//     console.error('禁用任务失败:', error)
//     throw error
//   }
// }

async function toggleTask(): Promise<void> {
  try {
    if (isRunning.value) {
      await invoke('stop_core')
      isRunning.value = false
    } else {
      console.log('开始执行任务')
      await invoke('start_core')
      isRunning.value = true
    }
  } catch (error) {
    isRunning.value = false
    if (error instanceof Error) {
      console.error('任务执行失败:', error.message)
    }
    throw error
  }
}

export const useTaskState = () => {
  return {
    currentTask,
    taskConfigs,
    isRunning,
    toggleTask,
    updateTaskConfig,
  }
}
