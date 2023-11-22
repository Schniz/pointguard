# @pointguard/nextjs

## 0.0.5

### Patch Changes

- [#10](https://github.com/Schniz/pointguard/pull/10) [`26ea3d4`](https://github.com/Schniz/pointguard/commit/26ea3d4af0c246fad952526599f17fbc3b5b1130) Thanks [@Schniz](https://github.com/Schniz)! - use the webhook openapi definition for types

  the Next.js adapter now returns 200 for all execution requests,
  because we successfuly applied them. But the JSON might be an error
  (that is managed by Pointguard). Errors are values, Exceptions are bugs.

## 0.0.4

### Patch Changes

- [#3](https://github.com/Schniz/pointguard/pull/3) [`8d0783b`](https://github.com/Schniz/pointguard/commit/8d0783b2aac867be3eaec57901b775df453b768f) Thanks [@Schniz](https://github.com/Schniz)! - testing CI
