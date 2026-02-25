# Hypernote in Pika: Integration Plan

Generative UI for Nostr bots, rendered natively in a SwiftUI chat app with cryptographic authorization.

## What This Is

Pika is a SwiftUI + Rust chat app that communicates over MLS-encrypted Nostr messages (kind 443 wrapper). Bots already have a CLI for sending text messages over the same encrypted channel, with support for different inner event kinds (text, reactions, typing indicators).

This plan adds a new inner kind: **hypernote** — markdown with JSX components that bots generate to create rich, interactive UI.

Most interactions are simple: the user picks an option, answers a question, submits a form. The response goes back to the bot as a structured chat message — like LLM tool-use responses. But when an action has real-world weight (authorizing a payment, publishing a post, voting in a poll), the app constructs and signs a Nostr event. The signed event is cryptographic proof — verifiable by anyone, forgeable by no one. Two tiers: chat for the common case, crypto when it matters.

## Parser: `hypernote-mdx` (this crate)

The parser is a pure Rust crate. No Zig dependency, no C FFI, no cross-compilation gymnastics. Pika adds it as a path dependency:

```toml
# In pika's rust/Cargo.toml
hypernote-mdx = { path = "../hypernote-mdx" }
```

This replaces the earlier `zig-mdx` prototype and the `feat/zig-mdx-rust-backend` branch. That branch's `build.rs` (Zig static lib, C ABI, iOS target mapping) is no longer needed.

### API

```rust
// Parse MDX source into AST
let ast = hypernote_mdx::parse(source);

// Serialize AST to JSON string (crosses UniFFI boundary to Swift)
let json = hypernote_mdx::serialize_tree(&ast);

// Render AST back to canonical MDX source
let source = hypernote_mdx::render(&ast);
```

### JSON AST Format

`serialize_tree()` returns a JSON object. This is the interface boundary — Swift receives this string via UniFFI and deserializes it into native Swift types for rendering.

```json
{
  "type": "root",
  "children": [
    { "type": "heading", "level": 1, "children": [{ "type": "text", "value": "Hello" }] },
    { "type": "paragraph", "children": [{ "type": "text", "value": "Some text" }] },
    {
      "type": "mdx_jsx_element",
      "name": "Card",
      "attributes": [{ "name": "title", "type": "literal", "value": "My Card" }],
      "children": [
        { "type": "mdx_jsx_self_closing", "name": "TextInput", "attributes": [
          { "name": "name", "type": "literal", "value": "message" },
          { "name": "placeholder", "type": "literal", "value": "Type here..." }
        ]}
      ]
    }
  ],
  "source": "...",
  "errors": []
}
```

### Node Types

| Type | Fields | Notes |
|------|--------|-------|
| `root` | `children`, `source`, `errors` | Top-level wrapper |
| `heading` | `level`, `children` | Level 1-6 |
| `paragraph` | `children` | Block of inline content |
| `text` | `value` | Raw text content |
| `strong` | `children` | Bold |
| `emphasis` | `children` | Italic |
| `code_inline` | `value` | Inline code |
| `code_block` | `value`, `lang?` | Fenced code block |
| `link` | `url`, `children` | Hyperlink |
| `image` | `url`, `children` | Image (children = alt text) |
| `blockquote` | `children` | Block quote |
| `list_unordered` | `children` | Bullet list |
| `list_ordered` | `children` | Numbered list |
| `list_item` | `children` | List item |
| `hr` | — | Horizontal rule |
| `hard_break` | — | Explicit line break |
| `mdx_jsx_element` | `name`, `attributes`, `children` | `<Card>...</Card>` |
| `mdx_jsx_self_closing` | `name`, `attributes` | `<TextInput />` |
| `mdx_jsx_fragment` | `children` | `<>...</>` |
| `mdx_text_expression` | `value` | `{form.name}` inline |
| `mdx_flow_expression` | `value` | `{expression}` block-level |
| `frontmatter` | `format`, `value` | `format` is `"yaml"` or `"json"` |

### JSX Attributes

Each attribute has `name`, `type`, and optional `value`:

```json
{ "name": "action", "type": "literal", "value": "approve" }
{ "name": "data", "type": "expression", "value": "form.message" }
```

`type` is `"literal"` (string value) or `"expression"` (dynamic `{...}` value).

### Frontmatter

