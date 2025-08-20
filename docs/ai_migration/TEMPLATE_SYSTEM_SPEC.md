# 🎨 TEMPLATE SYSTEM: TERA ENGINE SPECIFICATION

## 📊 Overview
This document specifies the Tera template engine integration with sophisticated context management, async rendering pipeline, and multi-dashboard template organization system.

## 🏗️ Template Architecture

### 1. Template Engine Configuration
**Purpose**: Centralized Tera template engine with dynamic template registration and context management

#### Core Components:
```rust
// Template Engine (in AppState)
pub tera: Tera,  // Thread-safe Tera instance

// Template Initialization
let mut tera = match Tera::new("dashboards/**/*.html") {
    Ok(t) => t,
    Err(e) => {
        println!("Warning: Template parsing error: {}", e);
        Tera::default()  // Fallback to empty engine
    }
};
```

### 2. Template Registration System
**Purpose**: Dynamic template path mapping for logical naming and backward compatibility

```rust
// Shared Components Registration (Backward Compatibility)
tera.add_template_file(
    "shared_components/theme_toggle.html", 
    Some("crypto/components/theme_toggle.html")
).expect("Failed to load legacy crypto theme_toggle.html");

tera.add_template_file(
    "shared_components/language_toggle.html", 
    Some("crypto/components/language_toggle.html")
).expect("Failed to load legacy crypto language_toggle.html");

// Crypto Dashboard Templates Registration
tera.add_template_file(
    "dashboards/crypto_dashboard/routes/reports/view.html", 
    Some("crypto/routes/reports/view.html")
).expect("Failed to load crypto reports view template");

tera.add_template_file(
    "dashboards/crypto_dashboard/routes/reports/pdf.html", 
    Some("crypto/routes/reports/pdf.html")
).expect("Failed to load crypto reports pdf template");

tera.add_template_file(
    "dashboards/crypto_dashboard/routes/reports/list.html", 
    Some("crypto/routes/reports/list.html")
).expect("Failed to load crypto reports list template");
```

### 3. Template Directory Structure
```
dashboards/
├── crypto_dashboard/
│   ├── pages/
│   │   ├── game.html
│   │   ├── game-ui.html  
│   │   └── report.html
│   └── routes/
│       └── reports/
│           ├── view.html      # Main report display
│           ├── pdf.html       # PDF-optimized template
│           └── list.html      # Report listing
├── stock_dashboard/
│   └── shared/
│       └── templates/
│           └── base.html
└── home.html                  # Homepage template

shared_components/
├── theme_toggle.html          # Theme switching component
└── language_toggle.html       # Language switching component
```

## 🎯 Async Template Rendering Pipeline

### 1. Core Rendering Function
**Purpose**: Centralized, async-safe template rendering with comprehensive context management

```rust
async fn render_crypto_template(
    tera: &tera::Tera, 
    template: &str,
    report: &Report,
    chart_modules_content: &str,
    additional_context: Option<HashMap<String, serde_json::Value>>
) -> Result<String, Box<dyn StdError + Send + Sync>> {
    // Step 1: Clone data for thread safety
    let tera_clone = tera.clone();
    let template_str = template.to_string();
    let report_clone = report.clone();
    let chart_content_clone = chart_modules_content.to_string();
    let additional_clone = additional_context.clone();
    
    // Step 2: Spawn blocking task for CPU-intensive template rendering
    let render_result = tokio::task::spawn_blocking(move || {
        let mut context = Context::new();
        
        // Base context - Always present
        context.insert("report", &report_clone);
        context.insert("chart_modules_content", &chart_content_clone);
        
        // Additional context from caller
        if let Some(extra) = additional_clone {
            for (key, value) in extra {
                context.insert(&key, &value);
            }
        }
        
        // Template-specific context injection
        if template_str.contains("view.html") {
            context.insert("current_route", "dashboard");
            context.insert("current_lang", "vi");
            context.insert("current_time", &chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
            let pdf_url = format!("/pdf-template/{}", report_clone.id);
            context.insert("pdf_url", &pdf_url);
        }
        
        // PDF template specific context
        if template_str.contains("pdf.html") {
            let created_display = (report_clone.created_at + chrono::Duration::hours(7))
                .format("%d-%m-%Y %H:%M").to_string();
            context.insert("created_at_display", &created_display);
        }

        // Execute template rendering
        tera_clone.render(&template_str, &context)
    }).await;
    
    // Step 3: Comprehensive error handling
    match render_result {
        Ok(Ok(html)) => Ok(html),
        Ok(Err(e)) => {
            eprintln!("Template render error: {:#?}", e);
            let mut src = e.source();
            while let Some(s) = src {
                eprintln!("Template render error source: {:#?}", s);
                src = s.source();
            }
            Err(format!("Template render error: {}", e).into())
        }
        Err(e) => {
            eprintln!("Task join error: {:#?}", e);
            Err(format!("Task join error: {}", e).into())
        }
    }
}
```

