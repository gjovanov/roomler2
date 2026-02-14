<template>
  <v-container fluid class="fill-height pa-0">
    <v-row no-gutters class="fill-height">
      <v-col class="d-flex flex-column fill-height">
        <!-- Conference header -->
        <v-toolbar density="compact" flat>
          <v-toolbar-title>
            {{ conferenceStore.current?.subject || 'Meeting' }}
          </v-toolbar-title>
          <v-spacer />
          <LayoutSwitcher
            v-if="joined"
            :prefs="layoutCtrl.prefs.value"
            @update:mode="layoutCtrl.setMode"
            @update:max-tiles="layoutCtrl.setMaxTiles"
            @update:hide-non-video="layoutCtrl.setHideNonVideo"
            @update:self-view-mode="layoutCtrl.setSelfViewMode"
          />
          <TranscriptionSwitcher
            v-if="joined"
            :enabled="conferenceStore.transcriptionEnabled"
            :selected-model="conferenceStore.selectedTranscriptionModel"
            @update:model="conferenceStore.setSelectedTranscriptionModel"
            @toggle="toggleTranscription"
          />
          <v-btn
            v-if="joined && conferenceStore.transcriptionEnabled"
            :icon="showTranscriptPanel ? 'mdi-text-box' : 'mdi-text-box-outline'"
            variant="text"
            :color="showTranscriptPanel ? 'primary' : undefined"
            @click="showTranscriptPanel = !showTranscriptPanel"
          />
          <v-btn
            v-if="joined"
            :icon="showChat ? 'mdi-message-text' : 'mdi-message-text-outline'"
            variant="text"
            :color="showChat ? 'primary' : undefined"
            @click="showChat = !showChat"
          />
          <v-chip size="small" :color="statusColor">
            {{ conferenceStore.current?.status }}
          </v-chip>
        </v-toolbar>

        <v-divider />

        <!-- Main content area -->
        <div class="flex-grow-1 d-flex overflow-hidden">
          <!-- Video grid -->
          <div class="flex-grow-1 d-flex align-center justify-center bg-black position-relative">
            <!-- Pre-join state -->
            <div v-if="!joined" class="text-center text-white">
              <v-icon size="64" class="mb-4">mdi-video</v-icon>
              <div class="text-h5">Ready to join?</div>
              <v-btn
                color="primary"
                size="large"
                class="mt-4"
                :loading="joining"
                @click="handleJoin"
              >
                {{ $t('conference.join') }}
              </v-btn>
            </div>

            <!-- Active conference: dynamic layout -->
            <component
              v-else
              :is="layoutComponent"
              v-bind="layoutProps"
              @toggle-pin="handleTogglePin"
              @request-pip="handlePiP"
            />

            <!-- Transcript subtitle overlay -->
            <TranscriptOverlay
              v-if="joined && conferenceStore.transcriptionEnabled"
              :segments="conferenceStore.transcriptSegments"
            />
          </div>

          <!-- Transcript panel -->
          <TranscriptPanel
            v-if="joined && showTranscriptPanel && conferenceStore.transcriptionEnabled"
            :segments="conferenceStore.transcriptSegments"
          />

          <!-- Chat panel -->
          <div
            v-if="joined && showChat"
            class="chat-panel d-flex flex-column"
          >
            <v-toolbar density="compact" flat>
              <v-toolbar-title class="text-body-1">
                {{ $t('conference.chat') }}
              </v-toolbar-title>
            </v-toolbar>
            <v-divider />

            <!-- Messages list -->
            <div ref="chatListRef" class="flex-grow-1 overflow-y-auto pa-3">
              <div
                v-if="conferenceStore.chatMessages.length === 0"
                class="text-center text-medium-emphasis mt-4"
              >
                {{ $t('conference.noChatMessages') }}
              </div>
              <div
                v-for="msg in conferenceStore.chatMessages"
                :key="msg.id"
                class="mb-3"
              >
                <div class="d-flex align-start">
                  <v-avatar size="28" color="primary" class="mr-2 mt-1">
                    <span class="text-caption">{{ msg.display_name.charAt(0).toUpperCase() }}</span>
                  </v-avatar>
                  <div class="flex-grow-1" style="min-width: 0;">
                    <div class="d-flex align-center ga-2">
                      <span class="text-subtitle-2 font-weight-bold text-truncate">
                        {{ msg.display_name }}
                      </span>
                      <span class="text-caption text-medium-emphasis flex-shrink-0">
                        {{ formatTime(msg.created_at) }}
                      </span>
                    </div>
                    <div class="text-body-2" style="word-break: break-word;">{{ msg.content }}</div>
                  </div>
                </div>
              </div>
            </div>

            <v-divider />

            <!-- Chat input -->
            <div class="pa-2">
              <v-text-field
                v-model="chatInput"
                :placeholder="$t('conference.chatPlaceholder')"
                variant="outlined"
                density="compact"
                hide-details
                append-inner-icon="mdi-send"
                @keydown.enter.exact.prevent="handleSendChat"
                @click:append-inner="handleSendChat"
              />
            </div>
          </div>
        </div>

        <!-- Controls bar -->
        <v-toolbar v-if="joined" density="compact" color="grey-darken-4">
          <v-spacer />
          <v-btn
            :icon="mediasoup.isMuted.value ? 'mdi-microphone-off' : 'mdi-microphone'"
            :color="mediasoup.isMuted.value ? 'error' : undefined"
            @click="mediasoup.toggleMute()"
          />
          <v-btn
            :icon="mediasoup.isVideoOn.value ? 'mdi-video' : 'mdi-video-off'"
            :color="!mediasoup.isVideoOn.value ? 'error' : undefined"
            @click="mediasoup.toggleVideo()"
          />
          <v-btn
            :icon="mediasoup.isScreenSharing.value ? 'mdi-monitor-off' : 'mdi-monitor-share'"
            :color="mediasoup.isScreenSharing.value ? 'success' : undefined"
            @click="handleScreenShare"
          />
          <v-btn
            v-if="pip.isSupported.value"
            icon="mdi-picture-in-picture-bottom-right"
            :color="pip.isPiPActive.value ? 'primary' : undefined"
            @click="togglePiP"
          />
          <v-btn icon="mdi-phone-hangup" color="error" @click="handleLeave" />
          <v-spacer />
        </v-toolbar>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, nextTick, type Component } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useConferenceStore } from '@/stores/conference'
