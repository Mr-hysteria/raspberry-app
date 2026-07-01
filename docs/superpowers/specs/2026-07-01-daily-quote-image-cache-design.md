# Daily Quote Image Cache Compatibility Design

## Goal

Load common daily-quote image formats reliably while keeping only the current valid background image on disk.

## Supported formats

The downloader will recognize JPEG, PNG, WebP, and GIF from file-signature bytes rather than trusting the URL suffix or HTTP filename. Each recognized image will be stored with its matching extension.

An empty response, an unsupported signature, or an image that Slint cannot decode is a failed image refresh.

## Cache lifecycle

The cache continues to contain one JSON record and at most one valid background image.

1. Download image bytes into memory with the existing 8 MiB limit.
2. Detect the format from the bytes.
3. Write the candidate to a temporary file beside the cache.
4. Verify that Slint can decode the temporary file.
5. Atomically rename it to `daily-quote.<format>`.
6. Update the JSON record to point to the new image.
7. Remove obsolete `daily-quote` image variants only after the new image and JSON are valid.

Temporary files are removed after failed validation. Cleanup is restricted to known image cache names and does not touch `daily-quote.json`.

## Failure behavior

If the image request, format detection, write, or decode validation fails, the new quote text may still be used and the previous valid image remains unchanged. A failed candidate must never overwrite or delete the previous image.

At startup, a cached image path that is absent or undecodable produces the existing plain background fallback.

## Memory behavior

Only the downloaded image bytes and the currently displayed decoded image are held during refresh. The 8 MiB download limit remains. Disk caching does not accumulate historical images: after a successful refresh, at most one recognized image variant remains.

## Testing

Unit tests will cover:

- Detection of JPEG, PNG, WebP, and GIF signatures.
- Rejection of empty and unsupported image data.
- Removal of obsolete known image variants while preserving the current image and JSON record.
- Preservation of the previous cached image when a candidate is invalid.

The complete Rust test suite and existing shell tests will be run after implementation.
