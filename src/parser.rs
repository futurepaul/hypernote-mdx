use crate::ast::*;
use crate::token::{Tag as TokenTag, Token};
use crate::tokenizer::Tokenizer;

const MAX_PARSE_ERRORS: usize = 4096;

#[derive(Debug, Clone)]
pub struct ParseOptions {
    pub normalize_emoji_shortcodes: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        ParseOptions {
            normalize_emoji_shortcodes: false,
        }
    }
}

pub struct Parser {
    source: String,
    token_tags: Vec<TokenTag>,
    token_starts: Vec<ByteOffset>,
    token_index: TokenIndex,
    nodes: Vec<Node>,
    extra_data: Vec<u32>,
    scratch: Vec<NodeIndex>,
    errors: Vec<Error>,
}

#[derive(Debug)]
pub enum ParseError {
    ParseError,
}

type PResult<T> = Result<T, ParseError>;

pub fn parse(source: &str) -> Ast {
    parse_with_options(source, &ParseOptions::default())
}

pub fn parse_with_options(source: &str, options: &ParseOptions) -> Ast {
    let source_owned = if options.normalize_emoji_shortcodes {
        normalize_emoji_shortcodes(source)
    } else {
        source.to_string()
    };

    // Phase 1: Tokenization
    let mut tokenizer = Tokenizer::new(&source_owned);
    let mut tokens: Vec<Token> = Vec::new();

    loop {
        let tok = tokenizer.next();
        tokens.push(tok);
        if tok.tag == TokenTag::Eof {
            break;
        }
    }

    // Phase 2: Parsing
    let token_tags: Vec<TokenTag> = tokens.iter().map(|t| t.tag).collect();
    let token_starts: Vec<ByteOffset> = tokens.iter().map(|t| t.loc.start).collect();

    let mut parser = Parser {
        source: source_owned.clone(),
        token_tags: token_tags.clone(),
        token_starts: token_starts.clone(),
        token_index: 0,
        nodes: Vec::new(),
        extra_data: Vec::new(),
        scratch: Vec::new(),
        errors: Vec::new(),
    };

    let _ = parser.parse_document();

    Ast {
        source: source_owned,
        token_tags,
        token_starts,
        nodes: parser.nodes,
        extra_data: parser.extra_data,
        errors: parser.errors,
    }
}

fn normalize_emoji_shortcodes(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut index: usize = 0;
    let bytes = source.as_bytes();

    while index < source.len() {
        if bytes[index] == b':' {
            if let Some((shortcode, end_index)) = parse_shortcode(source, index) {
                if let Some(emoji) = shortcode_to_emoji(shortcode) {
                    output.push_str(emoji);
                    index = end_index;
                    continue;
                }
            }
        }

        let ch = source[index..].chars().next().unwrap_or('\0');
        output.push(ch);
        index += ch.len_utf8();
    }

    output
}

fn parse_shortcode(source: &str, start: usize) -> Option<(&str, usize)> {
    let bytes = source.as_bytes();
    let mut index = start + 1;

    while index < bytes.len() {
        let b = bytes[index];
        let valid = b.is_ascii_alphanumeric() || b == b'_' || b == b'+' || b == b'-';
        if !valid {
            break;
        }
        index += 1;
    }

    if index == start + 1 || index >= bytes.len() || bytes[index] != b':' {
        return None;
    }

    Some((&source[start + 1..index], index + 1))
}

fn shortcode_to_emoji(shortcode: &str) -> Option<&'static str> {
    match shortcode {
        "thumbsup" | "+1" => Some("👍"),
        "thumbsdown" | "-1" => Some("👎"),
        "wave" => Some("👋"),
        "fire" => Some("🔥"),
        "rocket" => Some("🚀"),
        "sparkles" => Some("✨"),
        "tada" => Some("🎉"),
        "smile" => Some("😄"),
        "heart" => Some("❤️"),
        "white_check_mark" => Some("✅"),
        "x" => Some("❌"),
        "warning" => Some("⚠️"),
        "thinking" => Some("🤔"),
        "clap" => Some("👏"),
        "eyes" => Some("👀"),
        "point_up" => Some("☝️"),
        "point_right" => Some("👉"),
        "point_left" => Some("👈"),
        "point_down" => Some("👇"),
        "100" => Some("💯"),
        _ => None,
    }
}

impl Parser {
    // === Token consumption methods ===

    fn eat_token(&mut self, tag: TokenTag) -> Option<TokenIndex> {
        if self.token_tags[self.token_index as usize] == tag {
            let result = self.token_index;
            self.token_index += 1;
            Some(result)
        } else {
            None
        }
    }

    fn expect_token(&mut self, tag: TokenTag) -> PResult<TokenIndex> {
        if let Some(idx) = self.eat_token(tag) {
            Ok(idx)
        } else {
            self.warn(ErrorTag::ExpectedToken);
            Err(ParseError::ParseError)
        }
    }

    fn next_token(&mut self) -> TokenIndex {
        let result = self.token_index;
        self.token_index += 1;
        result
    }

    fn peek_token(&self, offset: u32) -> TokenTag {
        let index = (self.token_index + offset) as usize;
        if index >= self.token_tags.len() {
            TokenTag::Eof
        } else {
            self.token_tags[index]
        }
    }

    fn current_tag(&self) -> TokenTag {
        self.token_tags[self.token_index as usize]
    }

    // === Node creation methods ===

    fn add_node(&mut self, node: Node) -> NodeIndex {
        let index = self.nodes.len() as NodeIndex;
        self.nodes.push(node);
        index
    }

