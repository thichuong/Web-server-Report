# Language Toggle - Quick Start Guide

## 🚀 Thêm Language Toggle vào page mới (5 phút)

### 1. Include Component trong HTML
```html
<!-- Thêm vào <body> -->
{% include 'crypto/components/language_toggle.html' %}
```

### 2. Load Scripts (cuối trang)
```html
<script src="/crypto_dashboard/assets/translations.js" defer></script>
<script src="/shared_components/core/language-toggle.js" defer></script>
```

### 3. Thêm data-i18n attributes
```html
<h1 data-i18n="page-title">Tiêu đề trang</h1>
<p data-i18n="description">Mô tả nội dung</p>
<button data-i18n="save-btn">Lưu</button>
```

### 4. Thêm translations trong translations.js
```javascript
const translations_data = {
    'page-title': { vi: 'Tiêu đề trang', en: 'Page Title' },
    'description': { vi: 'Mô tả nội dung', en: 'Content description' },
    'save-btn': { vi: 'Lưu', en: 'Save' },
    // ... existing translations
};
```

### 5. Test 🎉
- Mở trang trong browser
- Click language toggle button (VI ↔ EN)
- Verify text changes correctly

---

## 💡 Patterns thường dùng

### Multi-language Content Blocks
```html
<!-- Method 1: Toggle visibility -->
<div id="content-vi" style="display: block;">Nội dung tiếng Việt</div>
<div id="content-en" style="display: none;">English content</div>

<!-- Method 2: data-i18n (recommended) -->
<p data-i18n="welcome-msg">Chào mừng bạn</p>
```

### Dynamic Translation trong JavaScript
```javascript
// Get current language
const currentLang = window.languageManager.currentLanguage;

// Get translated text
const text = window.languageManager.getTranslatedText('key');

// Format numbers
const formatted = window.languageManager.formatNumberLocalized(1234567);
// VI: "1,234,567" | EN: "1,234,567"

// Listen for language changes
window.addEventListener('languageChanged', function(event) {
    const newLang = event.detail.language;
    // Update your dynamic content
});
```

### Common Translation Categories

**UI Elements:**
```javascript
'loading': { vi: 'Đang tải...', en: 'Loading...' },
'save': { vi: 'Lưu', en: 'Save' },
'cancel': { vi: 'Hủy', en: 'Cancel' },
'close': { vi: 'Đóng', en: 'Close' },
```

**Navigation:**
```javascript
'home': { vi: 'Trang chủ', en: 'Home' },
'reports': { vi: 'Báo cáo', en: 'Reports' },
'settings': { vi: 'Cài đặt', en: 'Settings' },
```

**Messages:**
```javascript
'success-msg': { vi: 'Thành công!', en: 'Success!' },
'error-msg': { vi: 'Đã xảy ra lỗi', en: 'An error occurred' },
'no-data': { vi: 'Không có dữ liệu', en: 'No data available' },
```

---

## 🐛 Troubleshooting

### Language toggle không hoạt động?
1. Check console errors (F12)
2. Verify script loading order
3. Check network requests trong DevTools

### Translations không xuất hiện?
1. Verify data-i18n attributes
2. Check translations.js có đúng key không
3. Test bằng console: `getTranslationsData()`

### Scripts loading issues?
- Đảm bảo đường dẫn đúng: `/crypto_dashboard/assets/`
- Không dùng: `/dashboards/crypto_dashboard/assets/`

---

## 📋 Checklist cho new page

- [ ] Include language_toggle.html component
- [ ] Load translations.js và language-toggle.js scripts
- [ ] Add data-i18n attributes cho text elements
- [ ] Add translation keys vào translations.js
- [ ] Test language switching hoạt động
- [ ] Check console không có errors
- [ ] Verify both VI và EN text hiển thị correctly

---

**📖 Full Documentation**: `/docs/LANGUAGE_TOGGLE_ARCHITECTURE.md`