import { useWsStore } from '@/stores/ws'
import { useMediasoup } from '@/composables/useMediasoup'
import { useActiveSpeaker } from '@/composables/useActiveSpeaker'
import { useConferenceLayout } from '@/composables/useConferenceLayout'
import { usePictureInPicture } from '@/composables/usePictureInPicture'
import LayoutSwitcher from '@/components/conference/LayoutSwitcher.vue'
import TranscriptionSwitcher from '@/components/conference/TranscriptionSwitcher.vue'
import TranscriptOverlay from '@/components/conference/TranscriptOverlay.vue'
import TranscriptPanel from '@/components/conference/TranscriptPanel.vue'
import TiledLayout from '@/components/conference/layouts/TiledLayout.vue'
import SpotlightLayout from '@/components/conference/layouts/SpotlightLayout.vue'
import SidebarLayout from '@/components/conference/layouts/SidebarLayout.vue'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()
const conferenceStore = useConferenceStore()
const wsStore = useWsStore()
const mediasoup = useMediasoup()
const activeSpeaker = useActiveSpeaker(mediasoup.remoteStreams)
const pip = usePictureInPicture()

const tenantId = computed(() => route.params.tenantId as string)
const conferenceId = computed(() => route.params.conferenceId as string)
const joined = ref(false)
const joining = ref(false)
const showChat = ref(false)
const showTranscriptPanel = ref(false)
const chatInput = ref('')
const chatListRef = ref<HTMLElement | null>(null)

const localDisplayName = computed(() => authStore.user?.display_name ?? 'You')

function getDisplayName(userId: string): string {
  const participant = conferenceStore.participants.find((p) => p.user_id === userId)
  return participant?.display_name || userId.slice(0, 8)
}

const layoutCtrl = useConferenceLayout(
  mediasoup.localStream,
  mediasoup.remoteStreams,
  mediasoup.isMuted,
  activeSpeaker.audioLevels,
  activeSpeaker.activeSpeakerKey,
  getDisplayName,
  localDisplayName,
)

const layoutComponent = computed<Component>(() => {
  switch (layoutCtrl.layout.value.effectiveMode) {
    case 'spotlight': return SpotlightLayout
    case 'sidebar': return SidebarLayout
    default: return TiledLayout
  }
})

const layoutProps = computed(() => {
  const l = layoutCtrl.layout.value
  const selfP = layoutCtrl.selfParticipant.value
  const speakerKey = activeSpeaker.activeSpeakerKey.value

  if (l.effectiveMode === 'tiled') {
    return {
      participants: l.primary,
      selfParticipant: l.selfViewFloating ? selfP : null,
      selfViewFloating: l.selfViewFloating,
      selfViewMode: layoutCtrl.prefs.value.selfViewMode,
      activeSpeakerKey: speakerKey,
    }
  }
  // spotlight or sidebar
  return {
    primary: l.primary,
    secondary: l.secondary,
    selfParticipant: l.selfViewFloating ? selfP : null,
    selfViewFloating: l.selfViewFloating,
    activeSpeakerKey: speakerKey,
  }
})

