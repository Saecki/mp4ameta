# rust-mp4ameta
[![Crate](https://img.shields.io/crates/v/mp4ameta.svg)](https://crates.io/crates/mp4ameta)
[![Documentation](https://docs.rs/mp4ameta/badge.svg)](https://docs.rs/mp4ameta)
![CI](https://github.com/Saecki/rust-mp4ameta/workflows/CI/badge.svg)

A library for reading and writing iTunes style MPEG-4 audio metadata.

## Usage
```rust
fn main() {
  	let mut tag = mp4ameta::Tag::read_from_path("music.m4a").unwrap();

  	println!("{}", tag.artist().unwrap());

  	tag.set_artist("<artist>");

  	tag.write_to_path("music.m4a").unwrap();
}
```

## Supported Filetypes
- M4A
- M4B
- M4P
- M4V

## Useful Links
- [AtomicParsley Doc](http://atomicparsley.sourceforge.net/mpeg-4files.html)
- [Mutagen Doc](https://mutagen.readthedocs.io/en/latest/api/mp4.html)
- QuickTime Doc 
    - [Movie Atoms](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFChap2/qtff2.html)
    - [Metadata](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html)
