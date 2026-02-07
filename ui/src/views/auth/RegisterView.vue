<template>
  <v-container class="fill-height" fluid>
    <v-row align="center" justify="center">
      <v-col cols="12" sm="8" md="4">
        <v-card class="pa-4">
          <v-card-title class="text-center text-h5 mb-4">
            <v-icon color="primary" class="mr-2">mdi-forum</v-icon>
            {{ $t('auth.register') }}
          </v-card-title>

          <v-form @submit.prevent="handleRegister">
            <v-text-field
              v-model="email"
              :label="$t('auth.email')"
              prepend-inner-icon="mdi-email"
              type="email"
              required
              autofocus
            />
            <v-text-field
              v-model="username"
              :label="$t('auth.username')"
              prepend-inner-icon="mdi-account"
              required
            />
            <v-text-field
              v-model="displayName"
              :label="$t('auth.displayName')"
              prepend-inner-icon="mdi-badge-account"
              required
            />
            <v-text-field
              v-model="password"
              :label="$t('auth.password')"
              prepend-inner-icon="mdi-lock"
              type="password"
              required
            />

            <v-alert v-if="auth.error" type="error" density="compact" class="mb-4">
              {{ auth.error }}
            </v-alert>

            <v-btn
              type="submit"
              color="primary"
              block
              size="large"
              :loading="auth.loading"
            >
              {{ $t('auth.register') }}
            </v-btn>
          </v-form>

          <v-card-text class="text-center">
            {{ $t('auth.hasAccount') }}
            <router-link to="/login">{{ $t('auth.login') }}</router-link>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useWsStore } from '@/stores/ws'

const auth = useAuthStore()
const ws = useWsStore()
const router = useRouter()

const email = ref('')
const username = ref('')
const displayName = ref('')
const password = ref('')

async function handleRegister() {
  try {
    await auth.register(email.value, username.value, password.value, displayName.value)
    ws.connect(auth.token!)
    router.push({ name: 'dashboard' })
  } catch {
    // error handled by store
  }
}
</script>