    fn reserve_node(&mut self, tag: NodeTag) -> NodeIndex {
        let index = self.nodes.len() as NodeIndex;
        self.nodes.push(Node {
            tag,
            main_token: 0,
            data: NodeData::None,
        });
        index
    }

    fn set_node(&mut self, index: NodeIndex, node: Node) -> NodeIndex {
        self.nodes[index as usize] = node;
        index
    }

    // === Extra data methods ===

    fn add_extra_heading(&mut self, heading: &Heading) -> u32 {
        let start = self.extra_data.len() as u32;
        self.extra_data.push(heading.level as u32);
        self.extra_data.push(heading.children_start);
        self.extra_data.push(heading.children_end);
        start
    }

    fn add_extra_list_item(&mut self, data: &ListItemData) -> u32 {
        let start = self.extra_data.len() as u32;
        self.extra_data.push(match data.checked {
            None => 0,
            Some(false) => 1,
            Some(true) => 2,
        });
        self.extra_data.push(data.children_start);
        self.extra_data.push(data.children_end);
        start
    }

    fn add_extra_jsx_element(&mut self, elem: &JsxElement) -> u32 {
        let start = self.extra_data.len() as u32;
        self.extra_data.push(elem.name_token);
        self.extra_data.push(elem.attrs_start);
        self.extra_data.push(elem.attrs_end);
        self.extra_data.push(elem.children_start);
        self.extra_data.push(elem.children_end);
        start
    }

    fn add_extra_link(&mut self, link: &Link) -> u32 {
        let start = self.extra_data.len() as u32;
        self.extra_data.push(link.text_node.unwrap_or(u32::MAX));
        self.extra_data.push(link.url_token);
        start
    }

    fn add_extra_range(&mut self, range: &Range) -> u32 {
        let start = self.extra_data.len() as u32;
        self.extra_data.push(range.start);
        self.extra_data.push(range.end);
        start
    }

    fn add_extra_frontmatter(
        &mut self,
        format: FrontmatterFormat,
        content_start: u32,
        content_end: u32,
    ) -> u32 {
        let start = self.extra_data.len() as u32;
        self.extra_data.push(match format {
            FrontmatterFormat::Yaml => 0,
            FrontmatterFormat::Json => 1,
        });
        self.extra_data.push(content_start);
        self.extra_data.push(content_end);
        start
    }

    fn list_to_span(&mut self, items: &[NodeIndex]) -> Range {
        let start = self.extra_data.len() as u32;
        self.extra_data.extend_from_slice(items);
        Range {
            start,
            end: self.extra_data.len() as u32,
        }
    }

    // === Error handling ===

    fn warn(&mut self, tag: ErrorTag) {
        self.warn_at(tag, self.token_index);
    }

    fn warn_at(&mut self, tag: ErrorTag, token: TokenIndex) {
        if self.errors.len() >= MAX_PARSE_ERRORS {
            return;
        }
        let byte_offset = self.byte_offset_for_token(token);
        self.errors.push(Error {
            tag,
            token,
            byte_offset,
        });
    }

    fn byte_offset_for_token(&self, token: TokenIndex) -> ByteOffset {
        if (token as usize) < self.token_starts.len() {
            self.token_starts[token as usize]
        } else {
            self.source.len() as ByteOffset
        }
    }

    // === Parsing methods ===

    fn parse_document(&mut self) -> PResult<NodeIndex> {
        let scratch_top = self.scratch.len();

        // Check for YAML frontmatter
        if let Some(fm_start) = self.eat_token(TokenTag::FrontmatterStart) {
            if let Ok(fm_node) = self.parse_yaml_frontmatter(fm_start) {
                self.scratch.push(fm_node);
            }
        } else if self.is_json_frontmatter() {
            // Check for JSON frontmatter (```hnmd ... ```)
            if let Ok(fm_node) = self.parse_json_frontmatter() {
                self.scratch.push(fm_node);
            }
        }

        // Parse top-level blocks
        while self.current_tag() != TokenTag::Eof {
            // Skip newlines and blank lines between blocks
            while self.current_tag() == TokenTag::BlankLine
                || self.current_tag() == TokenTag::Newline
            {
                self.token_index += 1;
            }

            if self.current_tag() == TokenTag::Eof {
                break;
            }

            let before = self.token_index;
            match self.parse_block() {
                Ok(block) => {
                    self.scratch.push(block);
                }
                Err(_) => {
                    // Stop after the first parse failure; callers can fall back to
                    // plain-text rendering when `ast.errors` is non-empty.
                    break;
                }
            }
            // Keep forward-progress guard for pathological inputs.
            if self.token_index == before {
                self.warn(ErrorTag::UnexpectedToken);
                self.token_index += 1;
            }
        }

        let children: Vec<NodeIndex> = self.scratch[scratch_top..].to_vec();
        self.scratch.truncate(scratch_top);
        let children_span = self.list_to_span(&children);

        Ok(self.add_node(Node {
            tag: NodeTag::Document,
            main_token: 0,
            data: NodeData::Children(children_span),
        }))
    }

    fn parse_yaml_frontmatter(&mut self, start_token: TokenIndex) -> PResult<NodeIndex> {
        // Skip newline after ---
        self.eat_token(TokenTag::Newline);

        // Consume content until closing ---
        let content_start = self.token_index;
        while self.current_tag() != TokenTag::Hr && self.current_tag() != TokenTag::Eof {
            self.token_index += 1;
        }
        let content_end = self.token_index;

        // Expect closing ---
        if self.current_tag() != TokenTag::Hr {
            self.warn(ErrorTag::UnclosedFrontmatter);
            return Err(ParseError::ParseError);
        }
        self.next_token(); // consume hr

        let extra_index =
            self.add_extra_frontmatter(FrontmatterFormat::Yaml, content_start, content_end);

        Ok(self.add_node(Node {
            tag: NodeTag::Frontmatter,
            main_token: start_token,
            data: NodeData::Extra(extra_index),
        }))
    }

