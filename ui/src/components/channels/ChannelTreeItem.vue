<template>
  <div>
    <v-list-item
      :to="`/tenant/${tenantId}/channel/${channel.id}`"
      :style="{ paddingLeft: `${depth * 24 + 16}px` }"
      density="compact"
    >
      <template #prepend>
        <v-icon size="small">
          {{ channelIcon }}
        </v-icon>
      </template>

      <v-list-item-title class="text-body-2">
        {{ channel.name }}
      </v-list-item-title>

      <template #append>
        <v-chip v-if="channel.member_count > 0" size="x-small" variant="text">
          {{ channel.member_count }}
        </v-chip>
      </template>
    </v-list-item>

    <!-- Children -->
    <template v-if="children.length > 0">
      <channel-tree-item
        v-for="child in children"
        :key="child.id"
        :channel="child"
        :tenant-id="tenantId"
        :depth="depth + 1"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useChannelStore } from '@/stores/channels'

interface Channel {
  id: string
  channel_type: string
  name: string
  is_private: boolean
  member_count: number
}

const props = defineProps<{
  channel: Channel
  tenantId: string
  depth: number
}>()

const channelStore = useChannelStore()

const children = computed(() => channelStore.childrenOf(props.channel.id))

const channelIcon = computed(() => {
  if (props.channel.is_private) return 'mdi-lock'
  switch (props.channel.channel_type) {
    case 'voice': return 'mdi-volume-high'
    case 'announcement': return 'mdi-bullhorn'
    case 'forum': return 'mdi-forum'
    case 'category': return 'mdi-folder'
    default: return 'mdi-pound'
  }
})
</script>
