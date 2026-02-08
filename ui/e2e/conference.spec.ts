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
    // The toolbar should show the subject once loaded
    await expect(page.getByText(/meeting/i).first()).toBeVisible({ timeout: 10000 })
  })

  test('join conference shows local video', async ({ page, context }) => {
    // Grant camera/mic permissions
    await context.grantPermissions(['camera', 'microphone'])

    // Start conference via API first
    await fetch(`${API_URL}/api/tenant/${tenantId}/conference/${conferenceId}/start`, {
      method: 'POST',
      headers: { Authorization: `Bearer ${token}` },
    })

    await page.goto(`/tenant/${tenantId}/conference/${conferenceId}`)
    await expect(page.getByRole('button', { name: /join/i })).toBeVisible({ timeout: 10000 })

    // Click join
    await page.getByRole('button', { name: /join/i }).click()

    // Should see the video tile with "You"
    await expect(page.getByText('You')).toBeVisible({ timeout: 15000 })
  })

  test('mute/unmute toggles audio icon', async ({ page, context }) => {
    await context.grantPermissions(['camera', 'microphone'])

    await fetch(`${API_URL}/api/tenant/${tenantId}/conference/${conferenceId}/start`, {
      method: 'POST',
      headers: { Authorization: `Bearer ${token}` },
    })

    await page.goto(`/tenant/${tenantId}/conference/${conferenceId}`)
    await page.getByRole('button', { name: /join/i }).click()
    await expect(page.getByText('You')).toBeVisible({ timeout: 15000 })

    // Click mute button (microphone icon)
    const muteBtn = page.locator('button:has(.mdi-microphone)').first()
    await muteBtn.click()

    // After mute, icon should change to microphone-off
    await expect(page.locator('.mdi-microphone-off').first()).toBeVisible({ timeout: 5000 })
  })

  test('camera toggle changes icon', async ({ page, context }) => {
    await context.grantPermissions(['camera', 'microphone'])

    await fetch(`${API_URL}/api/tenant/${tenantId}/conference/${conferenceId}/start`, {
      method: 'POST',
      headers: { Authorization: `Bearer ${token}` },
    })

    await page.goto(`/tenant/${tenantId}/conference/${conferenceId}`)
    await page.getByRole('button', { name: /join/i }).click()
    await expect(page.getByText('You')).toBeVisible({ timeout: 15000 })

    // Click camera-off button (video icon)
    const cameraBtn = page.locator('button:has(.mdi-video)').first()
    await cameraBtn.click()

    // After toggle, icon should change to video-off
    await expect(page.locator('.mdi-video-off').first()).toBeVisible({ timeout: 5000 })
  })

  test('leave conference returns to dashboard', async ({ page, context }) => {
    await context.grantPermissions(['camera', 'microphone'])

    await fetch(`${API_URL}/api/tenant/${tenantId}/conference/${conferenceId}/start`, {
      method: 'POST',
      headers: { Authorization: `Bearer ${token}` },
    })

    await page.goto(`/tenant/${tenantId}/conference/${conferenceId}`)
    await page.getByRole('button', { name: /join/i }).click()
    await expect(page.getByText('You')).toBeVisible({ timeout: 15000 })

    // Click hangup button
    const hangupBtn = page.locator('button:has(.mdi-phone-hangup)').first()
    await hangupBtn.click()

    // Should navigate back to tenant dashboard
    await expect(page).toHaveURL(new RegExp(`/tenant/${tenantId}`), { timeout: 10000 })
  })
})
