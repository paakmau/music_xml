#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unzip .mxl file failed, details: {source:?}")]
    UnzipFailed {
        #[from]
        source: zip::result::ZipError,
    },
    #[error("an io error occurred, details: {source:?}")]
    Io {
        #[from]
        source: std::io::Error,
    },
    #[error("xml document parse failed, details: {source:?}")]
    XmlDocParseFailed {
        #[from]
        source: roxmltree::Error,
    },
    #[error("node {tag:?} not found in parent node {parent_tag:?}")]
    NodeNotFound {
        tag: &'static str,
        parent_tag: String,
    },
    #[error("duplicated nodes {tag:?} found in parent node {parent_tag:?}")]
    DuplicatedNodesFound {
        tag: &'static str,
        parent_tag: String,
    },
    #[error("not any node in exclusive group {tags:?} found in parent node {parent_tag:?}")]
    ExclusiveNodeGroupNotFound {
        tags: Vec<&'static str>,
        parent_tag: &'static str,
    },
    #[error("exclusive node in group {tags:?} found in parent node {parent_tag:?}")]
    ExclusiveNodeFound {
        tags: Vec<&'static str>,
        parent_tag: &'static str,
    },
    #[error("attr {attr:?} of node {tag:?} not found")]
    AttrNotFound { attr: &'static str, tag: String },
    #[error("node {tag:?} text {text:?} parse as {ty:?} failed")]
    NodeTextParseFailed {
        tag: &'static str,
        text: String,
        ty: &'static str,
    },
    #[error("attr {attr:?} of node {tag:?} values {v:?} parse as {ty:?} failed")]
    AttrValueParseFailed {
        attr: String,
        tag: String,
        v: String,
        ty: &'static str,
    },
    #[error("text in node {tag:?} is empty")]
    NodeTextEmpty { tag: &'static str },
}

pub type Result<T, E = Error> = core::result::Result<T, E>;
