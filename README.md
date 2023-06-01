<div align="center">

# synthahol-phase-plant

A library to read and write presets for the
[Phase Plant](https://kilohearts.com/products/phase_plant)
synthesizer.

[![crates.io][crates.io-badge]][crates.io]
[![Docs][docs-badge]][docs]
[![Workflows][workflows-badge]][workflows]
</div>

## Overview

synthahol-phase-plant is a library to read and write presets for the
[Phase Plant](https://kilohearts.com/products/phase_plant)
synthesizer by [Kilohearts](https://kilohearts.com).

This library was developed independently by Sheldon Young. It is not a
product of Kilohearts, please do not contact them for support.

## Reading and Writing a Preset

```rust
use std::fs::File;
use phase_plant::io::{ReadPreset, WritePreset};
use phase_plant::kilohearts::phase_plant::Preset;

fn main() -> std::io::Result<()> {
    // Read
    let preset = Preset::read_file("Example.phaseplant")?;
    let author = preset.metadata.author.unwrap_or("anonymous".to_owned());
    println!("The preset was created by {author}");

    // Write
    let mut preset = Preset::default();
    preset.metadata.name = Some("Example Preset".to_owned());

    let mut preset_file = File::create("example.phaseplant")?;
    let write_result = preset.write(&mut preset_file)?;
    for message in write_result.messages {
        println!("{message}");
    }

    Ok(())
}
```

## Known Limitations

* Writing is a work in progress.
* Presets created by version of Phase Plant before the public release version  
  of 1.7.0 are not supporter. Some of the early factory presets were created
  with a pre-release version of Phase Plant.
* Modulation routing is a work in progress.
* Snapin hosts like Multipass, Slice Eq and Snap Heap are not yet fully
  supported. CarveEQ is not supported because it is stored like a host in
  the preset.

## Other Libraries

Use [kibank](https://crates.io/crates/kibank) to combine presets into a bank.

## Issues

If you have any problems with or questions about this project, please contact
the developers by creating a
[GitHub issue](https://github.com/softdevca/synthahol-phase-plant/issues).

## Contributing

You are invited to contribute to new features, fixes, or updates, large or
small; we are always thrilled to receive pull requests, and do our best to
process them as fast as we can.

Before you start to code, we recommend discussing your plans through a
[GitHub issue](https://github.com/softdevca/synthahol-phase-plant/issues),
especially for more ambitious contributions. This gives other contributors a
chance to point you in the right direction, give you feedback on your design,
and help you find out if someone else is working on the same thing.

The copyrights of contributions to this project are retained by their
contributors. No copyright assignment is required to contribute to this
project.

## License

Licensed under the Apache License, Version 2.0 (the "License"); you may not use
this file except in compliance with the License. You may obtain a copy of the
License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the
specific language governing permissions and limitations under the License.

[crates.io]: https://crates.io/crates/synthahol-phase-plant

[crates.io-badge]: https://img.shields.io/crates/v/synthahol-phase-plant?logo=rust&logoColor=white&style=flat-square

[docs]: https://docs.rs/synthahol-phase-plant

[docs-badge]: https://docs.rs/synthahol-phase-plant/badge.svg

[workflows]: https://github.com/softdevca/synthahol-phase-plant/actions/workflows/ci.yml

[workflows-badge]: https://github.com/softdevca/synthahol-phase-plant/actions/workflows/ci.yml/badge.svg
