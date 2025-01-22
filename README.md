A project to learn how to code in Rust. The objective is to get a working FIX client and server then have each expose a simple API to allow a python client to send orders and get updates/executions from either end.

Phase 1 : write it using asnyc Tokio, make it FIX 4.2 Compliant ( focus on Session, Recovery and Resets ) <br>
Phase 2 : Create a Fork and rewrite using mio / single-threaded.<br>
Phase 3 : Create a Fork and make Tokio implementation as idiomatic as possible, delegating as much as possible to pre-existing crates<br>
Phase 4 : Create a Fork and make mio implementation that is a fast as possible with a few dependencies as possible<br>
Phase 5 : Add in support for as many of the msg_types / groups etc ( potentially going full FIX5.0 sp2 ) 
