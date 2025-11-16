# SIA CLI Architecture

## Design Principles

### Backend (Rust Agent)
- **Returns raw, unformatted metrics** in a standard JSON structure
- **Performance optimized**: Minimal data transformation, efficient serialization
- **Resource efficient**: Only sends necessary numeric values
- **Format**: Standard JSON with clear field names

### Frontend (TypeScript CLI)
- **Handles all formatting and presentation** logic
- **User-friendly display**: Converts raw data to human-readable format
- **Visual enhancements**: Adds progress bars, colors, and formatting
- **Responsive**: Adapts to terminal width

## Data Flow

```
Backend (Rust)                    Frontend (TypeScript)
─────────────────                 ─────────────────────
Raw Metrics                       Formatted Display
─────────────────                 ─────────────────────
cpu_usage: 45.5          →       45.5% [████████░░░░░░░░░░░░]
memory_used_mb: 1024     →       1.0 GB
memory_total_mb: 4096    →       4.0 GB
memory_percent: 25.0     →       25.0% [█████░░░░░░░░░░░░░░░]
```

## Backend Response Format

The backend returns metrics in a clean, structured format:

```json
{
  "success": true,
  "data": {
    "status": "running",
    "uptime_seconds": 3600,
    "metrics": {
      "cpu_usage": 45.5,           // Raw float percentage
      "memory_used_mb": 1024,      // Raw integer MB
      "memory_total_mb": 4096,     // Raw integer MB
      "memory_percent": 25.0       // Raw float percentage
    },
    "collectors": {
      "cpu": "active",
      "memory": "active"
    },
    "events": {
      "critical": 0,
      "warning": 2,
      "info": 5
    }
  }
}
```

## Frontend Formatting

The TypeScript CLI enhances the raw data with:

1. **Percentage Formatting**: Adds visual progress bars
   - `45.5% [████████░░░░░░░░░░░░]`

2. **Byte Formatting**: Converts MB to human-readable units
   - `1024 MB` → `1.0 GB`
   - `512 MB` → `512.0 MB`
   - `2048 MB` → `2.0 GB`

3. **Color Coding**: Visual indicators for status
   - Green: Normal (< 60% CPU, < 70% Memory)
   - Yellow: Warning (60-80% CPU, 70-85% Memory)
   - Red: Critical (> 80% CPU, > 85% Memory)

4. **Uptime Formatting**: Human-readable time
   - `3600` → `1h 0m 0s`
   - `125` → `2m 5s`

## Benefits

✅ **Separation of Concerns**: Backend focuses on data, frontend on presentation
✅ **Performance**: Backend doesn't waste CPU on formatting
✅ **Flexibility**: Different clients can format data differently
✅ **Maintainability**: Formatting logic isolated in one place
✅ **Resource Efficient**: Minimal data transfer, no redundant formatting

## Example

**Backend sends:**
```json
{"cpu_usage": 75.5, "memory_used_mb": 2048, "memory_total_mb": 8192, "memory_percent": 25.0}
```

**Frontend displays:**
```
CPU Usage:  75.5% [███████████████░░░░░]  (yellow)
Memory:     25.0% [█████░░░░░░░░░░░░░░░]  2.0 GB / 8.0 GB  (green)
```

