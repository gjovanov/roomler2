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
  const localStream = ref<MediaStream | null>(null)
  const isMuted = ref(false)
  const isVideoOn = ref(true)

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
    await api.post(`/tenant/${tenantId}/conference/${conferenceId}/start`)
  }

  async function joinConference(tenantId: string, conferenceId: string) {
    const data = await api.post<Conference>(`/tenant/${tenantId}/conference/${conferenceId}/join`)
    current.value = data
    return data
  }

  async function leaveConference(tenantId: string, conferenceId: string) {
    await api.post(`/tenant/${tenantId}/conference/${conferenceId}/leave`)
    current.value = null
    participants.value = []
    stopLocalStream()
  }

  async function endConference(tenantId: string, conferenceId: string) {
    await api.post(`/tenant/${tenantId}/conference/${conferenceId}/end`)
    current.value = null
    participants.value = []
    stopLocalStream()
  }

  async function startLocalStream() {
    const stream = await navigator.mediaDevices.getUserMedia({
      audio: true,
      video: true,
    })
    localStream.value = stream
    return stream
  }

  function stopLocalStream() {
    if (localStream.value) {
      localStream.value.getTracks().forEach((t) => t.stop())
      localStream.value = null
    }
  }

  function toggleMute() {
    isMuted.value = !isMuted.value
    if (localStream.value) {
      localStream.value.getAudioTracks().forEach((t) => {
        t.enabled = !isMuted.value
      })
    }
  }

  function toggleVideo() {
    isVideoOn.value = !isVideoOn.value
    if (localStream.value) {
      localStream.value.getVideoTracks().forEach((t) => {
        t.enabled = isVideoOn.value
      })
    }
  }

  return {
    conferences,
    current,
    participants,
    loading,
    localStream,
    isMuted,
    isVideoOn,
    fetchConferences,
    createConference,
    startConference,
    joinConference,
    leaveConference,
    endConference,
    startLocalStream,
    stopLocalStream,
    toggleMute,
    toggleVideo,
  }
})
