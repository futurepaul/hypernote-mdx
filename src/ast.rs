use crate::token::Tag as TokenTag;

pub type TokenIndex = u32;
pub type NodeIndex = u32;
pub type ByteOffset = u32;
pub const AST_SCHEMA_NAME: &str = "hypernote-mdx-ast";
pub const AST_SCHEMA_VERSION: u32 = 1;

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
    Table,
    TableRow,
    TableCell,

    // Markdown inline nodes
    Text,
    Strong,
    Emphasis,
    Strikethrough,
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
            NodeTag::Strikethrough => "strikethrough",
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
            NodeTag::Table => "table",
            NodeTag::TableRow => "table_row",
            NodeTag::TableCell => "table_cell",
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
    pub byte_offset: ByteOffset,
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

    pub fn message(&self) -> &'static str {
        match self {
            ErrorTag::ExpectedToken => "Expected a specific token but found a different one.",
            ErrorTag::ExpectedBlockElement => "Expected a valid block-level element.",
            ErrorTag::ExpectedClosingTag => "Expected a closing JSX tag.",
            ErrorTag::UnclosedExpression => "Expression is missing a closing brace.",
            ErrorTag::UnclosedFrontmatter => "Frontmatter block is missing a closing delimiter.",
            ErrorTag::InvalidJsxAttribute => "Invalid JSX attribute syntax.",
            ErrorTag::BlankLineRequired => "A blank line is required before this construct.",
            ErrorTag::MismatchedTags => "JSX closing tag does not match opening tag.",
            ErrorTag::UnexpectedToken => "Unexpected token in current parsing context.",
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
    String,
    Number,
    Boolean,
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
    pub children_start: u32,
    pub children_end: u32,
    pub url_token: TokenIndex,
}

#[derive(Debug, Clone, Copy)]
pub struct ListItemData {
    pub checked: Option<bool>,
    pub children_start: u32,
    pub children_end: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableAlignment {
    None = 0,
    Left = 1,
    Center = 2,
    Right = 3,
}

#[derive(Debug, Clone, Copy)]
pub struct TableData {
    pub num_columns: u32,
    pub num_rows: u32,
    pub alignments_start: u32,
    pub rows_start: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: ByteOffset,
    pub end: ByteOffset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePosition {
    pub line: usize,
    pub column: usize,
}

impl Ast {
    fn node(&self, node_idx: NodeIndex) -> Option<&Node> {
        self.nodes.get(node_idx as usize)
    }

    fn extra_u32(&self, index: u32) -> Option<u32> {
        self.extra_data.get(index as usize).copied()
    }

    fn node_index_slice(&self, start: u32, end: u32) -> &[NodeIndex] {
        if start > end {
            return &[];
        }

        let start = start as usize;
        let end = end as usize;
        let Some(slice) = self.extra_data.get(start..end) else {
            return &[];
        };

        // SAFETY: NodeIndex and u32 have the same repr
        unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const NodeIndex, slice.len()) }
    }

    /// Get child node indices for a given node
    pub fn children(&self, node_idx: NodeIndex) -> &[NodeIndex] {
        let Some(node) = self.node(node_idx) else {
            return &[];
        };

        match node.tag {
            NodeTag::Document
            | NodeTag::Paragraph
            | NodeTag::Blockquote
            | NodeTag::ListUnordered
            | NodeTag::ListOrdered
            | NodeTag::Strong
            | NodeTag::Emphasis
            | NodeTag::Strikethrough
            | NodeTag::MdxJsxFragment
            | NodeTag::TableRow
            | NodeTag::TableCell => {
                if let NodeData::Children(range) = node.data {
                    self.node_index_slice(range.start, range.end)
                } else {
                    &[]
                }
            }
            NodeTag::Heading => {
                let info = self.heading_info(node_idx);
                self.node_index_slice(info.children_start, info.children_end)
            }
            NodeTag::ListItem => {
                let info = self.list_item_info(node_idx);
                self.node_index_slice(info.children_start, info.children_end)
            }
            NodeTag::Table => {
                let info = self.table_info(node_idx);
                self.node_index_slice(
                    info.rows_start,
                    info.rows_start.saturating_add(info.num_rows),
                )
            }
            NodeTag::MdxJsxElement => {
                let elem = self.jsx_element(node_idx);
                self.node_index_slice(elem.children_start, elem.children_end)
            }
            _ => &[],
        }
    }

    /// Get text slice for a token
    pub fn token_slice(&self, token_index: TokenIndex) -> &str {
        let Some(&start) = self.token_starts.get(token_index as usize) else {
            return "";
        };
        let start = start as usize;
        let end = if (token_index + 1) < self.token_starts.len() as u32 {
            self.token_starts[token_index as usize + 1] as usize
        } else {
            self.source.len()
        };
        self.source.get(start..end).unwrap_or("")
    }

