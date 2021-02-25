# grab-rs

A library for effortlessly grabbing input users provide to your CLI. Whether from
files, stdin or just plain text on the command line, we've got you covered.

## Installation

1. Add it to your Cargo.toml dependencies

    ```toml
    # Cargo.toml

    [dependencies]
    grab = "0.3"
    ```

2. Import and use it

    ```rust
    use grab::{ /* ... */ }
    ```

## Usage

The main type exposed by this library is `grab::Input`. An instance of `Input` can
be created via `grab::Config::parse` which takes a `&str` as input and attempts to
parse it into a known `Input` source.

Currently, known sources are:

- The actual text (`&str`) passed in as input
- Your program's stdin
- A file

The default `Config`uration recognizes `-` for stdin per the unix tradition, and
takes cues from `curl -d` by recognizing `@<file path>` as a file.

This means that you can easily support all three of the most common input sources
with a single type. For example, assume we had a simple CLI tool named `hello`
that prints out a hello to the user...

```rust
use structopt::StructOpt;
use grab::Input;

// We use the popular StructOpt library for our CLI skeleton
#[derive(StructOpt)]
struct HelloCLI {
   // Note we use Input instead of the typical String or Vec<u8>
   /// The name we're going to greet
   name: Input
}

fn main() -> Result<()> {
   // Parse our argument(s)
   let args = HelloCLI::from_args();

   // Access the user's input, reading it to a string
   let name = args.name.access()?.read_to_string()?;

   // Say hello!
   println!("Hello, {}!", &name);
   Ok(())
}
```

Now we can support all three of the following invocations with absolutely zero
extra code:

```bash
$ hello John
Hello, John!

$ echo Bob | hello -
Hello, Bob!

$ print Fred >name.txt ; hello @name.txt
Hello, Fred!
```

#### License

<sup>
Licensed under <a href="LICENSE">Apache License, Version 2.0</a>
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be licensed as above, without any additional terms or conditions.
</sub>
