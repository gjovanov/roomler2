import { test, expect } from '@playwright/test'
import { uniqueUser, registerViaUi, loginViaUi } from './fixtures/test-helpers'

test.describe('Authentication', () => {
  test('register new user and redirect to dashboard', async ({ page }) => {
    const user = uniqueUser()
    await registerViaUi(page, user.email, user.username, user.displayName, user.password)
    await expect(page).toHaveURL('/')
  })

  test('login with valid credentials', async ({ page }) => {
    const user = uniqueUser()
    // Register via UI first
    await registerViaUi(page, user.email, user.username, user.displayName, user.password)
    // Logout by clearing storage
    await page.evaluate(() => localStorage.clear())
    await page.goto('/login')

    // Login
    await loginViaUi(page, user.username, user.password)
    await expect(page).toHaveURL('/')
  })

  test('login with wrong password shows error', async ({ page }) => {
    await page.goto('/login')
    await page.locator('input').first().fill('nonexistent')
    await page.locator('input[type="password"]').fill('wrongpass')
    await page.getByRole('button', { name: /login/i }).click()
    // Should stay on login page with error
    await expect(page).toHaveURL(/\/login/)
  })

  test('unauthenticated user is redirected to login', async ({ page }) => {
    await page.goto('/')
    await expect(page).toHaveURL(/\/login/)
  })

  test('navigate between login and register', async ({ page }) => {
    await page.goto('/login')
    await page.getByRole('link', { name: /register/i }).click()
    await expect(page).toHaveURL(/\/register/)

    await page.getByRole('link', { name: /login/i }).click()
    await expect(page).toHaveURL(/\/login/)
  })
})