Both YAML and JSON frontmatter are supported:

- **YAML** (`---\n...\n---`) — standard markdown frontmatter
- **JSON** (`` ```hnmd\n{...}\n``` ``) — for `.hnmd` files with action definitions

The parser stores the raw string in `value` without interpreting it. Over Nostr events, there is no frontmatter — metadata lives in event tags. Frontmatter is only for `.hnmd` files on disk.

## The Format

### In a Nostr event (over MLS)

Hypernote messages use a new inner event kind inside the existing kind 443 MLS wrapper. Pika's message handling already switches on inner kind for text, reactions, and typing indicators — this is one more case.

**Content field:** Pure MDX. No fencing, no metadata, no JSON envelope. Just markdown and JSX.

```
# Invoice from @merchant

**Service:** API hosting (June 2026)

<Card>
  <Caption>Amount</Caption>
  <Heading>50,000 sats</Heading>
  <Caption>Expires in 12 minutes</Caption>
</Card>

<HStack gap="4">
  <SubmitButton action="reject" variant="secondary">Reject</SubmitButton>
  <SubmitButton action="approve" variant="danger">Authorize Payment</SubmitButton>
</HStack>
```

**Tags:** All metadata lives in tags. Action definitions use JSON-in-tag:

```json
["actions", "{\"approve\":{\"kind\":21121,\"content\":\"\",\"tags\":[[\"p\",\"bot-pubkey-hex\"],[\"invoice\",\"lnbc500u1pj9nrzy...\"],[\"amount\",\"50000\"],[\"memo\",\"API hosting — June 2026\"],[\"authorization\",\"single-use-payment\"]],\"confirm\":\"Pay 50,000 sats for API hosting?\"},\"reject\":{\"kind\":21122,\"content\":\"\",\"tags\":[[\"p\",\"bot-pubkey-hex\"],[\"invoice\",\"lnbc500u1pj9nrzy...\"],[\"status\",\"rejected\"]]}}"]
```

Simple metadata uses plain tags:

```json
["title", "Payment Authorization"]
```

### As a .hnmd file (optional)

For portability and hand-editing, `.hnmd` files can include a JSON headmatter block. This is a convenience for passing hypernotes around outside of Nostr events — the headmatter maps to what would otherwise be event tags.

````
```hnmd
{
  "title": "Payment Authorization",
  "actions": {
    "approve": {
      "kind": 21121,
      "tags": [["p", "{{bot.pubkey}}"], ["invoice", "lnbc500u1pj9nrzy..."]],
      "confirm": "Pay 50,000 sats for API hosting?"
    }
  }
}
```

# Invoice from @merchant

<Card>
  <Heading>50,000 sats</Heading>
</Card>
````

The headmatter is not part of the Nostr event format. It exists only for files.

### All messages go through the renderer

Every bot message is parsed by `hypernote-mdx`. Plain markdown is just the subset of MDX with no JSX components. A message that says `**hello**` renders with bold text. A message with `<Card>` renders a card. There is no flag or opt-in — the renderer handles the full spectrum from plain text to rich interactive UI.

## Architecture

Four layers. Three in Rust, one in Swift.

```
Bot sends MLS message (kind 443 wrapper, hypernote inner kind)
  │
  ▼
┌─────────────────────────────────────────┐
│  RUST                                   │
│                                         │
│  Layer 1: PARSE (this crate)            │
│  hypernote_mdx::parse() → AST           │
│  hypernote_mdx::serialize_tree() → JSON  │
│  Extract action defs from tags → struct  │
│                                         │
│  Layer 2: SCOPE                         │
│  Simple dot-path evaluation             │
│  (form.fieldName, bot.pubkey)           │
│                                         │
│  Layer 3: ACTIONS                       │
│  Tier 1: Chat action → MLS reply        │
│  Tier 2: Signed action → Nostr event    │
│                                         │
└──────────────────┬──────────────────────┘
                   │ AST JSON string (via UniFFI)
                   ▼
┌─────────────────────────────────────────┐
│  SWIFT                                  │
│                                         │
│  Layer 4: RENDER                        │
│  Walk AST JSON → SwiftUI views          │
│  Local @State for form inputs           │
│  On submit: dispatch {action, form} to  │
│  Rust                                   │
│                                         │
└─────────────────────────────────────────┘
```

