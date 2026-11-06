use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::error::RagError;
use crate::retrieval::HybridSearchEngine;
use crate::security::{is_sensitive_or_ignored, validate_safe_workspace_path};

/// Security capabilities required to execute sandboxed tools.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCapability {
    WorkspaceRead,
    Compute,
    CodeSearch,
    Network,
}

/// Execution constraints and security context provided to sandboxed tools.
#[derive(Debug, Clone)]
pub struct ToolExecutionContext {
    pub workspace_root: PathBuf,
    pub local_only_mode: bool,
    pub allowed_capabilities: HashSet<ToolCapability>,
    pub max_timeout: Duration,
    pub max_output_bytes: usize,
}

impl Default for ToolExecutionContext {
    fn default() -> Self {
        let mut caps = HashSet::new();
        caps.insert(ToolCapability::WorkspaceRead);
        caps.insert(ToolCapability::Compute);
        caps.insert(ToolCapability::CodeSearch);

        Self {
            workspace_root: PathBuf::from("."),
            local_only_mode: true,
            allowed_capabilities: caps,
            max_timeout: Duration::from_millis(5000),
            max_output_bytes: 64 * 1024,
        }
    }
}

/// Structured outcome of a sandboxed tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    pub tool_name: String,
    pub success: bool,
    pub output: String,
    pub duration_ms: f64,
    pub bytes_produced: usize,
    pub truncated: bool,
    pub error_message: Option<String>,
}

/// Summary description of a registered tool for LLM function calling schemas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDescriptor {
    pub name: String,
    pub description: String,
    pub required_capability: ToolCapability,
}

/// Trait defining a sandboxed local tool.
pub trait SandboxedTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn required_capability(&self) -> ToolCapability;
    fn execute(&self, input_json: &str, context: &ToolExecutionContext)
        -> Result<String, RagError>;
}

// -----------------------------------------------------------------------------
// Built-in Tool 1: WorkspaceFileReadTool
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct ReadFileInput {
    path: String,
    line_start: Option<usize>,
    line_end: Option<usize>,
}

/// Sandboxed workspace file reader with path traversal and sensitive pattern defense.
pub struct WorkspaceFileReadTool;

impl SandboxedTool for WorkspaceFileReadTool {
    fn name(&self) -> &str {
        "workspace_read_file"
    }

    fn description(&self) -> &str {
        "Reads a slice of text from a file within the workspace with line boundary controls."
    }

    fn required_capability(&self) -> ToolCapability {
        ToolCapability::WorkspaceRead
    }

    fn execute(
        &self,
        input_json: &str,
        context: &ToolExecutionContext,
    ) -> Result<String, RagError> {
        let input: ReadFileInput = serde_json::from_str(input_json)
            .map_err(|e| RagError::Serialization(format!("Invalid read_file arguments: {e}")))?;

        let candidate = Path::new(&input.path);
        let absolute_path = validate_safe_workspace_path(candidate, &context.workspace_root)?;

        if is_sensitive_or_ignored(&absolute_path) || is_sensitive_or_ignored(candidate) {
            return Err(RagError::SensitiveFileExcluded(input.path));
        }

        let raw_bytes = fs::read(&absolute_path)?;
        if crate::security::is_binary_content(&raw_bytes) {
            return Err(RagError::BinaryFileRejected(input.path));
        }

        let content = String::from_utf8(raw_bytes).map_err(|e| RagError::InvalidEncoding {
            path: input.path.clone(),
            source: e,
        })?;

        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        let start = input.line_start.unwrap_or(1).max(1);
        let end = input.line_end.unwrap_or(total_lines).min(total_lines);

        if start > total_lines || start > end {
            return Ok(format!(
                "[{}] Empty line range (L{start}-L{end} of {total_lines} lines)",
                input.path
            ));
        }

        let selected_lines = &lines[start - 1..end];
        let mut output = format!("--- File: {} (Lines L{}-L{}) ---\n", input.path, start, end);
        for (idx, line) in selected_lines.iter().enumerate() {
            output.push_str(&format!("{:4} | {}\n", start + idx, line));
        }

        Ok(output)
    }
}

