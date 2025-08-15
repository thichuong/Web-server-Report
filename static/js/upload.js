// Upload page JavaScript functionality

// Import WebSocket client
import { wsClient } from './modules/websocket-client.js';

// Function to notify all iframes about theme change
function notifyIframesThemeChange(theme) {
    const iframes = document.querySelectorAll('iframe');
    iframes.forEach(iframe => {
        try {
            iframe.contentWindow.postMessage({
                type: 'themeChange',
                theme: theme
            }, '*');
        } catch (e) {
            console.log('Could not send theme message to iframe:', e);
        }
    });
}

// Initialize upload page functionality
document.addEventListener('DOMContentLoaded', async function() {
    // Initialize WebSocket connection for progress tracking
    try {
        await wsClient.connect();
        console.log('[Upload] WebSocket connected successfully');
    } catch (error) {
        console.warn('[Upload] WebSocket connection failed, will use polling fallback:', error);
    }
    
    // Theme handling is now managed by theme-manager.js
    // Just need to notify iframes when theme changes
    const observer = new MutationObserver((mutations) => {
        mutations.forEach((mutation) => {
            if (mutation.type === 'attributes' && mutation.attributeName === 'data-theme') {
                const newTheme = document.documentElement.getAttribute('data-theme');
                notifyIframesThemeChange(newTheme);
            }
        });
    });
    
    observer.observe(document.documentElement, {
        attributes: true,
        attributeFilter: ['data-theme']
    });
    
    // Initialize iframe theme after a delay
    setTimeout(() => {
        const currentTheme = document.documentElement.getAttribute('data-theme') || 'light';
        notifyIframesThemeChange(currentTheme === 'light' ? null : currentTheme);
    }, 1000);
});

// File selection handling
const fileInput = document.getElementById('file-upload');
const fileInputContainer = document.getElementById('file-input-container');
const fileDisplay = document.getElementById('file-display');
const fileNameText = document.getElementById('file-name-text');
const fileTypeText = document.getElementById('file-type-text');
const fileIcon = document.getElementById('file-icon');

function getFileIcon(fileName) {
    const extension = fileName.split('.').pop().toLowerCase();
    switch(extension) {
        case 'docx':
            return 'fas fa-file-word';
        case 'odt':
            return 'fas fa-file-alt';
        default:
            return 'fas fa-file';
    }
}

function getFileType(fileName) {
    const extension = fileName.split('.').pop().toLowerCase();
    switch(extension) {
        case 'docx':
            return 'Microsoft Word Document';
        case 'odt':
            return 'OpenDocument Text';
        default:
            return 'Document';
    }
}

function removeFile() {
    fileInput.value = '';
    fileInputContainer.style.display = 'block';
    fileDisplay.style.display = 'none';
}

fileInput.addEventListener('change', () => {
    if (fileInput.files.length > 0) {
        const file = fileInput.files[0];
        const fileName = file.name;
        
        // Update file display
        fileNameText.textContent = fileName;
        fileTypeText.textContent = getFileType(fileName);
        fileIcon.className = `file-icon ${getFileIcon(fileName)}`;
        
        // Show selected file display and hide input
        fileInputContainer.style.display = 'none';
        fileDisplay.style.display = 'flex';
    }
});

// Form submission handling
const uploadForm = document.getElementById('upload-form');
const uploadFormContainer = document.getElementById('upload-form-container');
const processingMessage = document.getElementById('processing-message');
const successOverlay = document.getElementById('success-overlay');
const errorOverlay = document.getElementById('error-overlay');
const successMessage = document.getElementById('success-message');
const errorMessage = document.getElementById('error-message');
const submitBtn = document.getElementById('submit-btn');

// Game iframe reference
let gameIframe = null;