### Layer 1: Parse (Rust — this crate)

`hypernote-mdx` handles tokenization, parsing, AST construction, and JSON serialization. It's the only layer currently implemented.

In Pika's `core/storage.rs`, where messages are converted to `ChatMessage` structs, the integration point is:

```rust
use hypernote_mdx;

// When building a ChatMessage from a decrypted MLS message:
let content = decrypted_message.content;
let ast_json = hypernote_mdx::serialize_tree(&hypernote_mdx::parse(&content));

// Parse action definitions from inner event tags (if present)
let actions: Option<HashMap<String, ActionDef>> = inner_event.tags.iter()
    .find(|t| t[0] == "actions")
    .and_then(|t| serde_json::from_str(&t[1]).ok());
```

**Additions to Pika's Rust layer:**

```rust
/// Add to pika_core state types
#[derive(Debug, Clone, uniffi::Record)]
pub struct HypernoteData {
    pub ast_json: String,                        // JSON AST from hypernote_mdx
    pub actions: Option<String>,                 // Raw JSON of action definitions (if any)
    pub title: Option<String>,                   // From tags
}

/// Extend ChatMessage with optional hypernote data
pub struct ChatMessage {
    pub id: String,
    pub sender_pubkey: String,
    pub sender_name: Option<String>,
    pub content: String,                         // Raw MDX source (fallback)
    pub hypernote: Option<HypernoteData>,        // Parsed hypernote data (if hypernote kind)
    pub timestamp: i64,
    pub is_mine: Bool,
    pub delivery: MessageDeliveryState,
}

/// Action definition (parsed from ["actions", "{...}"] tag)
#[derive(Debug, Clone, Deserialize)]
pub struct ActionDef {
    pub kind: u16,                               // Nostr event kind to publish
    pub content: Option<String>,                 // Template: "{{form.message}}"
    pub tags: Option<Vec<Vec<String>>>,          // May contain {{}} templates
    pub confirm: Option<String>,                 // Confirmation prompt text
}
```

### Layer 2: Scope (Rust)

Minimal expression evaluator for template interpolation in action definitions. v1 supports only simple dot-path resolution:

- `form.fieldName` → value from the form dict Swift sends on action submit
- `bot.pubkey` → the bot's pubkey (the message sender)
- `user.pubkey` → the current user's pubkey

No pipe filters, no defaults operator, no complex expressions. These can be added later.

```rust
fn interpolate_template(
    template: &str,
    form: &HashMap<String, String>,
    bot_pubkey: &str,
    user_pubkey: &str,
) -> String {
    // Replace all {{path}} occurrences with resolved values
    // e.g. "{{form.message}}" → "Hello world"
    // e.g. "{{bot.pubkey}}" → "abcd1234..."
    let re = regex::Regex::new(r"\{\{(\w+)\.(\w+)\}\}").unwrap();
    re.replace_all(template, |caps: &regex::Captures| {
        match (&caps[1], &caps[2]) {
            ("form", field) => form.get(field).cloned().unwrap_or_default(),
            ("bot", "pubkey") => bot_pubkey.to_string(),
            ("user", "pubkey") => user_pubkey.to_string(),
            _ => String::new(),
        }
    }).to_string()
}
```

### Layer 3: Actions (Rust)

There are two tiers of actions. Most bot interactions are conversational — the user picks an option, answers a question, fills in a value. These don't need cryptography. Only actions with real-world weight (signing a payment, publishing a post, voting in a poll) produce signed Nostr events.

#### Tier 1: Chat actions (the common case)

The user taps a button or submits a form. Pika sends a structured JSON reply back to the bot over the existing MLS chat — the same way LLM tool-use responses work. No Nostr event, no signing, no relay publishing. Just a message the bot can parse.

```
User taps "Option B"
  → Pika sends chat message: {"action": "choose", "value": "B"}
  → Bot receives it as a tool result and continues the conversation
```

Chat actions are the default. If an action name is referenced by a `SubmitButton` but has no corresponding entry in the `["actions", "..."]` tag, it's a chat action. Rust collects the form data, wraps it as `{"action": "<name>", "form": {...}}`, and sends it as a **regular text message** (same inner kind as normal chat) back to the bot over MLS.

