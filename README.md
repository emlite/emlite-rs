# emlite
Emlite is a tiny JS bridge for native Rust code via Wasm. It can be used with either the wasm32-wasip1 or the wasm32-unknown-unknown targets, and is agnostic to the underlying toolchain.

## Usage
Add emlite to your Cargo.toml:
```toml
[dependencies]
emlite = "0.1"
```

Then you can import and use the Val wrapper and its associated methods:
```rust
use emlite::{argv, Console};

fn main() {
    let con = Console::get();
    con.log(&argv!["Hello from Emlite!"]);
}
```

```rust
use emlite::*;

fn main() {
    let document = Val::global("document");
    let elem = document.call("createElement", &argv!["BUTTON"]);
    elem.set("textContent", Val::from("Click"));
    let body = document.call("getElementsByTagName", &argv!["body"]).at(0);
    elem.call(
        "addEventListener",
        &argv![
            "click",
            Val::make_fn(|ev| {
                let console = Console::get();
                console.call("clear", &[]);
                console.log(&[ev[0].get("clientX")]);
                println!(
                    "client x: {}",
                    ev[0].get("clientX").as_::<i32>()
                );
                println!("hello from Rust");
                Val::undefined()
            })
        ],
    );
    body.call("appendChild", &argv![elem]);
}
```

```rust
use emlite::*;

fn main() {
    #[allow(non_snake_case)]
    let mut AudioContext = Val::global("AudioContext");
    if !AudioContext.as_::<bool>() {
        println!("No global AudioContext, trying webkitAudioContext");
        AudioContext = Val::global("webkitAudioContext");
    }

    println!("Got an AudioContext");
    let context = AudioContext.new(&[]);
    let oscillator = context.call("createOscillator", &[]);

    println!("Configuring oscillator");
    oscillator.set("type", "triangle");
    oscillator.get("frequency").set::<_, f64>("value", 261.63); // Middle C

    let document = Val::global("document");
    let elem = document.call("createElement", &argv!["BUTTON"]);
    elem.set("textContent", "Click");
    let body = document.call("getElementsByTagName", &argv!["body"]).at(0);
    elem.call(
        "addEventListener",
        &argv![
            "click",
            Val::make_fn(move |_| {
                println!("Playing");
                oscillator.call("connect", &argv![context.get("destination")]);
                oscillator.call("start", &argv![0]);
                println!("All done!");
                Val::undefined()
            })
        ],
    );
    body.call("appendChild", &argv![elem]);
}
```

## Building

### For the wasm32-wasip1 target

You need:
- wasm32-wasip1 target.

To get the rust target:
```bash
rustup target add wasm32-wasip1
```

Running the build, you only need to pass the target to cargo:
```
cargo build --target=wasm32-wasip1
```

## Passing necessary flags for javascript engines (browser, node ...etc)

The most convenient way to pass extra flags to the toolchain is via a .cargo/config.toml file:
```toml
[target.wasm32-wasip1]
rustflags = ["-Clink-args=--no-entry --allow-undefined --export-dynamic --export-if-defined=main --export-table --import-memory --export-memory --strip-all"]

[profile.release]
lto = true # to get smaller builds
```

### For the wasm32-unknown-unknown target

You need:
- wasm32-unknown-unknown target.

To get the rust target:
```bash
rustup target add wasm32-unknown-unknown
```

## Passing necessary flags for javascript engines (browser, node ...etc)

The most convenient way to pass extra flags to the toolchain is via a .cargo/config.toml file:
```toml
[target.wasm32-unknown-unknown]
rustflags = ["-Clink-args=--no-entry --allow-undefined --export-dynamic --export-if-defined=main --export-if-defined=add --export-table --import-memory --export-memory --strip-all"]

[profile.release]
lto = true # to get smaller builds
```

## Deployment

### For the wasip1 target

#### In the browser

To use it in your web stack, you will need a wasi javascript polyfill, here we use @bjorn3/browser_wasi_shim and the emlite npm packages:

```javascript
// see the index.html for an example
import { WASI, File, OpenFile, ConsoleStdout } from "@bjorn3/browser_wasi_shim";
import { Emlite } from "emlite";

async function main() {
    let fds = [
        new OpenFile(new File([])), // 0, stdin
        ConsoleStdout.lineBuffered(msg => console.log(`[WASI stdout] ${msg}`)), // 1, stdout
        ConsoleStdout.lineBuffered(msg => console.warn(`[WASI stderr] ${msg}`)), // 2, stderr
    ];
    let wasi = new WASI([], [], fds);
    const emlite = new Emlite();
    const bytes = await emlite.readFile(new URL("./bin/dom_test1.wasm", import.meta.url));
    let wasm = await WebAssembly.compile(bytes);
    let inst = await WebAssembly.instantiate(wasm, {
        "wasi_snapshot_preview1": wasi.wasiImport,
        "env": emlite.env,
    });
    emlite.setExports(inst.exports);
    // if your C/C++ has a main function, use: `wasi.start(inst)`. If not, use `wasi.initialize(inst)`.
    wasi.start(inst);
    // test our exported function `add` in tests/dom_test1.cpp works
    window.alert(inst.exports.add?.(1, 2));
}

await main();
```

#### With a javascript engine like nodejs

If you're vendoring the emlite.js file:
```javascript
import { Emlite } from "emlite";
import { WASI } from "node:wasi";
import { argv, env } from "node:process";

async function main() {
    const wasi = new WASI({
        version: 'preview1',
        args: argv,
        env,
    });
    
    const emlite = new Emlite();
    const bytes = await emlite.readFile(new URL("./bin/console.wasm", import.meta.url));
    const wasm = await WebAssembly.compile(bytes);
    const instance = await WebAssembly.instantiate(wasm, {
        wasi_snapshot_preview1: wasi.wasiImport,
        env: emlite.env,
    });
    wasi.start(instance);
    emlite.setExports(instance.exports);
    // if you have another exported function marked with EMLITE_USED, you can get it in the instance exports
    instance.exports.some_func();
}

await main();
```
Note that nodejs as of version 22.16 requires a _start function in the wasm module. That can be achieved by defining an `fn main() {}` function. It's also why we use `wasi.start(instance)` in the js module.

### For the wasm32-unknown-unknown target

#### Targeting the browser

Emlite-rs can be used with Rust's wasm32-unknown-unknown target:

```javascript
import { Emlite } from "./src/emlite.js";

async function main() => {
    const emlite = new Emlite();
    const bytes = await emlite.readFile(new URL("./target/wasm32-unknown-unknown/release/examples/audio.wasm", import.meta.url));
    let wasm = await WebAssembly.compile(bytes);
    let inst = await WebAssembly.instantiate(wasm, {
        env: emlite.env,
    });
    emlite.setExports(inst.exports);
    inst.exports.main();
}

await main();
```

#### Targeting node and other javascript engins

```javascript
import { Emlite } from "./src/emlite.js";

async function main() {
    const emlite = new Emlite();
    const bytes = await emlite.readFile(new URL("./bin/console.wasm", import.meta.url));
    const wasm = await WebAssembly.compile(bytes);
    const instance = await WebAssembly.instantiate(wasm, {
        env: emlite.env,
    });
    emlite.setExports(instance.exports);
    instance.exports.main();
}

await main();
```

#### Targeting emscripten
emlite-rs supports emscripten's default mode when it outputs js glue code. This will require building for the wasm32-unknown-emscripten target.
The most convenient way to pass extra flags to the toolchain is via a .cargo/config.toml file:
```toml
[target.wasm32-unknown-emscripten]
rustflags = ["-Clink-args=-sERROR_ON_UNDEFINED_SYMBOLS=0 -sALLOW_MEMORY_GROWTH=1 -sEXPORTED_RUNTIME_METHODS=wasmTable -Wl,--strip-all"]
```

You just need to load the emscripten code after emlite:
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Document</title>
</head>
<body>
    <script src="./bin/mywasms.js"></script>
</body>
</html>
```

If you pass the `-sMODULARIZE=1 -sEXPORT_ES6=1` flags to emscripten, you will have to initialize your module accordingly:
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Document</title>
</head>
<body>
    <script type="module">
        import initModule from "./bin/mywasm.js";
        window.onload = async () => {
            const mymain = await initModule();
        };
    </script>
</body>
</html>
```