function handleTogglePin(streamKey: string) {
  layoutCtrl.togglePin(streamKey)
}

function handlePiP(streamKey: string) {
  const videoEl = document.querySelector(
    `video[data-stream-key="${streamKey}"]`,
  ) as HTMLVideoElement | null
  if (videoEl) {
    pip.requestPiP(videoEl)
  }
}

function togglePiP() {
  if (pip.isPiPActive.value) {
    pip.exitPiP()
  } else {
    // PiP the active speaker or first remote stream
    const speakerKey = activeSpeaker.activeSpeakerKey.value
    const targetKey = speakerKey || (mediasoup.remoteStreams.keys().next().value as string | undefined)
    if (targetKey) {
      handlePiP(targetKey)
    }
  }
}

const statusColor = computed(() => {
  switch (conferenceStore.current?.status) {
    case 'InProgress': return 'success'
    case 'Scheduled': return 'info'
    case 'Ended': return 'grey'
    default: return 'warning'
  }
})

function formatTime(iso: string): string {
  const d = new Date(iso)
  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

function scrollChatToBottom() {
  if (chatListRef.value) {
    chatListRef.value.scrollTop = chatListRef.value.scrollHeight
  }
}

// Auto-scroll when new messages arrive
watch(
  () => conferenceStore.chatMessages.length,
  async () => {
    await nextTick()
    scrollChatToBottom()
  },
)

async function handleSendChat() {
  const content = chatInput.value.trim()
  if (!content) return
  chatInput.value = ''
  try {
    await conferenceStore.sendChatMessage(tenantId.value, conferenceId.value, content)
  } catch (err) {
    console.error('Failed to send chat message:', err)
  }
}

async function handleJoin() {
  if (joined.value || joining.value) return
  joining.value = true
  try {
    // Always call start to ensure the in-memory mediasoup room exists
    // (it may have been lost on server restart even if DB status is InProgress)
    await conferenceStore.startConference(tenantId.value, conferenceId.value)

    // REST join (adds participant to DB)
    await conferenceStore.joinConference(tenantId.value, conferenceId.value)

    // Fetch conference details
    await conferenceStore.fetchConference(tenantId.value, conferenceId.value)

    // mediasoup signaling join (loads device, connects transports)
    await mediasoup.joinRoom(conferenceId.value)

    // Produce local audio + video
    await mediasoup.produceLocalMedia()

    // Fetch participants for display names
    await conferenceStore.fetchParticipants(tenantId.value, conferenceId.value)

    // Fetch existing chat messages
    await conferenceStore.fetchChatMessages(tenantId.value, conferenceId.value)

    joined.value = true
    showChat.value = true

    // Start active speaker detection
    activeSpeaker.start()
  } catch (err) {
    console.error('Failed to join conference:', err)
  } finally {
    joining.value = false
  }
}

async function handleLeave() {
  // Stop active speaker detection
  activeSpeaker.stop()

  // Exit PiP if active
  if (pip.isPiPActive.value) {
    await pip.exitPiP()
  }

  mediasoup.leaveRoom()
  conferenceStore.clearChatMessages()
  conferenceStore.clearTranscript()
  conferenceStore.setTranscriptionEnabled(false)
  await conferenceStore.leaveConference(tenantId.value, conferenceId.value)
  joined.value = false
  showChat.value = false
  showTranscriptPanel.value = false
  router.push(`/tenant/${tenantId.value}`)
}

function toggleTranscription(newState: boolean) {
  wsStore.send('media:transcript_toggle', {
    conference_id: conferenceId.value,
    enabled: newState,
    model: conferenceStore.selectedTranscriptionModel,
  })
  // Optimistic update; server will confirm via media:transcript_status
  conferenceStore.setTranscriptionEnabled(newState)
  if (newState) {
    showTranscriptPanel.value = true
  }
}

function handleScreenShare() {
  if (mediasoup.isScreenSharing.value) {
    mediasoup.stopScreenShare()
  } else {
    mediasoup.startScreenShare()
  }
}

onMounted(async () => {
  // Load conference info on mount
  try {
    await conferenceStore.fetchConference(tenantId.value, conferenceId.value)
  } catch {
    // Conference not found
  }
})
</script>

<style scoped>
.chat-panel {
  width: 320px;
  min-width: 280px;
  border-left: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
  background: rgb(var(--v-theme-surface));
}
</style>
