use std::{any::type_name, str::FromStr};

use roxmltree::{Document, Node};

use crate::error::Error::{
    AttrNotFound, AttrValueParseFailed, DuplicatedNodesFound, ExclusiveNodeFound,
    ExclusiveNodeGroupNotFound, NodeNotFound, NodeTextEmpty, NodeTextParseFailed,
};
use crate::error::Result;

trait FromNode: Sized {
    fn tag() -> &'static str;
    fn from_node(node: &Node) -> Result<Self>;
}

fn parse_children<T: FromNode>(node: &Node) -> Result<Vec<T>> {
    node.children()
        .filter(|c| c.tag_name().name() == T::tag())
        .map(|c| T::from_node(&c))
        .collect::<Result<Vec<T>>>()
}

fn parse_option_chd<T: FromNode>(node: &Node) -> Result<Option<T>> {
    node.children()
        .find(|c| c.tag_name().name() == T::tag())
        .as_ref()
        .map(T::from_node)
        .transpose()
}

pub fn parse_optional_attr<T: FromStr>(node: &Node, attr: &str) -> Result<Option<T>> {
    match node.attribute(attr) {
        Some(v) => Some(v)
            .map(T::from_str)
            .transpose()
            .map_err(|_| AttrValueParseFailed {
                attr: attr.to_owned(),
                tag: node.tag_name().name().to_owned(),
                v: v.to_owned(),
                ty: type_name::<T>(),
            }),
        None => Ok(None),
    }
}

pub fn parse_attr<T: FromStr>(node: &Node, attr: &'static str) -> Result<T> {
    parse_optional_attr(node, attr)
        .transpose()
        .ok_or(AttrNotFound {
            attr,
            tag: node.tag_name().name().to_owned(),
        })?
}

pub fn parse_optional_chd_text<T: FromStr>(node: &Node, name: &'static str) -> Result<Option<T>> {
    match node
        .children()
        .filter(|c| c.tag_name().name() == name)
        .count()
    {
        1 => {}
        0 => return Ok(None),
        _ => {
            return Err(DuplicatedNodesFound {
                tag: name,
                parent_tag: node.tag_name().name().to_owned(),
            })
        }
    };

    let text = node
        .children()
        .find(|c| c.tag_name().name() == name)
        .unwrap()
        .text()
        .ok_or(NodeTextEmpty { tag: name })?
        .to_owned();

    Some(text.parse().map_err(|_| NodeTextParseFailed {
        tag: name,
        text,
        ty: type_name::<T>(),
    }))
    .transpose()
}

pub fn parse_chd_text<T: FromStr>(node: &Node, name: &'static str) -> Result<T> {
    match parse_optional_chd_text(node, name).transpose() {
        None => Err(NodeNotFound {
            tag: name,
            parent_tag: node.tag_name().name().to_owned(),
        }),
        Some(r) => r,
    }
}

#[derive(Debug)]
pub struct Clef {
    pub number: u8,
    pub sign: char,
    pub line: Option<u8>,
}

impl FromNode for Clef {
    fn tag() -> &'static str {
        "clef"
    }
    fn from_node(node: &roxmltree::Node) -> Result<Self> {
        Ok(Clef {
            number: parse_optional_attr(node, "number")?.unwrap_or(1),
            sign: parse_chd_text(node, "sign")?,
            line: parse_optional_chd_text(node, "line")?,
        })
    }
}

#[derive(Debug)]
pub struct Attribute {
    pub divisions: u8,
    pub staves: u8,
    pub clef: Vec<Clef>,
}

