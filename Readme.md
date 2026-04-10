# Realworld app - rust egui multiple architecture demo

This is prototype implementation of [realworl app](https://github.com/realworld-apps/realworld) using full stack rust libraries.
It is learning and programming fun project.

It contains multiple architecture
* fatclient (desktop client talks directly to sql server)
* desktop client (egui) and server using quic protocol
* wasm spa web client (egui) and http server based on axum

The ui and sever core was programmed only once.
There are only few lines of code that difference each architecture

It uses following rust libraries
* [egui]() (ui interface)
* [sea orm]() persistence to sql server
* [quinn]() quic protocol for desktop client server. Secure ssl based protocol using udp.
* [axum]() web server
* [postcard]() message serialization

# Goals of prototype

* learn rust and known libraries on 'realworld' application
* find out pattern for multi target rust architecture
* use common rust code for client and server
* full stack implementation using one programming language and shared code
* test rust in typical enterprise scenarios like db application
* evaluate reusing code for client and server, multi crate project
* evaluate rust pattern that can be used instead on dynamic binding and reflection
* try out another options beside html, java script, json and http for common enterprise applications

# Architectural principles and decisions

* prefer enum over trait
* prefer static polymorphism (generic and trait) over dynamic polymorphism (dyn trait)
* configuration at build time not at run time (no dynamic behavior injection at runtime). Use conditional compilation.
* prefer small code over some arbitrary architectural rules. Do not copy data only because they are passing some module boundaries.
* use existing libs directly. Do not create another abstractions without good reason.
* prefer readable easy code. Do not use generics even if code might looks similar

I am programming big enterprise systems for years.
I have started over 25 years ago at the time of object oriented boom, worked with Smalltalk, C#, Java but also having background in dynamic script languages
like Tcl and python.
But at very beginning it was Assembler, C and C++.
Now with rust finally I can run fast small programs without the runtime overhead of another languages.
I noticed to use rust effectively I should not copy the patterns I know from Java I rather need to find the best way in rust.

It is very refreshing to compile program to small executable and do not need huge runtime and tons of dependencies.

# Why Rust matters

For years similar application was done using following technology stack with strick separation of client and server development.
* server
 * java spring
 * node
 * python django
* frontend
 * browser with react, angular, html

Rust is the chance to dramatically increase performance and reduce memory and cpu usage for both server and client.
Rust is opportunity to use same code for client and server developing.
Rust with strict typed compiler give the chance to validate code at compile time and provide deep checks needed in time
the code can be generated automatically.

I am sure that html and browser are wrong basis for enterprise ui.
A one glance at memory usage of browser shows that it can not be right.
Egui as desktop and even egui as wasm in browser is much more performant and need less resources.

Wasm can be the key technology for secure and performant sand box environment for client and server.

For mostly all applications the developing cost were the main part of software total cost ownership.
But currently the developing is becoming cheaper due to use of AI.
So the runtime cost of software will be more important.
You can not ignore that rust written application are at factor 10-100 smaller and take less resources.

My assumption is that also developing with rust can be cheaper in long term when more good rust programmer are available 
and more best practices are establish.
Typical architectures produce overhead because client and server developing is split.
Using one language for both is biggest saving point.
Python or Typescript are interesting options but they can not beat the performance of compiled language.
Compiling rust for multiple targets is not the problem now. It is equal portable as vm based languages.

The older systems (smalltalk, delphi , visual basic) was very effective because one could develop everything in one language and one environment.
I want the power of old systems back.
It is quite a pleasure to use fast software with no leaks and it is even more important that some 
fancy design with animation gemicks.

# Non considered parts

The prototype does not consider
* security
* no performance test
* no polished ui
* user experience considerations

# Setup and run

You will need rust and postgres server to run this software.

The project is multi crate cargo workspace.
You can not build all creates in one step, because cargo does not support multiply target projects and web-client compile only for "wasm32-unknown-unknown" target.

## Set up data base

The database connection is configuered in .env file.
It is postgres://realworld:realworld@localhost/realworld. So postgres user and password "realworld" and database realworld on localhost.
You can also set the database url as env Variable "DATABASE_URL"

## Run database migration


## Start desktop fat client

   cargo run -p fatclient

## Start desktop quic server and client

Run 2 processes. Open 2 terminals

   cargo run -p quic-server

   cargo run -p quic-client

## Start wasm client (in browser) and web server (HTTP server)

You need to install wasn2 target and trunk

   rustup target add wasm32-unknown-unknown
   cargo install trunk   

Run 2 processes. Open 2 terminals.
In dev mode

   cargo run -p web-server

   cd web-client
   trunk serve

Open browser on http://locahost:8080.

In prod mode you should build using trunk

   cd web-client
   trunk build --release
   cargo run -p web-server

Open browser on http://locahost:8081.
It serves the frontend directly from web_client/dist folder (this can be set by FRONTEND_DIST env variable see .env file)

# Conclusions

# Outlook

There is a lot to do to make the demo suitable for production use.
I noticed that are lot of code could be generated. I do not think that using rust macros is the good way for such complex systems
and there is not possibiltiy to adapt generated code.

I am generally interested in frameworks for rapid application development such like
* ruby on rails
* python django
* python odoo
* java spring boot
* java hipster
* delphi or lazarus (old but indeed really good and productive)

There are a lot low code systems that allows user to program your own app using ui.
The state is hold in database as huge metadata repository.
So you program the system by manipulating database state.

But this approach is not very good for many reasons.
AI currenlty works better with text based definitions. 
Text is better to be managed by systems like git.

So I think if using text based mda with good language server support would be good approach.
I have made some tries with textx and it works good.

If you have ideas about it contact me mail@xdobry.de.




