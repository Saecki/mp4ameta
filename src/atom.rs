use std::fmt::{Debug, Display, Formatter, Result};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::ops::Deref;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{Content, data, Data, ErrorKind, Tag};
use crate::content::ContentT;
use crate::data::DataT;

/// A list of valid file types defined by the `ftyp` atom.
pub const VALID_FILE_TYPES: [&str; 5] = ["M4A ", "M4B ", "M4P ", "M4V ", "isom"];

/// Identifier of an atom information about the filetype.
pub const FILE_TYPE: Ident = Ident(*b"ftyp");
/// Identifier of an atom containing a sturcture of children storing metadata.
pub const MOVIE: Ident = Ident(*b"moov");
/// Identifier of an atom containing information about a single track.
pub const TRACK: Ident = Ident(*b"trak");
/// Identifier of an atom containing inforamtion about a tracks media type and data.
pub const MEDIA: Ident = Ident(*b"mdia");
/// Identifier of an atom specifying the characteristics of a media atom.
pub const MEDIA_HEADER: Ident = Ident(*b"mdhd");
/// Identifier of an atom containing user metadata.
pub const USER_DATA: Ident = Ident(*b"udta");
/// Identifier of an atom containing a metadata item list.
pub const METADATA: Ident = Ident(*b"meta");
/// Identifier of an atom containing a list of metadata atoms.
pub const ITEM_LIST: Ident = Ident(*b"ilst");
/// Identifier of an atom containing typed data.
pub const DATA: Ident = Ident(*b"data");

// iTunes 4.0 atoms
pub const ALBUM: Ident = Ident(*b"\xa9alb");
pub const ALBUM_ARTIST: Ident = Ident(*b"aART");
pub const ARTIST: Ident = Ident(*b"\xa9ART");
pub const ARTWORK: Ident = Ident(*b"covr");
pub const BPM: Ident = Ident(*b"tmpo");
pub const COMMENT: Ident = Ident(*b"\xa9cmt");
pub const COMPILATION: Ident = Ident(*b"cpil");
pub const COMPOSER: Ident = Ident(*b"\xa9wrt");
pub const COPYRIGHT: Ident = Ident(*b"cprt");
pub const CUSTOM_GENRE: Ident = Ident(*b"\xa9gen");
pub const DISK_NUMBER: Ident = Ident(*b"disk");
pub const ENCODER: Ident = Ident(*b"\xa9too");
pub const ADVISORY_RATING: Ident = Ident(*b"rtng");
pub const STANDARD_GENRE: Ident = Ident(*b"gnre");
pub const TITLE: Ident = Ident(*b"\xa9nam");
pub const TRACK_NUMBER: Ident = Ident(*b"trkn");
pub const YEAR: Ident = Ident(*b"\xa9day");

// iTunes 4.2 atoms
pub const GROUPING: Ident = Ident(*b"\xa9grp");
pub const MEDIA_TYPE: Ident = Ident(*b"stik");

// iTunes 4.9 atoms
pub const CATEGORY: Ident = Ident(*b"catg");
pub const KEYWORD: Ident = Ident(*b"keyw");
pub const PODCAST: Ident = Ident(*b"pcst");
pub const PODCAST_EPISODE_GLOBAL_UNIQUE_ID: Ident = Ident(*b"egid");
pub const PODCAST_URL: Ident = Ident(*b"purl");

// iTunes 5.0
pub const DESCRIPTION: Ident = Ident(*b"desc");
pub const LYRICS: Ident = Ident(*b"\xa9lyr");

// iTunes 6.0
pub const TV_EPISODE: Ident = Ident(*b"tves");
pub const TV_EPISODE_NUMBER: Ident = Ident(*b"tven");
pub const TV_NETWORK_NAME: Ident = Ident(*b"tvnn");
pub const TV_SEASON: Ident = Ident(*b"tvsn");
pub const TV_SHOW_NAME: Ident = Ident(*b"tvsh");

// iTunes 6.0.2
pub const PURCHASE_DATE: Ident = Ident(*b"purd");

// iTunes 7.0
pub const GAPLESS_PLAYBACK: Ident = Ident(*b"pgap");

// Work, Movement
pub const MOVEMENT_NAME: Ident = Ident(*b"\xa9mvn");
pub const MOVEMENT_COUNT: Ident = Ident(*b"\xa9mvc");
pub const MOVEMENT_INDEX: Ident = Ident(*b"\xa9mvi");
pub const WORK: Ident = Ident(*b"\xa9wrk");
pub const SHOW_MOVEMENT: Ident = Ident(*b"shwm");

/// A 4 byte atom identifier.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ident(pub [u8; 4]);

