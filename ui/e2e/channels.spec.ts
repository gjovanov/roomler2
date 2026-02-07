import { test, expect } from '@playwright/test'
import {
  uniqueUser,
  registerUserViaApi,
  createTenantViaApi,
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

  test('explore page loads with search', async ({ page }) => {
    await page.goto(`/tenant/${tenantId}/explore`)
    await expect(page.getByText(/explore/i).first()).toBeVisible()
    const searchInput = page.locator('input').first()
    await expect(searchInput).toBeVisible()
  })
})
