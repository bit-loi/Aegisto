//! Multi-agent orchestration: Explorer, Systemizer and Verifier agents that
//! reason over a static-analysis snapshot through an [`LlmProvider`].

use std::fmt::Write as _;

use anyhow::Result;
use async_trait::async_trait;

use crate::llm::{ChatMessage, ChatRequest, LlmProvider};

/// The three agent roles from the Aegisto pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentKind {
    Explorer,
    Systemizer,
    Verifier,
}

impl AgentKind {
    pub fn label(&self) -> &'static str {
        match self {
            AgentKind::Explorer => "Explorer",
            AgentKind::Systemizer => "Systemizer",
            AgentKind::Verifier => "Verifier",
        }
    }
}

/// Static-analysis snapshot handed to the agents. Populated by `aegisto-core`
/// once the TUI wires the agent layer into `:analyze`.
#[derive(Debug, Clone, Default)]
pub struct AgentInput {
    pub file_path: String,
    pub format: String,
    pub entry_point: u64,
    pub sections: Vec<String>,
    pub imports: Vec<String>,
    pub strings: Vec<String>,
    pub disassembly: Vec<String>,
}

impl AgentInput {
    /// Render the snapshot as a compact, prompt-friendly summary.
    pub fn summary(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "file: {}", self.file_path);
        let _ = writeln!(out, "format: {}", self.format);
        let _ = writeln!(out, "entry point: {:#x}", self.entry_point);

        if !self.sections.is_empty() {
            let _ = writeln!(out, "sections:");
            for line in &self.sections {
                let _ = writeln!(out, "  {line}");
            }
        }
        if !self.imports.is_empty() {
            let _ = writeln!(out, "imports:");
            for line in &self.imports {
                let _ = writeln!(out, "  {line}");
            }
        }
        if !self.strings.is_empty() {
            let _ = writeln!(out, "strings:");
            for line in self.strings.iter().take(30) {
                let _ = writeln!(out, "  {line}");
            }
            if self.strings.len() > 30 {
                let _ = writeln!(out, "  … {} more", self.strings.len() - 30);
            }
        }
        if !self.disassembly.is_empty() {
            let _ = writeln!(out, "disassembly:");
            for line in self.disassembly.iter().take(50) {
                let _ = writeln!(out, "  {line}");
            }
            if self.disassembly.len() > 50 {
                let _ = writeln!(out, "  … {} more", self.disassembly.len() - 50);
            }
        }
        out
    }
}

/// A single agent that reasons over an [`AgentInput`] via an [`LlmProvider`].
#[async_trait]
pub trait Agent: Send + Sync {
    fn kind(&self) -> AgentKind;

    /// Run the agent and return its raw text output.
    async fn run(&self, input: &AgentInput) -> Result<String>;
}

fn system_prompt(kind: AgentKind) -> &'static str {
    match kind {
        AgentKind::Explorer => {
            "You are the Explorer agent of a binary analysis framework. \
            Scan the binary's functions, API calls and strings; flag notable patterns such as \
            encryption routines, packer signatures and suspicious imports. \
            Report findings as concise bullet points with evidence."
        }
        AgentKind::Systemizer => {
            "You are the Systemizer agent of a binary analysis framework. \
            Structure the execution path: build a call graph, map data flow and summarize what \
            the binary does. Report a structured behavior summary."
        }
        AgentKind::Verifier => {
            "You are the Verifier agent of a binary analysis framework. \
            Check every hypothesis against the evidence (sections, imports, strings, \
            disassembly). Reject false positives, validate claims and report a confidence \
            level for each finding."
        }
    }
}

/// Explorer agent: flags notable functions and patterns.
#[derive(Debug)]
pub struct ExplorerAgent {
    pub llm: Box<dyn LlmProvider>,
}

impl ExplorerAgent {
    pub fn new(llm: Box<dyn LlmProvider>) -> Self {
        Self { llm }
    }
}

/// Systemizer agent: structures call graph and data flow.
#[derive(Debug)]
pub struct SystemizerAgent {
    pub llm: Box<dyn LlmProvider>,
}