impl Deref for Ident {
    type Target = [u8; 4];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Ident {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0.iter().map(|b| char::from(*b)).collect::<String>())
    }
}

/// A structure that represents a MPEG-4 audio metadata atom.
#[derive(Clone, PartialEq)]
pub struct Atom {
    /// The 4 byte identifier of the atom.
    pub ident: Ident,
    /// The offset in bytes separating the head from the content.
    pub offset: usize,
    /// The content of an atom.
    pub content: Content,
}

impl Debug for Atom {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Atom{{ {}, {}, {:#?} }}", self.ident, self.offset, self.content)
    }
}

impl Atom {
    /// Creates an atom containing the provided content at a n byte offset.
    pub fn with(ident: Ident, offset: usize, content: Content) -> Self {
        Self { ident, offset, content }
    }

    /// Creates an atom containing `Content::RawData` with the provided data.
    pub fn with_raw_data(ident: Ident, offset: usize, data: Data) -> Self {
        Self::with(ident, offset, Content::RawData(data))
    }

    /// Creates an atom containing `Content::TypedData` with the provided data.
    pub fn with_typed_data(ident: Ident, offset: usize, data: Data) -> Self {
        Self::with(ident, offset, Content::TypedData(data))
    }

    /// Creates a data atom containing `Content::TypedData` with the provided data.
    pub fn data_atom_with(data: Data) -> Self {
        Self::with(DATA, 0, Content::TypedData(data))
    }

    /// Returns the length in bytes.
    pub fn len(&self) -> usize {
        8 + self.offset + self.content.len()
    }

    /// Returns true if the atom has no content and only consists of it's 8 byte head.
    pub fn is_empty(&self) -> bool {
        self.offset + self.content.len() == 0
    }

    /// Returns a reference to the first children atom matching the identifier, if present.
    pub fn child(&self, ident: Ident) -> Option<&Self> {
        if let Content::Atoms(v) = &self.content {
            for a in v {
                if a.ident == ident {
                    return Some(a);
                }
            }
        }

        None
    }

    /// Returns a mutable reference to the first children atom matching the identifier, if present.
    pub fn mut_child(&mut self, ident: Ident) -> Option<&mut Self> {
        if let Content::Atoms(v) = &mut self.content {
            for a in v {
                if a.ident == ident {
                    return Some(a);
                }
            }
        }

        None
    }

    /// Return a reference to the first children atom, if present.
    pub fn first_child(&self) -> Option<&Self> {
        match &self.content {
            Content::Atoms(v) => v.first(),
            _ => None,
        }
    }

    /// Returns a mutable reference to the first children atom, if present.
    pub fn mut_first_child(&mut self) -> Option<&mut Self> {
        match &mut self.content {
            Content::Atoms(v) => v.first_mut(),
            _ => None,
        }
    }

    /// Attempts to write the atom to the writer.
    pub fn write_to(&self, writer: &mut impl Write) -> crate::Result<()> {
        writer.write_u32::<BigEndian>(self.len() as u32)?;
        writer.write_all(&*self.ident)?;
        writer.write_all(&vec![0u8; self.offset])?;

        self.content.write_to(writer)?;

        Ok(())
    }

    /// Checks if the filetype is valid, returns an error otherwise.
    pub fn check_filetype(&self) -> crate::Result<()> {
        match &self.content {
            Content::RawData(Data::Utf8(s)) => {
                for f in &VALID_FILE_TYPES {
                    if s.starts_with(f) {
                        return Ok(());
                    }
                }

                Err(crate::Error::new(
                    ErrorKind::InvalidFiletype(s.clone()),
                    "Invalid filetype.".into(),
                ))
            }
            _ => Err(crate::Error::new(
                ErrorKind::NoTag,
                "No filetype atom found.".into(),
            )),
        }
    }
}

/// A template representing a MPEG-4 audio metadata atom.
#[derive(Clone, PartialEq)]
pub struct AtomT {
    /// The 4 byte identifier of the atom.
    pub ident: Ident,
    /// The offset in bytes separating the head from the content.
    pub offset: usize,
    /// The content template of an atom template.
    pub content: ContentT,
}

impl Debug for AtomT {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Atom{{ {}, {}, {:#?} }}", self.ident, self.offset, self.content)
    }
}

impl AtomT {
    /// Creates an atom template containing the provided content at a n byte offset.
    pub fn with(ident: Ident, offset: usize, content: ContentT) -> Self {
        Self { ident, offset, content }
    }

    /// Creates an atom template containing `ContentT::RawData` with the provided data template.
    pub fn with_raw_data(ident: Ident, offset: usize, data: DataT) -> Self {
        Self::with(ident, offset, ContentT::RawData(data))
    }

