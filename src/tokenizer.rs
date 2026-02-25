use crate::token::{Tag, Token, Loc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Markdown,
    Jsx,
    Expression,
    InlineCode,
    CodeBlock,
}

pub struct Tokenizer<'a> {
    buffer: &'a [u8],
    index: u32,
    line_start: u32,
    mode: Mode,
    mode_stack: Vec<Mode>,
    strong_depth: u32,
    emphasis_depth: u32,
    after_link_text: bool,
    in_link_url: bool,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Tokenizer {
            buffer: source.as_bytes(),
            index: 0,
            line_start: 0,
            mode: Mode::Markdown,
            mode_stack: Vec::new(),
            strong_depth: 0,
            emphasis_depth: 0,
            after_link_text: false,
            in_link_url: false,
        }
    }

    pub fn next(&mut self) -> Token {
        match self.mode {
            Mode::Markdown => self.next_markdown(),
            Mode::Jsx => self.next_jsx(),
            Mode::Expression => self.next_expression(),
            Mode::InlineCode => self.next_inline_code(),
            Mode::CodeBlock => self.next_code_block(),
        }
    }

    fn next_markdown(&mut self) -> Token {
        let start = self.index;

        if self.index as usize >= self.buffer.len() {
            return self.make_token(Tag::Eof, start);
        }

        let at_line_start = self.index == self.line_start;

        if at_line_start {
            return self.next_markdown_sol(start);
        }

        self.next_markdown_inline(start)
    }

    fn next_markdown_sol(&mut self, start: u32) -> Token {
        let c = self.buf(self.index);

        match c {
            0 => self.make_token(Tag::Eof, start),
            b'\n' => {
                self.index += 1;
                self.line_start = self.index;
                self.make_token(Tag::BlankLine, start)
            }
            b'#' => {
                self.index += 1;
                // Count consecutive # characters
                while self.buf(self.index) == b'#' {
                    self.index += 1;
                }
                // Skip space after #
                if self.buf(self.index) == b' ' {
                    self.index += 1;
                }
                self.make_token(Tag::HeadingStart, start)
            }
            b'-' | b'*' | b'_' => {
                self.index += 1;
                self.hr_or_frontmatter(start, c)
            }
            b'`' => {
                if self.peek_ahead("```") {
                    self.index += 3;
                    self.push_mode(Mode::CodeBlock);
                    self.make_token(Tag::CodeFenceStart, start)
                } else {
                    self.next_markdown_inline(start)
                }
            }
            b'>' => {
                self.index += 1;
                self.make_token(Tag::BlockquoteStart, start)
            }
            b' ' | b'\t' => {
                let indent_start = self.index;
                while self.buf(self.index) == b' ' || self.buf(self.index) == b'\t' {
                    self.index += 1;
                }
                self.make_token(Tag::Indent, indent_start)
            }
            b'0'..=b'9' => {
                // Check for ordered list (e.g., "1. ")
                let mut temp_index = self.index;
                while (temp_index as usize) < self.buffer.len()
                    && self.buf(temp_index) >= b'0'
                    && self.buf(temp_index) <= b'9'
                {
                    temp_index += 1;
                }
                if (temp_index as usize) < self.buffer.len()
                    && self.buf(temp_index) == b'.'
                    && (temp_index as usize + 1) < self.buffer.len()
                    && self.buf(temp_index + 1) == b' '
                {
                    self.index = temp_index + 2;
                    self.make_token(Tag::ListItemOrdered, start)
                } else {
                    self.next_markdown_inline(start)
                }
            }
            _ => self.next_markdown_inline(start),
        }
    }

    fn hr_or_frontmatter(&mut self, start: u32, first_char: u8) -> Token {
        let mut count: u32 = 1;

        while self.buf(self.index) == first_char {
            count += 1;
            self.index += 1;
        }

        // Check for frontmatter (--- at start of file)
        if first_char == b'-' && count >= 3 && start == 0 {
            let next = self.buf(self.index);
            if next == b'\n' || next == 0 {
                return self.make_token(Tag::FrontmatterStart, start);
            }
        }

        // Check for HR (3+ consecutive -, *, or _)
        if count >= 3 {
            let next = self.buf(self.index);
            if next == b'\n' || next == 0 {
                return self.make_token(Tag::Hr, start);
            }
        }

        // Check for list item
        if first_char == b'-' || first_char == b'*' {
            if self.buf(self.index) == b' ' {
                return self.make_token(Tag::ListItemUnordered, start);
            }
        }

        // Special case: * or ** at line start could be emphasis/strong
        if first_char == b'*' {
            self.index = start + 1;
            return self.maybe_strong_or_emphasis(start);
        }

        // Otherwise, treat as text
        self.text(start)
    }

    fn next_markdown_inline(&mut self, start: u32) -> Token {
        let c = self.buf(self.index);

        match c {
            0 => self.make_token(Tag::Eof, start),
            b'\n' => {
                self.index += 1;
                self.line_start = self.index;
                self.make_token(Tag::Newline, start)
            }
            b'\\' => {
                if self.index as usize + 1 < self.buffer.len()
                    && self.buf(self.index + 1) == b'\n'
                {
                    self.index += 2;
                    self.line_start = self.index;
                    self.make_token(Tag::HardBreak, start)
                } else {
                    self.text(start)
                }
            }
            b' ' => {
                let mut space_count: u32 = 0;
                let mut temp_idx = self.index;
                while (temp_idx as usize) < self.buffer.len() && self.buf(temp_idx) == b' ' {
                    space_count += 1;
                    temp_idx += 1;
                }
                if space_count >= 2
                    && (temp_idx as usize) < self.buffer.len()
                    && self.buf(temp_idx) == b'\n'
                {
                    self.index = temp_idx + 1;
                    self.line_start = self.index;
                    self.make_token(Tag::HardBreak, start)
                } else {
                    self.text(start)
                }
            }
            b'{' => {
                self.index += 1;
                self.push_mode(Mode::Expression);
                self.make_token(Tag::ExprStart, start)
            }
            b'<' => {
                if self.is_jsx_start() {
                    self.push_mode(Mode::Jsx);
                    self.next_jsx()
                } else {
                    self.text(start)
                }
            }
            b'*' => {
                self.index += 1;
                self.maybe_strong_or_emphasis(start)
            }
            b'`' => {
                self.index += 1;
                self.push_mode(Mode::InlineCode);
                self.make_token(Tag::CodeInlineStart, start)
            }
            b'[' => {
                self.index += 1;
                self.after_link_text = false;
                self.make_token(Tag::LinkStart, start)
            }
            b']' => {
                self.index += 1;
                if self.buf(self.index) == b'(' {
                    self.after_link_text = true;
                    self.make_token(Tag::LinkEnd, start)
                } else {
                    self.after_link_text = false;
                    self.text(start)
                }
            }
            b'(' => {
                if self.after_link_text {
                    self.index += 1;
                    self.after_link_text = false;
                    self.in_link_url = true;
                    self.make_token(Tag::LinkUrlStart, start)
                } else {
                    self.text(start)
                }
            }
            b')' => {
                if self.in_link_url {
                    self.index += 1;
                    self.in_link_url = false;
                    self.make_token(Tag::LinkUrlEnd, start)
                } else {
                    self.text(start)
                }
            }
            b'!' => {
                if self.index as usize + 1 < self.buffer.len()
                    && self.buf(self.index + 1) == b'['
                {
                    self.index += 2;
                    self.make_token(Tag::ImageStart, start)
                } else {
                    self.index += 1;
                    self.text(start)
                }
            }
            _ => self.text(start),
        }
    }

    fn maybe_strong_or_emphasis(&mut self, start: u32) -> Token {
        if self.buf(self.index) == b'*' {
            self.index += 1;
            if self.strong_depth > 0 {
                self.strong_depth -= 1;
                self.make_token(Tag::StrongEnd, start)
            } else {
                self.strong_depth += 1;
                self.make_token(Tag::StrongStart, start)
            }
        } else if self.emphasis_depth > 0 {
            self.emphasis_depth -= 1;
            self.make_token(Tag::EmphasisEnd, start)
        } else {
            self.emphasis_depth += 1;
            self.make_token(Tag::EmphasisStart, start)
        }
    }

    fn text(&mut self, start: u32) -> Token {
        while (self.index as usize) < self.buffer.len() {
            let ch = self.buf(self.index);
            match ch {
                0 | b'\n' | b'{' | b'<' | b'*' | b'`' | b'[' => break,
                b']' => {
                    if self.index as usize + 1 < self.buffer.len()
                        && self.buf(self.index + 1) == b'('
                    {
                        break;
                    }
                    self.index += 1;
                }
                b'(' => {
                    if self.after_link_text {
                        break;
                    }
                    self.index += 1;
                }
                b')' => {
                    if self.in_link_url {
                        break;
                    }
                    self.index += 1;
                }
                b'!' => {
                    if self.index as usize + 1 < self.buffer.len()
                        && self.buf(self.index + 1) == b'['
                    {
                        break;
                    }
                    self.index += 1;
                }
                _ => self.index += 1,
            }
        }

        // Check if we have a hard break pattern at the end
        if (self.index as usize) < self.buffer.len() && self.buf(self.index) == b'\n' {
            // Check for backslash immediately before newline
            if self.index > start && self.buf(self.index - 1) == b'\\' {
                self.index -= 1;
                if self.index == start {
                    self.index += 2;
                    self.line_start = self.index;
                    return self.make_token(Tag::HardBreak, start);
                }
            } else {
                // Check for two+ trailing spaces
                let mut end_idx = self.index;
                let mut spaces: u32 = 0;
                while end_idx > start && self.buf(end_idx - 1) == b' ' {
                    spaces += 1;
                    end_idx -= 1;
                }
                if spaces >= 2 {
                    if end_idx == start {
                        self.index += 1;
                        self.line_start = self.index;
                        return self.make_token(Tag::HardBreak, start);
                    }
                    self.index = end_idx;
                }
            }
        }

        self.make_token(Tag::Text, start)
    }

    fn next_jsx(&mut self) -> Token {
        let start = self.index;

        if self.index as usize >= self.buffer.len() {
            return self.make_token(Tag::Eof, start);
        }

        let c = self.buf(self.index);

        match c {
            0 => self.make_token(Tag::Eof, start),
            b'<' => {
                self.index += 1;
                if self.buf(self.index) == b'/' {
                    self.index += 1;
                    self.make_token(Tag::JsxCloseTag, start)
                } else if self.buf(self.index) == b'>' {
                    self.index += 1;
                    self.make_token(Tag::JsxFragmentStart, start)
                } else {
                    self.make_token(Tag::JsxTagStart, start)
                }
            }
            b'>' => {
                self.index += 1;
                self.pop_mode();
                self.make_token(Tag::JsxTagEnd, start)
            }
            b'/' => {
                if self.buf(self.index + 1) == b'>' {
                    self.index += 2;
                    self.pop_mode();
                    self.make_token(Tag::JsxSelfClose, start)
                } else {
                    self.index += 1;
                    self.make_token(Tag::Invalid, start)
                }
            }
            b'{' => {
                self.index += 1;
                self.push_mode(Mode::Expression);
                self.make_token(Tag::JsxAttrExprStart, start)
            }
            b'=' => {
                self.index += 1;
                self.make_token(Tag::JsxEqual, start)
            }
            b'"' | b'\'' => self.next_jsx_string(c),
            b'.' => {
                self.index += 1;
                self.make_token(Tag::JsxDot, start)
            }
            b':' => {
                self.index += 1;
                self.make_token(Tag::JsxColon, start)
            }
            b' ' | b'\t' | b'\n' => {
                while (self.index as usize) < self.buffer.len() {
                    let ch = self.buf(self.index);
                    if ch != b' ' && ch != b'\t' && ch != b'\n' {
                        break;
                    }
                    self.index += 1;
                }
                self.next()
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.next_jsx_identifier(),
            _ => {
                self.index += 1;
                self.make_token(Tag::Invalid, start)
            }
        }
    }

    fn next_jsx_identifier(&mut self) -> Token {
        let start = self.index;
        while (self.index as usize) < self.buffer.len() {
            match self.buf(self.index) {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-' => self.index += 1,
                _ => break,
            }
        }
        self.make_token(Tag::JsxIdentifier, start)
    }

    fn next_jsx_string(&mut self, quote: u8) -> Token {
        let start = self.index;
        self.index += 1; // Skip opening quote

        while (self.index as usize) < self.buffer.len() {
            let c = self.buf(self.index);
            if c == quote {
                self.index += 1;
                return self.make_token(Tag::JsxString, start);
            }
            if c == b'\\' {
                self.index += 2;
            } else {
                self.index += 1;
            }
        }

        self.make_token(Tag::Invalid, start)
    }

    fn next_expression(&mut self) -> Token {
        let start = self.index;

        if self.index as usize >= self.buffer.len() {
            return self.make_token(Tag::Eof, start);
        }

        let c = self.buf(self.index);

        match c {
            0 => self.make_token(Tag::Eof, start),
            b'}' => {
                self.index += 1;
                self.pop_mode();
                self.make_token(Tag::ExprEnd, start)
            }
            b'{' => {
                self.index += 1;
                self.push_mode(Mode::Expression);
                self.make_token(Tag::ExprStart, start)
            }
            _ => {
                while (self.index as usize) < self.buffer.len() {
                    let ch = self.buf(self.index);
                    if ch == b'{' || ch == b'}' || ch == 0 {
                        break;
                    }
                    self.index += 1;
                }
                self.make_token(Tag::Text, start)
            }
        }
    }

    fn next_inline_code(&mut self) -> Token {
        let start = self.index;

        if self.index as usize >= self.buffer.len() {
            return self.make_token(Tag::Eof, start);
        }

        let c = self.buf(self.index);

        match c {
            0 => self.make_token(Tag::Eof, start),
            b'`' => {
                self.index += 1;
                self.pop_mode();
                self.make_token(Tag::CodeInlineEnd, start)
            }
            _ => {
                while (self.index as usize) < self.buffer.len() {
                    let ch = self.buf(self.index);
                    if ch == b'`' || ch == 0 {
                        break;
                    }
                    self.index += 1;
                }
                self.make_token(Tag::Text, start)
            }
        }
    }

    fn next_code_block(&mut self) -> Token {
        let start = self.index;

        if self.index as usize >= self.buffer.len() {
            return self.make_token(Tag::Eof, start);
        }

        let c = self.buf(self.index);

        // Check for closing fence at start of line
        if self.index == self.line_start && c == b'`' && self.peek_ahead("```") {
            self.index += 3;
            self.pop_mode();
            return self.make_token(Tag::CodeFenceEnd, start);
        }

        match c {
            0 => self.make_token(Tag::Eof, start),
            b'\n' => {
                self.index += 1;
                self.line_start = self.index;
                self.make_token(Tag::Newline, start)
            }
            _ => {
                while (self.index as usize) < self.buffer.len() {
                    let ch = self.buf(self.index);
                    if ch == b'\n' || ch == 0 {
                        break;
                    }
                    if self.index == self.line_start && ch == b'`' && self.peek_ahead("```") {
                        break;
                    }
                    self.index += 1;
                }
                self.make_token(Tag::Text, start)
            }
        }
    }

    fn is_jsx_start(&self) -> bool {
        if self.index as usize + 1 >= self.buffer.len() {
            return false;
        }
        let next_char = self.buf(self.index + 1);
        match next_char {
            b'/' | b'>' => true,
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => true,
            _ => false,
        }
    }

    fn peek_ahead(&self, needle: &str) -> bool {
        let needle = needle.as_bytes();
        let idx = self.index as usize;
        if idx + needle.len() > self.buffer.len() {
            return false;
        }
        &self.buffer[idx..idx + needle.len()] == needle
    }

    fn buf(&self, idx: u32) -> u8 {
        let i = idx as usize;
        if i < self.buffer.len() {
            self.buffer[i]
        } else {
            0
        }
    }

    fn make_token(&self, tag: Tag, start: u32) -> Token {
        Token {
            tag,
            loc: Loc {
                start,
                end: self.index,
            },
        }
    }

    fn push_mode(&mut self, mode: Mode) {
        self.mode_stack.push(self.mode);
        self.mode = mode;
    }

    fn pop_mode(&mut self) {
        self.mode = self.mode_stack.pop().unwrap_or(Mode::Markdown);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_heading() {
        let source = "# Hello World\n";
        let mut tokenizer = Tokenizer::new(source);

        let tok1 = tokenizer.next();
        assert_eq!(Tag::HeadingStart, tok1.tag);

        let tok2 = tokenizer.next();
        assert_eq!(Tag::Text, tok2.tag);
        assert_eq!("Hello World", &source[tok2.loc.start as usize..tok2.loc.end as usize]);

        let tok3 = tokenizer.next();
        assert_eq!(Tag::Newline, tok3.tag);
    }

    #[test]
    fn tokenize_jsx_element() {
        let source = "<Component />";
        let mut tokenizer = Tokenizer::new(source);

        let tok1 = tokenizer.next();
        assert_eq!(Tag::JsxTagStart, tok1.tag);

        let tok2 = tokenizer.next();
        assert_eq!(Tag::JsxIdentifier, tok2.tag);
        assert_eq!(
            "Component",
            &source[tok2.loc.start as usize..tok2.loc.end as usize]
        );

        let tok3 = tokenizer.next();
        assert_eq!(Tag::JsxSelfClose, tok3.tag);
    }

    #[test]
    fn tokenize_expression() {
        let source = "{state.count}";
        let mut tokenizer = Tokenizer::new(source);

        let tok1 = tokenizer.next();
        assert_eq!(Tag::ExprStart, tok1.tag);

        let tok2 = tokenizer.next();
        assert_eq!(Tag::Text, tok2.tag);
        assert_eq!(
            "state.count",
            &source[tok2.loc.start as usize..tok2.loc.end as usize]
        );

        let tok3 = tokenizer.next();
        assert_eq!(Tag::ExprEnd, tok3.tag);
    }

    #[test]
    fn tokenize_frontmatter() {
        let source = "---\ntitle: Hello\n---\n";
        let mut tokenizer = Tokenizer::new(source);

        let tok1 = tokenizer.next();
        assert_eq!(Tag::FrontmatterStart, tok1.tag);
    }
}