impl SystemizerAgent {
    pub fn new(llm: Box<dyn LlmProvider>) -> Self {
        Self { llm }
    }
}

/// Verifier agent: validates hypotheses against evidence.
#[derive(Debug)]
pub struct VerifierAgent {
    pub llm: Box<dyn LlmProvider>,
}

impl VerifierAgent {
    pub fn new(llm: Box<dyn LlmProvider>) -> Self {
        Self { llm }
    }
}

macro_rules! impl_agent {
    ($ty:ty, $kind:expr) => {
        #[async_trait]
        impl Agent for $ty {
            fn kind(&self) -> AgentKind {
                $kind
            }

            async fn run(&self, input: &AgentInput) -> Result<String> {
                let request = ChatRequest::new(
                    self.llm.model().to_string(),
                    vec![
                        ChatMessage::system(system_prompt($kind)),
                        ChatMessage::user(format!(
                            "Static analysis snapshot:\n{}",
                            input.summary()
                        )),
                    ],
                );
                Ok(self.llm.chat(request).await?.content)
            }
        }
    };
}

impl_agent!(ExplorerAgent, AgentKind::Explorer);
impl_agent!(SystemizerAgent, AgentKind::Systemizer);
impl_agent!(VerifierAgent, AgentKind::Verifier);

// ---------------------------------------------------------------------------
// Tests (mock provider, no network)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{ChatResponse, LlmProvider};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct MockProvider {
        reply: String,
        received: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl LlmProvider for MockProvider {
        fn id(&self) -> &'static str {
            "mock"
        }

        fn model(&self) -> &str {
            "mock-model"
        }

        async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
            let transcript = request
                .messages
                .iter()
                .map(|m| format!("{}: {}", m.role.as_str(), m.content))
                .collect::<Vec<_>>()
                .join("\n");
            self.received.lock().unwrap().push(transcript);
            Ok(ChatResponse {
                content: self.reply.clone(),
                model: request.model,
                usage: None,
            })
        }
    }

    fn sample_input() -> AgentInput {
        AgentInput {
            file_path: "sample.exe".to_string(),
            format: "PE".to_string(),
            entry_point: 0x401000,
            sections: vec![".text size=0x1000 vaddr=0x401000 [RAX]".to_string()],
            imports: vec!["VirtualAllocEx (KERNEL32.dll)".to_string()],
            strings: vec!["secret_key".to_string()],
            disassembly: vec!["0x401000: push rbp".to_string()],
        }
    }

    #[tokio::test]
    async fn explorer_agent_sends_snapshot() {
        let received = Arc::new(Mutex::new(Vec::new()));
        let llm = Box::new(MockProvider {
            reply: "findings".to_string(),
            received: received.clone(),
        });
        let agent = ExplorerAgent::new(llm);
        let out = agent.run(&sample_input()).await.unwrap();
        assert_eq!(out, "findings");

        let transcript = &received.lock().unwrap()[0];
        assert!(transcript.contains("Explorer"));
        assert!(transcript.contains("sample.exe"));
        assert!(transcript.contains("VirtualAllocEx"));
        assert!(transcript.contains("0x401000: push rbp"));
    }

    #[tokio::test]
    async fn verifier_agent_uses_verifier_prompt() {
        let received = Arc::new(Mutex::new(Vec::new()));
        let llm = Box::new(MockProvider {
            reply: "verified".to_string(),
            received: received.clone(),
        });
        let agent = VerifierAgent::new(llm);
        let _ = agent.run(&sample_input()).await.unwrap();
        let transcript = &received.lock().unwrap()[0];
        assert!(transcript.contains("Verifier agent"));
        assert!(transcript.contains("confidence"));
    }

    #[test]
    fn summary_renders_every_section() {
        let s = sample_input().summary();
        assert!(s.contains("file: sample.exe"));
        assert!(s.contains("format: PE"));
        assert!(s.contains("entry point: 0x401000"));
        assert!(s.contains("sections:"));
        assert!(s.contains("imports:"));
        assert!(s.contains("strings:"));
        assert!(s.contains("disassembly:"));
    }
}