    /// Creates a data atom template containing `ContentT::TypedData`.
    pub fn data_atom() -> Self {
        Self::with(DATA, 0, ContentT::TypedData)
    }

    /// Returns a reference to the first children atom template matching the identifier, if present.
    pub fn child(&self, ident: Ident) -> Option<&Self> {
        if let ContentT::Atoms(v) = &self.content {
            for a in v {
                if a.ident == ident {
                    return Some(a);
                }
            }
        }

        None
    }

    /// Returns a mutable reference to the first children atom template matching the identifier, if
    /// present.
    pub fn mut_child(&mut self, ident: Ident) -> Option<&mut Self> {
        if let ContentT::Atoms(v) = &mut self.content {
            for a in v {
                if a.ident == ident {
                    return Some(a);
                }
            }
        }

        None
    }

    /// Returns a reference to the first children atom template, if present.
    pub fn first_child(&self) -> Option<&Self> {
        match &self.content {
            ContentT::Atoms(v) => v.first(),
            _ => None,
        }
    }

    /// Returns a mutable reference to the first children atom template, if present.
    pub fn mut_first_child(&mut self) -> Option<&mut Self> {
        match &mut self.content {
            ContentT::Atoms(v) => v.first_mut(),
            _ => None,
        }
    }

    /// Attempts to parse an atom, matching the template, from the reader. This should only be used
    /// if the atom has to be in this exact position, if the parsed atom's identifier doesn't match
    /// the expected one this will return an error.
    pub fn parse(&self, reader: &mut (impl Read + Seek)) -> crate::Result<Atom> {
        let (length, ident) = match parse_head(reader) {
            Ok(h) => h,
            Err(e) => return Err(e),
        };

        if ident == self.ident {
            match self.parse_content(reader, length) {
                Ok(c) => Ok(Atom::with(self.ident, self.offset, c)),
                Err(e) => Err(crate::Error::new(
                    e.kind,
                    format!(
                        "Error reading {}: {}",
                        ident,
                        e.description
                    ),
                )),
            }
        } else {
            Err(crate::Error::new(
                ErrorKind::AtomNotFound(self.ident),
                format!(
                    "Expected {} found {}",
                    self.ident,
                    ident
                ),
            ))
        }
    }

    /// Attempts to parse the list of atoms, matching the templates, from the reader.
    pub fn parse_atoms(atoms: &[AtomT], reader: &mut (impl Read + Seek), length: usize) -> crate::Result<Vec<Atom>> {
        let mut parsed_bytes = 0;
        let mut parsed_atoms = Vec::with_capacity(atoms.len());

        while parsed_bytes < length {
            let (atom_length, atom_ident) = parse_head(reader)?;

            let mut parsed = false;
            for a in atoms {
                if atom_ident == a.ident {
                    match a.parse_content(reader, atom_length) {
                        Ok(c) => {
                            parsed_atoms.push(Atom::with(a.ident, a.offset, c));
                            parsed = true;
                        }
                        Err(e) => {
                            return Err(crate::Error::new(
                                e.kind,
                                format!(
                                    "Error reading {}: {}",
                                    atom_ident,
                                    e.description
                                ),
                            ));
                        }
                    }
                    break;
                }
            }

            if atom_length > 8 && !parsed {
                reader.seek(SeekFrom::Current((atom_length - 8) as i64))?;
            }

            parsed_bytes += atom_length;
        }

        if parsed_bytes < length {
            reader.seek(SeekFrom::Current((length - parsed_bytes) as i64))?;
        }

        Ok(parsed_atoms)
    }

