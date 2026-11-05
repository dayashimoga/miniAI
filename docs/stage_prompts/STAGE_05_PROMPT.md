# Stage 5 Technical Execution Prompt: RAG, Document/Code Indexing & Secure Tool Integration

## 1. Objective & Dependencies
- **Objective:** Deliver local Retrieval-Augmented Generation (RAG), syntax-aware document/code chunking, hybrid vector + BM25 search with exact file/line citations, and capability-sandboxed local tool execution.
- **Dependencies:** Stage 4 Gate Verified.
- **Active Skills:** `miniai-rust-core`, `miniai-python-ml`, `miniai-offline-first`, `miniai-security-privacy`, `miniai-flutter-ui`, `miniai-testing-qa`.

## 2. Phases & Subphases
- **Phase 5.1: Document & Code Ingestion Pipeline (`crates/miniai-rag`)**
  - *Subphase 5.1.1:* Incremental ingestion of Markdown, plain text, PDF text, and source code with content-addressed SHA-256 chunk deduplication.
- **Phase 5.2: Hybrid Retrieval & Verifiable Citations**
  - *Subphase 5.2.1:* Local CPU-first embedding adapter + BM25 lexical index with Reciprocal Rank Fusion (RRF) and exact `file_path#Lstart-Lend` citation spans.
- **Phase 5.3: Sandboxed Local Tool Runner & Knowledge UI**
  - *Subphase 5.3.1:* Explicit capability allowlist for local tools (workspace-scoped read, calculator, deterministic code search) with execution timeouts and zero network access in `LocalOnlyPrivacyMode`.
  - *Subphase 5.3.2:* Flutter Knowledge Base index manager and interactive citation inspector in Chat UI.

## 3. Security & Edge Cases
- Block symlink escapes, `..` traversal, and binary/secret file indexing (`.env`, `.pem`, `.key`).
- Enforce strict tool execution timeouts and output byte caps.

## 4. Acceptance Tests & Exit Criteria
- End-to-end offline RAG query returns accurate grounded citations.
- Unauthorized tool paths and network calls are deterministically blocked.
- >= 90% coverage on `miniai-rag` and tool sandbox logic.

