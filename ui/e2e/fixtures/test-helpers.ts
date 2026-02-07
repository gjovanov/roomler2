import { type Page, expect } from '@playwright/test'

const API_URL = process.env.E2E_API_URL || 'http://localhost:5001'

let userCounter = 0

/** Generate a unique test user for each test */
export function uniqueUser() {
  userCounter++
  const id = `e2e_${Date.now()}_${userCounter}`
  return {
    email: `${id}@test.local`,
    username: id,
    displayName: `E2E User ${userCounter}`,
    password: 'TestPass123!',
  }
}

/** Register a user via the API and return credentials */
export async function registerUserViaApi(user: ReturnType<typeof uniqueUser>) {
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
  if (!resp.ok) throw new Error(`Register failed: ${resp.status}`)
  return (await resp.json()) as { access_token: string; user: { id: string } }
}

/** Create a tenant via the API */
export async function createTenantViaApi(token: string, name: string, slug: string) {
  const resp = await fetch(`${API_URL}/api/tenant`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify({ name, slug }),
  })
  if (!resp.ok) throw new Error(`Create tenant failed: ${resp.status}`)
  return (await resp.json()) as { id: string; name: string; slug: string }
}

/** Login through the UI */
export async function loginViaUi(page: Page, username: string, password: string) {
  await page.goto('/login')
  await page.locator('input').first().fill(username)
  await page.locator('input[type="password"]').fill(password)
  await page.getByRole('button', { name: /login/i }).click()
  await expect(page).toHaveURL(/\/$/, { timeout: 5000 })
}

/** Register through the UI */
export async function registerViaUi(
  page: Page,
  email: string,
  username: string,
  displayName: string,
  password: string,
) {
  await page.goto('/register')
  const inputs = page.locator('input')
  await inputs.nth(0).fill(email)
  await inputs.nth(1).fill(username)
  await inputs.nth(2).fill(displayName)
  await page.locator('input[type="password"]').fill(password)
  await page.getByRole('button', { name: /register/i }).click()
  await expect(page).toHaveURL(/\/$/, { timeout: 5000 })
}
