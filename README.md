# using

[![Crates.io Version](https://img.shields.io/crates/v/using)](https://crates.io/crates/using)
[![docs.rs](https://img.shields.io/docsrs/using)](https://docs.rs/using)
![Crates.io License](https://img.shields.io/crates/l/using)

The `using` macro allows simplified implementation and usage of the builder pattern
without having to return `&mut Self` or `Self` from the builder methods:

```rs
let vec3 = using!(Vec3Builder::default() => {
    .x(4.27);
    .y(9.71);
    .z(13.37);
    .build()
});

#[derive(Default, Debug, Copy, Clone)]
struct Vec3Builder {
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>,
}

impl Vec3Builder {
    pub fn x(&mut self, x: f32) {
        self.x = Some(x);
    }

    pub fn y(&mut self, y: f32) {
        self.y = Some(y);
    }

    pub fn z(&mut self, z: f32) {
        self.z = Some(z);
    }

    //this also works with `self` instead of `&mut self`
    pub fn build(&mut self) -> Vec3 {
        Vec3 {
            x: self.x.unwrap(),
            y: self.y.unwrap(),
            z: self.z.unwrap(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}
```

The `using` macro allows calling multiple methods on the same object,
which is also known as [method cascading](https://en.wikipedia.org/wiki/Method_cascading).
In the example above, the following code is generated:

```rs
let vec3 = {
    let mut target = Vec3Builder::default();
    target.x(4.27);
    target.y(9.71);
    target.z(13.37);
    target.build()
};
```

This allows more flexibility for implementing and using builders
and also makes more complicated use-cases more ergonomic:

```rs
// "Conventional" builder with method chaining:

let mut builder = SomeBuilder::new()
    .x(...)
    .y(...);
if some_condition {
    builder.z(...);
}
if some_other_condition {
    some_function(&mut builder);
}
let thing = builder.build();



// Using Builder with `using`:

let thing = using!(builder @ SomeBuilder::new() => {
    .x(...);
    .y(...);
    if some_condition {
        .z(...);
    }
    if some_other_condition {
        some_function(&mut builder);
    }
    .build()
});
```

Although `using` was primarily designed for the builder pattern,
it is not limited to that as it can be used on basically every type:

```rs
let hello_world = using!(Vec::new() => {
    .push("Hello");
    .push("World!");
    .join(", ")
});
assert_eq!(hello_world, "Hello, World!");
```
