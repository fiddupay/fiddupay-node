// Test Progress Tracker
// Tracks and reports testing progress across all phases

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestProgress {
    pub phase_name: String,
    pub total_tests: usize,
    pub completed_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub status: TestStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub test_results: Vec<TestResult>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestResult {
    pub test_id: String,
    pub test_name: String,
    pub status: TestStatus,
    pub duration_ms: Option<u64>,
    pub error_message: Option<String>,
    pub executed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TestStatus {
    NotStarted,
    InProgress,
    Passed,
    Failed,
    Skipped,
}

impl TestStatus {
    pub fn emoji(&self) -> &'static str {
        match self {
            TestStatus::NotStarted => "⏳",
            TestStatus::InProgress => "🔄",
            TestStatus::Passed => "✅",
            TestStatus::Failed => "❌",
            TestStatus::Skipped => "⏭️",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OverallProgress {
    pub phases: HashMap<String, TestProgress>,
    pub total_tests: usize,
    pub completed_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub overall_progress_percent: f64,
    pub started_at: Option<DateTime<Utc>>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

impl OverallProgress {
    pub fn new() -> Self {
        let mut phases = HashMap::new();
        
        // Initialize all test phases
        phases.insert("phase1".to_string(), TestProgress::new("Phase 1: Core Infrastructure", 5));
        phases.insert("phase2".to_string(), TestProgress::new("Phase 2: Wallet Management", 12));
        phases.insert("phase3".to_string(), TestProgress::new("Phase 3: Gas Validation", 15));
        phases.insert("phase4".to_string(), TestProgress::new("Phase 4: API Endpoints", 16));
        phases.insert("phase5".to_string(), TestProgress::new("Phase 5: Frontend Integration", 10));
        phases.insert("phase6".to_string(), TestProgress::new("Phase 6: Security & Monitoring", 12));
        phases.insert("integration".to_string(), TestProgress::new("Integration Testing", 8));
        phases.insert("performance".to_string(), TestProgress::new("Performance Testing", 5));
        phases.insert("security".to_string(), TestProgress::new("Security Testing", 6));
        
        let total_tests = phases.values().map(|p| p.total_tests).sum();
        
        Self {
            phases,
            total_tests,
            completed_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            overall_progress_percent: 0.0,
            started_at: None,
            estimated_completion: None,
        }
    }

    pub fn start_phase(&mut self, phase_id: &str) {
        if let Some(phase) = self.phases.get_mut(phase_id) {
            phase.status = TestStatus::InProgress;
            phase.started_at = Some(Utc::now());
            
            if self.started_at.is_none() {
                self.started_at = Some(Utc::now());
            }
        }
    }

    pub fn complete_test(&mut self, phase_id: &str, test_result: TestResult) {
        if let Some(phase) = self.phases.get_mut(phase_id) {
            phase.completed_tests += 1;
            
            match test_result.status {
                TestStatus::Passed => {
                    phase.passed_tests += 1;
                    self.passed_tests += 1;
                }
                TestStatus::Failed => {
                    phase.failed_tests += 1;
                    self.failed_tests += 1;
                }
                _ => {}
            }
            
            phase.test_results.push(test_result);
            self.completed_tests += 1;
            
            // Update phase status
            if phase.completed_tests == phase.total_tests {
                phase.status = if phase.failed_tests == 0 {
                    TestStatus::Passed
                } else {
                    TestStatus::Failed
                };
                phase.completed_at = Some(Utc::now());
            }
            
            // Update overall progress
            self.overall_progress_percent = (self.completed_tests as f64 / self.total_tests as f64) * 100.0;
        }
    }

    pub fn print_progress_report(&self) {
        println!("\n📊 Hybrid Non-Custodial System - Test Progress Report");
        println!("=" .repeat(70));
        
        println!("🎯 Overall Progress: {:.1}% ({}/{})", 
            self.overall_progress_percent, self.completed_tests, self.total_tests);
        println!("✅ Passed: {} | ❌ Failed: {} | ⏳ Remaining: {}", 
            self.passed_tests, self.failed_tests, self.total_tests - self.completed_tests);
        
        if let Some(started) = self.started_at {
            let elapsed = Utc::now().signed_duration_since(started);
            println!("⏱️ Elapsed Time: {}m {}s", elapsed.num_minutes(), elapsed.num_seconds() % 60);
        }
        
        println!("\n📋 Phase Progress:");
        println!("-" .repeat(70));
        
        let phase_order = vec![
            "phase1", "phase2", "phase3", "phase4", "phase5", 
            "phase6", "integration", "performance", "security"
        ];
        
        for phase_id in phase_order {
            if let Some(phase) = self.phases.get(phase_id) {
                let progress_percent = if phase.total_tests > 0 {
                    (phase.completed_tests as f64 / phase.total_tests as f64) * 100.0
                } else {
                    0.0
                };
                
                println!("{} {} - {:.1}% ({}/{}) - ✅{} ❌{}", 
                    phase.status.emoji(),
                    phase.phase_name,
                    progress_percent,
                    phase.completed_tests,
                    phase.total_tests,
                    phase.passed_tests,
                    phase.failed_tests
                );
            }
        }
        
        // Show failed tests if any
        if self.failed_tests > 0 {
            println!("\n❌ Failed Tests:");
            println!("-" .repeat(70));
            
            for phase in self.phases.values() {
                for test in &phase.test_results {
                    if test.status == TestStatus::Failed {
                        println!("  • {}: {} - {}", 
                            test.test_id, 
                            test.test_name,
                            test.error_message.as_deref().unwrap_or("Unknown error")
                        );
                    }
                }
            }
        }
        
        println!("=" .repeat(70));
    }

    pub fn generate_html_report(&self) -> String {
        let mut html = String::new();
        
        html.push_str("<!DOCTYPE html><html><head>");
        html.push_str("<title>Hybrid Non-Custodial System - Test Report</title>");
        html.push_str("<style>");
        html.push_str("body { font-family: Arial, sans-serif; margin: 20px; }");
        html.push_str(".header { background: #f0f8ff; padding: 20px; border-radius: 8px; }");
        html.push_str(".phase { margin: 10px 0; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }");
        html.push_str(".passed { background: #f0fff0; }");
        html.push_str(".failed { background: #fff0f0; }");
        html.push_str(".progress-bar { width: 100%; height: 20px; background: #f0f0f0; border-radius: 10px; overflow: hidden; }");
        html.push_str(".progress-fill { height: 100%; background: #4caf50; transition: width 0.3s; }");
        html.push_str("</style></head><body>");
        
        html.push_str(&format!("<div class='header'><h1>🧪 Hybrid Non-Custodial System Test Report</h1>"));
        html.push_str(&format!("<p><strong>Overall Progress:</strong> {:.1}% ({}/{})</p>", 
            self.overall_progress_percent, self.completed_tests, self.total_tests));
        html.push_str(&format!("<div class='progress-bar'><div class='progress-fill' style='width: {:.1}%'></div></div>", 
            self.overall_progress_percent));
        html.push_str("</div>");
        
        for phase in self.phases.values() {
            let class = match phase.status {
                TestStatus::Passed => "phase passed",
                TestStatus::Failed => "phase failed",
                _ => "phase",
            };
            
            let progress_percent = if phase.total_tests > 0 {
                (phase.completed_tests as f64 / phase.total_tests as f64) * 100.0
            } else {
                0.0
            };
            
            html.push_str(&format!("<div class='{}'><h3>{} {}</h3>", class, phase.status.emoji(), phase.phase_name));
            html.push_str(&format!("<p>Progress: {:.1}% ({}/{}) | Passed: {} | Failed: {}</p>", 
                progress_percent, phase.completed_tests, phase.total_tests, phase.passed_tests, phase.failed_tests));
            
            if !phase.test_results.is_empty() {
                html.push_str("<ul>");
                for test in &phase.test_results {
                    html.push_str(&format!("<li>{} {} - {}</li>", 
                        test.status.emoji(), test.test_name, 
                        test.error_message.as_deref().unwrap_or("OK")));
                }
                html.push_str("</ul>");
            }
            
            html.push_str("</div>");
        }
        
        html.push_str("</body></html>");
        html
    }

    pub fn is_complete(&self) -> bool {
        self.completed_tests == self.total_tests
    }

    pub fn is_success(&self) -> bool {
        self.is_complete() && self.failed_tests == 0
    }
}

impl TestProgress {
    pub fn new(phase_name: &str, total_tests: usize) -> Self {
        Self {
            phase_name: phase_name.to_string(),
            total_tests,
            completed_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            status: TestStatus::NotStarted,
            started_at: None,
            completed_at: None,
            test_results: Vec::new(),
        }
    }
}

impl TestResult {
    pub fn new_passed(test_id: &str, test_name: &str, duration_ms: Option<u64>) -> Self {
        Self {
            test_id: test_id.to_string(),
            test_name: test_name.to_string(),
            status: TestStatus::Passed,
            duration_ms,
            error_message: None,
            executed_at: Utc::now(),
        }
    }

    pub fn new_failed(test_id: &str, test_name: &str, error: &str, duration_ms: Option<u64>) -> Self {
        Self {
            test_id: test_id.to_string(),
            test_name: test_name.to_string(),
            status: TestStatus::Failed,
            duration_ms,
            error_message: Some(error.to_string()),
            executed_at: Utc::now(),
        }
    }
}
