// Trufflehog Activity Detector - OPTIMIZED VERSION
// Combined best features from V3 and Final for 100% Bash compatibility

use crate::detectors::{Finding, RiskLevel};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Detect Trufflehog secret scanning activity - V3 enhanced version for precise Bash matching
pub fn check_trufflehog_activity<P: AsRef<Path>>(scan_dir: P) -> Vec<Finding> {
    crate::colors::print_status(
        crate::colors::Color::Blue,
        "Checking for Trufflehog activity and secret scanning...",
    );

    let mut findings = Vec::new();
    let mut high_risk_files = std::collections::HashSet::new(); // BASH: track HIGH RISK files

    // Look for trufflehog binary files (always HIGH RISK) - V3 enhanced
    for entry in WalkDir::new(&scan_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let filename = entry.file_name().to_string_lossy();
        if filename.to_lowercase().contains("trufflehog") {
            findings.push(Finding::new(
                entry.path().to_path_buf(),
                "Trufflehog binary found".to_string(),
                RiskLevel::High,
                "trufflehog_binary",
            ));
        }
    }

    // Look for trufflehog references in code - V3 enhanced logic
    let extensions = &["js", "py", "sh", "json"];
    for entry in WalkDir::new(&scan_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| extensions.contains(&ext))
                .unwrap_or(false)
        })
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            let lowercase_content = content.to_lowercase();

            if lowercase_content.contains("trufflehog") {
                let path_str = entry.path().to_string_lossy();

                // Skip documentation - V3 logic
                if path_str.ends_with(".md") || path_str.ends_with(".txt") {
                    continue;
                }

                // Enhanced context checking - V3 approach
                let risk = if content.contains("subprocess") && content.contains("curl") {
                    RiskLevel::High
                } else if path_str.contains("/node_modules/")
                    || path_str.contains("\\node_modules\\")
                {
                    RiskLevel::Medium
                } else {
                    RiskLevel::Medium
                };

                let category = match risk {
                    RiskLevel::High => "trufflehog_execution",
                    _ => "trufflehog_reference",
                };

                findings.push(Finding::new(
                    entry.path().to_path_buf(),
                    "Contains trufflehog references in source code".to_string(),
                    risk,
                    category,
                ));
            }

            // Enhanced credential pattern detection - BASH EXACT logic (line 716-738)
            if content.contains("AWS_ACCESS_KEY")
                || content.contains("GITHUB_TOKEN")
                || content.contains("NPM_TOKEN")
            {
                let path_str = entry.path().to_string_lossy();

                // Skip type definitions and docs - BASH line 718
                if path_str.ends_with(".d.ts") || path_str.ends_with(".md") {
                    continue;
                }

                // BASH line 721-723: node_modules = LOW RISK
                if path_str.contains("/node_modules/") || path_str.contains("\\node_modules\\") {
                    findings.push(Finding::new(
                        entry.path().to_path_buf(),
                        "Credential patterns in node_modules".to_string(),
                        RiskLevel::Low,
                        "credential_node_modules",
                    ));
                    continue;
                }

                // BASH EXACT: Check content_sample (first 20 lines) for webhook config patterns
                if (content.contains("DefinePlugin") || content.contains("webpack"))
                    && path_str.contains("webpack.config")
                {
                    continue; // Bash: webpack config is legitimate
                }

                // BASH EXACT: Use content_sample (first 20 lines) like Bash does
                let content_sample: String = content.lines().take(20).collect::<Vec<_>>().join(" ");
                if content_sample.contains("webhook.site")
                    || content_sample.contains("curl")
                    || content_sample.contains("https.request")
                {
                    high_risk_files.insert(entry.path().to_path_buf()); // BASH: mark as HIGH RISK
                    findings.push(Finding::new(
                        entry.path().to_path_buf(),
                        "Credential patterns with potential exfiltration".to_string(),
                        RiskLevel::High,
                        "credential_exfiltration",
                    ));
                } else if !high_risk_files.contains(entry.path()) {
                    // BASH: only MEDIUM if not already HIGH RISK
                    findings.push(Finding::new(
                        entry.path().to_path_buf(),
                        "Contains credential scanning patterns".to_string(),
                        RiskLevel::Medium,
                        "credential_patterns",
                    ));
                }
            }

            // Environment variable scanning - BASH line 744-777
            if content.contains("process.env")
                || content.contains("os.environ")
                || content.contains("getenv")
            {
                let path_str = entry.path().to_string_lossy();

                // BASH EXACT: Skip if file already has HIGH RISK credential pattern
                if high_risk_files.contains(entry.path()) {
                    continue; // BASH: no duplicate risk classification for same file
                }

                // BASH line 750-755: node_modules/build_output = LOW RISK (if not legitimate)
                if path_str.contains("/node_modules/") || path_str.contains("\\node_modules\\") {
                    // Check if it's legitimate pattern first - Vue/webpack/etc
                    let content_sample: String =
                        content.lines().take(20).collect::<Vec<_>>().join(" ");
                    let is_legit = content_sample.contains("webpack")
                        || content_sample.contains("vite")
                        || content_sample.contains("rollup")
                        || content_sample.contains("Vue")
                        || content_sample.contains("createApp")
                        || content_sample.contains("DefinePlugin")
                        || content_sample.contains("NODE_ENV");

                    if !is_legit {
                        findings.push(Finding::new(
                            entry.path().to_path_buf(),
                            "Environment variable access in node_modules".to_string(),
                            RiskLevel::Low,
                            "env_node_modules",
                        ));
                    }
                    continue;
                }

                // BASH EXACT: is_legitimate_pattern function
                // Vue.js development patterns
                if content.contains("process.env.NODE_ENV") && content.contains("production") {
                    continue; // Bash: legitimate
                }

                // Common framework patterns (createApp, Vue)
                if content.contains("createApp") || content.contains("Vue") {
                    continue; // Bash: legitimate
                }

                // Package manager and build tool patterns
                if content.contains("webpack")
                    || content.contains("vite")
                    || content.contains("rollup")
                {
                    continue; // Bash: legitimate
                }

                // BASH EXACT: Ethers.js legitimate crypto usage (from legitimate-crypto test)
                if content.contains("ethers") && content.contains("sendTransaction") {
                    continue; // Bash: legitimate crypto tools
                }

                // BASH EXACT: Use content_sample for environment scanning too
                let content_sample: String = content.lines().take(20).collect::<Vec<_>>().join(" ");
                if content_sample.contains("webhook.site") && content_sample.contains("exfiltrat") {
                    findings.push(Finding::new(
                        entry.path().to_path_buf(),
                        "Environment scanning with exfiltration".to_string(),
                        RiskLevel::High,
                        "env_exfiltration",
                    ));
                } else if (content_sample.contains("scan") || content_sample.contains("harvest") || content_sample.contains("steal"))
                    && !high_risk_files.contains(entry.path()) 
                    && !content_sample.contains("webpack") // BASH: additional filtering like is_legitimate_pattern
                    && !content_sample.contains("vite")
                    && !content_sample.contains("rollup")
                    && !content_sample.contains("Vue") // BASH: Vue.js patterns are legitimate
                    && !content_sample.contains("createApp") // BASH: framework patterns are legitimate
                    && !content_sample.contains("DefinePlugin") // BASH: webpack DefinePlugin is legitimate
                    && !content_sample.contains("NODE_ENV")
                // BASH: NODE_ENV usage is legitimate
                {
                    findings.push(Finding::new(
                        entry.path().to_path_buf(),
                        "Potentially suspicious environment variable access".to_string(),
                        RiskLevel::Medium,
                        "env_suspicious",
                    ));
                }
            }
        }
    }

    findings
}
