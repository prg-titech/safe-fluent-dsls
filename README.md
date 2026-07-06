# safe-fluent-dsls
This repository is for research on embedding fluent DSLs into host languages safely. Static/Compile-time checks are supposed to ensure that embedded-DSL terms are syntactically correct, and well-formed.

## Related Work

Fluent API generators, which when given a grammar generate a fluent API for an
EDSL, are already widely available.
Tools like [Erilex](https://link.springer.com/chapter/10.1007/978-3-642-13953-6_11), 
or Fajita(no reference available), can generate a fluent API from an LL(1) grammar.
More recent research is also able to parse LR(1) grammars, which corresponds
to the class of all deterministic context-free grammars, thus ideal for programming
languages.

Syntax checking of the generated API, while not widely available,

### [Generating a fluent API with syntax checking from an LR grammar](http://localhost)

Generates fluent APIs which uses the type system of the host language to perform
type checking. However, numeric and string literals are only checked for correctness
at runtime.

### [Yet Another Generating Method of Fluent Interfaces Supporting Flat- and Sub-chaining Styles](https://dl.acm.org/doi/10.1145/3567512.3567533)

Works for LL(1) grammars. Allows both flat-(fluent) and sub-chaining (tree) calling
styles. The choice is up to the user of the generated API.

Example of Flat-Chaining:
```
Graph.digraph("test")
     .node("A")
     .node("B")
     .node("C")
     .node("D")
     .node("E")
     .edge("A").to("B")
     .edge("B").to("C")
     .edge("C").to("D")
     .edge("D").to("E")
     .edge("E").to("A")
     .end()
```

Example of Sub-chaining:

```
Graph.digraph("test")
     .node("A")
     .node("B")
     .node("C")
     .node("D")
     .node("E")
     .edge(Edge("A").to("B"))
     .edge(Edge("B").to("C"))     
     .edge(Edge("C").to("D"))
     .edge(Edge("D").to("E"))
     .edge(Edge("E").to("A"))
     .end()
```

### [LR parsing for strings with placeholders](https://dl.acm.org/doi/10.1016/j.ipl.2026.106627)

Parsing with placeholders is very relevant, because even with sophisticated data-flow analysis
there will be cases where the result of a function is unknown. However, we still want to provide
static analysis for fluent APIs with such placeholders.

Example of fluent API use which needs placeholders:

```
Query.select("*").from(foo())
```

Here, `foo()` might make a remote network call or do some other operations
that cannot be analyzed by even the best data-flow analysis, meaning that
we only now what the return type should be. As a consequence:

- Static Analysis should allow for placeholders during parsing
- There should still be dynamic errors in case the result of `foo()` causes
invalid syntax
- Optional: Warn the user that this kind of dynamic call could cause errors at runtime