uploadForm.addEventListener('submit', function(event) {
    event.preventDefault(); // Prevent normal form submission
    
    // Scroll to Game Card immediately when submit is clicked
    setTimeout(() => {
        const gameIframe = document.getElementById('main-game-iframe');
        if (gameIframe) {
            
            // Send message to iframe to scroll to game-header
            setTimeout(() => {
                if (gameIframe.contentWindow) {
                    gameIframe.contentWindow.postMessage({ 
                        type: 'scrollToHeader'
                    }, '*');
                }
            }, 200);
        }
    }, 100);
    
    // Check if required fields are filled
    const apiKey = document.getElementById('gemini-key').value;
    const file = document.getElementById('file-upload').files.length;
    
    if (!apiKey || file === 0) {
        showError('Vui lòng cung cấp đủ tệp và API Key.');
        return;
    }

    // Show processing state
    showProcessingState();
    //startGame();

    // Create FormData to send file and data
    const formData = new FormData(uploadForm);

    // Send AJAX request
    fetch(uploadForm.action, {
        method: 'POST',
        body: formData
    })
    .then(response => response.json())
    .then(data => {
        // Hide processing state
        hideProcessingState();
        stopGame();
        
        if (data.success) {
            showSuccess(data.message);
        } else {
            showError(data.message);
        }
    })
    .catch(error => {
        // Hide processing state
        hideProcessingState();
        stopGame();
        showError('Đã xảy ra lỗi kết nối. Vui lòng thử lại.');
        console.error('Error:', error);
    });
});

function showProcessingState() {
    uploadFormContainer.classList.add('processing');
    processingMessage.classList.add('show');
    submitBtn.innerHTML = '<i class="fas fa-spinner fa-spin mr-3"></i>Đang xử lý...';
    submitBtn.disabled = true;
}

function hideProcessingState() {
    uploadFormContainer.classList.remove('processing');
    processingMessage.classList.remove('show');
    submitBtn.innerHTML = '<i class="fas fa-cogs mr-3"></i>Xử lý và Tạo Báo cáo';
    submitBtn.disabled = false;
}

function showSuccess(message) {
    successMessage.textContent = message + ' Click nút bên dưới để xem báo cáo mới.';
    successOverlay.style.display = 'flex';
}

function showError(message) {
    errorMessage.textContent = message;
    errorOverlay.style.display = 'flex';
}

function goToHome() {
    // Navigate to home page to view new report
    window.location.href = "/";
}

function closeErrorOverlay() {
    errorOverlay.style.display = 'none';
}

// Game iframe control functions
function startGame() {
    gameIframe = document.getElementById('main-game-iframe');
    if (gameIframe && gameIframe.contentWindow) {
        gameIframe.contentWindow.postMessage({ type: 'startGame' }, '*');
    }
}

function stopGame() {
    if (gameIframe && gameIframe.contentWindow) {
        gameIframe.contentWindow.postMessage({ type: 'stopGame' }, '*');
    }
}

function pauseGame() {
    if (gameIframe && gameIframe.contentWindow) {
        gameIframe.contentWindow.postMessage({ type: 'pauseGame' }, '*');
    }
}

function resumeGame() {
    if (gameIframe && gameIframe.contentWindow) {
        gameIframe.contentWindow.postMessage({ type: 'resumeGame' }, '*');
    }
}

// Auto-resize iframe to fit content
function resizeIframe(iframe) {
    try {
        if (iframe && iframe.contentWindow && iframe.contentWindow.document) {
            const doc = iframe.contentWindow.document;
            const body = doc.body;
            const html = doc.documentElement;
            
            const height = Math.max(
                body.scrollHeight,
                body.offsetHeight,
                html.clientHeight,
                html.scrollHeight,
                html.offsetHeight
            );
            
            iframe.style.height = Math.max(height + 50, 1000) + 'px'; // Add 50px padding, minimum 1000px
        }
    } catch (e) {
        // Cross-origin restrictions - use fallback height
        iframe.style.height = '1200px';
    }
}

// Initialize iframe auto-resize
document.addEventListener('DOMContentLoaded', function() {
    const gameIframe = document.getElementById('main-game-iframe');
    if (gameIframe) {
        gameIframe.onload = function() {
            resizeIframe(gameIframe);
            // Re-check size periodically
            setInterval(() => resizeIframe(gameIframe), 2000);
        };
    }
});

// Listen for messages from game iframe
window.addEventListener('message', function(event) {
    if (event.data.type === 'gameStarted') {
        console.log('Game started in iframe');
    } else if (event.data.type === 'gameStopped') {
        console.log('Game stopped in iframe');
    }
});
