import { test, expect } from '@playwright/test'
import {
  uniqueUser,
  registerUserViaApi,
  createTenantViaApi,
  createConferenceViaApi,
  startConferenceViaApi,
  joinConferenceViaApi,
  sendConferenceChatViaApi,
  loginViaUi,
} from './fixtures/test-helpers'

test.describe('Conference Chat', () => {
  let user: ReturnType<typeof uniqueUser>
  let token: string
  let tenantId: string
  let conferenceId: string

  test.beforeEach(async ({ page }) => {
    user = uniqueUser()
    const result = await registerUserViaApi(user)
    token = result.access_token
    const tenant = await createTenantViaApi(token, 'Chat Org', `chat-${Date.now()}`)
    tenantId = tenant.id

    const conf = await createConferenceViaApi(token, tenantId, 'Chat Meeting')
    conferenceId = conf.id

    await startConferenceViaApi(token, tenantId, conferenceId)
    await joinConferenceViaApi(token, tenantId, conferenceId)

    await loginViaUi(page, user.username, user.password)
  })

  test('conference chat panel toggles visibility', async ({ page, context }) => {
    await context.grantPermissions(['camera', 'microphone'])
    await page.goto(`/tenant/${tenantId}/conference/${conferenceId}`)

    // Before joining, chat toggle should not be visible
    await expect(page.locator('[class*="mdi-message-text"]')).not.toBeVisible()

    // Join the conference
    await page.getByRole('button', { name: /join/i }).click()

    // After joining, chat panel should be visible (auto-opens)
    await expect(page.getByText('Chat')).toBeVisible({ timeout: 15000 })

    // Click the chat toggle to hide
    await page.locator('button:has(.mdi-message-text)').first().click()
    await expect(page.getByText('No messages yet')).not.toBeVisible()

    // Click again to show
    await page.locator('button:has(.mdi-message-text-outline)').first().click()
    await expect(page.getByText('No messages yet')).toBeVisible()
  })

  test('send and receive conference chat message', async ({ page, context }) => {
    await context.grantPermissions(['camera', 'microphone'])
    await page.goto(`/tenant/${tenantId}/conference/${conferenceId}`)

    // Join the conference
    await page.getByRole('button', { name: /join/i }).click()
    await expect(page.getByText('Chat')).toBeVisible({ timeout: 15000 })

    // Type and send a message
    const chatInput = page.getByPlaceholder('Type a message...')
    await chatInput.fill('Hello from E2E!')
    await chatInput.press('Enter')

    // Verify the message appears in the chat panel
    await expect(page.getByText('Hello from E2E!')).toBeVisible({ timeout: 5000 })
  })
})