    fn is_json_frontmatter(&self) -> bool {
        if self.peek_token(0) != TokenTag::CodeFenceStart {
            return false;
        }
        if self.peek_token(1) != TokenTag::Text {
            return false;
        }
        // Check that the text token is "hnmd"
        let text_idx = self.token_index + 1;
        let text = self.token_slice(text_idx);
        text.trim() == "hnmd"
    }

    fn parse_json_frontmatter(&mut self) -> PResult<NodeIndex> {
        let start_token = self.next_token(); // consume CodeFenceStart

        // Skip "hnmd" text token
        self.expect_token(TokenTag::Text)?;
        // Skip newline after ```hnmd
        self.eat_token(TokenTag::Newline);

        // Consume content until closing ```
        let content_start = self.token_index;
        while self.current_tag() != TokenTag::CodeFenceEnd && self.current_tag() != TokenTag::Eof {
            self.token_index += 1;
        }
        let content_end = self.token_index;

        // Expect closing ```
        if self.current_tag() != TokenTag::CodeFenceEnd {
            self.warn(ErrorTag::UnclosedFrontmatter);
            return Err(ParseError::ParseError);
        }
        self.next_token(); // consume CodeFenceEnd

        let extra_index =
            self.add_extra_frontmatter(FrontmatterFormat::Json, content_start, content_end);

        Ok(self.add_node(Node {
            tag: NodeTag::Frontmatter,
            main_token: start_token,
            data: NodeData::Extra(extra_index),
        }))
    }

    fn parse_block(&mut self) -> PResult<NodeIndex> {
        match self.current_tag() {
            TokenTag::HeadingStart => self.parse_heading(),
            TokenTag::CodeFenceStart => self.parse_code_block(),
            TokenTag::Hr => self.parse_hr(),
            TokenTag::BlockquoteStart => self.parse_blockquote(),
            TokenTag::ListItemUnordered | TokenTag::ListItemOrdered => self.parse_list(),
            TokenTag::Pipe => self.parse_table(),
            TokenTag::JsxTagStart => self.parse_jsx_element(),
            _ => self.parse_paragraph(),
        }
    }

    fn parse_heading(&mut self) -> PResult<NodeIndex> {
        let heading_token = self.next_token();

        // Count # characters to determine level
        let heading_text = self.token_slice(heading_token);
        let level = heading_text.bytes().take_while(|&ch| ch == b'#').count() as u8;

        let node_index = self.reserve_node(NodeTag::Heading);

        let children_span = match self.parse_inline_content(TokenTag::Newline) {
            Ok(span) => span,
            Err(e) => {
                let empty_heading = self.add_extra_heading(&Heading {
                    level,
                    children_start: 0,
                    children_end: 0,
                });
                self.set_node(
                    node_index,
                    Node {
                        tag: NodeTag::Heading,
                        main_token: heading_token,
                        data: NodeData::Extra(empty_heading),
                    },
                );
                return Err(e);
            }
        };

        let heading_index = self.add_extra_heading(&Heading {
            level,
            children_start: children_span.start,
            children_end: children_span.end,
        });

        Ok(self.set_node(
            node_index,
            Node {
                tag: NodeTag::Heading,
                main_token: heading_token,
                data: NodeData::Extra(heading_index),
            },
        ))
    }

    fn parse_paragraph(&mut self) -> PResult<NodeIndex> {
        let start_token = self.token_index;
        let node_index = self.reserve_node(NodeTag::Paragraph);

        let children_span = match self.parse_inline_content(TokenTag::BlankLine) {
            Ok(span) => span,
            Err(e) => {
                self.set_node(
                    node_index,
                    Node {
                        tag: NodeTag::Paragraph,
                        main_token: start_token,
                        data: NodeData::Children(Range { start: 0, end: 0 }),
                    },
                );
                return Err(e);
            }
        };

        Ok(self.set_node(
            node_index,
            Node {
                tag: NodeTag::Paragraph,
                main_token: start_token,
                data: NodeData::Children(children_span),
            },
        ))
    }

    fn parse_inline_content(&mut self, end_tag: TokenTag) -> PResult<Range> {
        let scratch_top = self.scratch.len();

        while self.current_tag() != end_tag
            && self.current_tag() != TokenTag::Eof
            && self.current_tag() != TokenTag::BlankLine
        {
            // Skip newlines within inline content (soft breaks)
            if self.current_tag() == TokenTag::Newline {
                self.next_token();
                continue;
            }

            let before = self.token_index;
            let inline_node = self.parse_inline()?;
            self.scratch.push(inline_node);
            if self.token_index == before {
                self.token_index += 1;
            }
        }

        self.eat_token(end_tag);

        let children: Vec<NodeIndex> = self.scratch[scratch_top..].to_vec();
        self.scratch.truncate(scratch_top);
        Ok(self.list_to_span(&children))
    }

    fn parse_inline(&mut self) -> PResult<NodeIndex> {
        match self.current_tag() {
            TokenTag::Text | TokenTag::Indent | TokenTag::Space => self.parse_text(),
            TokenTag::StrongStart => self.parse_strong(),
            TokenTag::EmphasisStart => self.parse_emphasis(),
            TokenTag::CodeInlineStart => self.parse_code_inline(),
            TokenTag::LinkStart => self.parse_link(),
            TokenTag::ImageStart => self.parse_image(),
            TokenTag::HardBreak => self.parse_hard_break(),
            TokenTag::ExprStart => self.parse_text_expression(),
            TokenTag::JsxTagStart => self.parse_jsx_element(),
            _ => {
                self.warn(ErrorTag::UnexpectedToken);
                self.next_token();
                Err(ParseError::ParseError)
            }
        }
    }

