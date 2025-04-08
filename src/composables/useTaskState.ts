import { invoke } from '@tauri-apps/api/core'

import { reactive, ref } from 'vue'

import { useConfig } from './useConfig'

interface TaskItem {
  name: string
  checked: boolean
}

export interface TaskConfig {
  label: string
  type: 'text' | 'select' | 'multiSelect' | 'checkbox'
  value: string | boolean | string[]
  options?: string[]
  displayMap?: Record<string, string>
}

interface TaskParams {
  enable?: boolean
  index?: number
  [key: string]: any
}

type TaskConverter = (configs: TaskConfig[]) => TaskParams

const taskConverters: Record<string, TaskConverter> = {
  StartUp: (configs) => ({
    client_type:
      { 官服: 'Official', B服: 'Bilibili' }[
        configs.find((c) => c.label === '客户端版本')?.value as string
      ] || '',
    start_game_enabled:
      (configs.find((c) => c.label === '自动启动客户端')?.value as boolean) ||
      false,
  }),
  Fight: (configs) => ({
    stage: (configs.find((c) => c.label === '关卡名称')?.value as string) || '',
    series: Number(configs.find((c) => c.label === '连续作战次数')?.value) || 1,
  }),
  Infrast: (configs) => ({
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
  }),
  Award: (configs) => ({
    award:
      (configs.find((c) => c.label === '领取任务奖励')?.value as boolean) ??
      true,
    mail:
      (configs.find((c) => c.label === '领取邮件奖励')?.value as boolean) ??
      false,
    recruit:
      (configs.find((c) => c.label === '使用免费单抽')?.value as boolean) ??
      false,
  }),
  Mall: (configs) => ({
    shopping:
      (configs.find((c) => c.label === '购买商品')?.value as boolean) ?? false,
    buy_first: (
      (configs.find((c) => c.label === '优先购买')?.value as string) || ''
    )
      .split(',')
      .map((item) => item.trim())
      .filter(Boolean),
    blacklist: (
      (configs.find((c) => c.label === '黑名单')?.value as string) || ''
    )
      .split(',')
      .map((item) => item.trim())
      .filter(Boolean),
  }),
}

const currentTask = ref<string>('开始唤醒')
const isRunning = ref<boolean>(false)

const navigation = ref<TaskItem[]>([
  { name: '开始唤醒', checked: false },
  { name: '自动公招', checked: false },
  { name: '基建换班', checked: false },
  { name: '获取信用', checked: false },
  { name: '刷理智', checked: false },
  { name: '领取奖励', checked: false },
  { name: '集成战略', checked: false },
  { name: '生息演算', checked: false },
])

const taskTypeMap: Record<string, string> = {
  开始唤醒: 'StartUp',
  刷理智: 'Fight',
  基建换班: 'Infrast',
  领取奖励: 'Award',
  获取信用: 'Mall',
}

const taskConfigs = reactive<Record<string, TaskConfig[]>>({
  开始唤醒: [
    {
      label: '客户端版本',
      type: 'select',
      value: '官服',
      options: ['官服', 'B服'],
    },
    {
      label: '自动启动客户端',
      type: 'checkbox',
      value: false,
    },
  ],
  刷理智: [
    {
      label: '关卡名称',
      type: 'select',
      value: '1-7',
      options: ['1-7'],
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
      value: [
        'Mfg',
        'Trade',
        'Power',
        'Control',
        'Reception',
        'Office',
        'Dorm',
      ],
      options: [
        'Mfg',
        'Trade',
        'Power',
        'Control',
        'Reception',
        'Office',
        'Dorm',
      ],
      displayMap: {
        Mfg: '制造站',
        Trade: '贸易站',
        Power: '发电站',
        Control: '控制中枢',
        Reception: '会客室',
        Office: '办公室',
        Dorm: '宿舍',
      },
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
      displayMap: {
        _NotUse: '不使用',
        Money: '龙门币',
        SyntheticJade: '合成玉',
        CombatRecord: '作战记录',
        PureGold: '赤金',
        OriginStone: '源石',
        Chip: '芯片',
      },
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
  领取奖励: [
    {
      label: '领取任务奖励',
      type: 'checkbox',
      value: true,
    },
    {
      label: '领取邮件奖励',
      type: 'checkbox',
      value: false,
    },
    {
      label: '使用免费单抽',
      type: 'checkbox',
      value: false,
    },
  ],
  获取信用: [
    {
      label: '购买商品',
      type: 'checkbox',
      value: false,
    },
    {
      label: '优先购买',
      type: 'text',
      value: '',
    },
    {
      label: '黑名单',
      type: 'text',
      value: '',
    },
  ],
})

const configKeyMap: Record<string, Record<string, string>> = {
  开始唤醒: {
    客户端版本: 'client_type',
    自动启动客户端: 'start_game_enabled',
  },
  刷理智: {
    关卡名称: 'stage',
    连续作战次数: 'series',
  },
  基建换班: {
    设施列表: 'facility',
    无人机用途: 'drones',
    心情阈值: 'threshold',
    源石碎片自动补货: 'replenish',
    启用宿舍未进驻: 'dorm_notstationed_enabled',
    填充信赖未满干员: 'dorm_trust_enabled',
  },
  领取奖励: {
    领取任务奖励: 'award',
    领取邮件奖励: 'mail',
    使用免费单抽: 'recruit',
  },
  获取信用: {
    购买商品: 'shopping',
    优先购买: 'buy_first',
    黑名单: 'blacklist',
  },
}

async function updateTaskConfig(
  taskName: string,
  enable: boolean,
  fullUpdate: boolean = false,
): Promise<void> {
  const taskType = taskTypeMap[taskName]
  if (!taskType) return

  try {
    const taskIndex = navigation.value.findIndex(
      (item) => item.name === taskName,
    )
    const baseParams: TaskParams = { enable, index: taskIndex }

    const params =
      fullUpdate && taskConverters[taskType]
        ? { ...baseParams, ...taskConverters[taskType](taskConfigs[taskName]) }
        : baseParams

    console.log('name:', taskType)
    console.log('params:', params)

    await invoke('update_config', { name: taskType, params })
  } catch (error) {
    console.error(`更新任务配置失败 (${taskType}):`, error)
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
      isRunning.value = false
    }
  } catch (error) {
    isRunning.value = false
    console.error(
      '任务执行失败:',
      error instanceof Error ? error.message : String(error),
    )
    throw error
  }
}

async function updateTaskEnable(
  taskName: string,
  enable: boolean,
): Promise<void> {
  await updateTaskConfig(taskName, enable, false)
}

export const useTaskState = () => {
  const { configs, loadConfigs: loadConfigsBase } = useConfig()

  const loadConfigs = () =>
    loadConfigsBase(navigation.value, taskTypeMap, taskConfigs, configKeyMap)

  return {
    currentTask,
    taskConfigs,
    isRunning,
    toggleTask,
    updateTaskConfig: (enable: boolean) =>
      updateTaskConfig(currentTask.value, enable, true),
    updateTaskEnable,
    navigation,
    configs,
    loadConfigs,
  }
}
