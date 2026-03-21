import { test, expect } from '@playwright/test'
import {
  uniqueUser,
  registerUserViaApi,
  createTenantViaApi,
  createRoomViaApi,
  joinRoomViaApi,
  sendMessageViaApi,
  addTenantMemberViaApi,
  loginViaUi,
} from './fixtures/test-helpers'

const API_URL = process.env.E2E_API_URL || 'http://localhost:5001'

test.describe('Email-related flows', () => {
  test.describe('Registration & Activation', () => {
    test('registration returns activation message', async () => {
      const user = uniqueUser()
      const resp = await fetch(`${API_URL}/api/auth/register`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          email: user.email,
          username: user.username,
          password: user.password,
          display_name: user.displayName,
        }),
      })

      expect(resp.status).toBe(201)
      const json = (await resp.json()) as { message: string }
      // The backend returns an activation message prompting user to check email
      expect(json.message).toBeDefined()
      expect(json.message.toLowerCase()).toContain('activate')
    })

    test('unactivated user cannot login', async () => {
      const user = uniqueUser()
      // Register but do NOT activate
      await fetch(`${API_URL}/api/auth/register`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          email: user.email,
          username: user.username,
          password: user.password,
          display_name: user.displayName,
        }),
      })

      // Attempt login should fail
      const loginResp = await fetch(`${API_URL}/api/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          email: user.email,
          password: user.password,
        }),
      })
      expect(loginResp.status).toBe(401)
    })

    test('activation endpoint exists and rejects invalid tokens', async () => {
      const resp = await fetch(`${API_URL}/api/auth/activate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          user_id: '000000000000000000000000',
          token: 'invalid-activation-token',
        }),
      })

      // Should be 400 (bad request) for invalid/expired token
      expect(resp.status).toBe(400)
    })

    test('activated user can login successfully', async () => {
      // registerUserViaApi auto-activates and returns tokens
      const user = uniqueUser()
      const auth = await registerUserViaApi(user)
      expect(auth.access_token).toBeDefined()
      expect(auth.access_token.length).toBeGreaterThan(0)
    })
  })

  test.describe('Mention Notification flows', () => {
    let adminUser: ReturnType<typeof uniqueUser>
    let memberUser: ReturnType<typeof uniqueUser>
    let adminToken: string
    let memberToken: string
    let memberUserId: string
    let tenantId: string
    let roomId: string

    test.beforeEach(async () => {
      adminUser = uniqueUser()
      memberUser = uniqueUser()

      const adminAuth = await registerUserViaApi(adminUser)
      adminToken = adminAuth.access_token

      const memberAuth = await registerUserViaApi(memberUser)
      memberToken = memberAuth.access_token
      memberUserId = memberAuth.user.id

      const tenant = await createTenantViaApi(
        adminToken,
        'Email Test Org',
        `email-test-${Date.now()}`,
      )
      tenantId = tenant.id

      await addTenantMemberViaApi(adminToken, tenantId, memberUserId)

      const room = await createRoomViaApi(adminToken, tenantId, 'email-test-room', true)
      roomId = room.id

      await joinRoomViaApi(adminToken, tenantId, roomId)
      await joinRoomViaApi(memberToken, tenantId, roomId)
    })

    test('sending a mention creates a notification for the mentioned user', async () => {
      // Admin sends a message mentioning the member
      const mentionContent = `Hey @${memberUser.username} please review this`
      await sendMessageViaApi(adminToken, tenantId, roomId, mentionContent)

      // Wait a moment for the notification to be created
      await new Promise((r) => setTimeout(r, 1000))

      // Check the mentioned user's unread notification count
      const resp = await fetch(`${API_URL}/api/notification/unread-count`, {
        headers: { Authorization: `Bearer ${memberToken}` },
      })
      expect(resp.ok).toBeTruthy()
      const countData = (await resp.json()) as { count: number }
      // Should have at least one notification from the mention
      expect(countData.count).toBeGreaterThanOrEqual(1)
    })

    test('notifications API returns mention notification details', async () => {
      // Send mention
      const mentionContent = `Heads up @${memberUser.username}!`
      await sendMessageViaApi(adminToken, tenantId, roomId, mentionContent)

      await new Promise((r) => setTimeout(r, 1000))

      // Fetch the member's unread notifications
      const resp = await fetch(`${API_URL}/api/notification/unread`, {
        headers: { Authorization: `Bearer ${memberToken}` },
      })
      expect(resp.ok).toBeTruthy()
      const notifications = (await resp.json()) as Array<{
        id: string
        type: string
        is_read: boolean
      }>
      // Expect at least one unread notification
      expect(notifications.length).toBeGreaterThanOrEqual(1)
      expect(notifications.some((n) => !n.is_read)).toBe(true)
    })

    test('mark-all-read clears unread notifications', async () => {
      // Create a notification via mention
      await sendMessageViaApi(
        adminToken,
        tenantId,
        roomId,
        `Hey @${memberUser.username}`,
      )
      await new Promise((r) => setTimeout(r, 1000))

      // Mark all as read
      const markResp = await fetch(`${API_URL}/api/notification/read-all`, {
        method: 'POST',
        headers: { Authorization: `Bearer ${memberToken}` },
      })
      expect(markResp.ok).toBeTruthy()

      // Verify unread count is now 0
      const countResp = await fetch(`${API_URL}/api/notification/unread-count`, {
        headers: { Authorization: `Bearer ${memberToken}` },
      })
      expect(countResp.ok).toBeTruthy()
      const countData = (await countResp.json()) as { count: number }
      expect(countData.count).toBe(0)
    })

    test('mention notification is visible in UI notification panel', async ({ page }) => {
      // Send mention
      await sendMessageViaApi(
        adminToken,
        tenantId,
        roomId,
        `Urgent @${memberUser.username} please respond`,
      )

      // Login as the mentioned user
      await loginViaUi(page, memberUser.username, memberUser.password)

      // Wait for notifications to load
      await page.waitForTimeout(2000)

      // Open the notification panel via bell icon
      await page.locator('.mdi-bell-outline').click()
      await page.waitForTimeout(1000)

      // The notification panel should be visible
      await expect(page.locator('.notification-panel')).toBeVisible({ timeout: 5000 })
    })
  })
})