This covers:
- Multiple-choice questions ("Which model? A, B, or C")
- Form submissions ("What's your name?")
- Confirmations ("Ready to proceed? Yes / No")
- Any interaction where the bot is the only audience

```rust
/// New AppAction variants
AppAction::HypernoteChatAction {
    chat_id: String,
    action_name: String,
    form: HashMap<String, String>,
}
// → Rust serializes {"action": "<name>", "form": {...}} and sends as regular MLS text message
```

#### Tier 2: Signed actions (when it matters)

When the `["actions", "..."]` tag defines an action with a `kind`, it's a signed action. Rust constructs a real Nostr event, signs it with the user's key, and publishes it to relays. The signed event is cryptographic proof — verifiable by anyone, forgeable by no one.

```
User taps "Authorize Payment"
  → Pika looks up action def: kind 21121, tags with invoice details
  → Interpolates {{form.amount}} etc.
  → Signs Nostr event with user's key
  → Publishes to relays
  → Bot (or any third party) can verify the signature
```

This is for:
- Payments and financial authorizations
- Publishing content to Nostr (posts, reactions, polls)
- Any action where a third party needs to verify the user's intent

```rust
AppAction::HypernoteSignedAction {
    chat_id: String,
    action_name: String,
    form: HashMap<String, String>,
}
// → Rust looks up ActionDef, interpolates templates, builds Nostr event, signs, publishes to relays
```

**Dispatch logic:**

When Swift dispatches either action, Rust:

1. Finds the `HypernoteData` for the message
2. Checks if the action name exists in the parsed actions JSON
3. **If no definition found** → chat action. Serialize `{"action": "<name>", "form": {...}}`, send as regular MLS text message
4. **If definition found** → signed action:
   a. Deserialize the `ActionDef`
   b. Interpolate `{{...}}` templates in content and tags
   c. Construct a Nostr event (kind, content, tags)
   d. Sign with the user's key (already available in Pika's session via `sess.keys`)
   e. Publish to relays via `client.send_event_to()`

Buttons are not disabled after use. Actions are idempotent — if a user taps twice, two responses are sent. The bot or receiving service handles deduplication.

### Layer 4: Render (Swift)

A recursive SwiftUI view builder that walks the AST JSON. This is a fresh implementation — don't build on the `feat/zig-mdx-rust-backend` branch's `MarkdownMessageContent`. Design from scratch for the full component catalog.

Swift receives the AST JSON string from `HypernoteData.ast_json` via UniFFI. Deserialize into Swift types:

```swift
struct AstNode: Decodable {
    let type: String
    let value: String?
    let level: Int?
    let name: String?
    let url: String?
    let lang: String?
    let format: String?
    let attributes: [AstAttribute]?
    let children: [AstNode]?
}

struct AstAttribute: Decodable {
    let name: String
    let type: String          // "literal" or "expression"
    let value: String?
}
```

Render recursively:

```swift
@ViewBuilder
func renderNode(_ node: AstNode, form: Binding<[String: String]>, onAction: @escaping (String) -> Void) -> some View {
    switch node.type {
    case "heading":
        // node.level determines font
    case "paragraph":
        // Render children inline
    case "text":
        Text(node.value ?? "")
    case "strong":
        // Render children with .bold()
    case "mdx_jsx_element", "mdx_jsx_self_closing":
        renderComponent(node, form: form, onAction: onAction)
    // ... etc
    default:
        EmptyView()
    }
}
```

**Markdown nodes → SwiftUI:**

| AST Node | SwiftUI |
|----------|---------|
| `heading` (1-6) | `Text().font(.title/.title2/.title3/...)` |
| `paragraph` | `Text` with inline children |
| `text` | `Text(value)` |
| `strong` | `.bold()` |
| `emphasis` | `.italic()` |
| `code_inline` | `.font(.system(.body, design: .monospaced))` |
| `code_block` | Monospace `Text` with background |
| `link` | `Link` or `Text` with `.underline()` |
| `image` | `AsyncImage` |
| `list_ordered/unordered` | `VStack` with bullets/numbers |
| `blockquote` | Styled with leading border |
| `hr` | `Divider()` |

**JSX components** map to the component catalog (see below).

**Unknown components** render their children with a subtle visual indicator (light dashed border or dimmed style) so content is never lost but it's visible that something wasn't recognized.

