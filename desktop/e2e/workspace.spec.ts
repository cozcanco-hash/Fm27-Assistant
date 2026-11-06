import { expect, test } from '@playwright/test';

test('dashboard shows provenance and a complete locally calculated lineup', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Bir sonraki hamleni planla.' })).toBeVisible();
  await expect(page.getByText('ÖRNEK VERİ', { exact: true })).toBeVisible();
  await expect(page.getByText('11/11 pozisyon dolu', { exact: true })).toBeVisible();
  await expect(page.locator('.pitch-player:not(:disabled)')).toHaveCount(11);
  await page.screenshot({ path: 'test-results/overview.png', fullPage: true });
});

test('player filters combine and a profile supports keyboard dismissal', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Kadro', exact: true }).click();
  await page.getByRole('textbox', { name: 'Oyuncu ara' }).fill('deniz');
  await page.getByLabel('Mevkiye göre filtrele').selectOption('GK');
  await expect(page.locator('tbody tr')).toHaveCount(1);
  await page.getByRole('button', { name: /Deniz Arslan/ }).first().click();
  await expect(page.getByRole('dialog')).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Deniz Arslan', exact: true })).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(page.getByRole('dialog')).toHaveCount(0);
});

test('formation selection uses another Rust-generated analysis', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Taktik laboratuvarı', exact: true }).click();
  await page.getByRole('button', { name: /^4–2–3–1/ }).click();
  await expect(page.locator('#formation')).toHaveValue('4231');
  await expect(page.locator('.pitch-player:not(:disabled)')).toHaveCount(11);
});

test('browser preview never invents an AI response or live game connection', async ({ page }) => {
  const errors: string[] = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.goto('/');
  await page.getByRole('button', { name: 'AI asistan', exact: true }).click();
  await page.getByRole('button', { name: 'Bu kadro için hangi dizilişi önerirsin?' }).click();
  await expect(page.getByRole('button', { name: 'Mesajı gönder' })).toBeDisabled();
  await expect(page.getByRole('button', { name: 'Bağlantıyı kontrol et' })).toBeDisabled();
  await page.getByRole('button', { name: 'Veri & bağlantı', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Verinin kaynağını bil.' })).toBeVisible();
  await expect(page.getByText('Henüz eklenmedi', { exact: true })).toBeVisible();
  expect(errors).toEqual([]);
});
