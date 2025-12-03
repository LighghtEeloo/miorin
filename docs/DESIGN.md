# Design Ideas

Here's a bunch of core ideas behind the syntax and semantics design of Miorin.
Miorin is all about structures - how they're discovered, reasoned, and archieved.

*Cube* is our flavor of "object" in an object-oriented paradiam.
Cubes describe the primitive semantics of all structure-based reasoning.
It does not include extra bias and can be viewed as a sort of ground truth. Cubes are graphs ~~(maybe plus sparse tables)~~. Higher order Cubes are JUST Cubes whose objects are other Cubes' morphisms.

*Cake* is cache for Cube. If some representation is better for performance then vanilla Cube, we can make a cake for the Cube. A sparse table is just an enriched Cube. Preorders and orders have their own cake.

*Prism* describes how Cubes can be rendered. All UI components are ideally Prisms.


## PreOrder

