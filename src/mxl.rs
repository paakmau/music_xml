use std::io::{self, Read};

use roxmltree::Document;
use zip::ZipArchive;

use crate::{
    error::{
        Error::{AttrNotFound, NodeNotFound},
        Result,
    },
    score::Score,
};

pub struct Mxl<R> {
    archive: ZipArchive<R>,
}

impl<R: Read + io::Seek> Mxl<R> {
    pub fn new(reader: R) -> Result<Mxl<R>> {
        Ok(Mxl {
            archive: ZipArchive::new(reader)?,
        })
    }

    pub fn parse_music_xml(&mut self) -> Result<Score> {
        let path = Self::parse_music_xml_path(&mut self.archive)?;

        let xml = Self::extra_text_file(&mut self.archive, &path)?;

        Score::from_xml(&xml)
    }

    fn extra_text_file(archive: &mut ZipArchive<R>, path: &str) -> Result<String> {
        let mut f = archive.by_name(path)?;
        let mut buf = Default::default();
        f.read_to_string(&mut buf)?;

        Ok(buf)
    }

    fn parse_music_xml_path(archive: &mut ZipArchive<R>) -> Result<String> {
        // TODO: should consider wrapping it to a struct
        let xml = Self::extra_text_file(archive, "META-INF/container.xml")?;

        let doc = Document::parse(&xml)?;
        let root = doc.root_element();
        let rootfiles = root
            .children()
            .find(|c| c.tag_name().name() == "rootfiles")
            .ok_or(NodeNotFound {
                tag: "rootfiles",
                parent_tag: root.tag_name().name().to_owned(),
            })?;
        rootfiles
            .children()
            .filter(|c| c.tag_name().name() == "rootfile")
            .find(|c| {
                matches!(
                    c.attribute("media-type"),
                    Some("application/vnd.recordare.musicxml+xml") | None
                )
            })
            .ok_or(NodeNotFound {
                tag: "rootfile",
                parent_tag: rootfiles.tag_name().name().to_owned(),
            })?
            .attribute("full-path")
            .map(str::to_owned)
            .ok_or(AttrNotFound {
                attr: "full-path",
                tag: "rootfile".to_owned(),
            })
    }
}
