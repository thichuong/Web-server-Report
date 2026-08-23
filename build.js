#!/usr/bin/env node

/**
 * Build Script for Web Server Report
 * 
 * Features:
 * - Bundle and minify JavaScript modules
 * - Generate source maps for debugging
 * - Watch mode for development
 * - Production optimization
 * - Tree-shaking for smaller bundles
 */

import * as esbuild from 'esbuild';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Build configuration
const isProduction = process.argv.includes('--production');
const isWatch = process.argv.includes('--watch');

const buildConfig = {
  production: isProduction,
  watch: isWatch,
  minify: isProduction,
  sourcemap: !isProduction,
  target: ['es2020', 'chrome90', 'firefox88', 'safari14'],
  format: 'esm'
};

console.log('🏗️  Build Configuration:');
console.log(`   Mode: ${isProduction ? 'PRODUCTION' : 'DEVELOPMENT'}`);
console.log(`   Watch: ${isWatch ? 'ENABLED' : 'DISABLED'}`);
console.log(`   Minify: ${buildConfig.minify ? 'ENABLED' : 'DISABLED'}`);
console.log(`   Source Maps: ${buildConfig.sourcemap ? 'ENABLED' : 'DISABLED'}`);
console.log('');

// Ensure dist directory exists
const distDir = join(__dirname, 'dist');
if (!existsSync(distDir)) {
  mkdirSync(distDir, { recursive: true });
  console.log('📁 Created dist/ directory\n');
}

/**
 * Build configurations for different bundles
 */
const bundles = [
  {
    name: 'market-indicators',
    entryPoint: 'shared_components/market-indicators/market-indicators-modular.js',
    outfile: 'dist/market-indicators.bundle.js',
    description: 'Market Indicators Dashboard Module'
  },

  {
    name: 'report-view-shadow-dom',
    entryPoint: 'dashboards/crypto_dashboard/assets/report-view-shadow-dom.js',
    outfile: 'dist/report-view-shadow-dom.bundle.js',
    description: 'Report View Shadow DOM Manager'
  },
  {
    name: 'date-formatter',
    entryPoint: 'dashboards/crypto_dashboard/assets/date-formatter-utility.js',
    outfile: 'dist/date-formatter.bundle.js',
    description: 'Date Formatter Utility'
  },
  {
    name: 'report-list-interactions',
    entryPoint: 'dashboards/crypto_dashboard/assets/report-list-interactions.js',
    outfile: 'dist/report-list-interactions.bundle.js',
    description: 'Report List Table Interactions'
  }
];

/**
 * Build a single bundle
 */
async function buildBundle(config) {
  const startTime = Date.now();
  const entryPath = join(__dirname, config.entryPoint);
  
  if (!existsSync(entryPath)) {
    console.error(`❌ Entry point not found: ${config.entryPoint}`);
    return null;
  }
  
  try {
    const buildOptions = {
      entryPoints: [entryPath],
      bundle: true,
      outfile: join(__dirname, config.outfile),
      minify: buildConfig.minify,
      sourcemap: buildConfig.sourcemap,
      target: buildConfig.target,
      format: buildConfig.format,
      platform: 'browser',
      
      // Optimization options
      treeShaking: true,
      legalComments: 'none',
      
      // Plugin to track bundle size
      metafile: true,
      
      // External dependencies (CDN loaded)
      external: [],
      
      // Banner for production builds
      banner: buildConfig.minify ? {
        js: `/* ${config.name} - Built: ${new Date().toISOString()} */`
      } : {}
    };
    
    const result = await esbuild.build(buildOptions);
    
    const buildTime = Date.now() - startTime;
    const outputPath = join(__dirname, config.outfile);
    const outputSize = existsSync(outputPath) 
      ? (readFileSync(outputPath).length / 1024).toFixed(2) 
      : '0';
    
    // Calculate gzipped size estimate (rough approximation: 30% of original)
    const gzippedSize = (outputSize * 0.3).toFixed(2);
    
    console.log(`✅ ${config.name}`);
    console.log(`   ${config.description}`);
    console.log(`   📦 Size: ${outputSize} KB (estimated gzip: ${gzippedSize} KB)`);
    console.log(`   ⏱️  Time: ${buildTime}ms`);
    
    // Log module breakdown if metafile is available
    if (result.metafile && !buildConfig.minify) {
      const modules = Object.keys(result.metafile.inputs).length;
      console.log(`   📚 Modules: ${modules} files bundled`);
    }
    
    console.log('');
    
    return {
      name: config.name,
      size: parseFloat(outputSize),
      gzippedSize: parseFloat(gzippedSize),
      time: buildTime
    };
  } catch (error) {
    console.error(`❌ Build failed for ${config.name}:`);
    console.error(error.message);
    if (error.errors) {
      error.errors.forEach(err => {
        console.error(`   ${err.text}`);
        if (err.location) {
          console.error(`   at ${err.location.file}:${err.location.line}:${err.location.column}`);
        }
      });
    }
    console.log('');
    return null;
  }
}

