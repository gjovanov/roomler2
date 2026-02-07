<template>
  <v-container fluid class="fill-height pa-0">
    <v-row no-gutters class="fill-height">
      <!-- Message list -->
      <v-col class="d-flex flex-column fill-height">
        <!-- Channel header -->
        <v-toolbar density="compact" flat>
          <v-toolbar-title>
            <v-icon class="mr-1" size="small">mdi-pound</v-icon>
            {{ channelStore.current?.name }}
          </v-toolbar-title>
          <v-spacer />
          <v-btn icon="mdi-pin" size="small" @click="showPinned = !showPinned" />
          <v-btn icon="mdi-account-group" size="small" @click="showMembers = !showMembers" />
        </v-toolbar>

        <v-divider />

        <!-- Messages -->
        <div ref="messageListRef" class="flex-grow-1 overflow-y-auto pa-4">
          <div v-if="messageStore.loading" class="text-center pa-4">
            <v-progress-circular indeterminate />
          </div>
          <div v-else-if="messageStore.messages.length === 0" class="text-center pa-8 text-medium-emphasis">
            {{ $t('channel.noMessages') }}
          </div>
          <div v-else>
            <div v-for="msg in messageStore.messages" :key="msg.id" class="mb-3">
              <message-bubble
                :message="msg"
                @reply="openThread(msg)"
                @react="(emoji) => handleReact(msg.id, emoji)"
                @pin="handlePin(msg.id)"
              />
            </div>
          </div>
        </div>

        <!-- Message input -->
        <v-divider />
        <div class="pa-3">
          <v-text-field
            v-model="newMessage"
            :placeholder="$t('message.placeholder')"
            variant="outlined"
            density="compact"
            hide-details
            append-inner-icon="mdi-send"
            @keydown.enter.exact.prevent="sendMessage"
            @click:append-inner="sendMessage"
            @input="handleTyping"
          />
        </div>
      </v-col>

      <!-- Thread panel -->
      <v-col v-if="activeThread" cols="4" class="border-s d-flex flex-column fill-height">
        <v-toolbar density="compact" flat>
          <v-toolbar-title class="text-body-1">Thread</v-toolbar-title>
          <v-spacer />
          <v-btn icon="mdi-close" size="small" @click="activeThread = null" />
        </v-toolbar>
        <v-divider />
        <div class="flex-grow-1 overflow-y-auto pa-4">
          <div v-for="msg in messageStore.threadMessages" :key="msg.id" class="mb-2">
            <message-bubble :message="msg" compact />
          </div>
        </div>
        <v-divider />
        <div class="pa-3">
          <v-text-field
            v-model="threadReply"
            placeholder="Reply in thread..."
            variant="outlined"
            density="compact"
            hide-details
            append-inner-icon="mdi-send"
            @keydown.enter.exact.prevent="sendThreadReply"
            @click:append-inner="sendThreadReply"
          />
        </div>
      </v-col>

      <!-- Members panel -->
      <v-col v-if="showMembers" cols="3" class="border-s">
        <v-toolbar density="compact" flat>
          <v-toolbar-title class="text-body-1">Members</v-toolbar-title>
          <v-spacer />
          <v-btn icon="mdi-close" size="small" @click="showMembers = false" />
        </v-toolbar>
        <v-divider />
        <v-list density="compact">
          <v-list-item prepend-icon="mdi-account" title="Members list coming soon" />
        </v-list>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useChannelStore } from '@/stores/channels'
import { useMessageStore } from '@/stores/messages'
import { useWsStore } from '@/stores/ws'
import MessageBubble from '@/components/chat/MessageBubble.vue'

const route = useRoute()
const channelStore = useChannelStore()
const messageStore = useMessageStore()
const wsStore = useWsStore()

const tenantId = computed(() => route.params.tenantId as string)
const channelId = computed(() => route.params.channelId as string)

const newMessage = ref('')
const threadReply = ref('')
const activeThread = ref<{ id: string } | null>(null)
const showPinned = ref(false)
const showMembers = ref(false)
const messageListRef = ref<HTMLElement | null>(null)

let typingTimeout: ReturnType<typeof setTimeout> | null = null

async function sendMessage() {
  const content = newMessage.value.trim()
  if (!content) return
  newMessage.value = ''
  await messageStore.sendMessage(tenantId.value, channelId.value, content)
  await nextTick()
  scrollToBottom()
}

async function sendThreadReply() {
  const content = threadReply.value.trim()
  if (!content || !activeThread.value) return
  threadReply.value = ''
  await messageStore.sendMessage(tenantId.value, channelId.value, content, activeThread.value.id)
}

function openThread(msg: { id: string }) {
  activeThread.value = msg
  messageStore.fetchThread(tenantId.value, channelId.value, msg.id)
}

async function handleReact(messageId: string, emoji: string) {
  await messageStore.addReaction(tenantId.value, channelId.value, messageId, emoji)
}

async function handlePin(messageId: string) {
  await messageStore.togglePin(tenantId.value, channelId.value, messageId)
}

function handleTyping() {
  if (typingTimeout) clearTimeout(typingTimeout)
  wsStore.sendTyping(channelId.value)
  typingTimeout = setTimeout(() => {
    wsStore.send('typing:stop', { channel_id: channelId.value })
  }, 3000)
}

function scrollToBottom() {
  if (messageListRef.value) {
    messageListRef.value.scrollTop = messageListRef.value.scrollHeight
  }
}

watch(channelId, async (id) => {
  if (id) {
    await messageStore.fetchMessages(tenantId.value, id)
    await nextTick()
    scrollToBottom()
  }
})

onMounted(async () => {
  if (channelId.value) {
    await messageStore.fetchMessages(tenantId.value, channelId.value)
    await nextTick()
    scrollToBottom()
  }
})
</script>