### 2. Response Creation Helper
**Purpose**: Consistent HTTP response creation with cache headers

```rust
fn create_cached_response(html: String, cache_status: &str) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/html; charset=utf-8")
        .header("cache-control", "public, max-age=300")  // 5 minutes cache
        .header("x-cache-status", cache_status)          // Cache debugging header
        .body(html)
        .unwrap()
}
```

## 📋 Context Management System

### 1. Base Context (Always Present)
```rust
// Core report data
context.insert("report", &report);

// Chart modules JavaScript code
context.insert("chart_modules_content", &chart_modules_content);
```

### 2. Template-Specific Context
#### View Template Context
```rust
if template_str.contains("view.html") {
    context.insert("current_route", "dashboard");
    context.insert("current_lang", "vi");
    context.insert("current_time", &chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
    let pdf_url = format!("/pdf-template/{}", report_clone.id);
    context.insert("pdf_url", &pdf_url);
}
```

#### PDF Template Context
```rust
if template_str.contains("pdf.html") {
    // Timezone adjustment: UTC+7 for Vietnam timezone
    let created_display = (report_clone.created_at + chrono::Duration::hours(7))
        .format("%d-%m-%Y %H:%M").to_string();
    context.insert("created_at_display", &created_display);
}
```

#### Dynamic Additional Context
```rust
// Caller can inject additional context
if let Some(extra) = additional_context {
    for (key, value) in extra {
        context.insert(&key, &value);
    }
}
```

## 📊 Report Data Model Integration

### Report Structure
```rust
pub struct Report {
    pub id: i32,
    pub html_content: String,       // Vietnamese HTML content
    pub css_content: String,        // CSS styles
    pub js_content: String,         // JavaScript code (Vietnamese)
    pub html_content_en: String,    // English HTML content  
    pub js_content_en: String,      // JavaScript code (English)
    pub created_at: DateTime<Utc>,  // Creation timestamp
}
```

### Template Access Patterns
```html
<!-- In templates, access report data: -->
{{ report.id }}
{{ report.html_content }}
{{ report.css_content }}  
{{ report.js_content }}

<!-- Language-specific content -->
{{ report.html_content_en }}
{{ report.js_content_en }}

<!-- Formatted timestamps -->
{{ created_at_display }}  <!-- PDF templates -->
{{ current_time }}        <!-- View templates -->
```

## 🔧 Template Engine Configuration

### Security Configuration
```rust
// Disable auto-escaping for safe content
tera.autoescape_on(vec![]);
```

### Error Handling Strategy
```rust
// Graceful fallback on template loading errors
let mut tera = match Tera::new("dashboards/**/*.html") {
    Ok(t) => t,
    Err(e) => {
        println!("Warning: Template parsing error: {}", e);
        Tera::default()  // Continue with empty template engine
    }
};

// Strict error handling for template registration
tera.add_template_file(path, logical_name)
    .expect("Failed to load required template");
```

## 🎨 Template Types & Use Cases

### 1. Report View Templates
**File**: `crypto/routes/reports/view.html`
**Purpose**: Interactive report display with charts and navigation
**Context**:
- Full report data
- Chart modules JavaScript
- Navigation context (current_route, current_lang)
- PDF generation link
- Current timestamp

