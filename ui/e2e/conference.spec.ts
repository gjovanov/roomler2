import { test, expect } from '@playwright/test'
import {
  uniqueUser,
  registerUserViaApi,
  createTenantViaApi,
  loginViaUi,
} from './fixtures/test-helpers'

const API_URL = process.env.E2E_API_URL || 'http://localhost:5001'

test.describe('Conference', () => {
  let user: ReturnType<typeof uniqueUser>
  let token: string
  let tenantId: string
  let conferenceId: string

  test.beforeEach(async ({ page }) => {
    user = uniqueUser()
    const result = await registerUserViaApi(user)
    token = result.access_token
    const tenant = await createTenantViaApi(token, 'Conf Org', `conf-${Date.now()}`)
    tenantId = tenant.id

    // Create a conference via API
    const resp = await fetch(`${API_URL}/api/tenant/${tenantId}/conference`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify({ subject: 'E2E Meeting' }),
    })
    const conf = (await resp.json()) as { id: string }
    conferenceId = conf.id

    await loginViaUi(page, user.username, user.password)
  })

  test('conference view loads with join button', async ({ page }) => {
    await page.goto(`/tenant/${tenantId}/conference/${conferenceId}`)
    await expect(page.getByText(/ready to join/i)).toBeVisible({ timeout: 10000 })
    await expect(page.getByRole('button', { name: /join/i })).toBeVisible()
  })

  test('conference view shows meeting subject', async ({ page }) => {
    await page.goto(`/tenant/${tenantId}/conference/${conferenceId}`)
    // The toolbar should show the subject once joined
    await expect(page.getByText(/meeting/i).first()).toBeVisible({ timeout: 10000 })
  })
})
