# VDFLex

VDFLex is a (de)serialization library for parsing the Valve Data File format with 
[serde](https://crates.io/crates/serde). VDF—or more generally, [KeyValues](https://developer.valvesoftware.com/wiki/KeyValues)—is 
a data format developed by Valve for use in Steam and the Source engine.

```text
LightmappedGeneric
{
    $basetexture "water/water_still"
    $surfaceprop water
    $translucent 1
    
    %compilewater 1
    %tooltexture "water/water_still_frame00"

    $abovewater 1
    $bottommaterial "water/water_still_beneath"
    
    $fogenable 1
    $fogcolor "{5 5 51}"
    $fogstart 0
    $fogend 200
    $lightmapwaterfog 1
    $flashlighttint 1
    
    Proxies
    {
        AnimatedTexture
        {
            animatedTextureVar $basetexture
            animatedTextureFrameNumVar $frame
            animatedTextureFrameRate 10
        }
    }
}
```

## Installation

Add the following to your project's `Cargo.toml`:

```toml
[dependencies]
serde = { version = "1.0.0", features = ["derive"] }
vdflex = "1.0.0"
```

## Feature Flags

- `default`: `["std"]`
- `no_std`: Don't use the standard library
- `preserve_order`: Preserve entry insertion order 

## Quick Start

```rust
use std::collections::BTreeMap;
use std::hash::Hash;
use serde::Serialize;

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize)]
struct AppId(u32);

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct AppBuild {
    #[serde(rename = "AppID")]
    app_id: AppId,
    desc: String,
    content_root: String,
    build_output: String,
    depots: BTreeMap<AppId, Depot>
}

#[derive(Serialize)]
struct Depot {
    #[serde(rename = "FileMapping")]
    file_mappings: Vec<FileMapping>
}

#[derive(Serialize)]
struct FileMapping {
    #[serde(rename = "LocalPath")]
    local_path: String,
    #[serde(rename = "DepotPath")]
    depot_path: String,
}

fn main() -> vdflex::Result<()> {
    let mut depots = BTreeMap::new();
    depots.insert(AppId(1234), Depot {
        file_mappings: vec![FileMapping { 
            local_path: String::from("*"),
            depot_path: String::from("."),
        }],
    });
    
    let build_script = AppBuild {
        app_id: AppId(1234),
        desc: String::from("My SteamPipe build script"),
        content_root: String::from("..\\assets\\"),
        build_output: String::from("..\\build\\"),
        depots,
    };
    
    let text: String = vdflex::kv_to_string("AppBuild", &build_script)?;
    println!("{text}");
    // "AppBuild"
    // {
    //     "AppID" "1234"
    //     "Desc" "My SteamPipe build script"
    //     "ContentRoot" "..\assets\"
    //     "BuildOutput" "..\build\"
    //     "Depots"
    //     {
    //         "1234"
    //         {
    //             "FileMapping"
    //             {
    //                 "LocalPath" "*"
    //                 "DepotPath" "."
    //             }
    //         }
    //     }
    // }
    
    Ok(())
}
```

## Supported Types

KeyValues is woefully underspecified, but in general it only supports strings and multimaps (objects). VDFLex attempts
to support every Rust type, but not all types necessarily have an "idiomatic" or "useful" representation. These are 
the types that VDFlex supports and how they are represented in KeyValues:


|              Type              | Notes                                                                                                  |
|:------------------------------:|:-------------------------------------------------------------------------------------------------------|
|             `bool`             | Serialized to `1` or `0`                                                                               |
|            integers            | KeyValues doesn't typically support `i128` or `u128`                                                   |
|          `f32`/`f64`           | Some implementations only support `f32`. Non-finite floats are also poorly supported.                  |
|     `char`/`String`/`str`      | -                                                                                                      |
|            `Option`            | KeyValues has no equivalent of `null`, so `Some<T>` is represented as `T` and `None` is simply omitted |
|       Unit/Unit Structs        | Serialized like `None`                                                                                 |
|         Unit Variants          | Represented as a string matching the name of the variant                                               |
|        Newtype Structs         | Represented as the wrapped type                                                                        |
|        Newtype Variants        | Represented as an object mapping the variant name to the wrapped type                                  |
| Sequences/Tuples/Tuple Structs | Represented by repeating the key for each element in the sequence                                      |
|         Tuple Variants         | Represented by a map containing a sequence of the tuple's fields, using the variant name as the key    |
|          Maps/Structs          | Represented by objects (a curly bracket-enclosed list of key-value pairs)                              |
|        Struct Variants         | Represented as an object mapping the variant name to the struct representation of its fields           |

### Limitations

- The *Bytes* type is unsupported, as there is no clear way to represent binary data in KeyValues. 
- Sequences are weird. It's not possible to serialize top-level or nested sequences. See 
  [`Error::UnrepresentableSequence`] for more. 

## Missing Features

This library is in an early state. As such, many features have not yet been implemented. 
Some missing features include: 

- Deserialization
  - Text parsing
  - Conversion to Rust types
- An easier API for [`Object`]
- A `keyvalues!` macro to create [`Object`]s
- Conditional tags
  - The [`ser::Formatter`] API supports conditional tags, but this is unsupported for the
    serde API.
- `#base` and `#include` directives
  - The [`ser::Formatter`] API supports macro formatting, but the serde API treats
    macros like normal fields. 

## License

This library is licensed under the MIT license (<https://opensource.org/license/MIT>).
