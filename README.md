<p align="center">
</p>

<h1 align="center">
  <img src="./packages/web-ui/src/logo.png" height="96"><br>
  Pointguard
</h1>

<blockquote align="center">
  An MVP-worthy background job server for PostgreSQL, written in Rust
</blockquote>

A simple background job server (database) on top of PostgreSQL, that can be used in _any_ language and _any_ environment.

> [!WARNING]
> this is obviously experimental, but it's for another project I'm working on on so... this is why issues are closed. feel free to [@ or DM me at Twitter](https://twitter.com/galstar) if you want to chat.

## Features

🪶 **Lightweight**: Pointguard is a single binary with no dependencies, written in Rust. The Docker image is so small it feels illegal to add it to your stack.

🔗 **HTTP based**: Jobs are invoked through HTTP calls, so you can keep using your favorite language and environment: Next.js, Remix, Rust, Go -- whether your app is serverless or containerized. HTTP is the only boundary needed!

📝 **Open API**: Pointguard exposes a well-documented OpenAPI-compatible HTTP API, so you can use it from any language or environment. So you can implement a client super easy.

⏰ **Delayed Jobs**: Pointguard supports delayed jobs, so you can schedule jobs to run in the future.

🔁 **Retries**: Pointguard will retry your jobs if they fail, so you can be sure your jobs will run.

💻 **Admin UI**: Pointguard comes with an admin UI, so you can see the status of your jobs.
