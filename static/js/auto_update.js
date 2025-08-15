// auto_update.js - Main Controller (Modularized)
import { UIController } from './modules/ui-controller.js';

// Application entry point
class AutoUpdateApp {
    constructor() {
        this.uiController = new UIController();
    }
    
    async init() {
        await this.uiController.init();
        console.log('[AUTO_UPDATE] Application initialized successfully');
    }
}

// Initialize application when DOM is ready
document.addEventListener('DOMContentLoaded', async function() {
    const app = new AutoUpdateApp();
    await app.init();
});
