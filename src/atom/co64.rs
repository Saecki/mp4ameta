use super::*;

/// A struct representing of a 64bit sample table chunk offset atom (`co64`).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Co64 {
    pub state: State,
    pub offsets: Vec<u64>,
}

impl Atom for Co64 {
    const FOURCC: Fourcc = SAMPLE_TABLE_CHUNK_OFFSET_64;
}

impl ParseAtom for Co64 {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ReadConfig,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let (version, _) = parse_full_head(reader)?;

        if version != 0 {
            return Err(crate::Error::new(
                crate::ErrorKind::UnknownVersion(version),
                "Unknown 64bit sample table chunk offset (co64) version",
            ));
        }

        let entries = reader.read_be_u32()?;
        if 8 + 8 * entries as u64 != size.content_len() {
            return Err(crate::Error::new(
                crate::ErrorKind::Parsing,
                "Sample table chunk offset 64 (co64) offset table size doesn't match atom length",
            ));
        }

        let mut offsets = Vec::with_capacity(entries as usize);
        for _ in 0..entries {
            let offset = reader.read_be_u64()?;
            offsets.push(offset);
        }

        Ok(Self { state: State::Existing(bounds), offsets })
    }
}

impl WriteAtom for Co64 {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        write_full_head(writer, 0, [0; 3])?;

        writer.write_be_u32(self.offsets.len() as u32)?;
        for o in self.offsets.iter() {
            writer.write_be_u64(*o)?;
        }

        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = 8 + 8 * self.offsets.len() as u64;
        Size::from(content_len)
    }
}

impl SimpleCollectChanges for Co64 {
    fn state(&self) -> &State {
        &self.state
    }

    fn existing<'a>(
        &'a self,
        _level: u8,
        bounds: &'a AtomBounds,
        changes: &mut Vec<Change<'a>>,
    ) -> i64 {
        changes.push(Change::UpdateChunkOffset(UpdateChunkOffsets {
            bounds,
            offsets: ChunkOffsets::Co64(&self.offsets),
        }));
        0
    }

    fn atom_ref(&self) -> AtomRef {
        AtomRef::Co64(self)
    }
}
