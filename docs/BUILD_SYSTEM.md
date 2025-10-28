# Build System Documentation

## Overview

This project uses **esbuild** as the build system to bundle and optimize JavaScript modules for production deployment.

## Quick Start

### Install Dependencies
```bash
npm install
```

### Build Commands

#### Development Build
```bash
npm run build
```
- Creates unminified bundles
- Includes source maps for debugging
- Larger file size but easier to debug

#### Production Build
```bash
npm run build:prod
```
- Minified and optimized bundles
- No source maps (smaller size)
- Tree-shaking enabled
- Best for deployment

#### Watch Mode (Development)
```bash
npm run build:watch
```
- Automatically rebuilds on file changes
- Perfect for active development
- Includes source maps

#### Clean Build Artifacts
```bash
npm run clean
```
- Removes `dist/` directory
- Fresh start for new builds

## Bundle Outputs

All bundles are generated in the `dist/` directory:

### 1. market-indicators.bundle.js
**Source**: `shared_components/market-indicators/market-indicators-modular.js`  
**Size**: 23.18 KB (6.95 KB gzipped)  
**Description**: Complete dashboard with WebSocket, data processing, and chart rendering

**Modules bundled**:
- WebSocketManager
- DataProcessor
- StateManager
- 7 Updaters (MarketCap, Volume, FearGreed, Dominance, RSI, CryptoPrice, StockIndex)
- ChartRenderer

### 2. report-view-iframe.bundle.js
**Source**: `dashboards/crypto_dashboard/assets/report-view-iframe.js`  
**Size**: 10.70 KB (3.21 KB gzipped)  
**Description**: Iframe communication and navigation management for report pages

**Features**:
- postMessage event handlers
- Sidebar navigation
- Scroll tracking
- Theme/language sync

### 3. date-formatter.bundle.js
**Source**: `dashboards/crypto_dashboard/assets/date-formatter-utility.js`  
**Size**: 1.24 KB (0.37 KB gzipped)  
**Description**: Date formatting with timezone and i18n support

**Features**:
- GMT+7 timezone
- vi-VN / en-US locales
- Auto-update on language change

### 4. report-list-interactions.bundle.js
**Source**: `dashboards/crypto_dashboard/assets/report-list-interactions.js`  
**Size**: 0.59 KB (0.18 KB gzipped)  
**Description**: Table row hover effects and interactions

## Performance Metrics

### Original vs Bundled Size

| Module | Original | Bundled (Dev) | Bundled (Prod) | Reduction |
|--------|----------|---------------|----------------|-----------|
| market-indicators | ~50 KB (13 files) | 42.58 KB | 23.18 KB | **53.6%** |
| report-view-iframe | 24 KB | 17.86 KB | 10.70 KB | **55.4%** |
| date-formatter | 2.9 KB | 2.08 KB | 1.24 KB | **57.2%** |
| report-list | 1.4 KB | 1.01 KB | 0.59 KB | **57.9%** |
| **TOTAL** | **~78 KB** | **63.53 KB** | **35.71 KB** | **54.2%** |

### Gzipped Size (Production)
- **Total**: 10.71 KB
- **Compression**: 70% from original size
- **Network transfer**: ~11 KB (vs ~78 KB original)

### Build Performance
- Total build time: **16ms**
- Fast rebuilds in watch mode
- Minimal overhead for CI/CD

## Build Configuration

### Target Browsers
- ES2020+
- Chrome 90+
- Firefox 88+
- Safari 14+

### Optimization Features
- ✅ **Tree-shaking**: Remove unused code
- ✅ **Minification**: Reduce file size
- ✅ **Dead code elimination**: Remove unreachable code
- ✅ **Scope hoisting**: Faster runtime
- ✅ **Source maps** (dev only): Debug original code

## Deployment Guide

### Step 1: Build Production Bundles
```bash
npm run build:prod
```

### Step 2: Copy Bundles to Server
Replace original files with bundled versions:

