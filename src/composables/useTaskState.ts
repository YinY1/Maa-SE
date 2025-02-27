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
    stage: (configs.find((c) => c.label === '关卡名称')?.value as string) || '',
  }
}

async function updateTaskConfig(enable: boolean): Promise<void> {
  const taskType = taskTypeMap[currentTask.value]
  if (!taskType) return

  try {
    let baseParams = { enable }
    let taskParams = {}

    // 根据任务类型添加特定参数
    if (taskType === 'StartUp') {
      taskParams = convertStartUpConfig(taskConfigs[currentTask.value])
    } else if (taskType === 'Fight') {
      taskParams = convertFightConfig(taskConfigs[currentTask.value])
    }

    const params = {
      ...baseParams,
      ...taskParams,
    }

    console.log('name:', taskType)
    console.log('params:', params)

    await invoke('update_config', {
      name: taskType,
      params: params,
    })
  } catch (error) {
    console.error('更新任务配置失败:', error)
    throw error
  }
}

async function toggleTask(): Promise<void> {
  try {
    if (isRunning.value) {
      await invoke('stop_core')
      isRunning.value = false
    } else {
      console.log('开始执行任务')
      await invoke('run_daily')
      isRunning.value = true
    }
  } catch (error) {
    isRunning.value = false
    const errorMessage =
      typeof error === 'string'
        ? error
        : error instanceof Error
          ? error.message
          : '未知错误'
    console.error('任务执行失败:', errorMessage)
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
