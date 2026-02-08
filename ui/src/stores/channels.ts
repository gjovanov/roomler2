import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { api } from '@/api/client'

interface Channel {
  id: string
  tenant_id: string
  parent_id?: string
  channel_type: string
  name: string
  path: string
  topic?: { value?: string }
  is_private: boolean
  member_count: number
  message_count: number
}

export const useChannelStore = defineStore('channels', () => {
  const channels = ref<Channel[]>([])
  const current = ref<Channel | null>(null)
  const loading = ref(false)

  const tree = computed(() => {
    const map = new Map<string | undefined, Channel[]>()
    for (const ch of channels.value) {
      const parentKey = ch.parent_id || undefined
      if (!map.has(parentKey)) map.set(parentKey, [])
      map.get(parentKey)!.push(ch)
    }
    return map
  })

  const rootChannels = computed(() => tree.value.get(undefined) || [])

  function childrenOf(parentId: string): Channel[] {
    return tree.value.get(parentId) || []
  }

  async function fetchChannels(tenantId: string) {
    loading.value = true
    try {
      channels.value = await api.get<Channel[]>(`/tenant/${tenantId}/channel`)
    } finally {
      loading.value = false
    }
  }

  async function createChannel(tenantId: string, payload: Partial<Channel>) {
    const channel = await api.post<Channel>(`/tenant/${tenantId}/channel`, payload)
    channels.value.push(channel)
    return channel
  }

  async function joinChannel(tenantId: string, channelId: string) {
    await api.post(`/tenant/${tenantId}/channel/${channelId}/join`)
  }

  async function leaveChannel(tenantId: string, channelId: string) {
    await api.post(`/tenant/${tenantId}/channel/${channelId}/leave`)
  }

  async function explore(tenantId: string, query: string) {
    return api.get<Channel[]>(
      `/tenant/${tenantId}/channel/explore?q=${encodeURIComponent(query)}`,
    )
  }

  function setCurrent(channel: Channel | null) {
    current.value = channel
  }

  return {
    channels,
    current,
    loading,
    rootChannels,
    tree,
    childrenOf,
    fetchChannels,
    createChannel,
    joinChannel,
    leaveChannel,
    explore,
    setCurrent,
  }
})