    /// Attempts to parse the atom template's content from the reader.
    pub fn parse_content(&self, reader: &mut (impl Read + Seek), length: usize) -> crate::Result<Content> {
        if length > 8 {
            if self.offset != 0 {
                reader.seek(SeekFrom::Current(self.offset as i64))?;
            }
            self.content.parse(reader, length - 8 - self.offset)
        } else {
            Ok(Content::Empty)
        }
    }
}

/// Attempts to read MPEG-4 audio metadata from the reader.
pub fn read_tag_from(reader: &mut (impl Read + Seek)) -> crate::Result<Tag> {
    let mut tag_atoms = Vec::new();
    let mut tag_readonly_atoms = Vec::new();

    let ftyp = filetype_atom().parse(reader)?;
    ftyp.check_filetype()?;
    tag_readonly_atoms.push(ftyp);

    let current_position = reader.seek(SeekFrom::Current(0))?;
    let complete_length = reader.seek(SeekFrom::End(0))?;
    let length = (complete_length - current_position) as usize;

    reader.seek(SeekFrom::Start(current_position))?;

    let moov_atoms = AtomT::parse_atoms(&[metadata_atom()], reader, length)?;
    let moov = match moov_atoms.first() {
        Some(m) => m,
        None => {
            return Err(crate::Error::new(
                ErrorKind::AtomNotFound(MOVIE),
                "No moov atom found".into(),
            ));
        }
    };

    if let Some(trak) = moov.child(TRACK) {
        if let Some(mdia) = trak.child(MEDIA) {
            if let Some(mdhd) = mdia.child(MEDIA_HEADER) {
                tag_readonly_atoms.push(mdhd.clone());
            }
        }
    }

    if let Some(udta) = moov.child(USER_DATA) {
        if let Some(meta) = udta.first_child() {
            if let Some(ilst) = meta.first_child() {
                if let Content::Atoms(atoms) = &ilst.content {
                    tag_atoms = atoms.to_vec();
                }
            }
        }
    }

    Ok(Tag::with(tag_atoms, tag_readonly_atoms))
}

/// Attempts to write the metadata atoms to the file inside the item list atom.
pub fn write_tag_to(file: &File, atoms: &[Atom]) -> crate::Result<()> {
    let mut reader = BufReader::new(file);
    let mut writer = BufWriter::new(file);

    let mut atom_pos_and_len = Vec::new();
    let mut destination = &item_list_atom_t();
    let ftyp = filetype_atom().parse(&mut reader)?;
    ftyp.check_filetype()?;

    while let Ok((length, ident)) = parse_head(&mut reader) {
        if ident == destination.ident {
            let pos = reader.seek(SeekFrom::Current(0))? as usize - 8;
            atom_pos_and_len.push((pos, length));

            reader.seek(SeekFrom::Current(destination.offset as i64))?;

            match destination.first_child() {
                Some(a) => destination = a,
                None => break,
            }
        } else {
            reader.seek(SeekFrom::Current(length as i64 - 8))?;
        }
    }

    let old_file_length = reader.seek(SeekFrom::End(0))?;
    let metadata_position = atom_pos_and_len[atom_pos_and_len.len() - 1].0 + 8;
    let old_metadata_length = atom_pos_and_len[atom_pos_and_len.len() - 1].1 - 8;
    let new_metadata_length = atoms.iter().map(|a| a.len()).sum::<usize>();
    let metadata_length_difference = new_metadata_length as i32 - old_metadata_length as i32;

    // reading additional data after metadata
    let mut additional_data = Vec::with_capacity(old_file_length as usize - (metadata_position + old_metadata_length));
    reader.seek(SeekFrom::Start((metadata_position + old_metadata_length) as u64))?;
    reader.read_to_end(&mut additional_data)?;

    // adjusting the file length
    file.set_len((old_file_length as i64 + metadata_length_difference as i64) as u64)?;

    // adjusting the atom lengths
    for (pos, len) in atom_pos_and_len {
        writer.seek(SeekFrom::Start(pos as u64))?;
        writer.write_u32::<BigEndian>((len as i32 + metadata_length_difference) as u32)?;
    }

    // writing metadata
    writer.seek(SeekFrom::Current(4))?;
    for a in atoms {
        a.write_to(&mut writer)?;
    }

    // writing additional data after metadata
    writer.write_all(&additional_data)?;
    writer.flush()?;

    Ok(())
}

/// Attempts to parse the atom's head containing a 32 bit unsigned integer determining the size
/// of the atom in bytes and the following 4 byte identifier from the reader.
pub fn parse_head(reader: &mut (impl Read + Seek)) -> crate::Result<(usize, Ident)> {
    let length = match reader.read_u32::<BigEndian>() {
        Ok(l) => l as usize,
        Err(e) => {
            return Err(crate::Error::new(
                ErrorKind::Io(e),
                "Error reading atom length".into(),
            ));
        }
    };
    let mut ident = [0u8; 4];
    if let Err(e) = reader.read_exact(&mut ident) {
        return Err(crate::Error::new(
            ErrorKind::Io(e),
            "Error reading atom identifier".into(),
        ));
    }

    Ok((length, Ident(ident)))
}

/// Returns a atom filetype hierarchy needed to parse the filetype.
pub fn filetype_atom() -> AtomT {
    AtomT::with_raw_data(FILE_TYPE, 0, DataT::with(data::UTF8))
}

/// Returns a atom metadata hierarchy template needed to parse metadata.
pub fn metadata_atom() -> AtomT {
    AtomT::with(MOVIE, 0, ContentT::atoms_t()
        .add_atom_t_with(TRACK, 0, ContentT::atoms_t()
            .add_atom_t_with(MEDIA, 0, ContentT::atoms_t()
                .add_atom_t_with(MEDIA_HEADER, 0, ContentT::RawData(
                    DataT::with(data::RESERVED)
                )),
            ),
        )
        .add_atom_t_with(USER_DATA, 0, ContentT::atoms_t()
            .add_atom_t_with(METADATA, 4, ContentT::atoms_t()
                .add_atom_t_with(ITEM_LIST, 0, ContentT::atoms_t()
                    .add_atom_t_with(ADVISORY_RATING, 0, ContentT::data_atom_t())
                    .add_atom_t_with(ALBUM, 0, ContentT::data_atom_t())
                    .add_atom_t_with(ALBUM_ARTIST, 0, ContentT::data_atom_t())
                    .add_atom_t_with(ARTIST, 0, ContentT::data_atom_t())
                    .add_atom_t_with(BPM, 0, ContentT::data_atom_t())
                    .add_atom_t_with(CATEGORY, 0, ContentT::data_atom_t())
                    .add_atom_t_with(COMMENT, 0, ContentT::data_atom_t())
                    .add_atom_t_with(COMPILATION, 0, ContentT::data_atom_t())
                    .add_atom_t_with(COMPOSER, 0, ContentT::data_atom_t())
                    .add_atom_t_with(COPYRIGHT, 0, ContentT::data_atom_t())
                    .add_atom_t_with(CUSTOM_GENRE, 0, ContentT::data_atom_t())
                    .add_atom_t_with(DESCRIPTION, 0, ContentT::data_atom_t())
                    .add_atom_t_with(DISK_NUMBER, 0, ContentT::data_atom_t())
                    .add_atom_t_with(ENCODER, 0, ContentT::data_atom_t())
                    .add_atom_t_with(GAPLESS_PLAYBACK, 0, ContentT::data_atom_t())
                    .add_atom_t_with(GROUPING, 0, ContentT::data_atom_t())
                    .add_atom_t_with(KEYWORD, 0, ContentT::data_atom_t())
                    .add_atom_t_with(LYRICS, 0, ContentT::data_atom_t())
                    .add_atom_t_with(MEDIA_TYPE, 0, ContentT::data_atom_t())
                    .add_atom_t_with(MOVEMENT_COUNT, 0, ContentT::data_atom_t())
                    .add_atom_t_with(MOVEMENT_INDEX, 0, ContentT::data_atom_t())
                    .add_atom_t_with(MOVEMENT_NAME, 0, ContentT::data_atom_t())
                    .add_atom_t_with(PODCAST, 0, ContentT::data_atom_t())
                    .add_atom_t_with(PODCAST_EPISODE_GLOBAL_UNIQUE_ID, 0, ContentT::data_atom_t())
                    .add_atom_t_with(PODCAST_URL, 0, ContentT::data_atom_t())
                    .add_atom_t_with(PURCHASE_DATE, 0, ContentT::data_atom_t())
                    .add_atom_t_with(SHOW_MOVEMENT, 0, ContentT::data_atom_t())
                    .add_atom_t_with(STANDARD_GENRE, 0, ContentT::data_atom_t())
                    .add_atom_t_with(TITLE, 0, ContentT::data_atom_t())
                    .add_atom_t_with(TRACK_NUMBER, 0, ContentT::data_atom_t())
                    .add_atom_t_with(TV_EPISODE, 0, ContentT::data_atom_t())
                    .add_atom_t_with(TV_EPISODE_NUMBER, 0, ContentT::data_atom_t())
                    .add_atom_t_with(TV_NETWORK_NAME, 0, ContentT::data_atom_t())
                    .add_atom_t_with(TV_SEASON, 0, ContentT::data_atom_t())
                    .add_atom_t_with(TV_SHOW_NAME, 0, ContentT::data_atom_t())
                    .add_atom_t_with(WORK, 0, ContentT::data_atom_t())
                    .add_atom_t_with(YEAR, 0, ContentT::data_atom_t())
                    .add_atom_t_with(ARTWORK, 0, ContentT::data_atom_t(),
                    ),
                ),
            ),
        ),
    )
}

/// Returns a atom metadata hierarchy.
pub fn item_list_atom_t() -> AtomT {
    AtomT::with(MOVIE, 0, ContentT::atom_t_with(
        USER_DATA, 0, ContentT::atom_t_with(
            METADATA, 4, ContentT::atom_t_with(
                ITEM_LIST, 0, ContentT::atoms_t(),
            ),
        ),
    ))
}