    fn parse_text(&mut self) -> PResult<NodeIndex> {
        let text_token = self.next_token();
        Ok(self.add_node(Node {
            tag: NodeTag::Text,
            main_token: text_token,
            data: NodeData::None,
        }))
    }

    fn parse_hard_break(&mut self) -> PResult<NodeIndex> {
        let break_token = self.next_token();
        Ok(self.add_node(Node {
            tag: NodeTag::HardBreak,
            main_token: break_token,
            data: NodeData::None,
        }))
    }

    fn parse_strong(&mut self) -> PResult<NodeIndex> {
        let start_token = self.next_token();
        let node_index = self.reserve_node(NodeTag::Strong);

        let children_span = match self.parse_inline_content(TokenTag::StrongEnd) {
            Ok(span) => span,
            Err(e) => {
                self.set_node(
                    node_index,
                    Node {
                        tag: NodeTag::Strong,
                        main_token: start_token,
                        data: NodeData::Children(Range { start: 0, end: 0 }),
                    },
                );
                return Err(e);
            }
        };

        Ok(self.set_node(
            node_index,
            Node {
                tag: NodeTag::Strong,
                main_token: start_token,
                data: NodeData::Children(children_span),
            },
        ))
    }

    fn parse_emphasis(&mut self) -> PResult<NodeIndex> {
        let start_token = self.next_token();
        let node_index = self.reserve_node(NodeTag::Emphasis);

        let children_span = match self.parse_inline_content(TokenTag::EmphasisEnd) {
            Ok(span) => span,
            Err(e) => {
                self.set_node(
                    node_index,
                    Node {
                        tag: NodeTag::Emphasis,
                        main_token: start_token,
                        data: NodeData::Children(Range { start: 0, end: 0 }),
                    },
                );
                return Err(e);
            }
        };

        Ok(self.set_node(
            node_index,
            Node {
                tag: NodeTag::Emphasis,
                main_token: start_token,
                data: NodeData::Children(children_span),
            },
        ))
    }

    fn parse_code_inline(&mut self) -> PResult<NodeIndex> {
        let start_token = self.next_token(); // `
        self.expect_token(TokenTag::Text)?; // code content
        self.expect_token(TokenTag::CodeInlineEnd)?; // `

        Ok(self.add_node(Node {
            tag: NodeTag::CodeInline,
            main_token: start_token,
            data: NodeData::Token(start_token + 1),
        }))
    }

    fn parse_link(&mut self) -> PResult<NodeIndex> {
        let start_token = self.next_token(); // [

        let text_node = if self.current_tag() == TokenTag::Text {
            Some(self.parse_text()?)
        } else {
            None
        };

        self.expect_token(TokenTag::LinkEnd)?; // ]
        self.expect_token(TokenTag::LinkUrlStart)?; // (
        let url_token = self.expect_token(TokenTag::Text)?;
        self.expect_token(TokenTag::LinkUrlEnd)?; // )

        let link_data = self.add_extra_link(&Link {
            text_node,
            url_token,
        });

        Ok(self.add_node(Node {
            tag: NodeTag::Link,
            main_token: start_token,
            data: NodeData::Extra(link_data),
        }))
    }

    fn parse_image(&mut self) -> PResult<NodeIndex> {
        let start_token = self.next_token(); // ![

        let text_node = if self.current_tag() == TokenTag::Text {
            Some(self.parse_text()?)
        } else {
            None
        };

        self.expect_token(TokenTag::LinkEnd)?; // ]
        self.expect_token(TokenTag::LinkUrlStart)?; // (
        let url_token = self.expect_token(TokenTag::Text)?;
        self.expect_token(TokenTag::LinkUrlEnd)?; // )

        let link_data = self.add_extra_link(&Link {
            text_node,
            url_token,
        });

        Ok(self.add_node(Node {
            tag: NodeTag::Image,
            main_token: start_token,
            data: NodeData::Extra(link_data),
        }))
    }

    fn parse_code_block(&mut self) -> PResult<NodeIndex> {
        let start_token = self.next_token(); // ```

        // Optional language identifier
        self.eat_token(TokenTag::Text);
        self.eat_token(TokenTag::Newline);

        // Consume until closing ```
        while self.current_tag() != TokenTag::CodeFenceEnd && self.current_tag() != TokenTag::Eof {
            self.token_index += 1;
        }

        self.expect_token(TokenTag::CodeFenceEnd)?;

        Ok(self.add_node(Node {
            tag: NodeTag::CodeBlock,
            main_token: start_token,
            data: NodeData::None,
        }))
    }

    fn parse_hr(&mut self) -> PResult<NodeIndex> {
        let hr_token = self.next_token();
        Ok(self.add_node(Node {
            tag: NodeTag::Hr,
            main_token: hr_token,
            data: NodeData::None,
        }))
    }

    fn parse_blockquote(&mut self) -> PResult<NodeIndex> {
        let start_token = self.next_token(); // >
        let node_index = self.reserve_node(NodeTag::Blockquote);

        // Skip space after >
        self.eat_token(TokenTag::Space);

        let children_span = match self.parse_inline_content(TokenTag::Newline) {
            Ok(span) => span,
            Err(e) => {
                self.set_node(
                    node_index,
                    Node {
                        tag: NodeTag::Blockquote,
                        main_token: start_token,
                        data: NodeData::Children(Range { start: 0, end: 0 }),
                    },
                );
                return Err(e);
            }
        };

        Ok(self.set_node(
            node_index,
            Node {
                tag: NodeTag::Blockquote,
                main_token: start_token,
                data: NodeData::Children(children_span),
            },
        ))
    }

