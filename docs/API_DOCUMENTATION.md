# RAG Engine API Documentation

## Overview

The RAG Engine provides a comprehensive RESTful API for document processing, semantic search, and content management. Built with Rust and Axum, it offers high-performance document extraction, embedding generation, and vector search capabilities.

**Base URL:** `http://localhost:3000`  
**Version:** 0.1.0  
**Content-Type:** `application/json`

---

## Table of Contents

1. [Health & System](#health--system)
2. [File Management](#file-management)
3. [Document Processing](#document-processing)
4. [Job Management](#job-management)
5. [Search](#search)
6. [Chunk Management](#chunk-management)
7. [Embedding Management](#embedding-management)
8. [Data Models](#data-models)
9. [Error Handling](#error-handling)

---

## Health & System

### Get API Root

Provides basic information about the API.

```http
GET /
```

**Response:**

```json
{
  "success": true,
  "data": "RAG Engine API - Clean Architecture",
  "error": null
}
```

### Health Check

Check if the API is healthy and responsive.

```http
GET /health
```

**Response:**

```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "version": "0.1.0",
    "uptime": "N/A"
  },
  "error": null
}
```

---

## File Management

### Upload File

Upload a document for processing. Supports PDF, TXT, DOCX, and other document formats.

```http
POST /upload
Content-Type: multipart/form-data
```

**Request:**

- **Body:** Form data with file attachment
- **Maximum file size:** 250MB

**Response:**

```json
{
  "success": true,
  "data": {
    "file_id": "123e4567-e89b-12d3-a456-426614174000",
    "file_name": "document.pdf",
    "file_size": 1024000,
    "file_hash": "sha256:abc123...",
    "content_type": "application/pdf",
    "message": "File uploaded successfully"
  },
  "error": null
}
```

### Upload and Process File

Upload a file and automatically queue it for processing.

```http
POST /upload-and-process
Content-Type: multipart/form-data
```

**Request:**

- **Body:** Form data with file attachment and optional parameters
- **Parameters:**
  - `file` (required): The file to upload
  - `auto_process` (optional): Whether to automatically process the file (default: true)
- **Maximum file size:** 250MB

**Response:**

```json
{
  "success": true,
  "data": {
    "file_id": "123e4567-e89b-12d3-a456-426614174000",
    "job_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "file_name": "document.pdf",
    "file_size": 1024000,
    "file_hash": "sha256:abc123...",
    "content_type": "application/pdf",
    "status": "processing",
    "message": "File uploaded and processing started successfully",
    "progress_stream_url": "/jobs/a1b2c3d4-e5f6-7890-abcd-ef1234567890/stream"
  },
  "error": null,
  "timestamp": "2025-01-02T12:00:00Z"
}
```

### List Files

Retrieve a paginated list of all uploaded files.

```http
GET /files?skip=0&limit=20
```

**Query Parameters:**

- `skip` (integer, optional): Number of records to skip (default: 0)
- `limit` (integer, optional): Maximum number of records to return (default: 20)

**Response:**

```json
{
  "success": true,
  "data": {
    "files": [
      {
        "id": "123e4567-e89b-12d3-a456-426614174000",
        "file_name": "document.pdf",
        "file_type": "application/pdf",
        "file_size": 1024000,
        "file_hash": "sha256:abc123...",
        "created_at": "2023-10-01T12:00:00Z",
        "updated_at": "2023-10-01T12:00:00Z",
        "processing_status": "completed"
      }
    ],
    "meta": {
      "offset": 0,
      "limit": 20,
      "total": 1
    }
  },
  "error": null
}
```

### Get File Details

Retrieve detailed information about a specific file.

```http
GET /files/{file_id}
```

**Path Parameters:**

- `file_id` (UUID): The unique identifier of the file

**Response:**

```json
{
  "success": true,
  "data": {
    "file_id": "123e4567-e89b-12d3-a456-426614174000",
    "file_name": "document.pdf",
    "file_size": 1024000,
    "file_hash": "sha256:abc123...",
    "content_type": "application/pdf",
    "processing_status": "completed",
    "created_at": "2023-10-01T12:00:00Z",
    "updated_at": "2023-10-01T12:00:00Z",
    "metadata": {
      "pages": 10,
      "word_count": 5000
    }
  },
  "error": null
}
```

### Update File

Update file metadata.

```http
PUT /files/{file_id}
Content-Type: application/json
```

**Path Parameters:**

- `file_id` (UUID): The unique identifier of the file

**Request Body:**

```json
{
  "file_name": "new_name.pdf",
  "content_type": "application/pdf"
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "file_id": "123e4567-e89b-12d3-a456-426614174000",
    "file_name": "new_name.pdf",
    "file_size": 1024000,
    "file_hash": "sha256:abc123...",
    "content_type": "application/pdf",
    "created_at": "2023-10-01T12:00:00Z",
    "updated_at": "2023-10-01T12:05:00Z",
    "processing_status": "completed"
  },
  "error": null,
  "timestamp": "2025-01-02T12:00:00Z"
}
```

### Delete File

Delete a file and all associated data.

```http
DELETE /files/{file_id}
```

**Path Parameters:**

- `file_id` (UUID): The unique identifier of the file

**Response:**

```json
{
  "success": true,
  "data": "File deleted successfully",
  "error": null,
  "timestamp": "2025-01-02T12:00:00Z"
}
```

### Get File Count

Get the total number of files in the system.

```http
GET /filesys/count
```

**Response:**

```json
{
  "success": true,
  "data": {
    "count": 150
  },
  "error": null,
  "timestamp": "2025-01-02T12:00:00Z"
}
```

### Get File Chunks

Retrieve text chunks for a specific file.

```http
GET /files/{file_id}/chunks?skip=0&limit=20
```

**Path Parameters:**

- `file_id` (UUID): The unique identifier of the file

**Query Parameters:**

- `skip` (integer, optional): Number of chunks to skip (default: 0)
- `limit` (integer, optional): Maximum number of chunks to return (default: 20)

**Response:**

```json
{
  "success": true,
  "data": {
    "file_id": "123e4567-e89b-12d3-a456-426614174000",
    "chunks": [
      {
        "chunk_id": "456e7890-e89b-12d3-a456-426614174001",
        "file_id": "123e4567-e89b-12d3-a456-426614174000",
        "chunk_text": "This is the content of the first chunk...",
        "chunk_index": 0,
        "word_count": 150,
        "page_number": 1,
        "section_path": "Introduction",
        "created_at": "2023-10-01T12:00:00Z"
      }
    ],
    "meta": {
      "offset": 0,
      "limit": 20,
      "total": 45
    }
  },
  "error": null
}
```

### Process File (Legacy)

Directly process a file to extract text and generate embeddings.

```http
POST /process/{file_id}
```

**Path Parameters:**

- `file_id` (UUID): The unique identifier of the file to process

**Response:**

```json
{
  "success": true,
  "data": {
    "file_id": "123e4567-e89b-12d3-a456-426614174000",
    "chunks_created": 45,
    "embeddings_created": 45,
    "processing_time_ms": 5000,
    "message": "File processed successfully"
  },
  "error": null
}
```

---

## Chunk Management

### Get Chunk

Get a specific chunk by ID.

```http
GET /chunks/{chunk_id}
```

**Path Parameters:**

- `chunk_id` (UUID): The unique identifier of the chunk

**Response:**

```json
{
  "success": true,
  "data": {
    "file_id": "123e4567-e89b-12d3-a456-426614174000",
    "chunks": [
      {
        "id": "chunk_uuid",
        "file_id": "123e4567-e89b-12d3-a456-426614174000",
        "chunk_text": "This is the content of the chunk...",
        "chunk_index": 0,
        "token_count": 150,
        "page_number": 1,
        "section_path": "Introduction",
        "created_at": "2025-01-02T12:00:00Z"
      }
    ],
    "total_chunks": 1,
    "skip": 0,
    "limit": 1
  },
  "error": null,
  "timestamp": "2025-01-02T12:00:00Z"
}
```

### Get File Chunks (Direct)

Get all chunks for a file with pagination.

```http
GET /chunks/file/{file_id}?skip=0&limit=50
```

**Path Parameters:**

- `file_id` (UUID): The unique identifier of the file

**Query Parameters:**

- `skip` (integer, optional): Number of chunks to skip (default: 0)
- `limit` (integer, optional): Maximum number of chunks to return (default: 50)

**Response:**

```json
{
  "success": true,
  "data": {
    "file_id": "123e4567-e89b-12d3-a456-426614174000",
    "chunks": [
      {
        "id": "chunk_uuid",
        "file_id": "123e4567-e89b-12d3-a456-426614174000",
        "chunk_text": "This is the content of the chunk...",
        "chunk_index": 0,
        "token_count": 150,
        "page_number": 1,
        "section_path": "Introduction",
        "created_at": "2025-01-02T12:00:00Z"
      }
    ],
    "total_chunks": 25,
    "skip": 0,
    "limit": 50
  },
  "error": null,
  "timestamp": "2025-01-02T12:00:00Z"
}
```

### Get Chunk Count

Get the number of chunks for a specific file.

```http
GET /chunks/file/{file_id}/count
```

**Path Parameters:**

- `file_id` (UUID): The unique identifier of the file

**Response:**

```json
{
  "success": true,
  "data": {
    "file_id": "123e4567-e89b-12d3-a456-426614174000",
    "chunk_count": 25
  },
  "error": null,
  "timestamp": "2025-01-02T12:00:00Z"
}
```

### Delete Chunk

Delete a specific chunk.

```http
DELETE /chunks/{chunk_id}
```

**Path Parameters:**

- `chunk_id` (UUID): The unique identifier of the chunk

**Response:**

```json
{
  "success": true,
  "data": "Chunk deleted successfully",
  "error": null,
  "timestamp": "2025-01-02T12:00:00Z"
}
```

### Delete File Chunks

Delete all chunks for a specific file.

```http
DELETE /chunks/file/{file_id}
```

**Path Parameters:**

- `file_id` (UUID): The unique identifier of the file

**Response:**

```json
{
  "success": true,
  "data": {
    "file_id": "123e4567-e89b-12d3-a456-426614174000",
    "deleted_chunks": 25
  },
  "error": null,
  "timestamp": "2025-01-02T12:00:00Z"
}
```

---

## Embedding Management

### Get Embedding

Get a specific embedding by ID.

```http
GET /embeddings/{embedding_id}
```

**Path Parameters:**

- `embedding_id` (UUID): The unique identifier of the embedding

**Response:**

```json
{
  "success": true,
  "data": {
    "id": "embedding_uuid",
    "chunk_id": "chunk_uuid",
    "model_name": "text-embedding-ada-002",
    "model_version": "v1.0",
    "vector_dimension": 1536,
    "created_at": "2025-01-02T12:00:00Z"
  },
  "error": null,
  "timestamp": "2025-01-02T12:00:00Z"
}
```

### Get Chunk Embedding

Get embedding for a specific chunk.

```http
GET /embeddings/chunk/{chunk_id}
```

**Path Parameters:**

- `chunk_id` (UUID): The unique identifier of the chunk

**Response:**

```json
{
  "success": true,
  "data": {
    "id": "embedding_uuid",
    "chunk_id": "chunk_uuid",
    "model_name": "text-embedding-ada-002",
    "model_version": "v1.0",
    "vector_dimension": 1536,
    "created_at": "2025-01-02T12:00:00Z"
  },
  "error": null,
  "timestamp": "2025-01-02T12:00:00Z"
}
```

### Get File Embeddings

Get all embeddings for a specific file.

```http
GET /embeddings/file/{file_id}
```

**Path Parameters:**

- `file_id` (UUID): The unique identifier of the file

**Response:**

```json
{
  "success": true,
  "data": {
    "file_id": "123e4567-e89b-12d3-a456-426614174000",
    "embeddings": [
      {
        "id": "embedding_uuid",
        "chunk_id": "chunk_uuid",
        "model_name": "text-embedding-ada-002",
        "model_version": "v1.0",
        "vector_dimension": 1536,
        "created_at": "2025-01-02T12:00:00Z"
      }
    ],
    "count": 25
  },
  "error": null,
  "timestamp": "2025-01-02T12:00:00Z"
}
```

### Semantic Search with Embeddings

Perform semantic similarity search using embeddings.

```http
POST /embeddings/search
Content-Type: application/json
```

**Request Body:**

```json
{
  "query_vector": [0.1, 0.2, 0.3, ...],
  "limit": 10,
  "similarity_threshold": 0.8,
  "file_id": "optional_file_uuid"
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "results": [
      {
        "similarity_score": 0.95,
        "chunk_id": "chunk_uuid",
        "file_id": "123e4567-e89b-12d3-a456-426614174000"
      }
    ],
    "total_results": 1
  },
  "error": null,
  "timestamp": "2025-01-02T12:00:00Z"
}
```

### Delete Embedding

Delete a specific embedding.

```http
DELETE /embeddings/{embedding_id}
```

**Path Parameters:**

- `embedding_id` (UUID): The unique identifier of the embedding

**Response:**

```json
{
  "success": true,
  "data": "Embedding deleted successfully",
  "error": null,
  "timestamp": "2025-01-02T12:00:00Z"
}
```

### Delete Chunk Embeddings

Delete embeddings for a specific chunk.

```http
DELETE /embeddings/chunk/{chunk_id}
```

**Path Parameters:**

- `chunk_id` (UUID): The unique identifier of the chunk

**Response:**

```json
{
  "success": true,
  "data": "Embeddings deleted successfully",
  "error": null,
  "timestamp": "2025-01-02T12:00:00Z"
}
```

### Delete File Embeddings

Delete all embeddings for a specific file.

```http
DELETE /embeddings/file/{file_id}
```

**Path Parameters:**

- `file_id` (UUID): The unique identifier of the file

**Response:**

```json
{
  "success": true,
  "data": {
    "file_id": "123e4567-e89b-12d3-a456-426614174000",
    "deleted_embeddings": 25
  },
  "error": null,
  "timestamp": "2025-01-02T12:00:00Z"
}
```

### Get Embedding Count

Get the total number of embeddings in the system.

```http
GET /embeddings/count
```

**Response:**

```json
{
  "success": true,
  "data": {
    "count": 1000
  },
  "error": null,
  "timestamp": "2025-01-02T12:00:00Z"
}
```

### Get Model Embedding Count

Get the number of embeddings for a specific model.

```http
GET /embeddings/count/model/{model_name}
```

**Path Parameters:**

- `model_name` (string): Name of the embedding model

**Response:**

```json
{
  "success": true,
  "data": {
    "model_name": "text-embedding-ada-002",
    "count": 500
  },
  "error": null,
  "timestamp": "2025-01-02T12:00:00Z"
}
```

---

## Search

### Search Content

Perform semantic search across all processed documents.

```http
GET /search?query=machine learning&limit=10&similarity_threshold=0.7&file_id=123e4567-e89b-12d3-a456-426614174000
```

**Query Parameters:**

- `query` (string, required): The search query text
- `limit` (integer, optional): Maximum number of results to return (default: 10)
- `similarity_threshold` (float, optional): Minimum similarity score (0.0-1.0)
- `file_id` (UUID, optional): Limit search to a specific file

**Response:**

```json
{
  "success": true,
  "data": {
    "query": "machine learning",
    "results": [
      {
        "chunk_id": "456e7890-e89b-12d3-a456-426614174001",
        "file_id": "123e4567-e89b-12d3-a456-426614174000",
        "chunk_text": "Machine learning is a subset of artificial intelligence...",
        "similarity_score": 0.89,
        "chunk_index": 5,
        "page_number": 2,
        "section_path": "Chapter 2: ML Fundamentals"
      }
    ],
    "total_results": 1,
    "search_time_ms": 150
  },
  "error": null
}
```

---

## Job Management

### Queue File Processing Job

Create a background job to process an uploaded file.

```http
POST /jobs/process/file/{file_id}
```

**Path Parameters:**

- `file_id` (UUID): The unique identifier of the file to process

**Response:**

```json
{
  "success": true,
  "data": {
    "job_id": "789e0123-e89b-12d3-a456-426614174002",
    "file_id": "123e4567-e89b-12d3-a456-426614174000",
    "job_type": {
      "type_name": "file_processing",
      "url": null
    },
    "status": "queued",
    "message": "File processing job queued successfully"
  },
  "error": null
}
```

### Queue URL Extraction Job

Create a background job to extract content from a URL.

```http
POST /jobs/process/url/{file_id}
```

**Path Parameters:**

- `file_id` (UUID): The file ID to associate with the extracted content

**Request Body:**

```json
{
  "url": "https://example.com/article"
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "job_id": "789e0123-e89b-12d3-a456-426614174002",
    "file_id": "123e4567-e89b-12d3-a456-426614174000",
    "job_type": {
      "type_name": "url_extraction",
      "url": "https://example.com/article"
    },
    "status": "queued",
    "message": "URL extraction job queued successfully"
  },
  "error": null
}
```

### Queue YouTube Extraction Job

Create a background job to extract transcript from a YouTube video.

```http
POST /jobs/process/youtube/{file_id}
```

**Path Parameters:**

- `file_id` (UUID): The file ID to associate with the extracted transcript

**Request Body:**

```json
{
  "url": "https://youtube.com/watch?v=dQw4w9WgXcQ"
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "job_id": "789e0123-e89b-12d3-a456-426614174002",
    "file_id": "123e4567-e89b-12d3-a456-426614174000",
    "job_type": {
      "type_name": "youtube_extraction",
      "url": "https://youtube.com/watch?v=dQw4w9WgXcQ"
    },
    "status": "queued",
    "message": "YouTube extraction job queued successfully"
  },
  "error": null
}
```

### Get Job Status

Check the status of a specific job.

```http
GET /jobs/{job_id}
```

**Path Parameters:**

- `job_id` (UUID): The unique identifier of the job

**Response:**

```json
{
  "success": true,
  "data": {
    "job_id": "789e0123-e89b-12d3-a456-426614174002",
    "file_id": "123e4567-e89b-12d3-a456-426614174000",
    "job_type": {
      "type_name": "file_processing",
      "url": null
    },
    "status": "processing",
    "progress": 0.65,
    "created_at": "2023-10-01T12:00:00Z",
    "started_at": "2023-10-01T12:01:00Z",
    "completed_at": null,
    "error_message": null,
    "result_summary": null,
    "estimated_completion": "2023-10-01T12:05:00Z",
    "duration_ms": 60000,
    "is_terminal": false
  },
  "error": null
}
```

### Cancel Job

Cancel a running or queued job.

```http
DELETE /jobs/{job_id}/cancel
```

**Path Parameters:**

- `job_id` (UUID): The unique identifier of the job

**Response:**

```json
{
  "success": true,
  "data": {
    "job_id": "789e0123-e89b-12d3-a456-426614174002",
    "status": "cancelled",
    "message": "Job cancelled successfully"
  },
  "error": null
}
```

### Get File Jobs

Get all jobs associated with a specific file.

```http
GET /jobs/file/{file_id}
```

**Path Parameters:**

- `file_id` (UUID): The unique identifier of the file

**Response:**

```json
{
  "success": true,
  "data": [
    {
      "job_id": "789e0123-e89b-12d3-a456-426614174002",
      "file_id": "123e4567-e89b-12d3-a456-426614174000",
      "job_type": {
        "type_name": "file_processing",
        "url": null
      },
      "status": "completed",
      "progress": 1.0,
      "created_at": "2023-10-01T12:00:00Z",
      "started_at": "2023-10-01T12:01:00Z",
      "completed_at": "2023-10-01T12:05:00Z",
      "error_message": null,
      "result_summary": {
        "chunks_created": 45,
        "embeddings_created": 45,
        "processing_time_ms": 240000,
        "extracted_text_length": 15000
      },
      "estimated_completion": null,
      "duration_ms": 240000,
      "is_terminal": true
    }
  ],
  "error": null
}
```

### Get Active Jobs

Get all currently active (non-terminal) jobs.

```http
GET /jobs/active
```

**Response:**

```json
{
  "success": true,
  "data": [
    {
      "job_id": "789e0123-e89b-12d3-a456-426614174002",
      "file_id": "123e4567-e89b-12d3-a456-426614174000",
      "job_type": {
        "type_name": "file_processing",
        "url": null
      },
      "status": "processing",
      "progress": 0.65,
      "created_at": "2023-10-01T12:00:00Z",
      "started_at": "2023-10-01T12:01:00Z",
      "completed_at": null,
      "error_message": null,
      "result_summary": null,
      "estimated_completion": "2023-10-01T12:05:00Z",
      "duration_ms": 60000,
      "is_terminal": false
    }
  ],
  "error": null
}
```

## Real-time Job Updates

### Job Progress Stream (SSE)

Stream real-time updates for a specific job using Server-Sent Events.

```http
GET /jobs/{job_id}/stream
Accept: text/event-stream
```

**Path Parameters:**

- `job_id` (UUID): The unique identifier of the job

**Response Stream:**

```
data: {"job_id":"789e0123-e89b-12d3-a456-426614174002","status":"processing","progress":0.25}

data: {"job_id":"789e0123-e89b-12d3-a456-426614174002","status":"processing","progress":0.50}

data: {"job_id":"789e0123-e89b-12d3-a456-426614174002","status":"completed","progress":1.0,"result_summary":{"chunks_created":45,"embeddings_created":45,"processing_time_ms":240000,"extracted_text_length":15000}}
```

### Multiple Jobs Stream (SSE)

Stream real-time updates for all active jobs.

```http
GET /jobs/stream
Accept: text/event-stream
```

**Response Stream:**

```
data: {"job_id":"789e0123-e89b-12d3-a456-426614174002","status":"processing","progress":0.65}

data: {"job_id":"456e7890-e89b-12d3-a456-426614174001","status":"queued","progress":0.0}
```

---

## Data Models

### File Statuses

- `uploaded` - File uploaded but not processed
- `processing` - File is being processed
- `completed` - Processing completed successfully
- `failed` - Processing failed

### Job Statuses

- `queued` - Job is waiting to be processed
- `processing` - Job is currently being processed
- `completed` - Job completed successfully
- `failed` - Job failed with an error
- `cancelled` - Job was cancelled by user

### Job Types

- `file_processing` - Process an uploaded file
- `url_extraction` - Extract content from a URL
- `youtube_extraction` - Extract transcript from YouTube video

### Content Types Supported

- `application/pdf` - PDF documents
- `text/plain` - Plain text files
- `application/vnd.openxmlformats-officedocument.wordprocessingml.document` - DOCX files
- And other document formats

---

## Error Handling

All API endpoints follow a consistent error response format:

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable error message",
    "details": {
      "field": "Additional error details"
    }
  }
}
```

### Common Error Codes

| HTTP Status | Error Code | Description |
|-------------|------------|-------------|
| 400 | `EMPTY_QUERY` | Search query cannot be empty |
| 400 | `INVALID_REQUEST` | Request validation failed |
| 404 | `FILE_NOT_FOUND` | Requested file does not exist |
| 404 | `JOB_NOT_FOUND` | Requested job does not exist |
| 413 | `FILE_TOO_LARGE` | Uploaded file exceeds size limit |
| 422 | `PROCESSING_FAILED` | File processing failed |
| 429 | `RATE_LIMITED` | Too many requests |
| 500 | `INTERNAL_ERROR` | Internal server error |
| 500 | `SEARCH_FAILED` | Search operation failed |

### Example Error Response

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "FILE_NOT_FOUND",
    "message": "File with ID 123e4567-e89b-12d3-a456-426614174000 not found",
    "details": null
  }
}
```

---

## Rate Limiting

The API implements rate limiting to ensure fair usage:

- **General endpoints:** 100 requests per minute per IP
- **Search endpoint:** 20 requests per minute per IP
- **Upload endpoint:** 10 requests per minute per IP

Rate limit headers are included in responses:

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1609459200
```

---

## Authentication

Currently, the API does not require authentication. In production environments, consider implementing:

- API key authentication
- JWT tokens
- OAuth 2.0

---

## CORS

Cross-Origin Resource Sharing (CORS) is enabled for all origins in development. In production, configure appropriate CORS policies.

---

## cURL Testing Commands

This section provides ready-to-use cURL commands for testing all API endpoints.

### Health & System

```bash
# Test API root
curl -X GET http://localhost:3000/

# Health check
curl -X GET http://localhost:3000/health
```

### File Management

```bash
# Upload a file (replace with your file path)
curl -X POST http://localhost:3000/upload \
  -F "file=@/path/to/your/document.pdf" \
  -v

# Upload with verbose output to see response headers
curl -X POST http://localhost:3000/upload \
  -F "file=@test.txt" \
  -H "Accept: application/json" \
  -w "Status: %{http_code}\nTime: %{time_total}s\n" \
  -v

# List all files (default pagination)
curl -X GET http://localhost:3000/files

# List files with pagination
curl -X GET "http://localhost:3000/files?skip=0&limit=10"

# Get specific file details (replace with actual file ID)
curl -X GET http://localhost:3000/files/123e4567-e89b-12d3-a456-426614174000

# Get file chunks
curl -X GET http://localhost:3000/files/123e4567-e89b-12d3-a456-426614174000/chunks

# Get file chunks with pagination
curl -X GET "http://localhost:3000/files/123e4567-e89b-12d3-a456-426614174000/chunks?skip=0&limit=5"

# Process file directly (legacy endpoint)
curl -X POST http://localhost:3000/process/123e4567-e89b-12d3-a456-426614174000
```

### Search

```bash
# Basic search
curl -X GET "http://localhost:3000/search?query=machine%20learning"

# Search with all parameters
curl -X GET "http://localhost:3000/search?query=artificial%20intelligence&limit=5&similarity_threshold=0.7&file_id=123e4567-e89b-12d3-a456-426614174000"

# Search with special characters (URL encoded)
curl -X GET "http://localhost:3000/search?query=what%20is%20AI%3F&limit=10"

# Search with JSON response formatting
curl -X GET "http://localhost:3000/search?query=neural%20networks" \
  -H "Accept: application/json" \
  | jq '.'
```

### Job Management

```bash
# Queue file processing job
curl -X POST http://localhost:3000/jobs/process/file/123e4567-e89b-12d3-a456-426614174000

# Queue URL extraction job
curl -X POST http://localhost:3000/jobs/process/url/123e4567-e89b-12d3-a456-426614174000 \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com/article"}'

# Queue YouTube extraction job
curl -X POST http://localhost:3000/jobs/process/youtube/123e4567-e89b-12d3-a456-426614174000 \
  -H "Content-Type: application/json" \
  -d '{"url": "https://youtube.com/watch?v=dQw4w9WgXcQ"}'

# Get job status
curl -X GET http://localhost:3000/jobs/789e0123-e89b-12d3-a456-426614174002

# Cancel a job
curl -X DELETE http://localhost:3000/jobs/789e0123-e89b-12d3-a456-426614174002/cancel

# Get all jobs for a file
curl -X GET http://localhost:3000/jobs/file/123e4567-e89b-12d3-a456-426614174000

# Get all active jobs
curl -X GET http://localhost:3000/jobs/active

# Stream job progress (Server-Sent Events)
curl -X GET http://localhost:3000/jobs/789e0123-e89b-12d3-a456-426614174002/stream \
  -H "Accept: text/event-stream" \
  -N

# Stream all job updates
curl -X GET http://localhost:3000/jobs/stream \
  -H "Accept: text/event-stream" \
  -N
```

### Advanced Testing Scenarios

```bash
# Test file upload with different file types
curl -X POST http://localhost:3000/upload -F "file=@document.pdf"
curl -X POST http://localhost:3000/upload -F "file=@text.txt"
curl -X POST http://localhost:3000/upload -F "file=@presentation.pptx"

# Test large file upload (monitor progress)
curl -X POST http://localhost:3000/upload \
  -F "file=@large_document.pdf" \
  --progress-bar \
  -o upload_response.json

# Test concurrent job creation
for i in {1..5}; do
  curl -X POST http://localhost:3000/jobs/process/file/123e4567-e89b-12d3-a456-426614174000 &
done
wait

# Test search with various query types
curl -X GET "http://localhost:3000/search?query=machine%20learning%20algorithms"
curl -X GET "http://localhost:3000/search?query=deep%20neural%20networks"
curl -X GET "http://localhost:3000/search?query=artificial%20intelligence"

# Test error scenarios
curl -X GET http://localhost:3000/files/nonexistent-id  # Should return 404
curl -X GET "http://localhost:3000/search?query="  # Should return 400 (empty query)
curl -X GET http://localhost:3000/jobs/invalid-job-id  # Should return 404
```

### Performance Testing

```bash
# Test API response times
curl -X GET http://localhost:3000/health \
  -w "Connect: %{time_connect}s\nTTFB: %{time_starttransfer}s\nTotal: %{time_total}s\n" \
  -o /dev/null -s

# Test search performance
curl -X GET "http://localhost:3000/search?query=test" \
  -w "Search time: %{time_total}s\n" \
  -o search_results.json -s

# Load test with multiple concurrent requests
for i in {1..10}; do
  curl -X GET "http://localhost:3000/search?query=test$i" -o "result_$i.json" -s &
done
wait
```

### Debugging Commands

```bash
# Get detailed response headers
curl -X GET http://localhost:3000/health -I

# Test with verbose output
curl -X GET http://localhost:3000/files -v

# Test connectivity and response codes
curl -X GET http://localhost:3000/ \
  -w "HTTP Code: %{http_code}\nRedirect URL: %{redirect_url}\nTotal Time: %{time_total}s\n" \
  -s -o /dev/null

# Save response and headers separately
curl -X GET http://localhost:3000/files \
  -D headers.txt \
  -o response.json

# Test rate limiting (make rapid requests)
for i in {1..25}; do
  echo "Request $i:"
  curl -X GET http://localhost:3000/health \
    -w "Status: %{http_code}\n" \
    -s -o /dev/null
  sleep 0.1
done
```

## Complete Workflow Example

Here's a complete workflow using real cURL commands:

```bash
#!/bin/bash
set -e

echo "🚀 RAG Engine API Testing Workflow"
echo "=================================="

# 1. Check API health
echo "1. Checking API health..."
curl -s http://localhost:3000/health | jq '.data.status'

# 2. Upload a test file
echo "2. Uploading test file..."
UPLOAD_RESPONSE=$(curl -s -X POST http://localhost:3000/upload \
  -F "file=@test_document.pdf")

FILE_ID=$(echo $UPLOAD_RESPONSE | jq -r '.data.file_id')
echo "Uploaded file ID: $FILE_ID"

# 3. Queue processing job
echo "3. Queuing processing job..."
JOB_RESPONSE=$(curl -s -X POST http://localhost:3000/jobs/process/file/$FILE_ID)
JOB_ID=$(echo $JOB_RESPONSE | jq -r '.data.job_id')
echo "Job ID: $JOB_ID"

# 4. Monitor job progress
echo "4. Monitoring job progress..."
while true; do
  STATUS_RESPONSE=$(curl -s http://localhost:3000/jobs/$JOB_ID)
  STATUS=$(echo $STATUS_RESPONSE | jq -r '.data.status')
  PROGRESS=$(echo $STATUS_RESPONSE | jq -r '.data.progress')
  
  echo "Status: $STATUS, Progress: $(echo "$PROGRESS * 100" | bc)%"
  
  if [ "$STATUS" = "completed" ] || [ "$STATUS" = "failed" ]; then
    break
  fi
  
  sleep 2
done

# 5. Search the processed content
if [ "$STATUS" = "completed" ]; then
  echo "5. Searching processed content..."
  curl -s "http://localhost:3000/search?query=machine%20learning&limit=3" | jq '.data.results'
else
  echo "❌ Job failed, cannot search content"
fi

echo "✅ Workflow completed!"
```

### Environment Variables for Testing

```bash
# Set base URL for easier testing
export RAG_API_BASE="http://localhost:3000"

# Use the variable in commands
curl -X GET $RAG_API_BASE/health
curl -X GET "$RAG_API_BASE/search?query=test"

# Test against different environments
export RAG_API_BASE="https://staging.yourapp.com"  # For staging
export RAG_API_BASE="https://api.yourapp.com"      # For production
```

### Testing with Different File Types

```bash
# Create test files for different formats
echo "This is a test document for the RAG engine." > test.txt
echo "# Test Markdown\nThis is a markdown document." > test.md

# Test with different file types
curl -X POST http://localhost:3000/upload -F "file=@test.txt"
curl -X POST http://localhost:3000/upload -F "file=@test.md"

# Test with binary files (should handle gracefully)
curl -X POST http://localhost:3000/upload -F "file=@image.jpg"  # Should be rejected or handled appropriately
```

---

## Examples

### JavaScript Example

```javascript
// Upload file
const formData = new FormData();
formData.append('file', fileInput.files[0]);

const uploadResponse = await fetch('/upload', {
  method: 'POST',
  body: formData
});

const uploadResult = await uploadResponse.json();
const fileId = uploadResult.data.file_id;

// Queue processing
const jobResponse = await fetch(`/jobs/process/file/${fileId}`, {
  method: 'POST'
});

const jobResult = await jobResponse.json();
const jobId = jobResult.data.job_id;

// Stream job progress
const eventSource = new EventSource(`/jobs/${jobId}/stream`);
eventSource.onmessage = (event) => {
  const update = JSON.parse(event.data);
  console.log(`Job ${update.job_id}: ${update.status} (${update.progress * 100}%)`);
  
  if (update.status === 'completed') {
    eventSource.close();
    // Job is done, can now search
    searchContent();
  }
};

// Search function
async function searchContent() {
  const searchResponse = await fetch(`/search?query=your search term&limit=10`);
  const searchResult = await searchResponse.json();
  console.log('Search results:', searchResult.data.results);
}
```

---

## Support

For API support and questions:

- Check the system health at `/health`
- Review error codes and messages
- Ensure proper request formatting
- Verify file types are supported

This documentation covers all available endpoints in the RAG Engine API. The system is designed to be simple, efficient, and scalable for document processing and semantic search applications.

---

## Changelog

### Version 1.0.0 (Latest)

- **Fixed Critical File ID Generation Issue**: Resolved foreign key constraint violations by ensuring database generates file IDs instead of application
- **Added Upload-and-Process Endpoint**: New `/upload-and-process` endpoint for seamless file upload and processing
- **Enhanced File Management**: Added update, delete, and count operations for files
- **New Chunk Management API**: Complete CRUD operations for content chunks
- **New Embedding Management API**: Complete CRUD operations for embeddings with similarity search
- **Improved Error Handling**: Better error messages and status codes
- **Enhanced Documentation**: Comprehensive API documentation with examples

### Key Fixes

- **File ID Consistency**: Database now generates all file IDs, preventing foreign key constraint violations
- **Race Condition Prevention**: Added blocking verification to ensure file persistence before processing
- **Transaction Isolation**: Improved database transaction handling for concurrent operations

### New Features

- Direct chunk and embedding management endpoints
- Advanced search capabilities with embedding vectors
- Real-time job progress streaming
- Comprehensive file metadata management
- Batch operations for chunks and embeddings