**Form state** is local `@State` / `@Observable` in Swift, scoped to the message view. TextInput binds to a local `[String: String]` dict keyed by the input's `name` attribute. When a SubmitButton is tapped, Swift dispatches the action name + form dict to Rust:

```swift
// In the message view
@State private var formState: [String: String] = [:]

// When SubmitButton tapped:
manager.dispatch(.hypernoteChatAction(
    chatId: chat.id,
    actionName: "choose",
    form: formState
))
```

## Component Catalog

The catalog is not a fixed spec. The app supports a set of components. The bot operator describes available components in the LLM's system prompt. The LLM generates MDX using only what it was told about. Unknown components degrade gracefully.

### Starting catalog for Pika

**Layout:**

| Component | Props | SwiftUI | Notes |
|-----------|-------|---------|-------|
| `Card` | `title?` | `GroupBox` or styled `VStack` | Visual container with boundary |
| `VStack` | `gap?` | `VStack(spacing:)` | Vertical layout |
| `HStack` | `gap?` | `HStack(spacing:)` | Horizontal layout |

**Content:**

| Component | Props | SwiftUI | Notes |
|-----------|-------|---------|-------|
| `Heading` | `level? (1-3)` | `Text().font(.title/.title2/.title3)` | Section heading |
| `Body` | — | `Text().font(.body)` | Paragraph text |
| `Caption` | — | `Text().font(.caption).foregroundStyle(.secondary)` | Muted text |

**Nostr Data:**

| Component | Props | SwiftUI | Notes |
|-----------|-------|---------|-------|
| `Profile` | `pubkey` | Custom view (avatar + name) | Lazy-fetched via Pika's existing relay infra |
| `Note` | `id` | Custom view (note content) | Lazy-fetched via Pika's existing relay infra |

**Interactive:**

| Component | Props | SwiftUI | Notes |
|-----------|-------|---------|-------|
| `TextInput` | `name`, `placeholder?` | `TextField` | Binds to local form state by `name` |
| `SubmitButton` | `action`, `variant?` | `Button` | Chat action (default) or signed action if defined in tags |

**SubmitButton variants** communicate intent, not color:
- `primary` (default) → `.borderedProminent`
- `secondary` → `.bordered`
- `danger` → `.borderedProminent` + destructive role

### Nostr embeds

`<Profile pubkey="npub1..."/>` and `<Note id="note1..."/>` use Pika's existing lazy-fetch pattern:

1. Swift renders a placeholder (loading indicator)
2. Dispatches a fetch request to Rust (new `AppAction` variant)
3. Rust fetches the profile/event from relays (using the existing profile cache and relay pool)
4. Rust emits a state update with the resolved data
5. Swift re-renders with the actual content

This is consistent with how Pika already handles profile lookups for chat participants.

## Transport

Hypernote messages use the existing MLS-encrypted transport. No new relay connections, no new encryption schemes.

```
Bot CLI
  │
  │  new command: send-hypernote
  │  args: --content <mdx-string>
  │        --actions <json-string>   (optional)
  │        --title <string>          (optional)
  │
  ▼
Kind 443 MLS wrapper
  └── Inner event: new hypernote kind (pick a kind number, e.g. 42 or similar)
        ├── content: raw MDX
        ├── tags: [["actions", "{...}"], ["title", "..."], ...]
        └── (same MLS group, same relays, same everything)
```

**Bot CLI integration:**

The bot CLI already has a command for sending text messages. Add a parallel command for hypernote messages that:

1. Takes MDX content as the primary argument (string or file path)
2. Optionally takes an actions JSON string
3. Constructs the inner event with the hypernote kind + appropriate tags
4. Wraps in MLS and publishes through the existing bot infrastructure

**Pika message handling:**

In `core/session.rs` where inner events are processed, add a match arm for the hypernote kind:

```rust
match inner_kind {
    Kind::Custom(9) => { /* existing text message handling */ }
    Kind::Custom(7) => { /* existing reaction handling */ }
    Kind::Custom(HYPERNOTE_KIND) => {
        // Parse with hypernote_mdx, extract tags, build HypernoteData
        // Attach to ChatMessage as .hypernote field
    }
    // ...
}
```

In `ChatView.swift`, check `message.hypernote` — if present, render with the hypernote renderer instead of the plain text/markdown renderer.

## Implementation Checklist

### Rust changes (in pika_core)

