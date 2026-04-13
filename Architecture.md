## Architectural Principles and Decisions

* Prefer enums over traits
* Prefer static polymorphism (generics and traits) over dynamic polymorphism (dyn Trait)
* Prefer build-time configuration over runtime configuration (no dynamic behavior injection at runtime). Use conditional compilation instead
* Prefer simpler code over enforcing arbitrary architectural rules. Do not duplicate data just to satisfy module boundaries
* Use existing libraries directly. Do not introduce additional abstractions without a good reason
* Prefer clear, readable code. Avoid unnecessary use of generics, even if it results in some code duplication

I have been developing large enterprise systems for many years.

I started over 25 years ago during the object-oriented boom, working with Smalltalk, C#, and Java, but I also have experience with dynamic scripting languages such as Tcl and Python. 
At the very beginning, I worked with Assembly, C, and C++.

With Rust, I can finally build fast and small programs without the runtime overhead of other languages. I have learned that to use Rust effectively, I should not copy patterns from Java, but instead find idiomatic solutions that fit Rust.

It is very refreshing to compile a program into a small executable without requiring a large runtime or many dependencies.

## Why Not Hexagonal (or Similar) Architectures

It was not important for me to follow any predefined architectural style.

I am not a fan of breaking dependencies by duplicating structures that contain the same data. The logical dependency still exists, but the compiler can no longer help if the structures diverge.

For this reason, the UI and server share the same struct/entity definitions, even though they are “polluted” with persistence annotations from SeaORM.

Thanks to the compiler, the SeaORM dependencies are not included in the UI binary, so this is a practical solution for me. 
A cleaner approach might be to define pure Rust structs and apply database annotations separately.

## Module Dependencies

Most of the code is located in the ui, core, and server-core modules.
The model code was generated automatically using the SeaORM CLI:

   `sea-orm-cli generate entity --database-url postgres://realworld:realworld@localhost/realworld --output-dir ./src/entity --entity-format dense --with-serde both`

![screenshot](docu_res/module_dependencies.png)

## Entities

![screenshot](docu_res/er_diagramm.png)

## Concrete solved challenges

* Single-threaded UI (egui):
egui runs on a single thread, while database interaction requires async operations that cannot be executed directly in UI code.
To solve this, messages (e.g., “load user”, “create user”) are sent to an async-capable worker via a message bus (based on mpsc).
This results in an architecture similar to the Elm pattern, where the UI communicates with the backend via async messages.
* Async abstraction for desktop and WASM:
The desktop UI uses Tokio for async communication with the database or a QUIC server.
The WASM UI cannot use Tokio, so it uses poll-promise.
A command bus abstraction hides these differences and provides a unified interface.
* Binary message protocol:
Messages are serialized into a compact binary format using postcard and serde.
The web server exposes a single endpoint for all UI messages/commands. This is not RESTful; WebSocket or WebTransport would likely be a better fit.
The same serialization format is used for both QUIC and HTTP communication.
* UI abstraction:
An egui page trait abstracts the use of the command bus for backend communication.
* Shared entities:
SeaORM entities are used both in the backend and in the UI to avoid duplicating data structures.