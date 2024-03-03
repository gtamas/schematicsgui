# Build instructions

First you need to install Rust as well as a linker and the usual build tools such as gcc, make etc.

 See [this page](https://doc.rust-lang.org/book/ch01-01-installation.html) and follow instructions to install everything.

Once you have these, get the code:

```bash
git clone git@github.com:gtamas/schematicsgui.git /some/path
```

Finally, run build:

```bash
cd /some/path
cargo build
cargo run
```

This should build and start the app.

The above command will create a dev build with debug stuff included. If you want a optimized ("release") build,
run this instead:

```bash
cargo build --release
```

Once the build is done, you can find the binary here:

```bash
./target/release/schematics-gui
```