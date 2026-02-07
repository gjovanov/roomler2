<template>
  <div :class="['message-bubble', { 'message-bubble--compact': compact }]">
    <div class="d-flex align-start">
      <v-avatar size="32" color="primary" class="mr-2 mt-1" v-if="!compact">
        <span class="text-caption">{{ authorInitial }}</span>
      </v-avatar>

      <div class="flex-grow-1">
        <div class="d-flex align-center mb-1">
          <span class="text-subtitle-2 font-weight-bold mr-2">
            {{ message.author_name || 'Unknown' }}
          </span>
          <span class="text-caption text-medium-emphasis">
            {{ formatTime(message.created_at) }}
          </span>
          <v-spacer />
          <!-- Action buttons (shown on hover) -->
          <div class="message-actions">
            <v-btn icon="mdi-emoticon-outline" size="x-small" variant="text" @click="showEmojiPicker = true" />
            <v-btn icon="mdi-reply" size="x-small" variant="text" @click="$emit('reply')" />
            <v-btn
              :icon="message.is_pinned ? 'mdi-pin-off' : 'mdi-pin'"
              size="x-small"
              variant="text"
              @click="$emit('pin')"
            />
          </div>
        </div>

        <div class="text-body-2">{{ message.content }}</div>

        <!-- Reactions -->
        <div v-if="message.reaction_summary.length > 0" class="d-flex flex-wrap ga-1 mt-1">
          <v-chip
            v-for="r in message.reaction_summary"
            :key="r.emoji"
            size="small"
            variant="tonal"
            @click="$emit('react', r.emoji)"
          >
            {{ r.emoji }} {{ r.count }}
          </v-chip>
        </div>

        <!-- Thread indicator -->
        <div
          v-if="message.is_thread_root"
          class="mt-1 text-caption text-primary cursor-pointer"
          @click="$emit('reply')"
        >
          View thread
        </div>
      </div>
    </div>

    <!-- Emoji picker (simplified) -->
    <v-menu v-model="showEmojiPicker" :close-on-content-click="true">
      <template #activator="{ props: _props }">
        <span />
      </template>
      <v-card>
        <div class="pa-2 d-flex flex-wrap ga-1">
          <v-btn
            v-for="emoji in quickEmojis"
            :key="emoji"
            variant="text"
            size="small"
            @click="$emit('react', emoji)"
          >
            {{ emoji }}
          </v-btn>
        </div>
      </v-card>
    </v-menu>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

interface Reaction {
  emoji: string
  count: number
}

interface Message {
  id: string
  author_name?: string
  content: string
  is_pinned: boolean
  is_thread_root: boolean
  reaction_summary: Reaction[]
  created_at: string
}

const props = defineProps<{
  message: Message
  compact?: boolean
}>()

defineEmits<{
  reply: []
  react: [emoji: string]
  pin: []
}>()

const showEmojiPicker = ref(false)
const quickEmojis = ['👍', '👎', '❤️', '😂', '🎉', '😮', '🤔', '👀']

const authorInitial = computed(() =>
  (props.message.author_name || '?').charAt(0).toUpperCase(),
)

function formatTime(iso: string): string {
  const d = new Date(iso)
  return d.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })
}
</script>

<style scoped>
.message-bubble {
  padding: 4px 8px;
  border-radius: 4px;
}
.message-bubble:hover {
  background: rgba(255, 255, 255, 0.04);
}
.message-actions {
  opacity: 0;
  transition: opacity 0.15s;
}
.message-bubble:hover .message-actions {
  opacity: 1;
}
</style>
