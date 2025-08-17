# Language Toggle API Reference

## 🔧 JavaScript API

### Global Objects

#### `window.languageManager`
Main API object for language operations.

```javascript
window.languageManager = {
    currentLanguage: 'vi' | 'en',
    getTranslatedText: (key: string) => string,
    formatNumberLocalized: (number: number) => string,
    setTranslations: (data: object) => void
}
```

### Core Functions

#### `getTranslationsData()`
**Description**: Dynamic translation data retrieval (similar to callInitializeReportVisuals pattern)
**Returns**: `object` - Translation data or empty object as fallback
**Usage**:
```javascript
const data = getTranslationsData();
console.log(data); // { 'key': { vi: 'text', en: 'text' }, ... }
```

#### `updateUI(lang)`
**Description**: Update entire UI based on language preference
**Parameters**: 
- `lang` (string): 'vi' or 'en'
**Usage**:
```javascript
updateUI('en'); // Switch entire page to English
```

#### `getPreferredLanguage()`
**Description**: Get saved language from localStorage
**Returns**: `string` - 'vi' (default) or 'en'
**Usage**:
```javascript
const currentLang = getPreferredLanguage();
```

#### `setPreferredLanguage(lang)`
**Description**: Save language preference and update UI
**Parameters**: 
- `lang` (string): 'vi' or 'en'
**Side Effects**: 
- Saves to localStorage
- Calls updateUI()
- Dispatches 'languageChanged' event
**Usage**:
```javascript
setPreferredLanguage('en');
```

#### `toggleLanguage()`
**Description**: Switch between vi ↔ en
**Usage**:
```javascript
toggleLanguage(); // vi → en or en → vi
```

### Language Manager API

#### `languageManager.getTranslatedText(key)`
**Description**: Get translation for specific key
**Parameters**: 
- `key` (string): Translation key
**Returns**: `string` - Translated text or original key as fallback
**Usage**:
```javascript
const text = window.languageManager.getTranslatedText('site-title');
// Returns: "Crypto Market Overview" (if current lang is 'en')
```

#### `languageManager.formatNumberLocalized(num)`
**Description**: Format number according to current language locale
**Parameters**: 
- `num` (number): Number to format
**Returns**: `string` - Formatted number with locale-specific formatting
**Usage**:
```javascript
const formatted = window.languageManager.formatNumberLocalized(1234567.89);
// VI: "1,234,567.89" 
// EN: "1,234,567.89"

// Large numbers:
window.languageManager.formatNumberLocalized(1500000000);
// VI: "1.50 tỷ"
// EN: "1.50B"
```

#### `languageManager.currentLanguage`
**Description**: Get/Set current active language
**Type**: `string` ('vi' | 'en')
**Usage**:
```javascript
// Get current language
const lang = window.languageManager.currentLanguage; // 'vi' or 'en'

// Set language (triggers UI update)
window.languageManager.currentLanguage = 'en';
```

### Translation Data Functions

#### `get_translations_data()`
**Description**: Core function that returns translation data (defined in translations.js)
**Returns**: `object` - Complete translation mapping
**Usage**:
```javascript
const allTranslations = get_translations_data();
/*
Returns:
{
  'site-title': { vi: 'Toàn Cảnh Thị Trường', en: 'Market Overview' },
  'loading': { vi: 'Đang tải...', en: 'Loading...' },
  // ... all translations
}
*/
```

#### `setTranslations(data)`
**Description**: Inject translation data (used by translations.js auto-registration)
**Parameters**: 
- `data` (object): Translation data object
**Usage**:
```javascript
// Usually called automatically, but can be used manually:
const customTranslations = {
  'custom-key': { vi: 'Văn bản tùy chỉnh', en: 'Custom text' }
};
setTranslations(customTranslations);
```

## 🎭 Events

### `languageChanged`
**Triggered**: When language preference changes
**Event Detail**: `{ language: 'vi' | 'en' }`
**Usage**:
```javascript
window.addEventListener('languageChanged', function(event) {
    const newLang = event.detail.language;
    console.log('Language changed to:', newLang);
    
    // Update dynamic content
    updateDynamicElements(newLang);
});
```

### `languageToggleReady`
**Triggered**: When language-toggle.js is fully loaded and initialized
**Usage**:
```javascript
window.addEventListener('languageToggleReady', function() {
    console.log('Language toggle system ready');
    // Safe to call language functions
});
```

## 🏷️ HTML Attributes

### `data-i18n`
**Description**: Mark elements for automatic translation
**Usage**:
```html
<!-- Basic usage -->
<span data-i18n="key-name">Default text</span>

<!-- Complex elements -->
<h1 data-i18n="page-title">Tiêu đề trang</h1>
<button data-i18n="save-button">Lưu</button>
<p data-i18n="description">Mô tả nội dung</p>

<!-- Works with any HTML element -->
<div data-i18n="container-label">Container</div>
<input placeholder="Search..." data-i18n="search-placeholder">
```

### Special Element IDs

#### Language-specific content containers
```html
<!-- Auto-toggled based on language -->
<div id="report-content-vi">Nội dung tiếng Việt</div>
<div id="report-content-en">English content</div>

<div id="title-vi">Tiêu đề tiếng Việt</div>
<div id="title-en">English Title</div>
```

#### Language toggle button
```html
<div id="language-toggle">
    <button>
        <span class="lang-text">VI</span> <!-- Auto-updated -->
    </button>
</div>
```

## 🔄 Translation Data Structure

### Basic Structure
```javascript
const translations_data = {
    'key-name': {
        vi: 'Vietnamese text',
        en: 'English text'
    }
};
```

