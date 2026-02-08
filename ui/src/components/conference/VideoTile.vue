<template>
  <v-card class="video-tile" color="grey-darken-3" elevation="2">
    <div class="video-container" style="aspect-ratio: 16/9; position: relative; overflow: hidden">
      <!-- Video element -->
      <video
        v-show="hasVideoTrack"
        ref="videoRef"
        autoplay
        playsinline
        :muted="isLocal"
        class="w-100 h-100"
        style="object-fit: cover"
      />

      <!-- Avatar fallback when video is off -->
      <div
        v-show="!hasVideoTrack"
        class="d-flex align-center justify-center w-100 h-100 bg-grey-darken-4"
      >
        <v-avatar size="64" color="primary">
          <span class="text-h5">{{ initial }}</span>
        </v-avatar>
      </div>

      <!-- Muted indicator -->
      <v-icon
        v-if="isMuted"
        class="mute-indicator"
        color="error"
        size="20"
      >
        mdi-microphone-off
      </v-icon>
    </div>

    <!-- Name label -->
    <v-card-text class="pa-2 text-caption text-center text-truncate">
      {{ displayName }}{{ isLocal ? ' (You)' : '' }}
    </v-card-text>
  </v-card>
</template>

<script setup lang="ts">
import { ref, watch, computed, onMounted, onBeforeUnmount } from 'vue'

const props = defineProps<{
  stream: MediaStream | null
  displayName: string
  isMuted: boolean
  isLocal: boolean
}>()

const videoRef = ref<HTMLVideoElement | null>(null)

const initial = computed(() =>
  props.displayName ? props.displayName.charAt(0).toUpperCase() : '?',
)

const hasVideoTrack = computed(() => {
  if (!props.stream) return false
  return props.stream.getVideoTracks().some((t) => t.enabled && t.readyState === 'live')
})

function attachStream() {
  if (videoRef.value && props.stream) {
    videoRef.value.srcObject = props.stream
  }
}

watch(() => props.stream, attachStream)

onMounted(attachStream)

onBeforeUnmount(() => {
  if (videoRef.value) {
    videoRef.value.srcObject = null
  }
})
</script>

<style scoped>
.video-tile {
  border-radius: 8px;
  overflow: hidden;
}

.mute-indicator {
  position: absolute;
  bottom: 8px;
  right: 8px;
  background: rgba(0, 0, 0, 0.6);
  border-radius: 50%;
  padding: 4px;
}
</style>