impl FromNode for Attribute {
    fn tag() -> &'static str {
        "attributes"
    }
    fn from_node(node: &Node) -> Result<Self> {
        Ok(Attribute {
            divisions: parse_chd_text(node, "divisions")?,
            staves: parse_chd_text(node, "staves")?,
            clef: parse_children(node)?,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Rest();

impl FromNode for Rest {
    fn tag() -> &'static str {
        "rest"
    }
    fn from_node(_node: &Node) -> Result<Self> {
        Ok(Rest())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Pitch {
    pub step: u8,
    pub alter: u8,
    pub octave: u8,
}

impl FromNode for Pitch {
    fn tag() -> &'static str {
        "pitch"
    }
    fn from_node(node: &Node) -> Result<Self> {
        Ok(Pitch {
            // map step to jianpu
            step: parse_chd_text::<char>(node, "step").map(|s| (s as u8 + 5 - b'A') % 7 + 1)?,
            alter: parse_optional_chd_text(node, "alter")?.unwrap_or(0),
            octave: parse_chd_text(node, "octave")?,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum NoteType {
    Rest(Rest),
    Pitch(Pitch),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Note {
    pub note_type: NoteType,
    pub duration: u8,
}

impl FromNode for Note {
    fn tag() -> &'static str {
        "note"
    }
    fn from_node(node: &Node) -> Result<Self> {
        let rest = parse_option_chd(node)?.map(NoteType::Rest);
        let pitch = parse_option_chd(node)?.map(NoteType::Pitch);

        if rest.as_ref().and(pitch.as_ref()).is_some() {
            return Err(ExclusiveNodeFound {
                tags: vec![Rest::tag(), Pitch::tag()],
                parent_tag: Self::tag(),
            });
        }

        // TODO: wrap it to a exclusive enum type
        let note_type: NoteType = match rest.or(pitch) {
            Some(ty) => ty,
            None => {
                return Err(ExclusiveNodeGroupNotFound {
                    tags: vec![Rest::tag(), Pitch::tag()],
                    parent_tag: Self::tag(),
                })
            }
        };

        // TODO: should consider grace note
        let duration = parse_chd_text(node, "duration").unwrap_or(0);

        Ok(Note {
            note_type,
            duration,
        })
    }
}

#[derive(Debug)]
pub struct Measure {
    pub number: u16,
    pub attr: Option<Attribute>,
    pub notes: Vec<Note>,
}

impl FromNode for Measure {
    fn tag() -> &'static str {
        "measure"
    }
    fn from_node(node: &Node) -> Result<Self> {
        Ok(Measure {
            number: parse_attr(node, "number")?,
            attr: parse_option_chd(node)?,
            notes: parse_children(node)?,
        })
    }
}

#[derive(Debug)]
pub struct Part {
    pub measures: Vec<Measure>,
}

impl FromNode for Part {
    fn tag() -> &'static str {
        "part"
    }
    fn from_node(node: &Node) -> Result<Self> {
        Ok(Part {
            measures: parse_children(node)?,
        })
    }
}

#[derive(Debug)]
pub struct Score {
    pub parts: Vec<Part>,
}

impl FromNode for Score {
    fn tag() -> &'static str {
        "score-partwise"
    }
    fn from_node(node: &Node) -> Result<Self> {
        Ok(Score {
            parts: parse_children(node)?,
        })
    }
}

impl Score {
    pub fn from_xml(xml: &str) -> Result<Self> {
        let doc = Document::parse(xml)?;

        Score::from_node(&doc.root_element())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_attr_ok() {
        let xml = r#"<slur type="start" />"#;
        let doc = Document::parse(&xml).unwrap();
        let node = doc.root_element();

        let slur_ty = parse_attr::<String>(&node, "type");
        assert!(slur_ty.is_ok());
        assert_eq!(slur_ty.unwrap(), "start");
    }

    #[test]
    fn parse_absent_attr_err() {
        let xml = r#"<slur />"#;
        let doc = Document::parse(&xml).unwrap();
        let node = doc.root_element();

        let slur_ty = parse_attr::<String>(&node, "type");
        assert!(slur_ty.is_err());
        assert!(
            matches!(slur_ty, Err(AttrNotFound { attr, tag }) if attr == "type" && tag == "slur" )
        );
    }

    #[test]
    fn note_pitch_ok() {
        let xml = r#"
            <note>
                <pitch>
                    <step>E</step>
                    <octave>4</octave>
                </pitch>
                <duration>60</duration>
            </note>"#;
        let doc = Document::parse(xml).unwrap();
        let node = doc.root_element();

        let note = Note::from_node(&node);
        assert!(note.is_ok());
        assert_eq!(
            note.unwrap(),
            Note {
                note_type: NoteType::Pitch(Pitch {
                    step: 3,
                    alter: 0,
                    octave: 4
                }),
                duration: 60
            }
        );
    }

    #[test]
    fn note_rest_ok() {
        let xml = r#"
            <note>
                <rest />
                <duration>60</duration>
            </note>"#;
        let doc = Document::parse(xml).unwrap();
        let node = doc.root_element();

        let note = Note::from_node(&node);
        assert!(note.is_ok());
        assert_eq!(
            note.unwrap(),
            Note {
                note_type: NoteType::Rest(Rest()),
                duration: 60
            }
        );
    }
}
