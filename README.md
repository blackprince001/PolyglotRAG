# **PolyglotRAG - A Multi-Source Personal Knowledge Retrieval Engine**

PolyglotRAG is a high-performance, locally-run Retrieval-Augmented Generation (RAG) engine designed to unify and make searchable a wide array of personal knowledge sources. It ingests information from disparate formats (websites, videos, PDFs), processes them into a standardized format, and creates a semantically searchable vector database. The system is built in Rust for efficiency, scalability, and extensibility to serve as the core infrastructure for future AI-powered applications.

#### **Architectural Overview**

The system follows a modular, pipeline-based architecture centered around an event scheduler, allowing for easy integration of new data sources ("pipes") and processing logic.

**High-Level System Architecture:**

This diagram shows the main components and how data flows between them.

```mermaid
flowchart TD
    A[Event Scheduler<br>Main Controller]

    subgraph IngestPipes [Ingestion Pipes]
        B[Web/HTML Pipe<br>HTML -> Markdown]
        C[YouTube Pipe<br>Video -> Transcript]
        D[PDF Pipe<br>PDF -> JSON Text]
        E[Audio Pipe<br>Whisper STT -> Text]
    end

    F[Shared Memory<br>File Storage]

    G[File Processor & Embedder<br>Chunks text & calls Embedding API]

    H[Vector Database<br>PGVector]

    I[Query API<br>REST Endpoint]

    J[Chatbot/Search UI<br>Client Application]

    A -- orchestrates --> IngestPipes
    IngestPipes -- writes extracted text to --> F
    A -- triggers --> G
    G -- reads from --> F
    G -- stores vectors in --> H
    I -- queries --> H
    J -- sends search request --> I
    I -- returns results --> J
```

**Detailed Data Flow Sequence:**

This sequence diagram details the step-by-step process for ingesting and querying data.

```mermaid
sequenceDiagram
    actor User
    participant Scheduler
    participant Pipe as HTML/YT/PDF Pipe
    participant Memory as Shared Memory
    participant Processor
    participant EmbedAPI as Embedding API
    participant DB as PGVector DB
    participant QueryAPI
    participant Client

    Note over User, Client: INGESTION PROCESS
    User->>Scheduler: Trigger pipe with source (e.g., URL)
    Scheduler->>Pipe: Process source
    Pipe->>Memory: Extract & save content as text file
    Scheduler->>Processor: New file ready
    Processor->>Memory: Read text file
    Processor->>Processor: Chunk text
    loop Embed Batches
        Processor->>EmbedAPI: Send text batch
        EmbedAPI-->>Processor: Return vector embeddings
    end
    Processor->>DB: Store chunks & vectors

    Note over User, Client: QUERY PROCESS
    Client->>QueryAPI: Send search query
    QueryAPI->>EmbedAPI: Embed query
    EmbedAPI-->>QueryAPI: Return query vector
    QueryAPI->>DB: Perform similarity search (L2/Cosine)
    DB-->>QueryAPI: Return top K results
    QueryAPI-->>Client: Return relevant text passages
```

#### **Key Components & Technologies**

*   **Language:** Rust (for performance and memory safety)
*   **Data Sources:** HTML (via `html2md`), YouTube Transcripts, PDFs (text extraction), Audio (OpenAI Whisper)
*   **Concurrency:** Multi-threading, System Channels, and Shared Memory for inter-process communication.
*   **Database:** PostgreSQL with PGVector extension for efficient vector similarity search.
*   **Embeddings:** External API calls to state-of-the-art language models (e.g., OpenAI, Hugging Face).
*   **API:** RESTful API (built with Axum/Actix-web) to handle search queries.

#### **Purpose & Value**

This project solves the "fragmented knowledge" problem. Instead of having information siloed in browser bookmarks, YouTube history, and folders of PDFs, PolyglotRAG creates a unified, semantic search index across all of it. This allows for powerful queries like "find me concepts related to neural attention mechanisms from all my saved articles, videos, and textbooks."
