# Conclusions

It was quite a fun to create the real world demo.
I have learned a lot. I could find out things that I am never excepted before.

# Why Rust matters

My opinion about Rust and why it is worth to bring rust as replacment for java, python or typescript.

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

I am quite sure that html and browser are wrong basis for enterprise ui.
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

# With the little help of AI

The like to write code on my own. But I am using AI a lot as documentation manual or knowledge base.
This way The AI makes the developing 

Anyway I want to understand every line of code.
But probably with some additional AI description this project can be made as template
for agent based AI developing.
I would be quite interesting if someone tries it out.

Adding new entries or modifying existing ui should be not big problem for AI.

AI can generate code from existing pattern but it is not good in creating new architectures or coding approaches
especially in some never language.
So perhaps some AI will main reader of this project and reproduce it somewhere else.

# My conclusions aout using rust as programmin language

Rust is powerfull low code system language.
Programming typicall applications with crates like sea orm, serde, axum and egui are more or less same easy as using java or python.
The rust abstraction are powerfull enough to write also "business" applications with similar efficieny like Java, C#, Typescript or Python.
Rust enum and match are very powerfull to write readable code.
Rust compiler can be your best friend because it support you to write valid typesafe code.

# Egui - UI Library (intermediate model)

Egui  was primary developed in gaming area as rust reimplementation of imgui.
The are not build in options for programmin powerfull data grid.
It is very pleasant for user because it is fast.
Probably egui should not be used to get html/css or some special corporate design.
Programming own special egui elements are quite easy because of immediate mode.

# Outlook

There is a lot to do to make the demo suitable for production use.
I noticed that are lot of code could be generated. I do not think that using rust macros is the good way for such complex systems
and there is not possibility to adapt generated code.

I am generally interested in frameworks for rapid application development such like
* ruby on rails
* python django
* [python odoo](https://www.odoo.com/)
* java spring boot
* java hipster
* delphi or [lazarus](https://www.lazarus-ide.org/) (old but indeed really good and productive)

There are a lot low code systems that allows user to program your own app using ui.
The state is hold in database as huge metadata repository.
So you program the system by manipulating database state.

But this approach is not very good for many reasons.
AI currently works better with text based definitions. 
Text is better to be managed by systems like git.

So I think if using text based mda with good language server support would be good approach.
I have made some tries with [textx](https://github.com/textx/textx) and it works good.

If you have ideas about it contact me mail@xdobry.de.
