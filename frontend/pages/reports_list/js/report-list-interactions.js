/**
 * Report List Table Interactions
 * Handles table row hover effects and interactions
 */

(function() {
    'use strict';
    
    /**
     * Initialize table row hover effects
     */
    function initTableRowHoverEffects() {
        const tableRows = document.querySelectorAll('tbody tr[onmouseover]');
        
        tableRows.forEach(row => {
            // Remove inline event handlers
            row.removeAttribute('onmouseover');
            row.removeAttribute('onmouseout');
            
            // Add event listeners
            row.addEventListener('mouseenter', function() {
                this.style.backgroundColor = 'var(--table-row-hover-bg)';
            });
            
            row.addEventListener('mouseleave', function() {
                this.style.backgroundColor = 'transparent';
            });
        });
        
        console.log('✅ Table hover effects: Initialized for', tableRows.length, 'rows');
    }
    
    /**
     * Initialize on DOM ready
     */
    function init() {
        initTableRowHoverEffects();
    }
    
    // Initialize when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
