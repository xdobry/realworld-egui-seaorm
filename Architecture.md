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

## Why not hexagonal or whats ever

It was not fundamental for me to follow any arbitrary architectural principles.
Im not a good fan for breaking dependencies by replicate structures with same data.
The logical dependency will be still there but the compiler will not help you if things diverged.
So the ui and server use same struct/enitity definition, which is "polluted" by persistency annotation "sea orm".
Thank to compiler indeed the sea orm dependencies are not in the binary of ui so for me it is practical approach.
Perhaps cleaner solution will be to have pure rust struct and than make db annotation in other place.

## Module Dependencies

The most code is located in ui, core and server-core.
The code of models was created automatically using sea orm cli.

   `sea-orm-cli generate entity --database-url postgres://realworld:realworld@localhost/realworld --output-dir ./src/entity --entity-format dense --with-serde both`

![screenshot](docu_res/module_dependencies.png)

## Concrete solved challenges

* egui runs one thread. The interaction with db requires async function calls that are directly not possible in egui code.
  To separate it messages like (load user, create uses) are send to async able worker via message bus (based on msp).
  This lead to somekind to ELM-Architecture where UI communicates to backend via async messages
* ui in desktop use tokio for async communication with db or quic server the wasm ui can not use tokio but use poll-promise the is abstraction as command bus that
  implement this for desktop and wasm variant differently
* messages are serialized to compact binary using postcard serde library. The web server has only one endpoint for all ui messages/commands. It is not REST conform and probably
  should be better WebSocket or WebTransport. Same serialization is used for quic or web server.
* egui page trait abstract the usage of command bus for backend communication
* the model sea entities are used for backend but also for ui to reuse the structs and not values.
