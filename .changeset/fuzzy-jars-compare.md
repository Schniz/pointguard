---
"@pointguard/nextjs": patch
"@pointguard/core": patch
---

use the webhook openapi definition for types

the Next.js adapter now returns 200 for all execution requests,
because we successfuly applied them. But the JSON might be an error
(that is managed by Pointguard). Errors are values, Exceptions are bugs.
