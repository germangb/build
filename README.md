## `build-rs`

### [[Browser Demo]](https://germangb.github.io/build/)

- Build Engine MAP file renderer (PoC).
- Software renderer.

### Example

```
cargo run --release --example example -- map/tests/maps/GERMAN.MAP
```

![](assets/example.gif)
![](assets/E1L1.gif)
![](assets/SIMPLE0.gif)
![](assets/wasm.gif)
![](assets/mapster.png)

### Limitations

- Maps with non-convex sectors break the renderer, meaning most maps from commercially-released games don't render properly... yet.
- Walking through walls also breaks the renderer (not really a limitation).
- Poorly-implemented movement controller which often breaks the renderer too.