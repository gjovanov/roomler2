import { onMounted } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { useWsStore } from '@/stores/ws'

export function useAuth() {
  const auth = useAuthStore()
  const ws = useWsStore()

  onMounted(async () => {
    if (auth.token) {
      await auth.fetchMe()
      ws.connect(auth.token)
    }
  })

  function logout() {
    ws.disconnect()
    auth.logout()
  }

  return { auth, logout }
}
