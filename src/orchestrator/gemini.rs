use super::cli_mapper::{LlmArg, LlmProvider, ResumeMode};
use super::prompts;
use super::{ModelSelector, ScriptRunner, UsageMetrics};
use crate::models::{AgentdConfig, Complexity};
use anyhow::{Context, Result};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

/// Self-review outcome from Gemini
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelfReviewResult {
    /// No issues found
    Pass,
    /// Issues found and files were edited
    NeedsRevision,
}

/// Gemini stream-json message response format for text content
#[derive(Debug, Deserialize)]
struct GeminiMessageResponse {
    #[serde(rename = "type")]
    response_type: Option<String>,
    content: Option<serde_json::Value>,
}

/// Detect self-review markers from Gemini stream-json output
///
/// Parses the output looking for `<review>PASS</review>` or `<review>NEEDS_REVISION</review>`.
/// If no marker is found, logs a warning and returns Pass (non-blocking, proceed to validation).
pub fn detect_self_review_marker(output: &str) -> SelfReviewResult {
    // Assemble all text content from the output
    let mut assembled_text = String::new();

    for line in output.lines() {
        // Try to parse as a message with content
        if let Ok(msg) = serde_json::from_str::<GeminiMessageResponse>(line) {
            if msg.response_type.as_deref() == Some("message") {
                if let Some(content) = msg.content {
                    // Content can be a string or an array of parts
                    match content {
                        serde_json::Value::String(s) => {
                            assembled_text.push_str(&s);
                        }
                        serde_json::Value::Array(parts) => {
                            for part in parts {
                                // Each part may have a "text" field
                                if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                    assembled_text.push_str(text);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Also check raw output for escaped markers
    let full_text = format!("{}\n{}", assembled_text, output);

    // Check for markers
    if full_text.contains("<review>NEEDS_REVISION</review>") {
        return SelfReviewResult::NeedsRevision;
    }
    if full_text.contains("<review>PASS</review>") {
        return SelfReviewResult::Pass;
    }

    // No marker found - log warning and treat as PASS
    eprintln!("Warning: Self-review marker not found in output, treating as PASS");
    SelfReviewResult::Pass
}

/// Find the session index for a given UUID by calling `gemini --list-sessions`
///
/// # Arguments
/// * `uuid` - The session UUID to find
/// * `project_root` - The project root directory (used as cwd for gemini command)
///
/// # Returns
/// The 1-indexed session index for use with `--resume <index>`
///
/// # Errors
/// - If the command fails (non-zero exit code)
/// - If the output format is unexpected
/// - If the UUID is not found in the session list
pub async fn find_session_index(uuid: &str, project_root: &PathBuf) -> Result<u32> {
    // Run gemini --list-sessions with project_root as cwd
    let output = Command::new("gemini")
        .arg("--list-sessions")
        .current_dir(project_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to execute gemini --list-sessions")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        anyhow::bail!(
            "gemini --list-sessions failed with exit code {:?}\nstdout: {}\nstderr: {}",
            output.status.code(),
            stdout,
            stderr
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse the output format:
    // Available sessions for this project (N):
    //   1. <prompt preview> (time ago) [UUID]
    //   2. <prompt preview> (time ago) [UUID]

    // Check for expected header
    if !stdout.contains("Available sessions") {
        anyhow::bail!(
            "Failed to parse session list: unexpected format (no header)\nRaw output: {}",
            stdout
        );
    }

    // Parse lines looking for the UUID in brackets
    // Format: "  1. <text> [UUID]"
    let re = Regex::new(r"^\s*(\d+)\.\s+.*\[([a-f0-9-]+)\]").unwrap();

    for line in stdout.lines() {
        if let Some(captures) = re.captures(line) {
            let index: u32 = captures
                .get(1)
                .unwrap()
                .as_str()
                .parse()
                .context("Failed to parse session index")?;
            let found_uuid = captures.get(2).unwrap().as_str();

            if found_uuid == uuid {
                return Ok(index);
            }
        }
    }

    anyhow::bail!("Session not found, please re-run proposal")
}

/// Gemini orchestrator for proposal and documentation tasks
pub struct GeminiOrchestrator<'a> {
    model_selector: ModelSelector<'a>,
    runner: ScriptRunner,
    project_root: PathBuf,
}

impl<'a> GeminiOrchestrator<'a> {
    pub fn new(config: &'a AgentdConfig, project_root: impl Into<PathBuf>) -> Self {
        let project_root = project_root.into();
        let model_selector = ModelSelector::new(config);
        let runner = ScriptRunner::new();

        Self {
            model_selector,
            runner,
            project_root,
        }
    }

    /// Build common Gemini environment variables
    fn build_env(&self, change_id: &str) -> HashMap<String, String> {
        let gemini_system_md = self
            .project_root
            .join("agentd")
            .join("changes")
            .join(change_id)
            .join("GEMINI.md");

        let mut env = HashMap::new();
        env.insert(
            "GEMINI_SYSTEM_MD".to_string(),
            gemini_system_md.to_string_lossy().to_string(),
        );
        env
    }

    /// Build common Gemini args with resume mode
    fn build_args_with_resume(&self, task: &str, complexity: Complexity, resume_mode: ResumeMode) -> Vec<String> {
        let model = self.model_selector.select_gemini(complexity);
        let args = vec![
            LlmArg::Task(task.to_string()),
            LlmArg::Model(model.to_cli_arg()),
            LlmArg::OutputFormat("stream-json".to_string()),
        ];
        LlmProvider::Gemini.build_args_with_resume(&args, resume_mode)
    }

    /// Build common Gemini args (convenience wrapper for bool resume)
    fn build_args(&self, task: &str, complexity: Complexity, resume: bool) -> Vec<String> {
        let resume_mode = if resume {
            ResumeMode::Latest
        } else {
            ResumeMode::None
        };
        self.build_args_with_resume(task, complexity, resume_mode)
    }

    /// Run proposal generation
    pub async fn run_proposal(
        &self,
        change_id: &str,
        description: &str,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        let prompt = prompts::gemini_proposal_prompt(change_id, description);
        let env = self.build_env(change_id);
        let args = self.build_args("agentd:proposal", complexity, false);

        self.runner.run_llm_with_cwd(LlmProvider::Gemini, args, env, &prompt, true, Some(&self.project_root)).await
    }

    /// Run self-review to check proposal files (resumes session by index)
    ///
    /// # Arguments
    /// * `change_id` - The change ID
    /// * `session_index` - The session index to resume (from find_session_index)
    /// * `complexity` - Complexity level for model selection
    ///
    /// # Returns
    /// Tuple of (output text, usage metrics). Use detect_self_review_marker() to parse the result.
    pub async fn run_self_review(
        &self,
        change_id: &str,
        session_index: u32,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        let prompt = prompts::proposal_self_review_prompt(change_id);
        let env = self.build_env(change_id);
        let args = self.build_args_with_resume("agentd:self-review", complexity, ResumeMode::ByIndex(session_index));

        self.runner.run_llm_with_cwd(LlmProvider::Gemini, args, env, &prompt, true, Some(&self.project_root)).await
    }

    /// Run reproposal (resume previous session for cached context)
    #[allow(dead_code)]
    pub async fn run_reproposal(
        &self,
        change_id: &str,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        let prompt = prompts::gemini_reproposal_prompt(change_id);
        let env = self.build_env(change_id);
        // Resume previous session (Plan stage)
        let args = self.build_args("agentd:reproposal", complexity, true);

        self.runner.run_llm_with_cwd(LlmProvider::Gemini, args, env, &prompt, true, Some(&self.project_root)).await
    }

    /// Run reproposal with fresh session (no resume - GEMINI.md provides all context)
    pub async fn run_reproposal_fresh(
        &self,
        change_id: &str,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        let prompt = prompts::gemini_reproposal_prompt(change_id);
        let env = self.build_env(change_id);
        // Fresh session - documents in GEMINI.md provide all necessary context
        let args = self.build_args("agentd:reproposal", complexity, false);

        self.runner.run_llm_with_cwd(LlmProvider::Gemini, args, env, &prompt, true, Some(&self.project_root)).await
    }

    /// Run reproposal with session index (resume by specific session)
    pub async fn run_reproposal_with_session(
        &self,
        change_id: &str,
        session_index: u32,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        let prompt = prompts::gemini_reproposal_prompt(change_id);
        let env = self.build_env(change_id);
        let args = self.build_args_with_resume("agentd:reproposal", complexity, ResumeMode::ByIndex(session_index));

        self.runner.run_llm_with_cwd(LlmProvider::Gemini, args, env, &prompt, true, Some(&self.project_root)).await
    }

    /// Run spec merging (merge delta specs back to main specs)
    pub async fn run_merge_specs(
        &self,
        change_id: &str,
        strategy: &str,
        spec_file: &str,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        let prompt = prompts::gemini_merge_specs_prompt(change_id, strategy, spec_file);
        let env = self.build_env(change_id);
        let args = self.build_args("agentd:merge-specs", complexity, false);

        self.runner.run_llm_with_cwd(LlmProvider::Gemini, args, env, &prompt, true, Some(&self.project_root)).await
    }

    /// Run changelog generation
    pub async fn run_changelog(&self, change_id: &str, complexity: Complexity) -> Result<(String, UsageMetrics)> {
        let prompt = prompts::gemini_changelog_prompt(change_id);
        let env = self.build_env(change_id);
        // First call in Archive stage, no resume
        let args = self.build_args("agentd:changelog", complexity, false);

        self.runner.run_llm_with_cwd(LlmProvider::Gemini, args, env, &prompt, true, Some(&self.project_root)).await
    }

    /// Run fillback (fill placeholders in files)
    pub async fn run_fillback(
        &self,
        change_id: &str,
        file_path: &str,
        placeholder: &str,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        let prompt = prompts::gemini_fillback_prompt(change_id, file_path, placeholder);
        let env = self.build_env(change_id);
        let args = self.build_args("agentd:fillback", complexity, false);

        self.runner.run_llm_with_cwd(LlmProvider::Gemini, args, env, &prompt, true, Some(&self.project_root)).await
    }

    /// Run archive fix (fix issues from archive review)
    pub async fn run_archive_fix(
        &self,
        change_id: &str,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        let prompt = prompts::gemini_archive_fix_prompt(change_id);
        let env = self.build_env(change_id);
        // Resume previous session (Archive stage)
        let args = self.build_args("agentd:archive-fix", complexity, true);

        self.runner.run_llm_with_cwd(LlmProvider::Gemini, args, env, &prompt, true, Some(&self.project_root)).await
    }

    /// Run a one-shot command without session reuse
    ///
    /// Each call is independent - context comes from files (via MCP), not session history.
    /// This is used for sequential generation where each phase reads the output of previous phases.
    ///
    /// # Arguments
    /// * `change_id` - The change ID (used for GEMINI.md context)
    /// * `prompt` - The prompt to send to Gemini
    /// * `complexity` - Complexity level for model selection
    ///
    /// # Returns
    /// Tuple of (output text, usage metrics)
    pub async fn run_one_shot(
        &self,
        change_id: &str,
        prompt: &str,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        let env = self.build_env(change_id);
        // No resume - each call is a fresh session
        let args = self.build_args("agentd:one-shot", complexity, false);

        self.runner.run_llm_with_cwd(LlmProvider::Gemini, args, env, prompt, true, Some(&self.project_root)).await
    }

    // =========================================================================
    // MCP-based Task Delivery (Agent-Agnostic)
    // =========================================================================

    /// Generate a short prompt instructing agent to call get_task MCP tool
    fn mcp_task_prompt(change_id: &str, task_type: &str, extra_params: &str) -> String {
        format!(
            r#"Call the `get_task` MCP tool to get your task instructions.

Parameters:
- change_id: "{}"
- task_type: "{}"
{}"#,
            change_id, task_type, extra_params
        )
    }

    /// Build args with prompt included (for headless mode, no stdin)
    fn build_args_with_mcp_prompt(&self, complexity: Complexity, prompt: &str) -> Vec<String> {
        let model = self.model_selector.select_gemini(complexity);
        let args = vec![
            LlmArg::Model(model.to_cli_arg()),
            LlmArg::OutputFormat("stream-json".to_string()),
            LlmArg::Prompt(prompt.to_string()),
        ];
        LlmProvider::Gemini.build_args_with_resume(&args, ResumeMode::None)
    }

    /// Run a task using MCP-based delivery (agent-agnostic)
    ///
    /// This method passes a short prompt via CLI args instead of stdin,
    /// and the agent retrieves full task instructions via the get_task MCP tool.
    ///
    /// # Arguments
    /// * `change_id` - The change ID to work on
    /// * `task_type` - Task type (e.g., "create_proposal", "create_spec")
    /// * `extra_params` - Extra parameters for the task (e.g., "- spec_id: \"auth-flow\"")
    /// * `complexity` - Complexity level for model selection
    pub async fn run_task_mcp(
        &self,
        change_id: &str,
        task_type: &str,
        extra_params: &str,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        let prompt = Self::mcp_task_prompt(change_id, task_type, extra_params);
        let env = self.build_env(change_id);
        let args = self.build_args_with_mcp_prompt(complexity, &prompt);

        // Empty string for stdin prompt - we're using CLI args instead
        self.runner.run_llm_with_cwd(LlmProvider::Gemini, args, env, "", true, Some(&self.project_root)).await
    }

    /// Run create_proposal task via MCP
    pub async fn run_create_proposal_mcp(
        &self,
        change_id: &str,
        description: &str,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        let extra = format!("- description: \"{}\"", description.replace('"', "\\\""));
        self.run_task_mcp(change_id, "create_proposal", &extra, complexity).await
    }

    /// Run review_proposal task via MCP
    pub async fn run_review_proposal_mcp(
        &self,
        change_id: &str,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        self.run_task_mcp(change_id, "review_proposal", "", complexity).await
    }

    /// Run create_spec task via MCP with optional dependencies
    pub async fn run_create_spec_mcp(
        &self,
        change_id: &str,
        spec_id: &str,
        dependencies: &[String],
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        let mut extra = format!("- spec_id: \"{}\"", spec_id);
        if !dependencies.is_empty() {
            let deps_json: Vec<String> = dependencies.iter().map(|d| format!("\"{}\"", d)).collect();
            extra.push_str(&format!("\n- dependencies: [{}]", deps_json.join(", ")));
        }
        self.run_task_mcp(change_id, "create_spec", &extra, complexity).await
    }

    /// Run create_tasks task via MCP
    pub async fn run_create_tasks_mcp(
        &self,
        change_id: &str,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        self.run_task_mcp(change_id, "create_tasks", "", complexity).await
    }

    /// Run revise_proposal task via MCP (fix proposal based on review feedback)
    pub async fn run_revise_proposal_mcp(
        &self,
        change_id: &str,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        self.run_task_mcp(change_id, "revise_proposal", "", complexity).await
    }

    /// Run revise_spec task via MCP (fix spec based on review feedback)
    pub async fn run_revise_spec_mcp(
        &self,
        change_id: &str,
        spec_id: &str,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        let extra = format!("- spec_id: \"{}\"", spec_id);
        self.run_task_mcp(change_id, "revise_spec", &extra, complexity).await
    }

    /// Run revise_tasks task via MCP (fix tasks based on review feedback)
    pub async fn run_revise_tasks_mcp(
        &self,
        change_id: &str,
        complexity: Complexity,
    ) -> Result<(String, UsageMetrics)> {
        self.run_task_mcp(change_id, "revise_tasks", "", complexity).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_orchestrator_creation() {
        let config = AgentdConfig::default();
        let orchestrator = GeminiOrchestrator::new(&config, "/tmp/test");
        assert_eq!(orchestrator.project_root, PathBuf::from("/tmp/test"));
    }

    // =========================================================================
    // Self-Review Marker Detection Tests
    // =========================================================================

    #[test]
    fn test_detect_self_review_marker_pass() {
        let output = r#"{"type":"message","content":"Reviewing files..."}
{"type":"message","content":"All looks good!\n<review>PASS</review>"}
{"type":"result","status":"success"}"#;

        assert_eq!(detect_self_review_marker(output), SelfReviewResult::Pass);
    }

    #[test]
    fn test_detect_self_review_marker_needs_revision() {
        let output = r#"{"type":"message","content":"Found issues, fixing..."}
{"type":"message","content":"<review>NEEDS_REVISION</review>"}
{"type":"result","status":"success"}"#;

        assert_eq!(
            detect_self_review_marker(output),
            SelfReviewResult::NeedsRevision
        );
    }

    #[test]
    fn test_detect_self_review_marker_in_raw_output() {
        // Test when marker appears in raw text (not parsed JSON content)
        let output = "Some raw text <review>PASS</review> more text";

        assert_eq!(detect_self_review_marker(output), SelfReviewResult::Pass);
    }

    #[test]
    fn test_detect_self_review_marker_multiline() {
        // Test marker spread across content with other text
        let output = r#"{"type":"message","content":"Reviewing proposal.md..."}
{"type":"message","content":"Reviewing tasks.md..."}
{"type":"message","content":"Reviewing specs/workflow.md..."}
{"type":"message","content":"No issues found.\n\n<review>PASS</review>"}
{"type":"result","status":"success"}"#;

        assert_eq!(detect_self_review_marker(output), SelfReviewResult::Pass);
    }

    #[test]
    fn test_detect_self_review_marker_array_content() {
        // Test content as array of parts (Gemini can emit nested text fields)
        let output = r#"{"type":"message","content":[{"text":"Found issues"},{"text":"\n<review>NEEDS_REVISION</review>"}]}"#;

        assert_eq!(
            detect_self_review_marker(output),
            SelfReviewResult::NeedsRevision
        );
    }

    #[test]
    fn test_detect_self_review_marker_no_marker_returns_pass() {
        // No marker found should return PASS with warning
        let output = r#"{"type":"message","content":"Done reviewing, everything looks fine."}
{"type":"result","status":"success"}"#;

        assert_eq!(detect_self_review_marker(output), SelfReviewResult::Pass);
    }

    #[test]
    fn test_detect_self_review_marker_empty_output() {
        assert_eq!(detect_self_review_marker(""), SelfReviewResult::Pass);
    }

    #[test]
    fn test_detect_self_review_marker_escaped_characters() {
        // Test with escaped characters in stream-json
        let output = r#"{"type":"message","content":"Files reviewed.\n\n\u003creview\u003ePASS\u003c/review\u003e"}"#;
        // \u003c = <, \u003e = >

        // The JSON parser will unescape these, so it should find the marker
        assert_eq!(detect_self_review_marker(output), SelfReviewResult::Pass);
    }

    #[test]
    fn test_detect_self_review_prefers_needs_revision() {
        // If both markers somehow appear, NEEDS_REVISION takes precedence
        let output = "<review>PASS</review>\n<review>NEEDS_REVISION</review>";

        assert_eq!(
            detect_self_review_marker(output),
            SelfReviewResult::NeedsRevision
        );
    }

    // =========================================================================
    // Session Index Lookup Tests (unit tests for parsing logic)
    // =========================================================================

    #[test]
    fn test_session_list_regex_parsing() {
        // Test the regex pattern used in find_session_index
        let re = Regex::new(r"^\s*(\d+)\.\s+.*\[([a-f0-9-]+)\]").unwrap();

        let line = "  1. Create proposal files in... (2 hours ago) [abc123-def456-789]";
        let captures = re.captures(line).expect("Should match");
        assert_eq!(captures.get(1).unwrap().as_str(), "1");
        assert_eq!(captures.get(2).unwrap().as_str(), "abc123-def456-789");

        let line2 = "  15. Another session (3 days ago) [fedcba98-7654-3210-abcd-ef0123456789]";
        let captures2 = re.captures(line2).expect("Should match");
        assert_eq!(captures2.get(1).unwrap().as_str(), "15");
        assert_eq!(
            captures2.get(2).unwrap().as_str(),
            "fedcba98-7654-3210-abcd-ef0123456789"
        );
    }

    #[test]
    fn test_session_list_no_match_for_invalid_lines() {
        let re = Regex::new(r"^\s*(\d+)\.\s+.*\[([a-f0-9-]+)\]").unwrap();

        // Header line shouldn't match
        assert!(re.captures("Available sessions for this project (5):").is_none());

        // Line without UUID shouldn't match
        assert!(re.captures("  1. Some session without uuid").is_none());
    }

    // =========================================================================
    // Session Lookup Failure Path Tests
    // =========================================================================

    /// Helper function to simulate parsing session list output
    /// This extracts the parsing logic for unit testing
    fn parse_session_list_for_uuid(stdout: &str, uuid: &str) -> Result<u32> {
        // Check for expected header
        if !stdout.contains("Available sessions") {
            anyhow::bail!(
                "Failed to parse session list: unexpected format (no header)\nRaw output: {}",
                stdout
            );
        }

        // Parse lines looking for the UUID in brackets
        let re = Regex::new(r"^\s*(\d+)\.\s+.*\[([a-f0-9-]+)\]").unwrap();

        for line in stdout.lines() {
            if let Some(captures) = re.captures(line) {
                let index: u32 = captures
                    .get(1)
                    .unwrap()
                    .as_str()
                    .parse()
                    .context("Failed to parse session index")?;
                let found_uuid = captures.get(2).unwrap().as_str();

                if found_uuid == uuid {
                    return Ok(index);
                }
            }
        }

        anyhow::bail!("Session not found, please re-run proposal")
    }

    #[test]
    fn test_parse_session_list_uuid_found() {
        // UUIDs must be lowercase hex with dashes to match the regex
        let output = r#"Available sessions for this project (3):
  1. Create proposal files in... (2 hours ago) [abc123-def456-789012]
  2. Fix validation errors... (1 day ago) [abcd1234-5678-90ab-cdef-123456789abc]
  3. Another session (3 days ago) [xyz789-123-456789]"#;

        let result = parse_session_list_for_uuid(output, "abcd1234-5678-90ab-cdef-123456789abc");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_parse_session_list_uuid_not_found() {
        let output = r#"Available sessions for this project (2):
  1. Session one (1 hour ago) [abc123-def456-789012]
  2. Session two (2 hours ago) [def456-abc789-012345]"#;

        let result = parse_session_list_for_uuid(output, "nonexistent-uuid-1234");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Session not found"));
    }

    #[test]
    fn test_parse_session_list_unexpected_format_no_header() {
        let output = r#"Some random output
  1. Session one [abc123-def456-789012]"#;

        let result = parse_session_list_for_uuid(output, "abc123-def456-789012");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Failed to parse session list"));
        assert!(err.contains("unexpected format"));
    }

    #[test]
    fn test_parse_session_list_malformed_lines() {
        // Valid header but malformed session lines (no brackets)
        let output = r#"Available sessions for this project (2):
  1. Session without uuid bracket
  2. Another malformed line"#;

        let result = parse_session_list_for_uuid(output, "any-uuid");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Session not found"));
    }

    #[test]
    fn test_parse_session_list_empty_output() {
        let output = "Available sessions for this project (0):";

        let result = parse_session_list_for_uuid(output, "any-uuid");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Session not found"));
    }
}
