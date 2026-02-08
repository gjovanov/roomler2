import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api } from '@/api/client'

interface Conference {
  id: string
  tenant_id: string
  subject: string
  status: string
  organizer_id: string
  participant_count: number
  created_at: string
}

interface Participant {
  id: string
  user_id?: string
  display_name: string
  is_muted: boolean
  is_video_on: boolean
  is_screen_sharing: boolean
  is_hand_raised: boolean
}

export const useConferenceStore = defineStore('conference', () => {
  const conferences = ref<Conference[]>([])
  const current = ref<Conference | null>(null)
  const participants = ref<Participant[]>([])
  const loading = ref(false)

  async function fetchConferences(tenantId: string) {
    loading.value = true
    try {
      const data = await api.get<{ items: Conference[] }>(`/tenant/${tenantId}/conference`)
      conferences.value = data.items
    } finally {
      loading.value = false
    }
  }

  async function createConference(tenantId: string, subject: string) {
    const conf = await api.post<Conference>(`/tenant/${tenantId}/conference`, { subject })
    conferences.value.push(conf)
    return conf
  }

  async function startConference(tenantId: string, conferenceId: string) {
    const result = await api.post<{ started: boolean; rtp_capabilities: unknown }>(
      `/tenant/${tenantId}/conference/${conferenceId}/start`,
    )
    return result
  }

  async function joinConference(tenantId: string, conferenceId: string) {
    const data = await api.post<{ participant_id: string; joined: boolean; transports?: unknown }>(
      `/tenant/${tenantId}/conference/${conferenceId}/join`,
    )
    return data
  }

  async function leaveConference(tenantId: string, conferenceId: string) {
    await api.post(`/tenant/${tenantId}/conference/${conferenceId}/leave`)
    current.value = null
    participants.value = []
  }

  async function endConference(tenantId: string, conferenceId: string) {
    await api.post(`/tenant/${tenantId}/conference/${conferenceId}/end`)
    current.value = null
    participants.value = []
  }

  async function fetchConference(tenantId: string, conferenceId: string) {
    const conf = await api.get<Conference>(`/tenant/${tenantId}/conference/${conferenceId}`)
    current.value = conf
    return conf
  }

  async function fetchParticipants(tenantId: string, conferenceId: string) {
    const parts = await api.get<Participant[]>(
      `/tenant/${tenantId}/conference/${conferenceId}/participant`,
    )
    participants.value = parts
    return parts
  }

  return {
    conferences,
    current,
    participants,
    loading,
    fetchConferences,
    createConference,
    startConference,
    joinConference,
    leaveConference,
    endConference,
    fetchConference,
    fetchParticipants,
  }
})
