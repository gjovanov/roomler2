<template>
  <v-container>
    <v-row>
      <v-col cols="12" class="d-flex align-center">
        <h1 class="text-h4">{{ $t('nav.channels') }}</h1>
        <v-spacer />
        <v-btn color="primary" prepend-icon="mdi-plus" @click="showCreate = true">
          {{ $t('channel.create') }}
        </v-btn>
      </v-col>
    </v-row>

    <v-row>
      <v-col cols="12">
        <v-card v-if="channelStore.loading">
          <v-card-text class="text-center">
            <v-progress-circular indeterminate />
          </v-card-text>
        </v-card>

        <v-list v-else>
          <template v-for="ch in channelStore.rootChannels" :key="ch.id">
            <channel-tree-item :channel="ch" :tenant-id="tenantId" :depth="0" />
          </template>
        </v-list>
      </v-col>
    </v-row>

    <!-- Create channel dialog -->
    <v-dialog v-model="showCreate" max-width="500">
      <v-card>
        <v-card-title>{{ $t('channel.create') }}</v-card-title>
        <v-card-text>
          <v-text-field v-model="newChannel.name" :label="$t('channel.name')" required />
          <v-select
            v-model="newChannel.type"
            :label="$t('channel.type')"
            :items="channelTypes"
          />
          <v-checkbox v-model="newChannel.is_private" :label="$t('channel.private')" />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="showCreate = false">{{ $t('common.cancel') }}</v-btn>
          <v-btn color="primary" @click="createChannel">{{ $t('common.save') }}</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-container>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, reactive } from 'vue'
import { useRoute } from 'vue-router'
import { useChannelStore } from '@/stores/channels'
import ChannelTreeItem from '@/components/channels/ChannelTreeItem.vue'

const route = useRoute()
const channelStore = useChannelStore()

const tenantId = computed(() => route.params.tenantId as string)
const showCreate = ref(false)
const channelTypes = ['text', 'voice', 'announcement', 'forum']

const newChannel = reactive({
  name: '',
  type: 'text',
  is_private: false,
})

async function createChannel() {
  await channelStore.createChannel(tenantId.value, {
    name: newChannel.name,
    channel_type: newChannel.type,
    is_private: newChannel.is_private,
  })
  showCreate.value = false
  newChannel.name = ''
  newChannel.type = 'text'
  newChannel.is_private = false
}

onMounted(() => {
  channelStore.fetchChannels(tenantId.value)
})
</script>
