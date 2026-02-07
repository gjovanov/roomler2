import { createApp } from 'vue'
import i18n from '@/plugins/i18n'
import vuetify from '@/plugins/vuetify'
import pinia from '@/plugins/pinia'
import router from '@/plugins/router'
import App from '@/App.vue'

const app = createApp(App)

// Plugin order matters: i18n FIRST, vuetify SECOND
app.use(i18n)
app.use(vuetify)
app.use(pinia)
app.use(router)

app.mount('#app')