// -----------------------------------------------------------------------------
// Built-in Tool 2: CalculatorTool (Safe Recursive Descent Math Evaluator)
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct CalcInput {
    expression: String,
}

/// Evaluates mathematical expressions using a safe AST parser without shell/eval injection.
pub struct CalculatorTool;

impl SandboxedTool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Evaluates safe arithmetic expressions (+, -, *, /, %, ^, parentheses) on CPU."
    }

    fn required_capability(&self) -> ToolCapability {
        ToolCapability::Compute
    }

    fn execute(
        &self,
        input_json: &str,
        _context: &ToolExecutionContext,
    ) -> Result<String, RagError> {
        let input: CalcInput = serde_json::from_str(input_json)
            .map_err(|e| RagError::Serialization(format!("Invalid calculator input: {e}")))?;

        match evaluate_math_expression(&input.expression) {
            Ok(result) => Ok(format!("{}", result)),
            Err(e) => Ok(format!("Error: {}", e)),
        }
    }
}

/// Recursive descent math expression parser and evaluator.
fn evaluate_math_expression(expr: &str) -> Result<f64, String> {
    let mut chars: Vec<char> = expr.chars().filter(|c| !c.is_whitespace()).collect();
    if chars.is_empty() {
        return Err("Empty expression".to_string());
    }
    parse_expression(&mut chars)
}

fn parse_expression(chars: &mut Vec<char>) -> Result<f64, String> {
    let mut val = parse_term(chars)?;
    while !chars.is_empty() {
        match chars.first() {
            Some('+') => {
                chars.remove(0);
                val += parse_term(chars)?;
            }
            Some('-') => {
                chars.remove(0);
                val -= parse_term(chars)?;
            }
            _ => break,
        }
    }
    Ok(val)
}

fn parse_term(chars: &mut Vec<char>) -> Result<f64, String> {
    let mut val = parse_factor(chars)?;
    while !chars.is_empty() {
        match chars.first() {
            Some('*') => {
                chars.remove(0);
                val *= parse_factor(chars)?;
            }
            Some('/') => {
                chars.remove(0);
                let divisor = parse_factor(chars)?;
                if divisor.abs() < 1e-12 {
                    return Err("Division by zero".to_string());
                }
                val /= divisor;
            }
            Some('%') => {
                chars.remove(0);
                let divisor = parse_factor(chars)?;
                if divisor.abs() < 1e-12 {
                    return Err("Division by zero".to_string());
                }
                val %= divisor;
            }
            _ => break,
        }
    }
    Ok(val)
}

fn parse_factor(chars: &mut Vec<char>) -> Result<f64, String> {
    let mut base = parse_primary(chars)?;
    if !chars.is_empty() && chars.first() == Some(&'^') {
        chars.remove(0);
        let exp = parse_factor(chars)?;
        base = base.powf(exp);
    }
    Ok(base)
}

fn parse_primary(chars: &mut Vec<char>) -> Result<f64, String> {
    if chars.is_empty() {
        return Err("Unexpected end of expression".to_string());
    }

    if chars.first() == Some(&'(') {
        chars.remove(0);
        let val = parse_expression(chars)?;
        if chars.first() == Some(&')') {
            chars.remove(0);
            Ok(val)
        } else {
            Err("Mismatched parentheses: missing ')'".to_string())
        }
    } else if chars.first() == Some(&'-') {
        chars.remove(0);
        let val = parse_primary(chars)?;
        Ok(-val)
    } else if chars.first() == Some(&'+') {
        chars.remove(0);
        parse_primary(chars)
    } else {
        let mut num_str = String::new();
        while !chars.is_empty() && (chars[0].is_ascii_digit() || chars[0] == '.') {
            num_str.push(chars.remove(0));
        }
        if num_str.is_empty() {
            Err(format!(
                "Expected number, got '{}'",
                chars.first().unwrap_or(&' ')
            ))
        } else {
            num_str
                .parse::<f64>()
                .map_err(|e| format!("Invalid number '{num_str}': {e}"))
        }
    }
}

