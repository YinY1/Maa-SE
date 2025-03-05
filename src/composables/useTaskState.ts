import { invoke } from '@tauri-apps/api/core'

import { ref } from 'vue'

import { useLogState } from './useLogState'

interface TaskConfig {
  label: string
  type: 'text' | 'select' | 'multiSelect' | 'checkbox'
  value: string | boolean | string[]
  options?: string[]
}

interface StartUpParams {
  client_type: string
  // start_game_enabled: boolean
}

interface FightParams {
  stage: string
  enable?: boolean
  series?: number
}

interface InfrastParams {
  enable?: boolean
  facility: string[]
  drones?: string
  threshold?: number
  replenish?: boolean
  dorm_notstationed_enabled?: boolean
  dorm_trust_enabled?: boolean
}

const currentTask = ref<string>('开始唤醒')
const isRunning = ref<boolean>(false)

const taskTypeMap: Record<string, string> = {
  开始唤醒: 'StartUp',
  刷理智: 'Fight',
  基建换班: 'Infrast',
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
    {
      label: '连续作战次数',
      type: 'select',
      value: '1',
      options: ['1', '2', '3', '4', '5', '6'],
    },
  ],
  基建换班: [
    {
      label: '设施列表',
      type: 'multiSelect',
      value: ['Mfg'],
      options: [
        'Mfg',
        'Trade',
        'Power',
        'Control',
        'Reception',
        'Office',
        'Dorm',
      ],
    },
    {
      label: '无人机用途',
      type: 'select',
      value: '_NotUse',
      options: [
        '_NotUse',
        'Money',
        'SyntheticJade',
        'CombatRecord',
        'PureGold',
        'OriginStone',
        'Chip',
      ],
    },
    {
      label: '心情阈值',
      type: 'text',
      value: '0.3',
    },
    {
      label: '源石碎片自动补货',
      type: 'checkbox',
      value: false,
    },
    {
      label: '启用宿舍未进驻',
      type: 'checkbox',
      value: false,
    },
    {
      label: '填充信赖未满干员',
      type: 'checkbox',
      value: false,
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
    series: Number(configs.find((c) => c.label === '连续作战次数')?.value) || 1,
  }
}

function convertInfrastConfig(configs: TaskConfig[]): InfrastParams {
  return {
    facility: (configs.find((c) => c.label === '设施列表')
      ?.value as string[]) || ['Mfg'],
    drones:
      (configs.find((c) => c.label === '无人机用途')?.value as string) ||
      '_NotUse',
    threshold:
      Number(configs.find((c) => c.label === '心情阈值')?.value) || 0.3,
    replenish:
      (configs.find((c) => c.label === '源石碎片自动补货')?.value as boolean) ||
      false,
    dorm_notstationed_enabled:
      (configs.find((c) => c.label === '启用宿舍未进驻')?.value as boolean) ||
      false,
    dorm_trust_enabled:
      (configs.find((c) => c.label === '填充信赖未满干员')?.value as boolean) ||
      false,
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
    } else if (taskType === 'Infrast') {
      taskParams = convertInfrastConfig(taskConfigs[currentTask.value])
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
      isRunning.value = false
      await invoke('stop_core')
    } else {
      console.log('开始执行任务')
      isRunning.value = true
      await invoke('run_daily')
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
