import { test, expect } from '@playwright/test';

test.describe('Plan Viewer', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for initial load
    await expect(page.locator('#change-id')).not.toBeEmpty();
  });

  test('displays change ID in sidebar', async ({ page }) => {
    await expect(page.locator('#change-id')).toContainText('plan-viewer-ui');
  });

  test('lists all files in navigation', async ({ page }) => {
    const nav = page.locator('#file-nav');
    await expect(nav.locator('.file-item')).toHaveCount(6);
    await expect(nav).toContainText('proposal.md');
    await expect(nav).toContainText('CHALLENGE.md');
    await expect(nav).toContainText('STATE.yaml');
    await expect(nav).toContainText('tasks.md');
  });

  test('loads first file automatically', async ({ page }) => {
    await expect(page.locator('#current-file')).toContainText('proposal.md');
    await expect(page.locator('#content-body')).not.toContainText('Loading...');
  });

  test('can navigate between files', async ({ page }) => {
    // Click on STATE.yaml
    await page.locator('.file-item', { hasText: 'STATE.yaml' }).click();
    await expect(page.locator('#current-file')).toContainText('STATE.yaml');
    await expect(page.locator('#content-body')).toContainText('phase');

    // Click on tasks.md
    await page.locator('.file-item', { hasText: 'tasks.md' }).click();
    await expect(page.locator('#current-file')).toContainText('tasks.md');
  });

  test('can navigate to specs files', async ({ page }) => {
    await page.locator('.file-item', { hasText: 'specs/plan-viewer.md' }).click();
    await expect(page.locator('#current-file')).toContainText('specs/plan-viewer.md');
    await expect(page.locator('#content-body h1')).toBeVisible();
  });

  test('renders markdown with headings', async ({ page }) => {
    // proposal.md should have headings with IDs
    const heading = page.locator('#content-body h1[id], #content-body h2[id]').first();
    await expect(heading).toBeVisible();
  });

  test('renders code blocks with syntax highlighting', async ({ page }) => {
    // Navigate to a file with code blocks
    await page.locator('.file-item', { hasText: 'specs/' }).first().click();
    await page.waitForTimeout(500);

    // Check for highlighted code (if present)
    const codeBlock = page.locator('#content-body pre code');
    if (await codeBlock.count() > 0) {
      await expect(codeBlock.first()).toBeVisible();
    }
  });

  test('shows comment count in sidebar', async ({ page }) => {
    await expect(page.locator('#comment-count')).toContainText('comment');
  });

  test('has approve and request changes buttons', async ({ page }) => {
    await expect(page.locator('#btn-approve')).toBeVisible();
    await expect(page.locator('#btn-request-changes')).toBeVisible();
  });
});

test.describe('Annotations', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#change-id')).not.toBeEmpty();
  });

  test('can open annotation modal by clicking heading', async ({ page }) => {
    // Wait for content to load
    await page.waitForSelector('#content-body h1[id], #content-body h2[id]');

    // Click on first heading
    const heading = page.locator('#content-body h1[id], #content-body h2[id]').first();
    await heading.click();

    // Modal should open
    await expect(page.locator('#annotation-modal')).toBeVisible();
  });

  test('can open annotation modal via comment button', async ({ page }) => {
    await page.waitForSelector('.section-comment-btn');

    // Click comment button
    await page.locator('.section-comment-btn').first().click();

    // Modal should open
    await expect(page.locator('#annotation-modal')).toBeVisible();
  });

  test('can close annotation modal', async ({ page }) => {
    // Open modal
    await page.waitForSelector('.section-comment-btn');
    await page.locator('.section-comment-btn').first().click();
    await expect(page.locator('#annotation-modal')).toBeVisible();

    // Close via X button
    await page.locator('#modal-close').click();
    await expect(page.locator('#annotation-modal')).not.toBeVisible();
  });

  test('can add annotation', async ({ page }) => {
    // Open modal
    await page.waitForSelector('.section-comment-btn');
    await page.locator('.section-comment-btn').first().click();

    // Use unique comment text
    const commentText = `Test comment ${Date.now()}`;
    await page.locator('#annotation-content').fill(commentText);

    // Save
    await page.locator('#btn-save').click();

    // Modal should close
    await expect(page.locator('#annotation-modal')).not.toBeVisible();

    // Annotation should appear in sidebar (use last card since it's the newest)
    await expect(page.locator('.annotation-card').last()).toContainText(commentText);

    // Comment count should update
    await expect(page.locator('#comment-count')).not.toContainText('No comments');
  });

  test('shows validation error for empty comment', async ({ page }) => {
    // Open modal
    await page.waitForSelector('.section-comment-btn');
    await page.locator('.section-comment-btn').first().click();

    // Try to save without content
    await page.locator('#btn-save').click();

    // Should show toast error
    await expect(page.locator('.toast')).toContainText('Please enter a comment');

    // Modal should stay open
    await expect(page.locator('#annotation-modal')).toBeVisible();
  });
});

test.describe('Review Actions', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#change-id')).not.toBeEmpty();
  });

  test('request changes button exists and is clickable', async ({ page }) => {
    const requestChangesBtn = page.locator('#btn-request-changes');
    await expect(requestChangesBtn).toBeVisible();
    await expect(requestChangesBtn).toBeEnabled();

    // Button should contain expected text
    await expect(requestChangesBtn).toContainText('Request Changes');
  });

  test('approve shows confirmation dialog', async ({ page }) => {
    // Set up dialog handler
    page.on('dialog', async dialog => {
      expect(dialog.type()).toBe('confirm');
      expect(dialog.message()).toContain('Approve');
      await dialog.dismiss();
    });

    await page.locator('#btn-approve').click();
  });
});

test.describe('Screenshots', () => {
  test('capture viewer screenshot', async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('#content-body h1, #content-body h2');
    await page.screenshot({ path: 'screenshots/viewer-main.png', fullPage: true });
  });

  test('capture annotation modal screenshot', async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.section-comment-btn');
    await page.locator('.section-comment-btn').first().click();
    await page.screenshot({ path: 'screenshots/annotation-modal.png' });
  });

  test('capture specs file screenshot', async ({ page }) => {
    await page.goto('/');
    await page.locator('.file-item', { hasText: 'specs/plan-viewer.md' }).click();
    await page.waitForSelector('#content-body h1');
    await page.screenshot({ path: 'screenshots/specs-file.png', fullPage: true });
  });
});
