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

test('advanced filters combine age, potential and a hidden attribute', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Kadro', exact: true }).click();
  await page.getByText('Gelişmiş filtreler', { exact: true }).click();
  await page.getByLabel('En yüksek yaş', { exact: true }).fill('20');
  await page.getByLabel('Potansiyel yetenek en az', { exact: true }).fill('150');
  await expect(page.locator('tbody tr')).toHaveCount(2);
  await page.getByRole('button', { name: 'Özellik filtresi ekle' }).click();
  await page.getByLabel('Özellik türü 1', { exact: true }).selectOption('hidden');
  await page.getByLabel('Alt sınır 1', { exact: true }).fill('16');
  await expect(page.locator('tbody tr')).toHaveCount(1);
  await expect(page.locator('tbody')).toContainText('Kaan Ekin');
  await page.screenshot({ path: 'test-results/scouting-filters.png', fullPage: true });
});

test('comparison caps selection at three and shows the same role with missing eligibility', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Kadro', exact: true }).click();
  for (const name of ['Deniz Arslan', 'Efe Ural', 'Mert Aydın']) await page.getByRole('checkbox', { name: `${name} karşılaştır`, exact: true }).check();
  await expect(page.getByRole('checkbox', { name: 'Bora Erden karşılaştır', exact: true })).toBeDisabled();
  const comparison = page.getByRole('region', { name: 'Oyuncu karşılaştırma' });
  await comparison.getByLabel('Karşılaştırılacak rol').selectOption('keeper');
  await expect(comparison.getByText('Mevki uygun değil', { exact: true })).toBeVisible();
  await comparison.getByLabel('Gizli özellikleri göster').check();
  await expect(comparison.getByText('Mevcut yetenek (CA)', { exact: true })).toBeVisible();
  await page.screenshot({ path: 'test-results/scouting-comparison.png', fullPage: true });
  await page.getByRole('button', { name: 'Karşılaştırmayı temizle' }).click();
  await expect(comparison).toHaveCount(0);
});

test('profile reveals optional hidden data and game-dated development', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Kadro', exact: true }).click();
  await page.getByRole('button', { name: 'Deniz Arslan ayrıntıları', exact: true }).click();
  const profile = page.getByRole('dialog');
  await expect(profile.getByText('Mevcut yetenek (CA)', { exact: true })).toHaveCount(0);
  await profile.getByLabel('CA/PA ve gizli özellikleri göster').check();
  await expect(profile.getByText('Mevcut yetenek (CA)', { exact: true })).toBeVisible();
  const history = profile.getByRole('region', { name: 'Oyuncu gelişimi' });
  await expect(history.locator('tbody tr')).toHaveCount(2);
  await expect(history.getByText('+1', { exact: true })).toBeVisible();
  await page.screenshot({ path: 'test-results/scouting-profile.png', fullPage: true });
});

test('preview lists show Rust-generated membership without pretending to persist edits', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Takip listeleri', exact: true }).click();
  await expect(page.locator('tbody tr')).toHaveCount(1);
  await expect(page.locator('tbody')).toContainText('Ali Sarp');
  await expect(page.getByRole('button', { name: 'Liste oluştur', exact: true })).toBeDisabled();
  await expect(page.getByText('Örnek listeyi inceliyorsun.', { exact: false })).toBeVisible();
});