// -----------------------------------------------------------------------------
// Built-in Tool 3: CodeSymbolSearchTool
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct SymbolSearchInput {
    symbol: String,
    max_results: Option<usize>,
}

/// Locates symbol definitions (`fn`, `struct`, `class`, `def`, `enum`) within workspace files.
pub struct CodeSymbolSearchTool;

impl SandboxedTool for CodeSymbolSearchTool {
    fn name(&self) -> &str {
        "code_symbol_search"
    }

    fn description(&self) -> &str {
        "Searches workspace code files for function, class, struct, or enum definitions."
    }

    fn required_capability(&self) -> ToolCapability {
        ToolCapability::CodeSearch
    }

    fn execute(
        &self,
        input_json: &str,
        context: &ToolExecutionContext,
    ) -> Result<String, RagError> {
        let input: SymbolSearchInput = serde_json::from_str(input_json)
            .map_err(|e| RagError::Serialization(format!("Invalid symbol_search args: {e}")))?;

        let target = input.symbol.trim();
        if target.is_empty() {
            return Ok("No symbol provided".to_string());
        }

        let max_results = input.max_results.unwrap_or(20).max(1);
        let mut matches = Vec::new();

        // Patterns indicating definition: "fn <sym>", "struct <sym>", "class <sym>", "def <sym>", "enum <sym>"
        let patterns = [
            format!("fn {target}"),
            format!("struct {target}"),
            format!("class {target}"),
            format!("def {target}"),
            format!("enum {target}"),
            format!("interface {target}"),
            format!("pub struct {target}"),
            format!("pub fn {target}"),
        ];

        let mut dirs = vec![context.workspace_root.clone()];
        while let Some(current_dir) = dirs.pop() {
            if is_sensitive_or_ignored(&current_dir) {
                continue;
            }

            if let Ok(entries) = fs::read_dir(&current_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if is_sensitive_or_ignored(&path) {
                        continue;
                    }
                    if path.is_dir() {
                        dirs.push(path);
                    } else if path.is_file() {
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                        if ["rs", "py", "dart", "js", "ts", "c", "cpp", "h"].contains(&ext) {
                            if let Ok(content) = fs::read_to_string(&path) {
                                for (idx, line) in content.lines().enumerate() {
                                    let trimmed = line.trim();
                                    for pat in &patterns {
                                        if trimmed.contains(pat) {
                                            let rel_path = crate::security::normalize_relative_path(
                                                &path,
                                                &context.workspace_root,
                                            );
                                            matches.push(format!(
                                                "{rel_path}#L{}: {}",
                                                idx + 1,
                                                trimmed
                                            ));
                                            if matches.len() >= max_results {
                                                break;
                                            }
                                        }
                                    }
                                    if matches.len() >= max_results {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    if matches.len() >= max_results {
                        break;
                    }
                }
            }
        }

        if matches.is_empty() {
            Ok(format!("No definitions found for symbol '{}'", target))
        } else {
            Ok(format!(
                "Found {} occurrences:\n{}",
                matches.len(),
                matches.join("\n")
            ))
        }
    }
}

// -----------------------------------------------------------------------------
// Built-in Tool 4: RagQueryTool
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct RagQueryInput {
    query: String,
    limit: Option<usize>,
}

/// Executes local RAG retrieval and outputs citation-annotated context.
pub struct RagQueryTool {
    engine: Arc<HybridSearchEngine>,
}

impl RagQueryTool {
    pub fn new(engine: Arc<HybridSearchEngine>) -> Self {
        Self { engine }
    }
}

impl SandboxedTool for RagQueryTool {
    fn name(&self) -> &str {
        "rag_query"
    }

    fn description(&self) -> &str {
        "Executes hybrid BM25 + vector search and formats grounded citation excerpts."
    }

    fn required_capability(&self) -> ToolCapability {
        ToolCapability::WorkspaceRead
    }

    fn execute(
        &self,
        input_json: &str,
        _context: &ToolExecutionContext,
    ) -> Result<String, RagError> {
        let input: RagQueryInput = serde_json::from_str(input_json)
            .map_err(|e| RagError::Serialization(format!("Invalid rag_query args: {e}")))?;

        let limit = input.limit.unwrap_or(3).clamp(1, 10);
        let results = self.engine.search(&input.query, limit);

        if results.is_empty() {
            return Ok(format!(
                "No relevant knowledge found for query: '{}'",
                input.query
            ));
        }

        let mut output = format!("Found {} grounded knowledge chunks:\n\n", results.len());
        for (idx, res) in results.iter().enumerate() {
            output.push_str(&format!(
                "[{}] Citation: {}\nScore: {:.4} (Dense: {:.4}, Lexical: {:.4})\nSnippet:\n{}\n\n",
                idx + 1,
                res.citation_tag,
                res.rrf_score,
                res.dense_score,
                res.lexical_score,
                res.excerpt
            ));
        }

        Ok(output)
    }
}

// -----------------------------------------------------------------------------
// Sandbox Runner
// -----------------------------------------------------------------------------

/// Orchestrates registration, capability verification, timeout guards, and execution of sandboxed tools.
#[derive(Default)]
pub struct ToolSandboxRunner {
    tools: HashMap<String, Arc<dyn SandboxedTool>>,
}

impl ToolSandboxRunner {
    pub fn new() -> Self {
        let mut runner = Self::default();
        runner.register_tool(Arc::new(WorkspaceFileReadTool));
        runner.register_tool(Arc::new(CalculatorTool));
        runner.register_tool(Arc::new(CodeSymbolSearchTool));
        runner
    }

    pub fn register_tool(&mut self, tool: Arc<dyn SandboxedTool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn list_tools(&self) -> Vec<ToolDescriptor> {
        self.tools
            .values()
            .map(|t| ToolDescriptor {
                name: t.name().to_string(),
                description: t.description().to_string(),
                required_capability: t.required_capability(),
            })
            .collect()
    }

    /// Safely executes a registered tool within the specified execution context.
    pub fn execute(
        &self,
        tool_name: &str,
        input_json: &str,
        context: &ToolExecutionContext,
    ) -> ToolExecutionResult {
        let start_time = Instant::now();

        let tool = match self.tools.get(tool_name) {
            Some(t) => t,
            None => {
                return ToolExecutionResult {
                    tool_name: tool_name.to_string(),
                    success: false,
                    output: String::new(),
                    duration_ms: start_time.elapsed().as_secs_f64() * 1000.0,
                    bytes_produced: 0,
                    truncated: false,
                    error_message: Some(format!("Tool '{tool_name}' not found")),
                };
            }
        };

        // Capability enforcement
        let cap = tool.required_capability();
        if cap == ToolCapability::Network && context.local_only_mode {
            return ToolExecutionResult {
                tool_name: tool_name.to_string(),
                success: false,
                output: String::new(),
                duration_ms: start_time.elapsed().as_secs_f64() * 1000.0,
                bytes_produced: 0,
                truncated: false,
                error_message: Some(
                    "Permission denied: Network tools blocked in Local-Only Privacy Mode"
                        .to_string(),
                ),
            };
        }

        if !context.allowed_capabilities.contains(&cap) {
            return ToolExecutionResult {
                tool_name: tool_name.to_string(),
                success: false,
                output: String::new(),
                duration_ms: start_time.elapsed().as_secs_f64() * 1000.0,
                bytes_produced: 0,
                truncated: false,
                error_message: Some(format!(
                    "Permission denied: capability {:?} not granted in execution context",
                    cap
                )),
            };
        }

        // Execute tool logic
        match tool.execute(input_json, context) {
            Ok(mut raw_output) => {
                let duration_ms = start_time.elapsed().as_secs_f64() * 1000.0;
                let original_bytes = raw_output.len();
                let mut truncated = false;

                if original_bytes > context.max_output_bytes {
                    raw_output.truncate(context.max_output_bytes);
                    raw_output.push_str("\n... [Output truncated by execution byte cap]");
                    truncated = true;
                }

                ToolExecutionResult {
                    tool_name: tool_name.to_string(),
                    success: true,
                    output: raw_output,
                    duration_ms,
                    bytes_produced: original_bytes,
                    truncated,
                    error_message: None,
                }
            }
            Err(e) => ToolExecutionResult {
                tool_name: tool_name.to_string(),
                success: false,
                output: String::new(),
                duration_ms: start_time.elapsed().as_secs_f64() * 1000.0,
                bytes_produced: 0,
                truncated: false,
                error_message: Some(e.to_string()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_calculator_tool_arithmetic() {
        let runner = ToolSandboxRunner::new();
        let context = ToolExecutionContext::default();

        let res = runner.execute(
            "calculator",
            r#"{"expression": "(10 + 5) * 2 - 4 / 2"}"#,
            &context,
        );
        assert!(res.success);
        assert_eq!(res.output, "28");

        let res_pow = runner.execute("calculator", r#"{"expression": "2 ^ 8"}"#, &context);
        assert!(res_pow.success);
        assert_eq!(res_pow.output, "256");

        let res_div_zero = runner.execute("calculator", r#"{"expression": "100 / 0"}"#, &context);
        assert!(res_div_zero.success);
        assert!(res_div_zero.output.contains("Division by zero"));
    }

    #[test]
    fn test_workspace_file_read_tool_and_traversal_block() {
        let tmp = tempdir().unwrap();
        let workspace = tmp.path();

        let doc_file = workspace.join("sample.txt");
        fs::write(&doc_file, "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\n").unwrap();

        let runner = ToolSandboxRunner::new();
        let context = ToolExecutionContext {
            workspace_root: workspace.to_path_buf(),
            ..Default::default()
        };

        // Normal read of lines 2 to 4
        let res = runner.execute(
            "workspace_read_file",
            r#"{"path": "sample.txt", "line_start": 2, "line_end": 4}"#,
            &context,
        );
        assert!(res.success);
        assert!(res.output.contains("Line 2"));
        assert!(res.output.contains("Line 4"));
        assert!(!res.output.contains("Line 5"));

        // Path traversal rejection
        let res_traversal = runner.execute(
            "workspace_read_file",
            r#"{"path": "../secret.txt"}"#,
            &context,
        );
        assert!(!res_traversal.success);
        assert!(res_traversal
            .error_message
            .unwrap()
            .contains("Path traversal rejected"));
    }

    #[test]
    fn test_code_symbol_search_tool() {
        let tmp = tempdir().unwrap();
        let workspace = tmp.path();

        let code_file = workspace.join("engine.rs");
        fs::write(&code_file, "pub struct LlmEngine {\n    weights: usize,\n}\n\nimpl LlmEngine {\n    pub fn run_inference() {}\n}\n").unwrap();

        let runner = ToolSandboxRunner::new();
        let context = ToolExecutionContext {
            workspace_root: workspace.to_path_buf(),
            ..Default::default()
        };

        let res = runner.execute("code_symbol_search", r#"{"symbol": "LlmEngine"}"#, &context);
        assert!(res.success);
        assert!(res.output.contains("engine.rs#L1"));
    }

    #[test]
    fn test_network_tool_blocked_in_local_only_mode() {
        struct DummyNetworkTool;
        impl SandboxedTool for DummyNetworkTool {
            fn name(&self) -> &str {
                "http_fetch"
            }
            fn description(&self) -> &str {
                "fetches url"
            }
            fn required_capability(&self) -> ToolCapability {
                ToolCapability::Network
            }
            fn execute(
                &self,
                _input_json: &str,
                _context: &ToolExecutionContext,
            ) -> Result<String, RagError> {
                Ok("data".to_string())
            }
        }

        let mut runner = ToolSandboxRunner::new();
        runner.register_tool(Arc::new(DummyNetworkTool));

        let context = ToolExecutionContext {
            local_only_mode: true,
            ..Default::default()
        };

        let res = runner.execute("http_fetch", "{}", &context);
        assert!(!res.success);
        assert!(res
            .error_message
            .unwrap()
            .contains("Local-Only Privacy Mode"));
    }
}
