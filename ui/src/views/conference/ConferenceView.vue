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
          <v-chip size="small" :color="statusColor">
            {{ conferenceStore.current?.status }}
          </v-chip>
        </v-toolbar>

        <v-divider />

        <!-- Video grid -->
        <div class="flex-grow-1 d-flex align-center justify-center bg-black">
          <v-row v-if="conferenceStore.current" class="pa-4" justify="center">
            <!-- Local video -->
            <v-col cols="12" sm="6" md="4" lg="3">
              <v-card class="video-tile" color="grey-darken-3">
                <video
                  ref="localVideoRef"
                  autoplay
                  muted
                  playsinline
                  class="w-100"
                  style="aspect-ratio: 16/9; object-fit: cover"
                />
                <v-card-text class="pa-2 text-caption text-center">
                  You {{ conferenceStore.isMuted ? '(muted)' : '' }}
                </v-card-text>
              </v-card>
            </v-col>

            <!-- Remote participants placeholder -->
            <v-col
              v-for="p in conferenceStore.participants"
              :key="p.id"
              cols="12" sm="6" md="4" lg="3"
            >
              <v-card class="video-tile" color="grey-darken-3">
                <div
                  class="d-flex align-center justify-center"
                  style="aspect-ratio: 16/9"
                >
                  <v-avatar size="64" color="primary">
                    <span class="text-h5">{{ p.display_name.charAt(0) }}</span>
                  </v-avatar>
                </div>
                <v-card-text class="pa-2 text-caption text-center">
                  {{ p.display_name }}
                  {{ p.is_muted ? '(muted)' : '' }}
                </v-card-text>
              </v-card>
            </v-col>
          </v-row>

          <div v-else class="text-center text-white">
            <v-icon size="64" class="mb-4">mdi-video</v-icon>
            <div class="text-h5">Ready to join?</div>
            <v-btn color="primary" size="large" class="mt-4" @click="handleJoin">
              {{ $t('conference.join') }}
            </v-btn>
          </div>
        </div>

        <!-- Controls bar -->
        <v-toolbar v-if="conferenceStore.current" density="compact" color="grey-darken-4">
          <v-spacer />
          <v-btn
            :icon="conferenceStore.isMuted ? 'mdi-microphone-off' : 'mdi-microphone'"
            :color="conferenceStore.isMuted ? 'error' : undefined"
            @click="conferenceStore.toggleMute()"
          />
          <v-btn
            :icon="conferenceStore.isVideoOn ? 'mdi-video' : 'mdi-video-off'"
            :color="!conferenceStore.isVideoOn ? 'error' : undefined"
            @click="conferenceStore.toggleVideo()"
          />
          <v-btn icon="mdi-monitor-share" />
          <v-btn icon="mdi-phone-hangup" color="error" @click="handleLeave" />
          <v-spacer />
        </v-toolbar>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useConferenceStore } from '@/stores/conference'

const route = useRoute()
const router = useRouter()
const conferenceStore = useConferenceStore()

const tenantId = computed(() => route.params.tenantId as string)
const conferenceId = computed(() => route.params.conferenceId as string)
const localVideoRef = ref<HTMLVideoElement | null>(null)

const statusColor = computed(() => {
  switch (conferenceStore.current?.status) {
    case 'InProgress': return 'success'
    case 'Scheduled': return 'info'
    case 'Ended': return 'grey'
    default: return 'warning'
  }
})

async function handleJoin() {
  await conferenceStore.joinConference(tenantId.value, conferenceId.value)
  const stream = await conferenceStore.startLocalStream()
  if (localVideoRef.value) {
    localVideoRef.value.srcObject = stream
  }
}

async function handleLeave() {
  await conferenceStore.leaveConference(tenantId.value, conferenceId.value)
  router.push(`/tenant/${tenantId.value}`)
}

watch(
  () => conferenceStore.localStream,
  (stream) => {
    if (localVideoRef.value && stream) {
      localVideoRef.value.srcObject = stream
    }
  },
)
</script>
