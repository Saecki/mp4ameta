# mp4ameta
[![Build](https://github.com/Saecki/mp4ameta/actions/workflows/build.yml/badge.svg)](https://github.com/Saecki/mp4ameta/actions/workflows/build.yml)
[![Style](https://github.com/Saecki/mp4ameta/actions/workflows/style.yml/badge.svg)](https://github.com/Saecki/mp4ameta/actions/workflows/style.yml)
[![Crate](https://img.shields.io/crates/v/mp4ameta.svg)](https://crates.io/crates/mp4ameta)
[![Documentation](https://img.shields.io/docsrs/mp4ameta?label=docs.rs)](https://docs.rs/mp4ameta)
![License](https://img.shields.io/crates/l/mp4ameta)
![LOC](https://tokei.rs/b1/github/saecki/mp4ameta?category=code)

A library for reading and writing iTunes style MPEG-4 audio metadata.
Most commonly this kind of metadata is found inside `m4a` or `m4b` files but basically any `mp4` container supports it.

## Examples

### The easy way
```rust
let mut tag = mp4ameta::Tag::read_from_path("music.m4a").unwrap();

println!("{}", tag.artist().unwrap());

tag.set_artist("artist");
tag.write_to_path("music.m4a").unwrap();
```

### The hard way
```rust
use mp4ameta::{Data, Fourcc, Tag};

let mut tag = Tag::read_from_path("music.m4a").unwrap();
let artist_ident = Fourcc(*b"\xa9ART");

let artist = tag.strings_of(&artist_ident).next().unwrap();
println!("{}", artist);

tag.set_data(artist_ident, Data::Utf8("artist".to_owned()));
tag.write_to_path("music.m4a").unwrap();
```

### Using freeform identifiers
```rust
use mp4ameta::{Data, FreeformIdent, Tag};

let mut tag = Tag::read_from_path("music.m4a").unwrap();
let isrc_ident = FreeformIdent::new("com.apple.iTunes", "ISRC");

let isrc = tag.strings_of(&isrc_ident).next().unwrap();
println!("{}", isrc);

tag.set_data(isrc_ident, Data::Utf8("isrc".to_owned()));
tag.write_to_path("music.m4a").unwrap();
```

## Useful Links
- QuickTime spec
    - [Overview of QTFF](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFChap1/qtff1.html)
    - [Movie Atoms](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFChap2/qtff2.html)
    - [Metadata](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html)
- [MultimediaWiki QuickTime container](https://wiki.multimedia.cx/index.php/QuickTime_container)
- [AtomicParsley docs](http://atomicparsley.sourceforge.net/mpeg-4files.html)
- [Mutagen docs](https://mutagen.readthedocs.io/en/latest/api/mp4.html)
- [Hydrogen audio tag mapping](https://wiki.hydrogenaud.io/index.php?title=Tag_Mapping)
- [MusicBrainz Picard tag mapping](https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html)
- [Filetype list](https://ftyps.com/)

## Testing
__Run all tests:__<br/>
`cargo test`

__Test this library on your collection:__<br/>
`cargo test -- --nocapture collection <path>`