    fn parse_list(&mut self) -> PResult<NodeIndex> {
        let first_item_tag = self.current_tag();
        let list_tag = if first_item_tag == TokenTag::ListItemOrdered {
            NodeTag::ListOrdered
        } else {
            NodeTag::ListUnordered
        };

        let start_token = self.token_index;
        let node_index = self.reserve_node(list_tag);

        let scratch_top = self.scratch.len();

        while self.current_tag() == first_item_tag {
            match self.parse_list_item() {
                Ok(item) => {
                    self.scratch.push(item);
                }
                Err(e) => {
                    let children: Vec<NodeIndex> = self.scratch[scratch_top..].to_vec();
                    self.scratch.truncate(scratch_top);
                    let empty_span = self.list_to_span(&children);
                    self.set_node(
                        node_index,
                        Node {
                            tag: list_tag,
                            main_token: start_token,
                            data: NodeData::Children(empty_span),
                        },
                    );
                    return Err(e);
                }
            }
        }

        let children: Vec<NodeIndex> = self.scratch[scratch_top..].to_vec();
        self.scratch.truncate(scratch_top);
        let children_span = self.list_to_span(&children);

        Ok(self.set_node(
            node_index,
            Node {
                tag: list_tag,
                main_token: start_token,
                data: NodeData::Children(children_span),
            },
        ))
    }

    fn parse_list_item(&mut self) -> PResult<NodeIndex> {
        let item_token = self.next_token();
        let node_index = self.reserve_node(NodeTag::ListItem);

        // Check for checkbox token
        let checked = if self.eat_token(TokenTag::CheckboxUnchecked).is_some() {
            Some(false)
        } else if self.eat_token(TokenTag::CheckboxChecked).is_some() {
            Some(true)
        } else {
            None
        };

        let children_span = match self.parse_inline_content(TokenTag::Newline) {
            Ok(span) => span,
            Err(e) => {
                let extra_idx = self.add_extra_list_item(&ListItemData {
                    checked,
                    children_start: 0,
                    children_end: 0,
                });
                self.set_node(
                    node_index,
                    Node {
                        tag: NodeTag::ListItem,
                        main_token: item_token,
                        data: NodeData::Extra(extra_idx),
                    },
                );
                return Err(e);
            }
        };

        let extra_idx = self.add_extra_list_item(&ListItemData {
            checked,
            children_start: children_span.start,
            children_end: children_span.end,
        });

        Ok(self.set_node(
            node_index,
            Node {
                tag: NodeTag::ListItem,
                main_token: item_token,
                data: NodeData::Extra(extra_idx),
            },
        ))
    }

    fn parse_table(&mut self) -> PResult<NodeIndex> {
        let start_token = self.token_index;
        let node_index = self.reserve_node(NodeTag::Table);

        let scratch_top = self.scratch.len();

        // Parse header row
        let header_row = self.parse_table_row()?;
        self.scratch.push(header_row);

        // Count columns from header row
        let header_children = match self.nodes[header_row as usize].data {
            NodeData::Children(range) => range,
            _ => Range { start: 0, end: 0 },
        };
        let num_columns = (header_children.end - header_children.start) as u32;

        // Parse separator row and extract alignments
        let mut alignments: Vec<TableAlignment> = Vec::new();
        if self.current_tag() == TokenTag::Pipe {
            self.next_token(); // consume leading |
            while self.current_tag() != TokenTag::Newline
                && self.current_tag() != TokenTag::Eof
                && self.current_tag() != TokenTag::BlankLine
            {
                let before = self.token_index;
                // Read cell content (should be dashes, colons, spaces)
                let mut has_left_colon = false;
                let mut has_right_colon = false;
                let mut has_dash = false;

                if self.current_tag() == TokenTag::Text {
                    let text = self.token_slice(self.token_index).trim();
                    if text.starts_with(':') {
                        has_left_colon = true;
                    }
                    if text.ends_with(':') {
                        has_right_colon = true;
                    }
                    has_dash = text.contains('-');
                    self.next_token(); // consume text
                } else if self.current_tag() == TokenTag::Space
                    || self.current_tag() == TokenTag::Indent
                {
                    self.next_token(); // skip whitespace
                    continue;
                }

                if has_dash {
                    let alignment = match (has_left_colon, has_right_colon) {
                        (true, true) => TableAlignment::Center,
                        (true, false) => TableAlignment::Left,
                        (false, true) => TableAlignment::Right,
                        (false, false) => TableAlignment::None,
                    };
                    alignments.push(alignment);
                }

                if self.current_tag() == TokenTag::Pipe {
                    self.next_token(); // consume |
                }
                if self.token_index == before {
                    self.warn(ErrorTag::UnexpectedToken);
                    self.token_index += 1;
                }
            }
            // Consume trailing newline
            self.eat_token(TokenTag::Newline);
        }

        // Pad alignments to match column count
        while (alignments.len() as u32) < num_columns {
            alignments.push(TableAlignment::None);
        }
        alignments.truncate(num_columns as usize);

        // Parse body rows
        while self.current_tag() == TokenTag::Pipe {
            let before = self.token_index;
            match self.parse_table_row() {
                Ok(row) => self.scratch.push(row),
                Err(_) => break,
            }
            if self.token_index == before {
                self.token_index += 1;
            }
        }

        let rows: Vec<NodeIndex> = self.scratch[scratch_top..].to_vec();
        self.scratch.truncate(scratch_top);

        let num_rows = rows.len() as u32;

        // Store extra data: [num_columns, num_rows, align_0..align_N-1, row_0..row_M-1]
        let extra_start = self.extra_data.len() as u32;
        self.extra_data.push(num_columns);
        self.extra_data.push(num_rows);
        for align in &alignments {
            self.extra_data.push(*align as u32);
        }
        for row in &rows {
            self.extra_data.push(*row);
        }

        Ok(self.set_node(
            node_index,
            Node {
                tag: NodeTag::Table,
                main_token: start_token,
                data: NodeData::Extra(extra_start),
            },
        ))
    }

