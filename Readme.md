# Realworld app - rust egui multiple architecture demo

This is prototype implementation of [realworl app](https://github.com/realworld-apps/realworld) using full stack rust libraries.
It is learning and programming fun project.
The prototype concerns only parts of needed functionality and features needed for production.

It contains multiple architecture
* fat client (desktop client talks directly to sql server)
* desktop client (egui) and server using quic protocol
* wasm spa web client (egui) and http server based on axum

![screenshot](docu_res/architecture_types.png)

The ui and sever core was programmed only once.
There are only few lines of code that difference each architecture.

It uses following rust libraries
* [egui](https://github.com/emilk/egui) (ui interface)
* [sea orm](https://www.sea-ql.org/SeaORM/) persistence to sql server
* [quinn](https://github.com/quinn-rs/quinn) quic protocol for desktop client server. Secure ssl based protocol using udp.
* [axum](https://github.com/tokio-rs/axum) web server
* [postcard](https://github.com/jamesmunns/postcard) message serialization

# Goals of prototype

* learn rust and known libraries on 'realworld' application
* find out pattern for multi target rust architecture
* use common rust code for client and server
* full stack implementation using one programming language and shared code
* test rust in typical enterprise scenarios like db application
* evaluate reusing code for client and server, multi crate project
* evaluate rust pattern that can be used instead on dynamic binding and reflection
* try out another options beside html, java script, json and http for common enterprise applications
* find out how write sql database independent code (sea orm)

# Non considered parts

The prototype does not consider
* security
* performance test
* polished ui
* user experience considerations

It is not intended as template or framework for such kind of applications.

# Documentation

* [Installation Manual](Installation.md)
* [Architecture Description](Architecture.md)
* [Final Report and Outlook](Final_Report.md)

# Feedback and Contributing

I welcome contributions from the community. If you find but or something odd, please open an issue — even small notes help.
I am also interesting to improve the use case and add additional functionality or make another architecture for example with (leptos, yew or dioxus)
You can also create github issue or just drop me an email mail@xdobry.de.