- [ ] Add `hypernote-mdx` as workspace dependency
- [ ] Define `HypernoteData` struct with UniFFI export
- [ ] Add `hypernote: Option<HypernoteData>` to `ChatMessage`
- [ ] Pick an inner kind number for hypernote messages
- [ ] Handle hypernote kind in session notification loop
- [ ] Parse MDX content with `hypernote_mdx::parse()` + `serialize_tree()`
- [ ] Extract `["actions", "..."]` tag into `HypernoteData.actions`
- [ ] Add `AppAction::HypernoteChatAction` — serialize form to JSON, send as MLS text
- [ ] Add `AppAction::HypernoteSignedAction` — interpolate templates, build event, sign, publish
- [ ] Implement `interpolate_template()` for `{{dot.path}}` resolution

### Swift changes (in iOS app)

- [ ] Add `AstNode` / `AstAttribute` Decodable types
- [ ] Build `HypernoteRenderer` view — recursive AST walker producing SwiftUI
- [ ] Implement markdown node rendering (heading, paragraph, text, strong, emphasis, code, link, image, list, blockquote, hr, hard_break)
- [ ] Implement component catalog (Card, VStack, HStack, Heading, Body, Caption, TextInput, SubmitButton)
- [ ] Unknown component fallback — render children with subtle dashed border
- [ ] Local `@State` form dict per message, bound to TextInput by `name`
- [ ] SubmitButton dispatches `HypernoteChatAction` or `HypernoteSignedAction`
- [ ] In `ChatView`, route `message.hypernote != nil` to `HypernoteRenderer`

### Bot CLI changes

- [ ] Add `send-hypernote` command (or flag on existing send command)
- [ ] Accept `--content` (MDX string or file) and `--actions` (JSON string)
- [ ] Construct inner event with hypernote kind + tags
- [ ] Send through existing MLS infrastructure

## Demo Scenarios

### Demo 1: Chat action (simple — no crypto)

Bot sends a message with no action tags — just MDX content:

```
Which language should I use for the backend?

<HStack gap="4">
  <SubmitButton action="choose" variant="secondary">Rust</SubmitButton>
  <SubmitButton action="choose" variant="secondary">Go</SubmitButton>
  <SubmitButton action="choose" variant="secondary">TypeScript</SubmitButton>
</HStack>
```

User taps "Rust". Pika sends `{"action": "choose", "form": {}}` as a regular text message back to the bot. The bot sees the tool result and continues the conversation. No signing, no relays. Just chat.

### Demo 2: Signed action (crypto)

Bot sends MDX with an action tag:

```
# Quick Note

<Card>
  <Caption>Post to Nostr</Caption>
  <TextInput name="message" placeholder="What's on your mind?" />
  <SubmitButton action="post" variant="primary">Publish</SubmitButton>
</Card>
```

With action tag:

```json
["actions", "{\"post\":{\"kind\":1,\"content\":\"{{form.message}}\",\"tags\":[]}}"]
```

User types a message, taps Publish. Pika sees that `post` has an `ActionDef` with `kind: 1`. It constructs a kind 1 Nostr event with the message as content, signs it with the user's key, publishes to relays. A real Nostr post, created from bot-generated UI.

### Demo 3: Plain markdown (no components)

Bot sends a regular message:

```
Here's what I found:

**3 matching results:**

1. nostr-sdk — Rust library for Nostr
2. nostr-tools — JavaScript library
3. python-nostr — Python library

Let me know which one to investigate.
```

Renders with rich formatting (bold, numbered list) even though there are no JSX components. Same renderer, same code path. The bot doesn't need to know or care about hypernote — plain markdown just works.

## Future Milestones (Not In Scope Now)

- **Richer expression evaluation:** pipe filters (`| format_date`), defaults (`// 'Anon'`), conditionals
- **Confirmation flows:** biometric auth for high-stakes actions (payments, transfers)
- **Nostr queries in tags:** let the bot declare relay subscriptions so the UI shows live data
- **Each/ForEach component:** iterate over lists of data
- **Select/Option components:** dropdowns and pickers
- **NumberInput component:** numeric input with keyboard type
- **Live-updating UI:** bot updates a previous message via replaceable events
- **Custom component registration:** bot declares new component types at runtime
- **Theming:** visual customization layer (the semantic model makes this additive)
