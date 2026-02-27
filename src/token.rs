/// Token represents a single lexical unit in MDX source.
/// Tokens track their position but not their text content -
/// use Loc indices into the source buffer to retrieve text.
#[derive(Debug, Clone, Copy)]
pub struct Token {
    pub tag: Tag,
    pub loc: Loc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Loc {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    // Markdown block-level tokens
    HeadingStart,
    ParagraphStart,
    CodeFenceStart,
    CodeFenceEnd,
    ListItemUnordered,
    ListItemOrdered,
    CheckboxUnchecked,
    CheckboxChecked,
    BlockquoteStart,
    Hr,
    BlankLine,

    // Table tokens
    Pipe,

    // Markdown inline tokens
    Text,
    StrongStart,
    StrongEnd,
    EmphasisStart,
    EmphasisEnd,
    CodeInlineStart,
    CodeInlineEnd,
    LinkStart,
    LinkEnd,
    LinkUrlStart,
    LinkUrlEnd,
    ImageStart,
    HardBreak,

    // MDX Expression tokens
    ExprStart,
    ExprEnd,

    // JSX tokens
    JsxTagStart,
    JsxTagEnd,
    JsxCloseTag,
    JsxSelfClose,
    JsxFragmentStart,
    JsxFragmentClose,
    JsxIdentifier,
    JsxDot,
    JsxColon,
    JsxEqual,
    JsxString,
    JsxAttrExprStart,

    // Frontmatter tokens
    FrontmatterStart,
    FrontmatterEnd,
    FrontmatterContent,

    // ESM tokens
    EsmImport,
    EsmExport,

    // Whitespace and structural
    Newline,
    Space,
    Indent,

    // Special
    Eof,
    Invalid,
}

impl Tag {
    pub fn symbol(&self) -> &'static str {
        match self {
            Tag::HeadingStart => "#",
            Tag::StrongStart | Tag::StrongEnd => "**",
            Tag::EmphasisStart | Tag::EmphasisEnd => "*",
            Tag::CodeInlineStart | Tag::CodeInlineEnd => "`",
            Tag::LinkStart => "[",
            Tag::LinkEnd => "]",
            Tag::LinkUrlStart => "(",
            Tag::LinkUrlEnd => ")",
            Tag::ImageStart => "![",
            Tag::ExprStart => "{",
            Tag::ExprEnd => "}",
            Tag::JsxTagStart => "<",
            Tag::JsxTagEnd => ">",
            Tag::JsxCloseTag => "</",
            Tag::JsxSelfClose => "/>",
            Tag::JsxFragmentStart => "<>",
            Tag::JsxFragmentClose => "</>",
            Tag::JsxDot => ".",
            Tag::JsxColon => ":",
            Tag::JsxEqual => "=",
            Tag::Pipe => "|",
            Tag::CheckboxUnchecked => "[ ]",
            Tag::CheckboxChecked => "[x]",
            Tag::Hr => "---",
            Tag::FrontmatterStart | Tag::FrontmatterEnd => "---",
            Tag::Newline => "\\n",
            Tag::Eof => "EOF",
            other => other.name(),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Tag::HeadingStart => "heading_start",
            Tag::ParagraphStart => "paragraph_start",
            Tag::CodeFenceStart => "code_fence_start",
            Tag::CodeFenceEnd => "code_fence_end",
            Tag::ListItemUnordered => "list_item_unordered",
            Tag::ListItemOrdered => "list_item_ordered",
            Tag::CheckboxUnchecked => "checkbox_unchecked",
            Tag::CheckboxChecked => "checkbox_checked",
            Tag::BlockquoteStart => "blockquote_start",
            Tag::Hr => "hr",
            Tag::Pipe => "pipe",
            Tag::BlankLine => "blank_line",
            Tag::Text => "text",
            Tag::StrongStart => "strong_start",
            Tag::StrongEnd => "strong_end",
            Tag::EmphasisStart => "emphasis_start",
            Tag::EmphasisEnd => "emphasis_end",
            Tag::CodeInlineStart => "code_inline_start",
            Tag::CodeInlineEnd => "code_inline_end",
            Tag::LinkStart => "link_start",
            Tag::LinkEnd => "link_end",
            Tag::LinkUrlStart => "link_url_start",
            Tag::LinkUrlEnd => "link_url_end",
            Tag::ImageStart => "image_start",
            Tag::HardBreak => "hard_break",
            Tag::ExprStart => "expr_start",
            Tag::ExprEnd => "expr_end",
            Tag::JsxTagStart => "jsx_tag_start",
            Tag::JsxTagEnd => "jsx_tag_end",
            Tag::JsxCloseTag => "jsx_close_tag",
            Tag::JsxSelfClose => "jsx_self_close",
            Tag::JsxFragmentStart => "jsx_fragment_start",
            Tag::JsxFragmentClose => "jsx_fragment_close",
            Tag::JsxIdentifier => "jsx_identifier",
            Tag::JsxDot => "jsx_dot",
            Tag::JsxColon => "jsx_colon",
            Tag::JsxEqual => "jsx_equal",
            Tag::JsxString => "jsx_string",
            Tag::JsxAttrExprStart => "jsx_attr_expr_start",
            Tag::FrontmatterStart => "frontmatter_start",
            Tag::FrontmatterEnd => "frontmatter_end",
            Tag::FrontmatterContent => "frontmatter_content",
            Tag::EsmImport => "esm_import",
            Tag::EsmExport => "esm_export",
            Tag::Newline => "newline",
            Tag::Space => "space",
            Tag::Indent => "indent",
            Tag::Eof => "eof",
            Tag::Invalid => "invalid",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_symbol() {
        assert_eq!("#", Tag::HeadingStart.symbol());
        assert_eq!("**", Tag::StrongStart.symbol());
        assert_eq!("{", Tag::ExprStart.symbol());
    }
}
