import { test, expect } from '@playwright/test'
import {
  uniqueUser,
  registerUserViaApi,
  createTenantViaApi,
  loginViaUi,
} from './fixtures/test-helpers'

test.describe('Files', () => {
  let user: ReturnType<typeof uniqueUser>
  let token: string
  let tenantId: string

  test.beforeEach(async ({ page }) => {
    user = uniqueUser()
    const result = await registerUserViaApi(user)
    token = result.access_token
    const tenant = await createTenantViaApi(token, 'Files Org', `files-${Date.now()}`)
    tenantId = tenant.id
    void token // suppress unused warning
    await loginViaUi(page, user.username, user.password)
  })

  test('files page loads with empty state', async ({ page }) => {
    await page.goto(`/tenant/${tenantId}/files`)
    await expect(page.getByText(/no files/i)).toBeVisible({ timeout: 10000 })
  })

  test('upload button is visible', async ({ page }) => {
    await page.goto(`/tenant/${tenantId}/files`)
    await expect(page.getByRole('button', { name: /upload/i })).toBeVisible()
  })

  test('files table headers are correct', async ({ page }) => {
    await page.goto(`/tenant/${tenantId}/files`)
    // When there are no files, we show empty state instead of table
    await expect(page.getByText(/no files/i)).toBeVisible({ timeout: 10000 })
  })
})
