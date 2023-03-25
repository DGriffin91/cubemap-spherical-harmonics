# Generate Spherical Harmonics from Cubemaps 

Example usage:
```bash
cargo run --release "example_cube.png"
```

Example output:
```rust
[
    Vec3(0.48503733, 0.48316428, 0.32759333),
    Vec3(-0.26511002, 0.2617326, -0.0001285886),
    Vec3(-0.26024902, -0.26002043, 0.26548815),
    Vec3(0.2476333, 0.24728885, 0.24757174),
    Vec3(0.00032010232, 0.00019238597, 0.0005412985),
    Vec3(0.0009668731, 0.0009325959, 0.00018943335),
    Vec3(-0.00040348107, -0.00046164956, 0.00018102766),
    Vec3(0.0044354284, 0.0049572727, 0.030710248),
    Vec3(0.002222224, 0.000540525, -0.08147548),
]
```

![Example Render](example_render.png)

Example wgsl shader (used for render above)
```glsl
struct FragmentInput {
    [[builtin(front_facing)]] is_front: bool;
    [[builtin(position)]] frag_coord: vec4<f32>;
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
};

fn irradianceSH(n: vec3<f32>) -> vec3<f32> {    
    let sh0 = vec3(0.48503733, 0.48316428, 0.32759333);
    let sh1 = vec3(-0.26511002, 0.2617326, -0.0001285886);
    let sh2 = vec3(-0.26024902, -0.26002043, 0.26548815);
    let sh3 = vec3(0.2476333, 0.24728885, 0.24757174);
    let sh4 = vec3(0.00032010232, 0.00019238597, 0.0005412985);
    let sh5 = vec3(0.0009668731, 0.0009325959, 0.00018943335);
    let sh6 = vec3(-0.00040348107, -0.00046164956, 0.00018102766);
    let sh7 = vec3(0.0044354284, 0.0049572727, 0.030710248);
    let sh8 = vec3(0.002222224, 0.000540525, -0.08147548);
    
    let z = -n.z; //z is inverted for Bevy
    let x = n.x;
    let y = n.y;

    return
          sh0
        + sh1 * (x)
        + sh2 * (y)
        + sh3 * (z)
        + sh4 * (x * z)
        + sh5 * (z * y)
        + sh6 * (y * x)
        + sh7 * (3.0 * z * z - 1.0)
        + sh8 * (x * x - y * y);
}

[[stage(fragment)]]
fn fragment(in: FragmentInput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(irradianceSH(normalize(in.world_normal.xyz)), 1.0);
}
```