    fn parse_table_row(&mut self) -> PResult<NodeIndex> {
        let start_token = self.token_index;
        self.expect_token(TokenTag::Pipe)?; // leading |

        let node_index = self.reserve_node(NodeTag::TableRow);
        let scratch_top = self.scratch.len();

        loop {
            // Check for end of row
            if self.current_tag() == TokenTag::Newline
                || self.current_tag() == TokenTag::Eof
                || self.current_tag() == TokenTag::BlankLine
            {
                break;
            }

            let before = self.token_index;

            // Parse cell content
            let cell = self.parse_table_cell()?;
            self.scratch.push(cell);

            // Expect pipe or end of row
            if self.current_tag() == TokenTag::Pipe {
                self.next_token(); // consume |

                // Check if this was a trailing pipe (next is newline/eof)
                if self.current_tag() == TokenTag::Newline
                    || self.current_tag() == TokenTag::Eof
                    || self.current_tag() == TokenTag::BlankLine
                {
                    break;
                }
            }

            // Forward-progress guard
            if self.token_index == before {
                self.token_index += 1;
            }
        }

        // Consume trailing newline
        self.eat_token(TokenTag::Newline);

        let cells: Vec<NodeIndex> = self.scratch[scratch_top..].to_vec();
        self.scratch.truncate(scratch_top);
        let children_span = self.list_to_span(&cells);

        Ok(self.set_node(
            node_index,
            Node {
                tag: NodeTag::TableRow,
                main_token: start_token,
                data: NodeData::Children(children_span),
            },
        ))
    }

    fn parse_table_cell(&mut self) -> PResult<NodeIndex> {
        let start_token = self.token_index;
        let node_index = self.reserve_node(NodeTag::TableCell);

        let scratch_top = self.scratch.len();

        // Skip leading space
        if self.current_tag() == TokenTag::Space || self.current_tag() == TokenTag::Indent {
            self.next_token();
        }

        // Parse inline content until Pipe or Newline
        while self.current_tag() != TokenTag::Pipe
            && self.current_tag() != TokenTag::Newline
            && self.current_tag() != TokenTag::Eof
            && self.current_tag() != TokenTag::BlankLine
        {
            let before = self.token_index;
            let inline_node = self.parse_inline()?;
            self.scratch.push(inline_node);
            if self.token_index == before {
                self.token_index += 1;
            }
        }

        let children: Vec<NodeIndex> = self.scratch[scratch_top..].to_vec();
        self.scratch.truncate(scratch_top);

        // Trim trailing space from children: if last child is a text node with trailing spaces,
        // we'll keep it as-is (the renderer can handle trimming if needed)

        let children_span = self.list_to_span(&children);

        Ok(self.set_node(
            node_index,
            Node {
                tag: NodeTag::TableCell,
                main_token: start_token,
                data: NodeData::Children(children_span),
            },
        ))
    }

    fn parse_text_expression(&mut self) -> PResult<NodeIndex> {
        let expr_start = self.expect_token(TokenTag::ExprStart)?;

        let content_start = self.token_index;
        let mut depth: u32 = 1;

        while depth > 0 && self.current_tag() != TokenTag::Eof {
            match self.current_tag() {
                TokenTag::ExprStart => depth += 1,
                TokenTag::ExprEnd => depth -= 1,
                _ => {}
            }
            if depth > 0 {
                self.token_index += 1;
            }
        }

        if depth > 0 {
            self.warn(ErrorTag::UnclosedExpression);
            return Err(ParseError::ParseError);
        }

        let content_end = self.token_index;

        self.expect_token(TokenTag::ExprEnd)?;

        let range_index = self.add_extra_range(&Range {
            start: content_start,
            end: content_end,
        });

        Ok(self.add_node(Node {
            tag: NodeTag::MdxTextExpression,
            main_token: expr_start,
            data: NodeData::Extra(range_index),
        }))
    }

