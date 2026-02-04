describe('Server Lifecycle', () => {
    it('should navigate to Main tab', async () => {
        const mainTabButton = await $('button=Main');
        await mainTabButton.click();

        const heading = await $('h2=Server Control');
        await expect(heading).toBeDisplayed();
    });

    it('should start the server', async () => {
        const startButton = await $('button=Start Server');
        await startButton.click();

        const statusText = await $('p=System Active');
        await expect(statusText).toBeDisplayed();

        const stopButton = await $('button=Stop Server');
        await expect(stopButton).toBeDisplayed();
    });

    it('should stop the server', async () => {
        const stopButton = await $('button=Stop Server');
        await stopButton.click();

        const statusText = await $('p=System Stopped');
        await expect(statusText).toBeDisplayed();
    });

    it('should copy subscription URL', async () => {
        const copyButton = await $('button >> svg[class*="lucide-copy"]').parentElement();
        await copyButton.click();

        // Success message should appear
        const successMsg = await $('div*=链接已复制');
        await expect(successMsg).toBeDisplayed();
    });
});
