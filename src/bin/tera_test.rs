use tera::{Tera, Context};
use serde_json::json;

fn main() {
    // Initialize Tera similarly to main.rs
    let mut tera = Tera::default();

    tera.add_template_file("shared_components/theme_toggle.html", Some("shared/components/theme_toggle.html")).expect("Failed to load shared theme_toggle.html");
    tera.add_template_file("shared_components/language_toggle.html", Some("shared/components/language_toggle.html")).expect("Failed to load shared language_toggle.html");

    tera.add_template_file("dashboards/crypto_dashboard/routes/reports/view.html", Some("crypto/routes/reports/view.html")).expect("Failed to load crypto reports view template");
    tera.add_template_file("dashboards/crypto_dashboard/routes/reports/pdf.html", Some("crypto/routes/reports/pdf.html")).expect("Failed to load crypto reports pdf template");
    tera.add_template_file("dashboards/crypto_dashboard/routes/reports/list.html", Some("crypto/routes/reports/list.html")).expect("Failed to load crypto reports list template");

    // Add legacy aliases
    tera.add_template_file("shared_components/theme_toggle.html", Some("crypto/components/theme_toggle.html")).expect("Failed to load legacy crypto theme_toggle.html");
    tera.add_template_file("shared_components/language_toggle.html", Some("crypto/components/language_toggle.html")).expect("Failed to load legacy crypto language_toggle.html");

    tera.autoescape_on(vec![]);

    // Non-empty reports context matching runtime dump to reproduce the error
    let reports = json!({
        "items": [
            {"id": 12, "created_date": "16/08/2025", "created_time": "18:10:56 UTC+7"},
            {"id": 11, "created_date": "16/08/2025", "created_time": "18:05:46 UTC+7"},
            {"id": 10, "created_date": "16/08/2025", "created_time": "17:41:06 UTC+7"},
            {"id": 9, "created_date": "16/08/2025", "created_time": "15:14:28 UTC+7"},
            {"id": 8, "created_date": "16/08/2025", "created_time": "14:54:40 UTC+7"},
            {"id": 7, "created_date": "16/08/2025", "created_time": "13:43:21 UTC+7"},
            {"id": 6, "created_date": "16/08/2025", "created_time": "10:38:20 UTC+7"},
            {"id": 5, "created_date": "16/08/2025", "created_time": "07:33:19 UTC+7"},
            {"id": 4, "created_date": "16/08/2025", "created_time": "04:29:43 UTC+7"},
            {"id": 3, "created_date": "16/08/2025", "created_time": "01:24:02 UTC+7"}
        ],
        "total": 12,
        "per_page": 10,
        "page": 1,
        "pages": 2,
        "has_prev": false,
        "has_next": true,
        "prev_num": 1,
        "next_num": 2,
        "page_numbers": [1,2],
        "display_start": 1,
        "display_end": 10
    });

    let mut ctx = Context::new();
    ctx.insert("reports", &reports);

    match tera.render("crypto/routes/reports/list.html", &ctx) {
        Ok(s) => println!("Rendered OK:\n{}", s),
        Err(e) => {
            eprintln!("Render error: {}", e);
            eprintln!("Debug: {:#?}", e);
        }
    }
}
