# Setup and run

You will need [rust](https://rust-lang.org/) programming language and [postgres server](https://www.postgresql.org/) to run this software.

The project is multi crate cargo workspace.
You can not build all creates in one step, because cargo does not support multiply target projects and web-client compile only for "wasm32-unknown-unknown" target.

## Set up data base

The database connection is configuered in .env file.
It is `postgres://realworld:realworld@localhost/realworld`. So postgres user and password "realworld" and database realworld on localhost.
You can also set the database url as env Variable "DATABASE_URL"

## Setup and Run database migration

[Migration Doc](migration/README.md)

## Start desktop fat client

   ```sh
   cargo run -p fatclient
   ```

## Start desktop quic server and client

Run 2 processes. Open 2 terminals

   ```sh
   cargo run -p quic-server

   cargo run -p quic-client http://localhost:4433
   ```

If you have problems with connection try also

   ```sh
   cargo run -p quic-server --listen 127.0.0.1:4433

   cargo run -p quic-client http://localhost:4433
   ```



## Start wasm client (in browser) and web server (HTTP server)

You need to install wasm32 target and trunk

   ```sh
   rustup target add wasm32-unknown-unknown
   cargo install trunk   
   ```

Run 2 processes. Open 2 terminals.
In dev mode

   ```sh
   cargo run -p web-server

   cd web-client
   trunk serve
   ```

Open browser on http://locahost:8080.

In prod mode you should build using trunk

   ```sh
   cd web-client
   trunk build --release
   cd ..
   cargo run -p web-server
   ```

Open browser on http://locahost:8081.
It serves the frontend directly from web_client/dist folder (this can be set by **FRONTEND_DIST** env variable see .env file)