    fn parse_jsx_element(&mut self) -> PResult<NodeIndex> {
        let open_bracket = self.expect_token(TokenTag::JsxTagStart)?;

        // Check for closing tag
        if self.eat_token(TokenTag::JsxCloseTag).is_some() {
            return self.parse_jsx_closing_tag();
        }

        // Check for fragment
        if self.peek_token(0) == TokenTag::JsxTagEnd {
            return self.parse_jsx_fragment();
        }

        let name = self.expect_token(TokenTag::JsxIdentifier)?;
        let open_name = self.token_slice(name).trim().to_string();

        // Parse attributes
        let attrs_start = self.extra_data.len() as u32;
        while self.current_tag() == TokenTag::JsxIdentifier {
            let attr_name = self.next_token();

            let (attr_value, attr_type) = if self.eat_token(TokenTag::JsxEqual).is_some() {
                self.parse_jsx_attribute_value()?
            } else {
                (None, JsxAttributeType::Boolean)
            };

            self.extra_data.push(attr_name);
            self.extra_data.push(attr_value.unwrap_or(u32::MAX));
            self.extra_data.push(Self::jsx_attr_type_to_raw(attr_type));
        }
        let attrs_end = self.extra_data.len() as u32;

        if !matches!(
            self.current_tag(),
            TokenTag::JsxSelfClose | TokenTag::JsxTagEnd
        ) {
            self.warn(ErrorTag::InvalidJsxAttribute);
            return Err(ParseError::ParseError);
        }

        // Check for self-closing
        if self.eat_token(TokenTag::JsxSelfClose).is_some() {
            let jsx_data = self.add_extra_jsx_element(&JsxElement {
                name_token: name,
                attrs_start,
                attrs_end,
                children_start: 0,
                children_end: 0,
            });

            return Ok(self.add_node(Node {
                tag: NodeTag::MdxJsxSelfClosing,
                main_token: open_bracket,
                data: NodeData::Extra(jsx_data),
            }));
        }

        self.expect_token(TokenTag::JsxTagEnd)?;

        // Parse children
        let scratch_top = self.scratch.len();

        while self.current_tag() != TokenTag::JsxCloseTag && self.current_tag() != TokenTag::Eof {
            let before = self.token_index;
            let tag = self.current_tag();

            match tag {
                TokenTag::JsxTagStart => {
                    if self.peek_token(1) == TokenTag::JsxCloseTag {
                        break;
                    }
                    let child = self.parse_block()?;
                    self.scratch.push(child);
                }
                TokenTag::ExprStart => {
                    let child = self.parse_text_expression()?;
                    self.scratch.push(child);
                }
                TokenTag::Text => {
                    let child = self.parse_text()?;
                    self.scratch.push(child);
                }
                TokenTag::Indent => {
                    let next_tag = self.peek_token(1);
                    if matches!(
                        next_tag,
                        TokenTag::JsxTagStart
                            | TokenTag::JsxCloseTag
                            | TokenTag::Newline
                            | TokenTag::BlankLine
                            | TokenTag::Eof
                    ) {
                        self.token_index += 1;
                    } else {
                        let child = self.parse_text()?;
                        self.scratch.push(child);
                    }
                }
                TokenTag::Space => {
                    let child = self.parse_text()?;
                    self.scratch.push(child);
                }
                TokenTag::CodeInlineStart => {
                    let child = self.parse_code_inline()?;
                    self.scratch.push(child);
                }
                TokenTag::StrongStart => {
                    let child = self.parse_strong()?;
                    self.scratch.push(child);
                }
                TokenTag::EmphasisStart => {
                    let child = self.parse_emphasis()?;
                    self.scratch.push(child);
                }
                TokenTag::LinkStart => {
                    let child = self.parse_link()?;
                    self.scratch.push(child);
                }
                TokenTag::ImageStart => {
                    let child = self.parse_image()?;
                    self.scratch.push(child);
                }
                TokenTag::HardBreak => {
                    let child = self.parse_hard_break()?;
                    self.scratch.push(child);
                }
                TokenTag::HeadingStart => {
                    let child = self.parse_heading()?;
                    self.scratch.push(child);
                }
                TokenTag::Newline | TokenTag::BlankLine => {
                    self.token_index += 1;
                }
                _ => {
                    self.token_index += 1;
                }
            }
            if self.token_index == before {
                self.warn(ErrorTag::UnexpectedToken);
                self.token_index += 1;
            }
        }

        let children_vec: Vec<NodeIndex> = self.scratch[scratch_top..].to_vec();
        self.scratch.truncate(scratch_top);
        let children_span = self.list_to_span(&children_vec);

        // Expect closing tag
        let close_tag_token = self.expect_token(TokenTag::JsxCloseTag)?;
        let close_name = self.expect_token(TokenTag::JsxIdentifier)?;
        if self.token_slice(close_name).trim() != open_name {
            self.warn_at(ErrorTag::MismatchedTags, close_tag_token);
            self.eat_token(TokenTag::JsxTagEnd);
            return Err(ParseError::ParseError);
        }
        self.expect_token(TokenTag::JsxTagEnd)?;

        let jsx_data = self.add_extra_jsx_element(&JsxElement {
            name_token: name,
            attrs_start,
            attrs_end,
            children_start: children_span.start,
            children_end: children_span.end,
        });

        Ok(self.add_node(Node {
            tag: NodeTag::MdxJsxElement,
            main_token: open_bracket,
            data: NodeData::Extra(jsx_data),
        }))
    }

    fn parse_jsx_attribute_value(&mut self) -> PResult<(Option<TokenIndex>, JsxAttributeType)> {
        if let Some(value_token) = self.eat_token(TokenTag::JsxString) {
            return Ok((Some(value_token), JsxAttributeType::String));
        }

        if self.eat_token(TokenTag::JsxAttrExprStart).is_some() {
            let expr_content_start = self.token_index;
            let mut depth: u32 = 1;

            while depth > 0 && self.current_tag() != TokenTag::Eof {
                match self.current_tag() {
                    TokenTag::ExprStart => depth += 1,
                    TokenTag::ExprEnd => depth -= 1,
                    _ => {}
                }
                if depth > 0 {
                    self.token_index += 1;
                }
            }

            if depth > 0 {
                self.warn(ErrorTag::UnclosedExpression);
                return Err(ParseError::ParseError);
            }

            self.expect_token(TokenTag::ExprEnd)?;
            let value_token = if expr_content_start == self.token_index.saturating_sub(1) {
                None
            } else {
                Some(expr_content_start)
            };
            return Ok((value_token, JsxAttributeType::Expression));
        }

        if let Some(value_token) = self.eat_token(TokenTag::JsxIdentifier) {
            let value = self.token_slice(value_token).trim();
            let value_type = Self::infer_unquoted_jsx_value_type(value);
            return Ok((Some(value_token), value_type));
        }

        if let Some(value_token) = self.eat_token(TokenTag::Text) {
            let value = self.token_slice(value_token).trim();
            let value_type = Self::infer_unquoted_jsx_value_type(value);
            return Ok((Some(value_token), value_type));
        }

        self.warn(ErrorTag::InvalidJsxAttribute);
        Err(ParseError::ParseError)
    }

