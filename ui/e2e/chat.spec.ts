import { test, expect } from '@playwright/test'
import {
  uniqueUser,
  registerUserViaApi,
  createTenantViaApi,
  loginViaUi,
} from './fixtures/test-helpers'

const API_URL = process.env.E2E_API_URL || 'http://localhost:5001'

test.describe('Chat', () => {
  let user: ReturnType<typeof uniqueUser>
  let token: string
  let tenantId: string
  let channelId: string

  test.beforeEach(async ({ page }) => {
    user = uniqueUser()
    const result = await registerUserViaApi(user)
    token = result.access_token
    const tenant = await createTenantViaApi(token, 'Chat Org', `chat-${Date.now()}`)
    tenantId = tenant.id

    // Create a channel via API
    const resp = await fetch(`${API_URL}/api/tenant/${tenantId}/channel`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify({ name: 'general', channel_type: 'text' }),
    })
    const channel = (await resp.json()) as { id: string }
    channelId = channel.id

    // Join channel
    await fetch(`${API_URL}/api/tenant/${tenantId}/channel/${channelId}/join`, {
      method: 'POST',
      headers: { Authorization: `Bearer ${token}` },
    })

    await loginViaUi(page, user.username, user.password)
  })

  test('chat view loads with empty state', async ({ page }) => {
    await page.goto(`/tenant/${tenantId}/channel/${channelId}`)
    await expect(page.getByText(/no messages/i)).toBeVisible({ timeout: 10000 })
  })

  test('message input is visible and functional', async ({ page }) => {
    await page.goto(`/tenant/${tenantId}/channel/${channelId}`)
    const input = page.getByPlaceholder(/type a message/i)
    await expect(input).toBeVisible({ timeout: 10000 })
    await input.fill('Hello from E2E!')
    await expect(input).toHaveValue('Hello from E2E!')
  })
})