### 2. PDF Templates  
**File**: `crypto/routes/reports/pdf.html`
**Purpose**: Print-optimized report layout for PDF generation
**Context**:
- Full report data
- Chart modules JavaScript (for rendering)
- Formatted creation date (localized timezone)
- Print-specific styling

### 3. Report List Templates
**File**: `crypto/routes/reports/list.html`
**Purpose**: Report listing and pagination
**Context**:
- Report summaries
- Pagination data
- Filter parameters
- Navigation context

### 4. Shared Component Templates
**Files**: 
- `crypto/components/theme_toggle.html`
- `crypto/components/language_toggle.html`

**Purpose**: Reusable UI components
**Context**: Minimal, component-specific

## ⚡ Performance Optimizations

### 1. Async Rendering Pipeline
```rust
// CPU-intensive template rendering in dedicated thread pool
tokio::task::spawn_blocking(move || {
    tera_clone.render(&template_str, &context)
}).await
```

### 2. Template Caching
- **Template compilation**: Cached automatically by Tera
- **Rendered output**: Cached at application level with cache headers
- **Context reuse**: Cloning for thread safety

### 3. Error Recovery
```rust
// Detailed error logging for debugging
eprintln!("Template render error: {:#?}", e);
let mut src = e.source();
while let Some(s) = src {
    eprintln!("Template render error source: {:#?}", s);
    src = s.source();
}
```

## 🔌 Integration Requirements

### Dependencies
```rust
use tera::{Tera, Context};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use serde_json::Value;
use tokio::task;
use axum::{http::StatusCode, response::Response};
```

### Template File Structure Requirements
```
Required template files:
- dashboards/crypto_dashboard/routes/reports/view.html
- dashboards/crypto_dashboard/routes/reports/pdf.html  
- dashboards/crypto_dashboard/routes/reports/list.html
- shared_components/theme_toggle.html
- shared_components/language_toggle.html
- dashboards/home.html

Optional extensions:
- dashboards/crypto_dashboard/pages/*.html
- dashboards/stock_dashboard/**/*.html
```

## 🚨 Error Handling Patterns

### Template Loading Errors
```rust
// Graceful degradation
match Tera::new("dashboards/**/*.html") {
    Ok(t) => t,
    Err(e) => {
        println!("Warning: Template parsing error: {}", e);
        Tera::default()  // Empty engine, handle in render calls
    }
}
```

### Rendering Errors
```rust
// Comprehensive error reporting
match render_result {
    Ok(Ok(html)) => Ok(html),
    Ok(Err(e)) => {
        // Log full error chain
        eprintln!("Template render error: {:#?}", e);
        let mut src = e.source();
        while let Some(s) = src {
            eprintln!("Template render error source: {:#?}", s);
            src = s.source();
        }
        Err(format!("Template render error: {}", e).into())
    }
    Err(e) => {
        // Tokio task join errors
        eprintln!("Task join error: {:#?}", e);
        Err(format!("Task join error: {}", e).into())
    }
}
```

### Missing Template Fallbacks
```rust
// Template registration with explicit error handling
tera.add_template_file(file_path, logical_name)
    .expect(&format!("Failed to load critical template: {}", file_path));
```

## 🎯 Migration Considerations

### Feature Isolation Strategy
When migrating to feature-based architecture:

1. **Template Engine**: Move to `shared/templates.rs` or per-feature
2. **Context Builders**: Feature-specific context management
3. **Template Organization**: Group templates by feature
4. **Shared Components**: Extract to `shared/components/`

### Template Namespacing
```rust
// Feature-based template logical names
crypto/routes/reports/view.html     → CryptoReportsViewTemplate
crypto/routes/reports/pdf.html      → CryptoReportsPdfTemplate  
crypto/routes/reports/list.html     → CryptoReportsListTemplate
crypto/components/theme_toggle.html → CryptoThemeToggleComponent
```

### Backwards Compatibility
- All template logical names preserved
- Context structure maintained
- Response format unchanged
- Error handling patterns consistent

---

**📝 Generated**: August 20, 2025  
**🔄 Version**: 1.0  
**📊 Source Lines**: 80+ lines of template rendering logic + Tera configuration  
**🎯 Migration Target**: `features/*/templates.rs` + `shared/template_engine.rs`
