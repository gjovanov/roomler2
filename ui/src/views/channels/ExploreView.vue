<template>
  <v-container>
    <v-row>
      <v-col cols="12">
        <h1 class="text-h4 mb-4">{{ $t('nav.explore') }}</h1>
        <v-text-field
          v-model="query"
          :label="$t('common.search')"
          prepend-inner-icon="mdi-magnify"
          clearable
          @input="debounceSearch"
        />
      </v-col>
    </v-row>

    <v-row>
      <v-col v-for="ch in results" :key="ch.id" cols="12" sm="6" md="4">
        <v-card>
          <v-card-title>
            <v-icon class="mr-2" size="small">
              {{ ch.is_private ? 'mdi-lock' : 'mdi-pound' }}
            </v-icon>
            {{ ch.name }}
          </v-card-title>
          <v-card-subtitle>{{ ch.member_count }} members</v-card-subtitle>
          <v-card-text v-if="ch.topic?.value">{{ ch.topic.value }}</v-card-text>
          <v-card-actions>
            <v-btn color="primary" @click="join(ch.id)">{{ $t('channel.join') }}</v-btn>
          </v-card-actions>
        </v-card>
      </v-col>
    </v-row>

    <v-row v-if="results.length === 0 && query">
      <v-col cols="12" class="text-center text-medium-emphasis pa-8">
        No channels found matching "{{ query }}"
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useChannelStore } from '@/stores/channels'

const route = useRoute()
const router = useRouter()
const channelStore = useChannelStore()

const tenantId = computed(() => route.params.tenantId as string)
const query = ref('')

interface Channel {
  id: string
  name: string
  is_private: boolean
  member_count: number
  topic?: { value?: string }
}

const results = ref<Channel[]>([])
let searchTimeout: ReturnType<typeof setTimeout> | null = null

function debounceSearch() {
  if (searchTimeout) clearTimeout(searchTimeout)
  searchTimeout = setTimeout(async () => {
    if (query.value.trim()) {
      const data = await channelStore.explore(tenantId.value, query.value)
      results.value = data.items
    } else {
      results.value = []
    }
  }, 300)
}

async function join(channelId: string) {
  await channelStore.joinChannel(tenantId.value, channelId)
  router.push(`/tenant/${tenantId.value}/channel/${channelId}`)
}
</script>
