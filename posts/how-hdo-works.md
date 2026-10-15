I wrote a macro called `hdo!` that gives Rust something like Haskell's do
notation. In Rust terms: it's like `?`, except it short-circuits the block instead of
the whole function, and it also works on `Vec`.

Here is a block:

```rust
let result = hdo! {
    x <- Some(10);
    y <- Some(20);

    let sum = x + y;

    guard sum > 5;

    pure sum * 2
};

assert_eq!(result, Some(60));
```

`x <- Some(10);` takes the value out of the `Option` and calls it `x`. If it
was `None`, the whole block stops there and is `None`, the same way `?` returns
early. `guard` stops the block when its condition is false, and `pure` is the
value you want out at the end.

Written by hand, that's a pile of nested closures:

```rust
let result = Some(10).and_then(|x| {
    Some(20).and_then(|y| {
        let sum = x + y;

        if sum > 5 {
            Some(sum * 2)
        } else {
            None
        }
    })
});
```

Two binds are fine. Five get ugly.

So what does the macro actually produce? It goes through the statements from the
bottom up and wraps what it has so far in the next one. A bind becomes a call
with the rest of the block as a closure. Simplified expansion of the block
above:

```rust
HdoBind::hdo_bind(Some(10), |x| {
    HdoBind::hdo_bind(Some(20), |y| {
        let sum = x + y;

        guard_option(
            sum > 5,
            || Pure(sum * 2),
        )
    })
})
```

Two things in there are not what you'd write by hand: `HdoBind` instead of
`and_then`, and `Pure`. Both exist because a proc macro only gets tokens. When
it reads `x <- Some(10);` it has no idea that `Some(10)` is an `Option`. It could be a `Result` or a `Vec`, and those are
sequenced differently. So the macro emits the same call every time and lets the
type checker pick:

```rust
impl<T, F, C> HdoBind<F> for Option<T>
where
    F: FnOnce(T) -> C,
    C: OptionContinuation,
{
    type Output = Option<C::Value>;

    fn hdo_bind(
        self,
        next: F,
    ) -> Self::Output {
        self.and_then(|value| {
            next(value).into_option()
        })
    }
}
```

`Result` and `Vec` have their own impls. For an `Option` it is just `and_then`.

`pure x` has the same problem. Should it build `Some(x)`, `Ok(x)` or `vec![x]`?
The macro can't tell, because that depends on what the block is bound to.

So it builds `Pure(x)`, which is only a wrapper around the value, and leaves
the decision to the next step. `Option`, `Result` and `Vec` each accept `Pure`
as the end of a block and turn it into themselves. That's what the
`OptionContinuation` bound in the impl above is: "whatever comes after this bind
must be able to become an `Option`".

`Option` accepts one more thing there: another `Option`. So a block can end in
a plain expression that already returns an `Option`, without `pure`:

```rust
let result: Option<i32> = hdo! {
    x <- Some(10i32);

    x.checked_add(1)
};
```

The first version of `hdo!` only allowed `pure`. If the last step already
returned an `Option`, you had to unwrap it just to `pure` it back. Later I let
the block end in an expression instead, and the same trait handled it.

Lists work the same way. For `Vec`, a bind is a `flat_map`:

```rust
let pairs: Vec<(i32, i32)> = hdo! {
    x <- vec![1, 2, 3];
    y <- vec![10, 20];

    guard x + y > 12;

    pure (x, y)
};

assert_eq!(
    pairs,
    vec![(1, 20), (2, 20), (3, 10), (3, 20)]
);
```

`guard` gives back an `Option`, and the list side accepts an `Option` as a list
of zero or one elements. That's how a guard drops an element.

So what does `guard` do in a `Result` block? `None` has no error in it, so there
you have to supply one:

```rust
guard x > 5 throw ApiError::TooSmall;
```

Forget it and you get this, instead of a wall of text about
`ResultContinuation`:

```
note: `guard condition;` produces `Option`;
      inside a `Result` block write
      `guard condition throw MyError;`
```

That comes from `#[diagnostic::on_unimplemented]` on the internal traits. These
messages are covered by compile-fail tests in `tests/ui/fail`, checked with
`trybuild` against the exact compiler output.

There is one more kind of bind. `?` converts the error with `From`, so a function returning `Result<_, Outer>`
can use it on a `Result<_, Inner>` as long as `Outer: From<Inner>`. `<-?` does
the same for a bind:

```rust
let result: Result<i32, Outer> = hdo! {
    x <-? inner();

    pure x * 2
};
```

Here `inner()` returns a `Result<i32, Inner>`. It's the one bind that skips
`HdoBind`. It expands to a `match`, because the
conversion needs to know the block's error type (`Outer` here). That has to come
from the type around the block, usually an annotation. Leave it out and you get
`type annotations needed`.

Async is the exception to all of this. `.await` is not allowed inside a plain closure, and the expansion above is made
of closures. So `hdo!(async { ... })` expands differently, into an `async move`
block where every statement runs in sequence and a failure leaves with `return`:

```rust
let result: Result<i32, ApiError> =
    hdo!(async {
        x <- fetch(10).await;
        y <-? from_store().await;

        guard x > 5 throw ApiError::TooSmall;

        pure x + y
    })
    .await;
```

You still write every `.await` yourself, and the block itself is a future.

The `return` is also the limit. An early return can leave with `None` or `Err`,
but a list has no equivalent, so inside `async` only `Option` and `Result` can
be bound.

Right now `hdo!` dispatches on a trait implemented for three types. The roadmap
has a real `Monad` trait, and `hdo!` should be defined on top of it once it
exists. I haven't tried that yet, so no promises.

The code is on [GitHub](https://github.com/bytematebot/raskell), and `raskell` is
on [crates.io](https://crates.io/crates/raskell).