    fn infer_unquoted_jsx_value_type(value: &str) -> JsxAttributeType {
        if value == "true" || value == "false" {
            JsxAttributeType::Boolean
        } else if value.parse::<f64>().is_ok() {
            JsxAttributeType::Number
        } else {
            JsxAttributeType::String
        }
    }

    fn jsx_attr_type_to_raw(value_type: JsxAttributeType) -> u32 {
        match value_type {
            JsxAttributeType::String => 0,
            JsxAttributeType::Number => 1,
            JsxAttributeType::Boolean => 2,
            JsxAttributeType::Expression => 3,
        }
    }

    fn parse_jsx_closing_tag(&mut self) -> PResult<NodeIndex> {
        let close_tag_token = self.token_index.saturating_sub(1);
        self.warn_at(ErrorTag::UnexpectedToken, close_tag_token);
        self.expect_token(TokenTag::JsxIdentifier)?;
        self.expect_token(TokenTag::JsxTagEnd)?;
        Err(ParseError::ParseError) // Closing tags shouldn't appear at block level
    }

    fn parse_jsx_fragment(&mut self) -> PResult<NodeIndex> {
        let open_bracket = self.token_index - 1; // jsx_tag_start
        self.expect_token(TokenTag::JsxTagEnd)?; // >

        let scratch_top = self.scratch.len();

        // Parse children until </>
        while !(self.current_tag() == TokenTag::JsxTagStart
            && self.peek_token(1) == TokenTag::JsxCloseTag)
        {
            if self.current_tag() == TokenTag::Eof {
                self.warn(ErrorTag::ExpectedClosingTag);
                return Err(ParseError::ParseError);
            }
            let before = self.token_index;
            let child = self.parse_block()?;
            self.scratch.push(child);
            if self.token_index == before {
                self.warn(ErrorTag::UnexpectedToken);
                self.token_index += 1;
            }
        }

        let children_vec: Vec<NodeIndex> = self.scratch[scratch_top..].to_vec();
        self.scratch.truncate(scratch_top);
        let children_span = self.list_to_span(&children_vec);

        // Expect </>
        self.expect_token(TokenTag::JsxTagStart)?;
        self.expect_token(TokenTag::JsxCloseTag)?;
        self.expect_token(TokenTag::JsxTagEnd)?;

        Ok(self.add_node(Node {
            tag: NodeTag::MdxJsxFragment,
            main_token: open_bracket,
            data: NodeData::Children(children_span),
        }))
    }

    fn token_slice(&self, token_index: TokenIndex) -> &str {
        let start = self.token_starts[token_index as usize] as usize;
        let end = if (token_index as usize + 1) < self.token_starts.len() {
            self.token_starts[token_index as usize + 1] as usize
        } else {
            self.source.len()
        };
        &self.source[start..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_heading() {
        let source = "# Hello World\n";
        let ast = parse(source);

        assert!(ast.nodes.len() >= 1);

        let heading_idx = ast
            .nodes
            .iter()
            .enumerate()
            .find(|(_, n)| n.tag == NodeTag::Heading)
            .map(|(i, _)| i as NodeIndex);

        assert!(heading_idx.is_some());

        if let Some(idx) = heading_idx {
            let info = ast.heading_info(idx);
            assert_eq!(1, info.level);
        }
    }

    #[test]
    fn parse_paragraph_with_expression() {
        let source = "Hello {name}\n";
        let ast = parse(source);

        let found_paragraph = ast.nodes.iter().any(|n| n.tag == NodeTag::Paragraph);
        assert!(found_paragraph);
    }

    #[test]
    fn parse_json_frontmatter() {
        let source = "```hnmd\n{\"title\": \"Hello\"}\n```\n\n# Content\n";
        let ast = parse(source);

        assert!(
            ast.errors.is_empty(),
            "Expected no errors, got: {:?}",
            ast.errors
        );

        let fm_idx = ast
            .nodes
            .iter()
            .enumerate()
            .find(|(_, n)| n.tag == NodeTag::Frontmatter)
            .map(|(i, _)| i as NodeIndex);

        assert!(fm_idx.is_some(), "Expected a Frontmatter node");

        if let Some(idx) = fm_idx {
            let info = ast.frontmatter_info(idx);
            assert_eq!(FrontmatterFormat::Json, info.format);
        }
    }

    #[test]
    fn parse_with_unclosed_heredoc_marker_in_jsx_text_terminates() {
        let source = r#"# Waffle

<Card>
<Caption>cat > "$HOME/.config/systemd/user/orange-wallet.service" <<EOF
[Unit]
Description=Orange Wallet
EOF
</Caption>
</Card>
"#;

        let ast = parse(source);
        assert!(!ast.nodes.is_empty(), "parser should return an AST");
        assert!(
            ast.errors.len() <= MAX_PARSE_ERRORS,
            "error list must stay bounded"
        );
    }

    #[test]
    fn parse_table_recovery_progresses_after_invalid_cell_start() {
        let source = "| [ |\n| --- |\n";
        let ast = parse(source);
        assert!(
            ast.errors.len() <= MAX_PARSE_ERRORS,
            "error list must stay bounded"
        );
    }
}
