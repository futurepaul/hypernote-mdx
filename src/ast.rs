use crate::token::Tag as TokenTag;

pub type TokenIndex = u32;
pub type NodeIndex = u32;
pub type ByteOffset = u32;

/// Abstract Syntax Tree for MDX documents.
pub struct Ast {
    pub source: String,
    pub token_tags: Vec<TokenTag>,
    pub token_starts: Vec<ByteOffset>,
    pub nodes: Vec<Node>,
    pub extra_data: Vec<u32>,
    pub errors: Vec<Error>,
}

#[derive(Debug, Clone, Copy)]
pub struct Node {
    pub tag: NodeTag,
    pub main_token: TokenIndex,
    pub data: NodeData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeTag {
    // Root
    Document,

    // Markdown block nodes
    Heading,
    Paragraph,
    CodeBlock,
    Blockquote,
    ListUnordered,
    ListOrdered,
    ListItem,
    Hr,

    // Markdown inline nodes
    Text,
    Strong,
    Emphasis,
    CodeInline,
    Link,
    Image,
    HardBreak,

    // MDX expression nodes
    MdxTextExpression,
    MdxFlowExpression,

    // MDX JSX nodes
    MdxJsxElement,
    MdxJsxSelfClosing,
    MdxJsxFragment,
    MdxJsxAttribute,

    // MDX ESM nodes
    MdxEsmImport,
    MdxEsmExport,

    // Frontmatter
    Frontmatter,
}

impl NodeTag {
    pub fn name(&self) -> &'static str {
        match self {
            NodeTag::Document => "document",
            NodeTag::Heading => "heading",
            NodeTag::Paragraph => "paragraph",
            NodeTag::CodeBlock => "code_block",
            NodeTag::Blockquote => "blockquote",
            NodeTag::ListUnordered => "list_unordered",
            NodeTag::ListOrdered => "list_ordered",
            NodeTag::ListItem => "list_item",
            NodeTag::Hr => "hr",
            NodeTag::Text => "text",
            NodeTag::Strong => "strong",
            NodeTag::Emphasis => "emphasis",
            NodeTag::CodeInline => "code_inline",
            NodeTag::Link => "link",
            NodeTag::Image => "image",
            NodeTag::HardBreak => "hard_break",
            NodeTag::MdxTextExpression => "mdx_text_expression",
            NodeTag::MdxFlowExpression => "mdx_flow_expression",
            NodeTag::MdxJsxElement => "mdx_jsx_element",
            NodeTag::MdxJsxSelfClosing => "mdx_jsx_self_closing",
            NodeTag::MdxJsxFragment => "mdx_jsx_fragment",
            NodeTag::MdxJsxAttribute => "mdx_jsx_attribute",
            NodeTag::MdxEsmImport => "mdx_esm_import",
            NodeTag::MdxEsmExport => "mdx_esm_export",
            NodeTag::Frontmatter => "frontmatter",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NodeData {
    None,
    Token(TokenIndex),
    Children(Range),
    Extra(u32),
}

#[derive(Debug, Clone, Copy)]
pub struct Range {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Error {
    pub tag: ErrorTag,
    pub token: TokenIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorTag {
    ExpectedToken,
    ExpectedBlockElement,
    ExpectedClosingTag,
    UnclosedExpression,
    UnclosedFrontmatter,
    InvalidJsxAttribute,
    BlankLineRequired,
    MismatchedTags,
    UnexpectedToken,
}

impl ErrorTag {
    pub fn name(&self) -> &'static str {
        match self {
            ErrorTag::ExpectedToken => "expected_token",
            ErrorTag::ExpectedBlockElement => "expected_block_element",
            ErrorTag::ExpectedClosingTag => "expected_closing_tag",
            ErrorTag::UnclosedExpression => "unclosed_expression",
            ErrorTag::UnclosedFrontmatter => "unclosed_frontmatter",
            ErrorTag::InvalidJsxAttribute => "invalid_jsx_attribute",
            ErrorTag::BlankLineRequired => "blank_line_required",
            ErrorTag::MismatchedTags => "mismatched_tags",
            ErrorTag::UnexpectedToken => "unexpected_token",
        }
    }
}

// Extra data structures for complex nodes

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontmatterFormat {
    Yaml,
    Json,
}

#[derive(Debug, Clone, Copy)]
pub struct FrontmatterData {
    pub format: FrontmatterFormat,
    pub content_start: u32,
    pub content_end: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsxAttributeType {
    Literal,
    Expression,
}

#[derive(Debug, Clone, Copy)]
pub struct JsxAttribute {
    pub name_token: TokenIndex,
    pub value_token: Option<TokenIndex>,
    pub value_type: JsxAttributeType,
}

#[derive(Debug, Clone, Copy)]
pub struct JsxElement {
    pub name_token: TokenIndex,
    pub attrs_start: u32,
    pub attrs_end: u32,
    pub children_start: u32,
    pub children_end: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Heading {
    pub level: u8,
    pub children_start: u32,
    pub children_end: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Link {
    pub text_node: Option<NodeIndex>,
    pub url_token: TokenIndex,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: ByteOffset,
    pub end: ByteOffset,
}

impl Ast {
    /// Get child node indices for a given node
    pub fn children(&self, node_idx: NodeIndex) -> &[NodeIndex] {
        let node = &self.nodes[node_idx as usize];
        match node.tag {
            NodeTag::Document
            | NodeTag::Paragraph
            | NodeTag::Blockquote
            | NodeTag::ListUnordered
            | NodeTag::ListOrdered
            | NodeTag::ListItem
            | NodeTag::Strong
            | NodeTag::Emphasis
            | NodeTag::MdxJsxFragment => {
                if let NodeData::Children(range) = node.data {
                    let slice = &self.extra_data[range.start as usize..range.end as usize];
                    // SAFETY: NodeIndex and u32 have the same repr
                    unsafe {
                        std::slice::from_raw_parts(slice.as_ptr() as *const NodeIndex, slice.len())
                    }
                } else {
                    &[]
                }
            }
            NodeTag::Heading => {
                let info = self.heading_info(node_idx);
                let slice =
                    &self.extra_data[info.children_start as usize..info.children_end as usize];
                unsafe {
                    std::slice::from_raw_parts(slice.as_ptr() as *const NodeIndex, slice.len())
                }
            }
            NodeTag::MdxJsxElement => {
                let elem = self.jsx_element(node_idx);
                let slice =
                    &self.extra_data[elem.children_start as usize..elem.children_end as usize];
                unsafe {
                    std::slice::from_raw_parts(slice.as_ptr() as *const NodeIndex, slice.len())
                }
            }
            _ => &[],
        }
    }

    /// Get text slice for a token
    pub fn token_slice(&self, token_index: TokenIndex) -> &str {
        let start = self.token_starts[token_index as usize] as usize;
        let end = if (token_index + 1) < self.token_starts.len() as u32 {
            self.token_starts[token_index as usize + 1] as usize
        } else {
            self.source.len()
        };
        &self.source[start..end]
    }

    /// Get the source text span for a node
    pub fn node_source(&self, node_index: NodeIndex) -> &str {
        let node = &self.nodes[node_index as usize];
        let start_token = node.main_token;
        let end_token = {
            let node_children = self.children(node_index);
            if !node_children.is_empty() {
                let last_child = node_children[node_children.len() - 1];
                self.nodes[last_child as usize].main_token + 1
            } else {
                start_token + 1
            }
        };

        let start = self.token_starts[start_token as usize] as usize;
        let end = if (end_token as usize) < self.token_starts.len() {
            self.token_starts[end_token as usize] as usize
        } else {
            self.source.len()
        };

        &self.source[start..end]
    }

    /// Extract extra data as Heading
    pub fn heading_info(&self, node_index: NodeIndex) -> Heading {
        let node = &self.nodes[node_index as usize];
        debug_assert!(node.tag == NodeTag::Heading);
        let idx = match node.data {
            NodeData::Extra(i) => i as usize,
            _ => panic!("heading node has wrong data type"),
        };
        Heading {
            level: self.extra_data[idx] as u8,
            children_start: self.extra_data[idx + 1],
            children_end: self.extra_data[idx + 2],
        }
    }

    /// Get JSX element details
    pub fn jsx_element(&self, node_index: NodeIndex) -> JsxElement {
        let node = &self.nodes[node_index as usize];
        debug_assert!(
            node.tag == NodeTag::MdxJsxElement || node.tag == NodeTag::MdxJsxSelfClosing
        );
        let idx = match node.data {
            NodeData::Extra(i) => i as usize,
            _ => panic!("jsx element node has wrong data type"),
        };
        JsxElement {
            name_token: self.extra_data[idx],
            attrs_start: self.extra_data[idx + 1],
            attrs_end: self.extra_data[idx + 2],
            children_start: self.extra_data[idx + 3],
            children_end: self.extra_data[idx + 4],
        }
    }

    /// Get JSX attributes for an element
    pub fn jsx_attributes(&self, node_index: NodeIndex) -> Vec<JsxAttribute> {
        let elem = self.jsx_element(node_index);
        if elem.attrs_start == elem.attrs_end {
            return Vec::new();
        }

        let mut attrs = Vec::new();
        let mut i = elem.attrs_start as usize;
        while i + 2 < elem.attrs_end as usize + 1 {
            let name_token = self.extra_data[i];
            let value_raw = self.extra_data[i + 1];
            let type_raw = self.extra_data[i + 2];

            let value_token = if value_raw == u32::MAX {
                None
            } else {
                Some(value_raw)
            };

            let value_type = if type_raw == 0 {
                JsxAttributeType::Literal
            } else {
                JsxAttributeType::Expression
            };

            attrs.push(JsxAttribute {
                name_token,
                value_token,
                value_type,
            });

            i += 3;
        }

        attrs
    }

    /// Get the byte span for a node
    pub fn node_span(&self, node_index: NodeIndex) -> Span {
        let node = &self.nodes[node_index as usize];
        let start = self.token_starts[node.main_token as usize];

        let end = {
            let node_children = self.children(node_index);
            if !node_children.is_empty() {
                let last_child = node_children[node_children.len() - 1];
                let child_span = self.node_span(last_child);
                child_span.end
            } else {
                let end_token = node.main_token + 1;
                if (end_token as usize) < self.token_starts.len() {
                    self.token_starts[end_token as usize]
                } else {
                    self.source.len() as ByteOffset
                }
            }
        };

        Span { start, end }
    }

    /// Find the deepest node containing a byte offset
    pub fn node_at_offset(&self, offset: ByteOffset) -> Option<NodeIndex> {
        if self.nodes.is_empty() {
            return None;
        }

        // Find document node
        let doc_idx = self.nodes.iter().enumerate().find_map(|(i, n)| {
            if n.tag == NodeTag::Document {
                Some(i as NodeIndex)
            } else {
                None
            }
        })?;

        self.node_at_offset_recursive(doc_idx, offset)
    }

    fn node_at_offset_recursive(
        &self,
        node_index: NodeIndex,
        offset: ByteOffset,
    ) -> Option<NodeIndex> {
        let span = self.node_span(node_index);

        if offset < span.start || offset >= span.end {
            return None;
        }

        let node_children = self.children(node_index);
        for &child_idx in node_children {
            if let Some(found) = self.node_at_offset_recursive(child_idx, offset) {
                return Some(found);
            }
        }

        Some(node_index)
    }

    /// Extract frontmatter info from extra_data (3 u32s: format, content_start, content_end)
    pub fn frontmatter_info(&self, node_index: NodeIndex) -> FrontmatterData {
        let node = &self.nodes[node_index as usize];
        debug_assert!(node.tag == NodeTag::Frontmatter);
        let idx = match node.data {
            NodeData::Extra(i) => i as usize,
            _ => panic!("frontmatter node has wrong data type"),
        };
        let format_raw = self.extra_data[idx];
        let format = if format_raw == 0 {
            FrontmatterFormat::Yaml
        } else {
            FrontmatterFormat::Json
        };
        FrontmatterData {
            format,
            content_start: self.extra_data[idx + 1],
            content_end: self.extra_data[idx + 2],
        }
    }

    /// Extract a Range from extra_data
    pub fn extra_range(&self, index: u32) -> Range {
        Range {
            start: self.extra_data[index as usize],
            end: self.extra_data[index as usize + 1],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_tag_names() {
        assert_eq!("document", NodeTag::Document.name());
        assert_eq!("heading", NodeTag::Heading.name());
        assert_eq!("mdx_jsx_element", NodeTag::MdxJsxElement.name());
    }
}