### Categories

#### Homepage
```javascript
'homepage-title': { vi: 'Trang chủ - Crypto Dashboard', en: 'Homepage - Crypto Dashboard' },
'welcome-message': { vi: 'Chào mừng đến Crypto Dashboard', en: 'Welcome to Crypto Dashboard' },
```

#### UI Elements
```javascript
'loading': { vi: 'Đang tải...', en: 'Loading...' },
'save': { vi: 'Lưu', en: 'Save' },
'cancel': { vi: 'Hủy', en: 'Cancel' },
```

#### Business Domain
```javascript
'btc-price': { vi: 'Giá BTC', en: 'BTC Price' },
'market-cap': { vi: 'Tổng Vốn Hóa', en: 'Market Capitalization' },
```

## ⚡ Performance APIs

### Batch Translation
```javascript
// Get multiple translations at once
function getMultipleTranslations(keys, lang) {
    const data = getTranslationsData();
    return keys.map(key => ({
        key,
        text: data[key] && data[key][lang] ? data[key][lang] : key
    }));
}

// Usage
const texts = getMultipleTranslations(['home', 'save', 'cancel'], 'en');
```

### Lazy Loading Check
```javascript
// Check if translations are loaded
function isTranslationsReady() {
    const data = getTranslationsData();
    return Object.keys(data).length > 0;
}

// Wait for translations
function waitForTranslations(callback, timeout = 5000) {
    const startTime = Date.now();
    
    function check() {
        if (isTranslationsReady()) {
            callback();
        } else if (Date.now() - startTime < timeout) {
            setTimeout(check, 100);
        } else {
            console.warn('Translations loading timeout');
        }
    }
    
    check();
}
```

## 🛡️ Error Handling APIs

### Safe Translation Get
```javascript
function getSafeTranslation(key, lang = null) {
    try {
        const currentLang = lang || window.languageManager.currentLanguage || 'vi';
        const data = getTranslationsData();
        
        if (data[key] && data[key][currentLang]) {
            return data[key][currentLang];
        }
        
        // Fallback to other language
        const fallbackLang = currentLang === 'vi' ? 'en' : 'vi';
        if (data[key] && data[key][fallbackLang]) {
            return data[key][fallbackLang];
        }
        
        // Return key as last resort
        return key;
    } catch (error) {
        console.warn('Translation error for key:', key, error);
        return key;
    }
}
```

### Validation APIs
```javascript
// Check missing translations
function findMissingTranslations(requiredKeys) {
    const data = getTranslationsData();
    const missing = [];
    
    requiredKeys.forEach(key => {
        if (!data[key]) {
            missing.push({ key, reason: 'key_not_found' });
        } else if (!data[key].vi || !data[key].en) {
            missing.push({ key, reason: 'incomplete_translation' });
        }
    });
    
    return missing;
}

// Check unused translations
function findUnusedTranslations() {
    const data = getTranslationsData();
    const usedKeys = Array.from(document.querySelectorAll('[data-i18n]'))
                          .map(el => el.getAttribute('data-i18n'));
    
    const definedKeys = Object.keys(data);
    const unused = definedKeys.filter(key => !usedKeys.includes(key));
    
    return unused;
}
```

## 📊 Debug/Development APIs

### Debug Information
```javascript
// Get system status
function getLanguageToggleStatus() {
    return {
        currentLanguage: window.languageManager?.currentLanguage,
        translationsLoaded: isTranslationsReady(),
        totalTranslations: Object.keys(getTranslationsData()).length,
        elementsWithI18n: document.querySelectorAll('[data-i18n]').length,
        lastUpdate: localStorage.getItem('preferred_language_updated'),
        version: '1.0.0'
    };
}

// Test all translations
function testAllTranslations() {
    const data = getTranslationsData();
    const results = {};
    
    Object.keys(data).forEach(key => {
        results[key] = {
            hasVi: !!data[key].vi,
            hasEn: !!data[key].en,
            viLength: data[key].vi?.length || 0,
            enLength: data[key].en?.length || 0
        };
    });
    
    return results;
}
```

### Console Helpers
```javascript
// Global debug helpers (available in console)
window.debugLangToggle = {
    switchTo: (lang) => setPreferredLanguage(lang),
    getStatus: () => getLanguageToggleStatus(),
    testTranslations: () => testAllTranslations(),
    findMissing: (keys) => findMissingTranslations(keys),
    forceUpdate: () => updateUI(window.languageManager.currentLanguage)
};
```

---

## 📚 Examples

### Complete Integration Example
```html
<!DOCTYPE html>
<html lang="vi">
<head>
    <title data-i18n="page-title">My Page</title>
</head>
<body>
    {% include 'crypto/components/language_toggle.html' %}
    
    <h1 data-i18n="welcome">Welcome</h1>
    <p data-i18n="description">Description</p>
    
    <script src="/crypto_dashboard/assets/translations.js" defer></script>
    <script src="/shared_components/core/language-toggle.js" defer></script>
    
    <script>
        // Listen for language changes
        window.addEventListener('languageChanged', function(event) {
            console.log('New language:', event.detail.language);
        });
        
        // Custom logic after page load
        document.addEventListener('DOMContentLoaded', function() {
            // Safe to use language APIs here
            setTimeout(() => {
                const current = window.languageManager.currentLanguage;
                console.log('Current language:', current);
            }, 500);
        });
    </script>
</body>
</html>
```

---

**API Version**: 1.0.0  
**Last Updated**: August 17, 2025
