/**
 * BaseUpdater - Base class for all market indicator updaters
 * 
 * Provides common functionality:
 * - Element management
 * - Animation
 * - Translation updates
 * - Skeleton removal
 */

const DEBUG_MODE = false;

function debugLog(...args) {
    if (DEBUG_MODE) console.log(...args);
}

export class BaseUpdater {
    constructor(elementId) {
        this.elementId = elementId;
        this.element = document.getElementById(elementId);
        this.updateAnimationDuration = 300;
        
        if (!this.element) {
            console.warn(`⚠️ Element not found: ${elementId}`);
        }
    }
    
    /**
     * Check if element exists
     * @returns {boolean}
     */
    exists() {
        return this.element !== null;
    }
    
    /**
     * Remove skeleton loader from element
     */
    removeSkeleton() {
        if (!this.element) return;
        
        const skeleton = this.element.querySelector('.skeleton-loader, .skeleton');
        if (skeleton) {
            skeleton.remove();
            debugLog(`🦴 Removed skeleton from ${this.elementId}`);
        }
    }
    
    /**
     * Animate element update with fade-in effect
     */
    animate() {
        if (!this.element) return;
        
        this.element.classList.add('fade-in');
        setTimeout(() => {
            this.element.classList.remove('fade-in');
        }, this.updateAnimationDuration);
    }
    
    /**
     * Update translations for internationalization
     * Triggers translation update if window.updateTranslations exists
     */
    updateTranslations() {
        if (!this.element) return;
        
        if (typeof window.updateTranslations === 'function') {
            window.updateTranslations(this.element);
        }
    }
    
    /**
     * Set element innerHTML
     * @param {string} html - HTML content
     */
    setHTML(html) {
        if (!this.element) return;
        
        this.removeSkeleton();
        this.element.innerHTML = html;
        this.updateTranslations();
        this.animate();
    }
    
    /**
     * Format large numbers with appropriate units (B, M, K)
     * @param {number} num - Number to format
     * @returns {Object} { number: string, unitText: string, unitKey: string }
     */
    formatLargeNumber(num) {
        if (num >= 1e12) {
            return {
                number: (num / 1e12).toFixed(2),
                unitText: 'T',
                unitKey: 'trillion'
            };
        } else if (num >= 1e9) {
            return {
                number: (num / 1e9).toFixed(2),
                unitText: 'B',
                unitKey: 'billion'
            };
        } else if (num >= 1e6) {
            return {
                number: (num / 1e6).toFixed(2),
                unitText: 'M',
                unitKey: 'million'
            };
        } else if (num >= 1e3) {
            return {
                number: (num / 1e3).toFixed(2),
                unitText: 'K',
                unitKey: 'thousand'
            };
        } else {
            return {
                number: num.toFixed(2),
                unitText: '',
                unitKey: ''
            };
        }
    }
    
    /**
     * Get change class and icon based on value
     * @param {number} changeValue - Change value (can be positive or negative)
     * @returns {Object} { class: string, icon: string, sign: string }
     */
    getChangeInfo(changeValue) {
        const isPositive = changeValue >= 0;
        return {
            class: isPositive ? 'positive' : 'negative',
            icon: isPositive ? '📈' : '📉',
            sign: isPositive ? '+' : ''
        };
    }
    
    /**
     * Update method - to be overridden by child classes
     * @param {*} data - Data to update
     */
    update(data) {
        throw new Error('update() must be implemented by child class');
    }
}
