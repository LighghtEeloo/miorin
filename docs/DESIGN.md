# Design Ideas

Here's a bunch of core ideas behind the syntax and semantics design of Miorin.
Miorin is all about structures - how they're discovered, reasoned, and archieved.

*Cube* is our flavor of "object" in an object-oriented paradiam.
Cubes describe the primitive semantics of all structure-based reasoning.
It does not include extra bias and can be viewed as a sort of ground truth.
Cubes are graphs, whose vertices are called objects and edges are called morphisms.
(Sparse table, another strong candidate, is discussed in the next section.)
Higher order Cubes are JUST Cubes whose objects are other Cubes' morphisms.

*Cake* is cache for Cube. (The cake is a lie.)
If some representation is better for performance then vanilla Cube, we can make a cake for the Cube.
A sparse table is just an enriched Cube, where entries and values are objects,
and properties are morphisms, at the same time objects of another cube.
Preorders and orders have their corresponding cakes as well.
A cake should be isomorphic to its original Cube, so it should be bidirectionally updated of and to a cube.

*Prism* describes how Cubes can be rendered. All UI components are ideally Prisms.
Prisms have more data than the underlying cube, which are the visual portion of information.
For example, if the user dragged elements around, we'd better recall the change on next session.
A conceptual set must be ordered during rendering, and we'd like it to be stable as well.
All Prisms are based on their corresponding Cake,
and they manipulate their Cake directly since they should be similar in nature.
Prisms should not add additional constraints on the underlying structure than the Cake.
In short, Cake is for data correctness and Prism is for visual stability.

*Delta* is a set of operations on a Cube that is (conceptually) atomically performed.
Deltas are log-based. When a Prism "sends" a Delta, it eventually propagates to all three sorts.
When a Cake receives a Delta from its corresponding Prism, it should always succeed,
and send the Delta to its Cube.
When a Cake receives a Delta from the Cube, it may not work and if so the Delta will be logged.
When a Cube receives a Delta from the Cake, it should always work since it's the most flexible.
The Cube will then send the Delta to all the Cakes.
Finally, when a Prism "receives" a Delta through the updated Cake,
it's automatically updated because it's a customized render of the Cake.

In conclusion, a Cube may have many corresponding Cakes, all isomorphic to the Cube itself,
and a Cake may have many Prisms based on it, but these Prism can differ.
A Prism can manipulate the Cube through the Cake, and propagate the Delta.
In this case, the Delta starts from the Prism, through the Cake, to the Cube,
and updates backwards (lazily) to all Cakes and Prisms that the Cube interferes.
If a Delta can't be performed in a reasonable way for a Cake, it will be stalled in the Cake,
until conflict is solved, either by weakening the Cake and Prisms (e.g. from Order to Preorder),
or by shallow / deep duplicating the Cube.
We may want reference counting of some sort to keep track of lifetimes of Cubes.

## PreOrder

