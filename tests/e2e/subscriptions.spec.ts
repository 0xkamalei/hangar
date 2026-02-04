describe('Subscription Management', () => {
    before(async () => {
        // Wait for app to load
        await browser.pause(2000);
    });

    it('should navigate to Subscriptions tab', async () => {
        const subTabButton = await $('button=Subscriptions');
        await subTabButton.click();

        const heading = await $('h1=Subscription Management');
        await expect(heading).toBeDisplayed();
    });

    it('should add a new subscription', async () => {
        const nameInput = await $('input[placeholder*="Premium Proxy"]');
        const urlInput = await $('input[placeholder*="https://example.com"]');
        const addButton = await $('button=Add Subscription');

        await nameInput.setValue('E2E Test Subscription');
        await urlInput.setValue('https://example.com/sub.yaml');
        await addButton.click();

        // Check for success message or the new row
        const row = await $('span=E2E Test Subscription');
        await expect(row).toBeDisplayed();
    });

    it('should toggle selection and show batch bar', async () => {
        const checkbox = await $('input[type="checkbox"]');
        await checkbox.click();

        const batchBar = await $('span*=items selected');
        await expect(batchBar).toBeDisplayed();

        const deleteButton = await $('button=Delete All');
        await expect(deleteButton).toBeDisplayed();
    });

    it('should handle pagination controls', async () => {
        const nextButton = await $('button >> svg[class*="lucide-chevron-right"]').parentElement();
        await expect(nextButton).toExist();
    });
});
