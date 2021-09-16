use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Trak {
    pub tkhd: Option<Tkhd>,
    pub tref: Option<Tref>,
    pub mdia: Option<Mdia>,
}

impl Atom for Trak {
    const FOURCC: Fourcc = TRACK;
}

impl ParseAtom for Trak {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ReadConfig,
        size: Size,
    ) -> crate::Result<Self> {
        let mut trak = Self::default();
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                TRACK_HEADER if cfg.read_chapters => {
                    trak.tkhd = Some(Tkhd::parse(reader, cfg, head.size())?)
                }
                TRACK_REFERENCE if cfg.read_chapters => {
                    trak.tref = Some(Tref::parse(reader, cfg, head.size())?)
                }
                MEDIA if cfg.read_chapters || cfg.read_audio_info => {
                    trak.mdia = Some(Mdia::parse(reader, cfg, head.size())?)
                }
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        Ok(trak)
    }
}

impl WriteAtom for Trak {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        if let Some(a) = &self.tkhd {
            a.write(writer)?;
        }
        if let Some(a) = &self.tref {
            a.write(writer)?;
        }
        if let Some(a) = &self.mdia {
            a.write(writer)?;
        }
        Ok(())
    }

    fn size(&self) -> Size {
        let content_len =
            self.tkhd.len_or_zero() + self.tref.len_or_zero() + self.mdia.len_or_zero();
        Size::from(content_len)
    }
}

#[derive(Default)]
pub struct TrakBounds {
    pub bounds: AtomBounds,
    pub tkhd: Option<Tkhd>,
    pub tref: Option<TrefBounds>,
    pub mdia: Option<MdiaBounds>,
}

impl FindAtom for Trak {
    type Bounds = TrakBounds;

    fn find_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self::Bounds> {
        let bounds = find_bounds(reader, size)?;
        let mut trak = TrakBounds { bounds, ..Default::default() };
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                TRACK_HEADER => trak.tkhd = Some(Tkhd::parse(reader, &READ_CONFIG, head.size())?),
                TRACK_REFERENCE => trak.tref = Some(Tref::find(reader, head.size())?),
                MEDIA => trak.mdia = Some(Mdia::find(reader, head.size())?),
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        Ok(trak)
    }
}
