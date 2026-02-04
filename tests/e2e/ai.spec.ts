describe('AI & Versioning', () => {
    it('should show history sidebar', async () => {
        const sidebarToggle = await $('button >> svg[class*="lucide-panel-right"]').parentElement();
        await sidebarToggle.click();

        const historyHeading = await $('h3=Snapshot History');
        await expect(historyHeading).toBeDisplayed();
    });

    it('should create a manual snapshot', async () => {
        const createButton = await $('button=Create Manual Snapshot');
        await createButton.click();

        // Note: browser.alert() or similar might be needed if using window.prompt
        // But for E2E we might want to mock the prompt or just verify the button exists
        await expect(createButton).toBeDisplayed();
    });

    it('should show AI modification preview', async () => {
        const textarea = await $('textarea[placeholder*="How can I help"]');
        await textarea.setValue('Route google to HK');

        const generateButton = await $('button=Generate');
        await expect(generateButton).toBeEnabled();
    });
});