    /// Get the source text span for a node
    pub fn node_source(&self, node_index: NodeIndex) -> &str {
        let Some(node) = self.node(node_index) else {
            return "";
        };
        let start_token = node.main_token;
        let end_token = {
            let node_children = self.children(node_index);
            if !node_children.is_empty() {
                let last_child = node_children[node_children.len() - 1];
                self.node(last_child)
                    .map(|child| child.main_token.saturating_add(1))
                    .unwrap_or_else(|| start_token.saturating_add(1))
            } else {
                start_token.saturating_add(1)
            }
        };

        let Some(&start) = self.token_starts.get(start_token as usize) else {
            return "";
        };
        let start = start as usize;
        let end = if (end_token as usize) < self.token_starts.len() {
            self.token_starts[end_token as usize] as usize
        } else {
            self.source.len()
        };

        self.source.get(start..end).unwrap_or("")
    }

    /// Extract extra data as Heading
    pub fn heading_info(&self, node_index: NodeIndex) -> Heading {
        let Some(node) = self.node(node_index) else {
            return Heading {
                level: 0,
                children_start: 0,
                children_end: 0,
            };
        };
        if node.tag != NodeTag::Heading {
            return Heading {
                level: 0,
                children_start: 0,
                children_end: 0,
            };
        }
        let idx = match node.data {
            NodeData::Extra(i) => i,
            _ => {
                return Heading {
                    level: 0,
                    children_start: 0,
                    children_end: 0,
                };
            }
        };
        Heading {
            level: self.extra_u32(idx).unwrap_or(0) as u8,
            children_start: self.extra_u32(idx.saturating_add(1)).unwrap_or(0),
            children_end: self.extra_u32(idx.saturating_add(2)).unwrap_or(0),
        }
    }

    /// Extract extra data as ListItemData
    pub fn list_item_info(&self, node_index: NodeIndex) -> ListItemData {
        let Some(node) = self.node(node_index) else {
            return ListItemData {
                checked: None,
                children_start: 0,
                children_end: 0,
            };
        };
        if node.tag != NodeTag::ListItem {
            return ListItemData {
                checked: None,
                children_start: 0,
                children_end: 0,
            };
        }
        let idx = match node.data {
            NodeData::Extra(i) => i,
            _ => {
                return ListItemData {
                    checked: None,
                    children_start: 0,
                    children_end: 0,
                };
            }
        };
        let checked_raw = self.extra_u32(idx).unwrap_or(0);
        let checked = match checked_raw {
            1 => Some(false),
            2 => Some(true),
            _ => None,
        };
        ListItemData {
            checked,
            children_start: self.extra_u32(idx.saturating_add(1)).unwrap_or(0),
            children_end: self.extra_u32(idx.saturating_add(2)).unwrap_or(0),
        }
    }

    /// Get JSX element details
    pub fn jsx_element(&self, node_index: NodeIndex) -> JsxElement {
        let Some(node) = self.node(node_index) else {
            return JsxElement {
                name_token: 0,
                attrs_start: 0,
                attrs_end: 0,
                children_start: 0,
                children_end: 0,
            };
        };
        if node.tag != NodeTag::MdxJsxElement && node.tag != NodeTag::MdxJsxSelfClosing {
            return JsxElement {
                name_token: 0,
                attrs_start: 0,
                attrs_end: 0,
                children_start: 0,
                children_end: 0,
            };
        }
        let idx = match node.data {
            NodeData::Extra(i) => i,
            _ => {
                return JsxElement {
                    name_token: 0,
                    attrs_start: 0,
                    attrs_end: 0,
                    children_start: 0,
                    children_end: 0,
                };
            }
        };
        JsxElement {
            name_token: self.extra_u32(idx).unwrap_or(0),
            attrs_start: self.extra_u32(idx.saturating_add(1)).unwrap_or(0),
            attrs_end: self.extra_u32(idx.saturating_add(2)).unwrap_or(0),
            children_start: self.extra_u32(idx.saturating_add(3)).unwrap_or(0),
            children_end: self.extra_u32(idx.saturating_add(4)).unwrap_or(0),
        }
    }

    /// Get link/image details
    pub fn link_info(&self, node_index: NodeIndex) -> Link {
        let Some(node) = self.node(node_index) else {
            return Link {
                children_start: 0,
                children_end: 0,
                url_token: 0,
            };
        };
        if node.tag != NodeTag::Link && node.tag != NodeTag::Image {
            return Link {
                children_start: 0,
                children_end: 0,
                url_token: 0,
            };
        }
        let idx = match node.data {
            NodeData::Extra(i) => i,
            _ => {
                return Link {
                    children_start: 0,
                    children_end: 0,
                    url_token: 0,
                };
            }
        };
        Link {
            children_start: self.extra_u32(idx).unwrap_or(0),
            children_end: self.extra_u32(idx.saturating_add(1)).unwrap_or(0),
            url_token: self.extra_u32(idx.saturating_add(2)).unwrap_or(0),
        }
    }

