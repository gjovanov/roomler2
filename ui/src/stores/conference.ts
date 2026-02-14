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

export interface ConferenceChatMsg {
  id: string
  conference_id: string
  author_id: string
  display_name: string
  content: string
  created_at: string
}

export interface TranscriptSegment {
  id: string
  user_id: string
  speaker_name: string
  text: string
  language?: string
  confidence?: number
  start_time: number
  end_time: number
  inference_duration_ms?: number
  received_at: number
}

export const useConferenceStore = defineStore('conference', () => {
  const conferences = ref<Conference[]>([])
  const current = ref<Conference | null>(null)
  const participants = ref<Participant[]>([])
  const loading = ref(false)
  const chatMessages = ref<ConferenceChatMsg[]>([])
  const transcriptSegments = ref<TranscriptSegment[]>([])
  const transcriptionEnabled = ref(false)
  const selectedTranscriptionModel = ref<'whisper' | 'canary'>('whisper')

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

  async function fetchChatMessages(tenantId: string, conferenceId: string) {
    const data = await api.get<{ items: ConferenceChatMsg[] }>(
      `/tenant/${tenantId}/conference/${conferenceId}/message`,
    )
    chatMessages.value = data.items
  }

  async function sendChatMessage(tenantId: string, conferenceId: string, content: string) {
    const msg = await api.post<ConferenceChatMsg>(
      `/tenant/${tenantId}/conference/${conferenceId}/message`,
      { content },
    )
    chatMessages.value.push(msg)
    return msg
  }

  function addChatMessageFromWs(msg: ConferenceChatMsg) {
    if (!chatMessages.value.some((m) => m.id === msg.id)) {
      chatMessages.value.push(msg)
    }
  }

  function clearChatMessages() {
    chatMessages.value = []
  }

  function addTranscriptFromWs(data: {
    user_id: string
    speaker_name: string
    text: string
    language?: string
    confidence?: number
    start_time: number
    end_time: number
    inference_duration_ms?: number
  }) {
    const segment: TranscriptSegment = {
      id: `${data.user_id}-${data.start_time}-${Date.now()}`,
      user_id: data.user_id,
      speaker_name: data.speaker_name,
      text: data.text,
      language: data.language,
      confidence: data.confidence,
      start_time: data.start_time,
      end_time: data.end_time,
      inference_duration_ms: data.inference_duration_ms,
      received_at: Date.now(),
    }
    transcriptSegments.value.push(segment)
  }

  function setTranscriptionEnabled(enabled: boolean) {
    transcriptionEnabled.value = enabled
  }

  function setSelectedTranscriptionModel(model: 'whisper' | 'canary') {
    selectedTranscriptionModel.value = model
  }

  function clearTranscript() {
    transcriptSegments.value = []
  }

  return {
    conferences,
    current,
    participants,
    loading,
    chatMessages,
    fetchConferences,
    createConference,
    startConference,
    joinConference,
    leaveConference,
    endConference,
    fetchConference,
    fetchParticipants,
    fetchChatMessages,
    sendChatMessage,
    addChatMessageFromWs,
    clearChatMessages,
    transcriptSegments,
    transcriptionEnabled,
    selectedTranscriptionModel,
    addTranscriptFromWs,
    setTranscriptionEnabled,
    setSelectedTranscriptionModel,
    clearTranscript,
  }
})