/**
 * Build all bundles
 */
async function buildAll() {
  console.log('🚀 Starting build process...\n');
  
  const results = [];
  
  for (const config of bundles) {
    const result = await buildBundle(config);
    if (result) {
      results.push(result);
    }
  }
  
  // Summary
  if (results.length > 0) {
    console.log('📊 Build Summary:');
    console.log('─'.repeat(60));
    
    const totalSize = results.reduce((sum, r) => sum + r.size, 0);
    const totalGzipped = results.reduce((sum, r) => sum + r.gzippedSize, 0);
    const totalTime = results.reduce((sum, r) => sum + r.time, 0);
    
    console.log(`   Total bundles: ${results.length}`);
    console.log(`   Total size: ${totalSize.toFixed(2)} KB`);
    console.log(`   Total gzipped: ${totalGzipped.toFixed(2)} KB`);
    console.log(`   Total time: ${totalTime}ms`);
    console.log(`   Compression: ${((1 - totalGzipped / totalSize) * 100).toFixed(1)}%`);
    console.log('─'.repeat(60));
    console.log('');
    
    // Generate build report
    const buildReport = {
      timestamp: new Date().toISOString(),
      mode: isProduction ? 'production' : 'development',
      bundles: results,
      summary: {
        totalSize,
        totalGzipped,
        totalTime,
        compressionRatio: ((1 - totalGzipped / totalSize) * 100).toFixed(1) + '%'
      }
    };
    
    const reportPath = join(__dirname, 'dist', 'build-report.json');
    writeFileSync(reportPath, JSON.stringify(buildReport, null, 2));
    console.log(`📄 Build report saved to: dist/build-report.json\n`);
  }
  
  if (results.length === bundles.length) {
    console.log('✅ Build completed successfully!\n');
  } else {
    console.log(`⚠️  Build completed with ${bundles.length - results.length} failure(s)\n`);
    process.exit(1);
  }
}

/**
 * Watch mode
 */
async function watch() {
  console.log('👀 Watch mode enabled - rebuilding on changes...\n');
  
  const contexts = [];
  
  for (const config of bundles) {
    const entryPath = join(__dirname, config.entryPoint);
    
    if (!existsSync(entryPath)) {
      console.error(`❌ Entry point not found: ${config.entryPoint}`);
      continue;
    }
    
    const buildOptions = {
      entryPoints: [entryPath],
      bundle: true,
      outfile: join(__dirname, config.outfile),
      minify: false,
      sourcemap: true,
      target: buildConfig.target,
      format: buildConfig.format,
      platform: 'browser',
      treeShaking: true,
      legalComments: 'none',
      logLevel: 'info'
    };
    
    try {
      const ctx = await esbuild.context(buildOptions);
      await ctx.watch();
      contexts.push(ctx);
      console.log(`👁️  Watching: ${config.name}`);
    } catch (error) {
      console.error(`❌ Failed to start watch for ${config.name}:`, error.message);
    }
  }
  
  console.log('\n✅ Watch mode active. Press Ctrl+C to stop.\n');
  
  // Keep process alive
  process.on('SIGINT', async () => {
    console.log('\n🛑 Stopping watch mode...');
    for (const ctx of contexts) {
      await ctx.dispose();
    }
    process.exit(0);
  });
}

/**
 * Main execution
 */
async function main() {
  try {
    if (isWatch) {
      await watch();
    } else {
      await buildAll();
    }
  } catch (error) {
    console.error('❌ Build failed:', error);
    process.exit(1);
  }
}

main();
