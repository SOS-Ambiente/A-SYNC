# MSSCS v4.0 API Documentation

## Base URL

```
http://localhost:8080
```

## Authentication

If API keys are configured, include the `X-API-Key` header in all requests:

```
X-API-Key: your-secret-key
```

## Endpoints

### Write File

Upload a file to the distributed storage system.

**Endpoint**: `POST /files`

**Request Body**:
```json
{
  "path": "example.txt",
  "content": "SGVsbG8sIFdvcmxkIQ=="
}
```

- `path` (string): File path/name
- `content` (string): Base64-encoded file content

**Response**: `201 Created`
```json
{
  "uuid": "550e8400-e29b-41d4-a716-446655440000",
  "blocks": 1
}
```

- `uuid` (string): UUID of the first block in the chain
- `blocks` (number): Number of blocks created

**Example**:
```bash
# Encode file to base64
base64 myfile.txt > encoded.txt

# Upload file
curl -X POST http://localhost:8080/files \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-key" \
  -d '{
    "path": "myfile.txt",
    "content": "'$(cat encoded.txt)'"
  }'
```

---

### Read File

Download a file from the distributed storage system.

**Endpoint**: `GET /files/:path`

**Parameters**:
- `path` (string): File path/name

**Response**: `200 OK`
```json
{
  "content": "SGVsbG8sIFdvcmxkIQ=="
}
```

- `content` (string): Base64-encoded file content

**Example**:
```bash
# Download and decode file
curl http://localhost:8080/files/myfile.txt \
  -H "X-API-Key: your-key" \
  | jq -r '.content' \
  | base64 -d > downloaded.txt
```

**Error Responses**:
- `404 Not Found`: File does not exist
```json
"File 'myfile.txt' not found"
```

---

### Delete File

Remove a file from the manifest (blocks remain until cleanup).

**Endpoint**: `DELETE /files/:path`

**Parameters**:
- `path` (string): File path/name

**Response**: `200 OK`
```json
{
  "status": "deleted"
}
```

**Example**:
```bash
curl -X DELETE http://localhost:8080/files/myfile.txt \
  -H "X-API-Key: your-key"
```

**Error Responses**:
- `404 Not Found`: File does not exist

---

### List Files

Get a list of all files in the system.

**Endpoint**: `GET /files`

**Response**: `200 OK`
```json
{
  "files": [
    "file1.txt",
    "file2.txt",
    "documents/report.pdf"
  ]
}
```

**Example**:
```bash
curl http://localhost:8080/files \
  -H "X-API-Key: your-key"
```

---

### Get Block Info

Retrieve metadata about a specific block.

**Endpoint**: `GET /blocks/:uuid`

**Parameters**:
- `uuid` (string): Block UUID

**Response**: `200 OK`
```json
{
  "uuid": "550e8400-e29b-41d4-a716-446655440000",
  "node_index": 0,
  "size": 1234
}
```

- `uuid` (string): Block UUID
- `node_index` (number): Position in chain
- `size` (number): Block size in bytes

**Example**:
```bash
curl http://localhost:8080/blocks/550e8400-e29b-41d4-a716-446655440000 \
  -H "X-API-Key: your-key"
```

**Error Responses**:
- `400 Bad Request`: Invalid UUID format
- `404 Not Found`: Block does not exist

---

### Health Check

Check if the node is running and connected to peers.

**Endpoint**: `GET /health`

**Response**: `200 OK`
```json
{
  "status": "healthy",
  "peers": 3
}
```

- `status` (string): Always "healthy" if responding
- `peers` (number): Number of connected peers

**Example**:
```bash
curl http://localhost:8080/health
```

---

### Metrics

Get system metrics and statistics.

**Endpoint**: `GET /metrics`

**Response**: `200 OK`
```json
{
  "block_count": 1234,
  "storage_bytes": 5242880,
  "peer_count": 3,
  "uptime_seconds": 3600,
  "requests_total": 100,
  "requests_failed": 2,
  "success_rate": 98.0
}
```

- `block_count` (number): Total blocks stored locally
- `storage_bytes` (number): Total storage used in bytes
- `peer_count` (number): Number of connected peers
- `uptime_seconds` (number): Time since node started
- `requests_total` (number): Total API requests
- `requests_failed` (number): Failed API requests
- `success_rate` (number): Success rate percentage

**Example**:
```bash
curl http://localhost:8080/metrics
```

---

## Error Codes

### 400 Bad Request
Invalid request format or parameters.

### 401 Unauthorized
Missing or invalid API key.

### 404 Not Found
Requested resource does not exist.

### 500 Internal Server Error
Server-side error occurred.

---

## Rate Limiting

Currently no rate limiting is implemented. Consider implementing rate limiting in production environments.

---

## CORS

CORS is enabled for all origins. Adjust in production as needed.

---

## Examples

### Python Client

```python
import requests
import base64

BASE_URL = "http://localhost:8080"
API_KEY = "your-secret-key"

headers = {"X-API-Key": API_KEY}

# Write file
with open("test.txt", "rb") as f:
    content = base64.b64encode(f.read()).decode()

response = requests.post(
    f"{BASE_URL}/files",
    json={"path": "test.txt", "content": content},
    headers=headers
)
print(response.json())

# Read file
response = requests.get(f"{BASE_URL}/files/test.txt", headers=headers)
content = base64.b64decode(response.json()["content"])
print(content.decode())

# List files
response = requests.get(f"{BASE_URL}/files", headers=headers)
print(response.json())

# Delete file
response = requests.delete(f"{BASE_URL}/files/test.txt", headers=headers)
print(response.json())
```

### JavaScript Client

```javascript
const BASE_URL = "http://localhost:8080";
const API_KEY = "your-secret-key";

const headers = {
  "Content-Type": "application/json",
  "X-API-Key": API_KEY
};

// Write file
const content = btoa("Hello, World!");
const response = await fetch(`${BASE_URL}/files`, {
  method: "POST",
  headers,
  body: JSON.stringify({
    path: "test.txt",
    content
  })
});
console.log(await response.json());

// Read file
const readResponse = await fetch(`${BASE_URL}/files/test.txt`, { headers });
const data = await readResponse.json();
console.log(atob(data.content));

// List files
const listResponse = await fetch(`${BASE_URL}/files`, { headers });
console.log(await listResponse.json());

// Delete file
const deleteResponse = await fetch(`${BASE_URL}/files/test.txt`, {
  method: "DELETE",
  headers
});
console.log(await deleteResponse.json());
```
