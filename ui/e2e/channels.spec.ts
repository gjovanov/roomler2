import { test, expect } from '@playwright/test'
import {
  uniqueUser,
  registerUserViaApi,
  createTenantViaApi,
  createChannelViaApi,
  loginViaUi,
} from './fixtures/test-helpers'

test.describe('Channels', () => {
  let user: ReturnType<typeof uniqueUser>
  let token: string
  let tenantId: string

  test.beforeEach(async ({ page }) => {
    user = uniqueUser()
    const result = await registerUserViaApi(user)
    token = result.access_token
    const tenant = await createTenantViaApi(token, 'Test Org', `test-${Date.now()}`)
    tenantId = tenant.id
    await loginViaUi(page, user.username, user.password)
  })

  test('channel list page loads', async ({ page }) => {
    await page.goto(`/tenant/${tenantId}/channels`)
    await expect(page.getByText(/channels/i).first()).toBeVisible()
  })

  test('create channel dialog opens and closes', async ({ page }) => {
    await page.goto(`/tenant/${tenantId}/channels`)
    await page.getByRole('button', { name: /create channel/i }).click()
    // Dialog should be visible
    await expect(page.getByText(/channel name/i).first()).toBeVisible()

    // Cancel
    await page.getByRole('button', { name: /cancel/i }).click()
  })

  test('create channel via UI and see it in channel list', async ({ page }) => {
    await page.goto(`/tenant/${tenantId}/channels`)

    // Open create dialog
    await page.getByRole('button', { name: /create channel/i }).click()
    await expect(page.getByText(/channel name/i).first()).toBeVisible()

    // Fill in channel name and save
    const nameInput = page.locator('.v-dialog input').first()
    await nameInput.fill('my-test-channel')
    await page.getByRole('button', { name: /save/i }).click()

    // Verify the new channel appears in the list
    await expect(page.getByText('my-test-channel')).toBeVisible({ timeout: 5000 })
  })

  test('create duplicate channel shows error alert', async ({ page }) => {
    // Pre-create a channel via API so we have a duplicate name to test against
    await createChannelViaApi(token, tenantId, 'duplicate-ch')

    await page.goto(`/tenant/${tenantId}/channels`)

    // Open create dialog
    await page.getByRole('button', { name: /create channel/i }).click()
    await expect(page.getByText(/channel name/i).first()).toBeVisible()

    // Try to create a channel with the same name
    const nameInput = page.locator('.v-dialog input').first()
    await nameInput.fill('duplicate-ch')
    await page.getByRole('button', { name: /save/i }).click()

    // The error alert should be displayed inside the dialog
    await expect(page.locator('.v-dialog .v-alert')).toBeVisible({ timeout: 5000 })
  })

  test('channel list displays channels created via API', async ({ page }) => {
    // Create channels via API
    await createChannelViaApi(token, tenantId, 'api-channel-1')
    await createChannelViaApi(token, tenantId, 'api-channel-2')

    await page.goto(`/tenant/${tenantId}/channels`)

    // Both channels should appear in the list (validates store parses plain array)
    await expect(page.getByText('api-channel-1')).toBeVisible({ timeout: 10000 })
    await expect(page.getByText('api-channel-2')).toBeVisible()
  })

  test('explore page loads with search', async ({ page }) => {
    await page.goto(`/tenant/${tenantId}/explore`)
    await expect(page.getByText(/explore/i).first()).toBeVisible()
    const searchInput = page.locator('input').first()
    await expect(searchInput).toBeVisible()
  })
})