```bash
# Market Indicators
cp dist/market-indicators.bundle.js shared_components/market-indicators/market-indicators.bundle.js

# Report View
cp dist/report-view-iframe.bundle.js dashboards/crypto_dashboard/assets/report-view-iframe.bundle.js

# Date Formatter
cp dist/date-formatter.bundle.js dashboards/crypto_dashboard/assets/date-formatter.bundle.js

# Report List
cp dist/report-list-interactions.bundle.js dashboards/crypto_dashboard/assets/report-list-interactions.bundle.js
```

### Step 3: Update HTML References

#### home.html
Replace:
```html
<script type="module" src="/shared_components/market-indicators/market-indicators-modular.js"></script>
```

With:
```html
<script src="/shared_components/market-indicators/market-indicators.bundle.js"></script>
```

#### view.html
Replace:
```html
<script src="/crypto_dashboard/assets/report-view-iframe.js" defer></script>
<script src="/crypto_dashboard/assets/date-formatter-utility.js" defer></script>
```

With:
```html
<script src="/crypto_dashboard/assets/report-view-iframe.bundle.js" defer></script>
<script src="/crypto_dashboard/assets/date-formatter.bundle.js" defer></script>
```

#### list.html
Replace:
```html
<script src="/crypto_dashboard/assets/report-list-interactions.js" defer></script>
```

With:
```html
<script src="/crypto_dashboard/assets/report-list-interactions.bundle.js" defer></script>
```

### Step 4: Clear Browser Cache
Add cache busting with version query:
```html
<script src="/path/to/bundle.js?v=1.0.0"></script>
```

### Step 5: Test Deployment
- ✅ Check WebSocket connections
- ✅ Verify chart rendering
- ✅ Test navigation and scroll
- ✅ Verify date formatting
- ✅ Check table interactions
- ✅ Test theme/language switching

## CI/CD Integration

### GitHub Actions Example
```yaml
name: Build Frontend Assets

on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          
      - name: Install dependencies
        run: npm ci
        
      - name: Build production bundles
        run: npm run build:prod
        
      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: bundles
          path: dist/
```

## Troubleshooting

### Build Errors

**Error: "Entry point not found"**
- Check file paths in `build.js`
- Ensure all source files exist

**Error: "Module not found"**
- Check import paths in source files
- Use relative paths with `.js` extension

**Warning: "CommonJS variable"**
- Remove `module.exports` from browser code
- Already fixed in current version

### Performance Issues

**Slow build times**
- Check for large dependencies
- Use `--production` flag for final builds
- Consider splitting into smaller bundles

**Large bundle sizes**
- Check for unused imports
- Use tree-shaking
- Consider code splitting for lazy loading

## Advanced Configuration

### Custom Bundle Configuration

Edit `build.js` to add new bundles:

```javascript
const bundles = [
  // ... existing bundles
  {
    name: 'my-new-bundle',
    entryPoint: 'path/to/entry.js',
    outfile: 'dist/my-bundle.js',
    description: 'My New Bundle'
  }
];
```

### External Dependencies

Exclude CDN-loaded libraries:

```javascript
external: ['chart.js', 'lodash']
```

### Code Splitting (Future)

For larger apps, consider splitting:
```javascript
splitting: true,
format: 'esm',
outdir: 'dist/chunks/'
```

## Best Practices

1. **Always build before deployment**
   ```bash
   npm run build:prod
   ```

2. **Test bundles locally**
   - Check bundle sizes
   - Verify functionality
   - Test in target browsers

3. **Version your bundles**
   - Use git tags
   - Update version in `package.json`
   - Add version to URLs

4. **Monitor bundle sizes**
   - Check `build-report.json`
   - Set size budgets
   - Alert on size increases

5. **Keep dependencies updated**
   ```bash
   npm update
   npm audit fix
   ```

## Resources

- [esbuild Documentation](https://esbuild.github.io/)
- [JavaScript Module Best Practices](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Modules)
- [Web Performance Optimization](https://web.dev/fast/)

---

**Last Updated**: 2024  
**Build System Version**: 1.0.0  
**esbuild Version**: 0.19.5
