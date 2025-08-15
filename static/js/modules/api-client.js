// api-client.js - Centralized API communication
export class APIClient {
    static async getProgress(sessionId) {
        try {
            const response = await fetch(`/api/progress/${sessionId}`);
            const data = await response.json();
            return data.success ? data.progress : null;
        } catch (error) {
            console.error('[API] Progress fetch error:', error);
            return null;
        }
    }
    
    static async getSchedulerStatus() {
        try {
            const response = await fetch('/scheduler-status');
            return await response.json();
        } catch (error) {
            console.error('[API] Status fetch error:', error);
            throw error;
        }
    }
    
    static async triggerReport() {
        try {
            const response = await fetch('/generate-auto-report', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/x-www-form-urlencoded',
                }
            });
            return await response.json();
        } catch (error) {
            console.error('[API] Report trigger error:', error);
            throw error;
        }
    }
}