    /// Get link/image child nodes
    pub fn link_children(&self, node_index: NodeIndex) -> &[NodeIndex] {
        let info = self.link_info(node_index);
        self.node_index_slice(info.children_start, info.children_end)
    }

    /// Get JSX attributes for an element
    pub fn jsx_attributes(&self, node_index: NodeIndex) -> Vec<JsxAttribute> {
        let elem = self.jsx_element(node_index);
        if elem.attrs_start == elem.attrs_end {
            return Vec::new();
        }

        let mut attrs = Vec::new();
        let attrs_end = (elem.attrs_end as usize).min(self.extra_data.len());
        let mut i = (elem.attrs_start as usize).min(attrs_end);
        while i + 2 < attrs_end {
            let name_token = self.extra_data[i];
            let value_raw = self.extra_data[i + 1];
            let type_raw = self.extra_data[i + 2];

            let value_token = if value_raw == u32::MAX {
                None
            } else {
                Some(value_raw)
            };

            let value_type = if type_raw == 0 {
                JsxAttributeType::String
            } else if type_raw == 1 {
                JsxAttributeType::Number
            } else if type_raw == 2 {
                JsxAttributeType::Boolean
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
        let Some(node) = self.node(node_index) else {
            return Span { start: 0, end: 0 };
        };
        let start = self
            .token_starts
            .get(node.main_token as usize)
            .copied()
            .unwrap_or(self.source.len() as ByteOffset);

        let end = {
            let node_children = self.children(node_index);
            if !node_children.is_empty() {
                let last_child = node_children[node_children.len() - 1];
                let child_span = self.node_span(last_child);
                child_span.end
            } else {
                let end_token = node.main_token.saturating_add(1);
                if (end_token as usize) < self.token_starts.len() {
                    self.token_starts[end_token as usize]
                } else {
                    self.source.len() as ByteOffset
                }
            }
        };

        Span { start, end }
    }

    /// Convert a byte offset into a one-based line and column.
    pub fn line_col(&self, byte_offset: ByteOffset) -> SourcePosition {
        let mut clamped = (byte_offset as usize).min(self.source.len());
        while clamped > 0 && !self.source.is_char_boundary(clamped) {
            clamped -= 1;
        }

        let mut line = 1usize;
        let mut column = 1usize;

        for (idx, ch) in self.source.char_indices() {
            if idx >= clamped {
                break;
            }

            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }

        SourcePosition { line, column }
    }

    /// Convert the start of a node span into a one-based line and column.
    pub fn node_position(&self, node_index: NodeIndex) -> SourcePosition {
        self.line_col(self.node_span(node_index).start)
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
        let Some(node) = self.node(node_index) else {
            return FrontmatterData {
                format: FrontmatterFormat::Yaml,
                content_start: 0,
                content_end: 0,
            };
        };
        if node.tag != NodeTag::Frontmatter {
            return FrontmatterData {
                format: FrontmatterFormat::Yaml,
                content_start: 0,
                content_end: 0,
            };
        }
        let idx = match node.data {
            NodeData::Extra(i) => i,
            _ => {
                return FrontmatterData {
                    format: FrontmatterFormat::Yaml,
                    content_start: 0,
                    content_end: 0,
                };
            }
        };
        let format_raw = self.extra_u32(idx).unwrap_or(0);
        let format = if format_raw == 0 {
            FrontmatterFormat::Yaml
        } else {
            FrontmatterFormat::Json
        };
        FrontmatterData {
            format,
            content_start: self.extra_u32(idx.saturating_add(1)).unwrap_or(0),
            content_end: self.extra_u32(idx.saturating_add(2)).unwrap_or(0),
        }
    }

    /// Extract table info from extra_data
    pub fn table_info(&self, node_index: NodeIndex) -> TableData {
        let Some(node) = self.node(node_index) else {
            return TableData {
                num_columns: 0,
                num_rows: 0,
                alignments_start: 0,
                rows_start: 0,
            };
        };
        if node.tag != NodeTag::Table {
            return TableData {
                num_columns: 0,
                num_rows: 0,
                alignments_start: 0,
                rows_start: 0,
            };
        }
        let idx = match node.data {
            NodeData::Extra(i) => i,
            _ => {
                return TableData {
                    num_columns: 0,
                    num_rows: 0,
                    alignments_start: 0,
                    rows_start: 0,
                };
            }
        };
        let num_columns = self.extra_u32(idx).unwrap_or(0);
        let num_rows = self.extra_u32(idx.saturating_add(1)).unwrap_or(0);
        TableData {
            num_columns,
            num_rows,
            alignments_start: idx.saturating_add(2),
            rows_start: idx.saturating_add(2).saturating_add(num_columns),
        }
    }

    /// Get table column alignments
    pub fn table_alignments(&self, node_index: NodeIndex) -> Vec<TableAlignment> {
        let info = self.table_info(node_index);
        (0..info.num_columns)
            .map(|i| {
                let raw = self
                    .extra_u32(info.alignments_start.saturating_add(i))
                    .unwrap_or(0);
                match raw {
                    1 => TableAlignment::Left,
                    2 => TableAlignment::Center,
                    3 => TableAlignment::Right,
                    _ => TableAlignment::None,
                }
            })
            .collect()
    }

    /// Extract a Range from extra_data
    pub fn extra_range(&self, index: u32) -> Range {
        Range {
            start: self.extra_u32(index).unwrap_or(0),
            end: self.extra_u32(index.saturating_add(1)).unwrap_or(0),
        }
    }

    /// Extract typed semantic details for a fenced code block.
    pub fn code_block_info(
        &self,
        node_index: NodeIndex,
    ) -> Option<crate::semantic::CodeBlockInfo<'_>> {
        crate::semantic::code_block_info(self, node_index)
    }

    /// Extract typed semantic details for a link node.
    pub fn link_view(&self, node_index: NodeIndex) -> Option<crate::semantic::LinkInfo<'_>> {
        crate::semantic::link_view(self, node_index)
    }

    /// Extract typed semantic details for an image node.
    pub fn image_view(&self, node_index: NodeIndex) -> Option<crate::semantic::ImageInfo<'_>> {
        crate::semantic::image_view(self, node_index)
    }

    /// Extract typed semantic details for an MDX expression node.
    pub fn expression_info(
        &self,
        node_index: NodeIndex,
    ) -> Option<crate::semantic::ExpressionInfo<'_>> {
        crate::semantic::expression_info(self, node_index)
    }

    /// Extract typed semantic details for a frontmatter node.
    pub fn frontmatter_view(
        &self,
        node_index: NodeIndex,
    ) -> Option<crate::semantic::FrontmatterInfoView<'_>> {
        crate::semantic::frontmatter_view(self, node_index)
    }

    /// Extract typed, decoded JSX attributes for an element node.
    pub fn jsx_attribute_views(
        &self,
        node_index: NodeIndex,
    ) -> Option<Vec<crate::semantic::JsxAttributeView<'_>>> {
        crate::semantic::jsx_attribute_views(self, node_index)
    }

    /// Extract typed semantic details for a JSX element node.
    pub fn jsx_element_view(
        &self,
        node_index: NodeIndex,
    ) -> Option<crate::semantic::JsxElementView<'_>> {
        crate::semantic::jsx_element_view(self, node_index)
    }

    /// Extract plain-text semantic parts for a node.
    pub fn plain_text_parts(
        &self,
        node_index: NodeIndex,
    ) -> Option<Vec<crate::semantic::PlainTextPart<'_>>> {
        crate::semantic::plain_text_parts(self, node_index)
    }

    /// Extract plain-text semantic parts for a child node slice.
    pub fn plain_text_parts_children(
        &self,
        children: &[NodeIndex],
    ) -> Vec<crate::semantic::PlainTextPart<'_>> {
        crate::semantic::plain_text_parts_children(self, children)
    }

    /// Flatten a node to plain text using default text options.
    pub fn plain_text(&self, node_index: NodeIndex) -> Option<String> {
        self.plain_text_with_options(node_index, &crate::semantic::PlainTextOptions::default())
    }

    /// Flatten a node to plain text using explicit options.
    pub fn plain_text_with_options(
        &self,
        node_index: NodeIndex,
        options: &crate::semantic::PlainTextOptions<'_>,
    ) -> Option<String> {
        crate::semantic::plain_text_with_options(self, node_index, options)
    }

    /// Flatten a child node slice to plain text using default text options.
    pub fn plain_text_children(&self, children: &[NodeIndex]) -> String {
        self.plain_text_children_with_options(
            children,
            &crate::semantic::PlainTextOptions::default(),
        )
    }

    /// Flatten a child node slice to plain text using explicit options.
    pub fn plain_text_children_with_options(
        &self,
        children: &[NodeIndex],
        options: &crate::semantic::PlainTextOptions<'_>,
    ) -> String {
        crate::semantic::plain_text_children_with_options(self, children, options)
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
