use std::hint::black_box;

fn build_chat_corpus() -> Vec<String> {
    let mut corpus = Vec::with_capacity(512);

    for i in 0..512usize {
        let message = match i % 10 {
            0 => tiny_message_status(i),
            1 => tiny_message_follow_up(i),
            2 => tiny_message_link(i),
            3 => tiny_message_checklist(i),
            4 => tiny_message_quote(i),
            5 => tiny_message_inline_code(i),
            6 => tiny_message_reply(i),
            7 => medium_message_release_note(i),
            8 => medium_message_table(i),
            _ => mdx_message_card(i),
        };
        corpus.push(message);
    }

    corpus
}

fn tiny_message_status(i: usize) -> String {
    format!(
        "status update {i}: user-{owner} finished review batch {batch} and queued deploy window {window}\n",
        owner = i % 19,
        batch = (i % 7) + 1,
        window = (i % 5) + 1,
    )
}

fn tiny_message_follow_up(i: usize) -> String {
    format!(
        "- [ ] follow up with user-{owner}\n- [x] attach note-{i}\n",
        owner = i % 23,
    )
}

fn tiny_message_link(i: usize) -> String {
    format!("can someone review [ticket-{i}](https://example.com/tickets/{i}) before standup?\n")
}

fn tiny_message_checklist(i: usize) -> String {
    format!(
        "1. sync branch-{i}\n2. rerun `cargo test --test chat_workload`\n3. post summary for room-{room}\n",
        room = i % 11,
    )
}

fn tiny_message_quote(i: usize) -> String {
    format!(
        "> note from ops-{ops}: keep rollout group {group} under threshold {threshold}\n",
        ops = i % 9,
        group = i % 6,
        threshold = (i % 4) + 2,
    )
}

fn tiny_message_inline_code(i: usize) -> String {
    format!(
        "captured failure in `worker_{i}` after retry count `{retries}`\n",
        retries = (i % 5) + 1,
    )
}

fn tiny_message_reply(i: usize) -> String {
    format!(
        "replying to note-{parent}: markdown is fine here, but keep body under {limit} lines for mobile\n",
        parent = i.saturating_sub(1),
        limit = (i % 3) + 2,
    )
}

fn medium_message_release_note(i: usize) -> String {
    format!(
        "# release note {i}\n\nowner: user-{owner}\n\nThe chat payload for room-{room} now includes richer markdown sections, nested lists, and stable links for audit-{audit}.\n\n- status: ready\n- reviewer: user-{reviewer}\n- docs: [runbook-{i}](https://example.com/runbooks/{i})\n\n> ship after the final smoke pass for batch-{batch}\n",
        owner = i % 17,
        room = i % 13,
        audit = i % 29,
        reviewer = (i + 3) % 17,
        batch = (i % 5) + 1,
    )
}

fn medium_message_table(i: usize) -> String {
    format!(
        "| item | state | owner |\n| :--- | ---: | :--- |\n| alpha-{i} | {alpha_state} | user-{alpha_owner} |\n| beta-{i} | {beta_state} | user-{beta_owner} |\n| gamma-{i} | {gamma_state} | user-{gamma_owner} |\n",
        alpha_state = (i % 9) + 1,
        alpha_owner = i % 14,
        beta_state = (i % 7) + 2,
        beta_owner = (i + 1) % 14,
        gamma_state = (i % 5) + 3,
        gamma_owner = (i + 2) % 14,
    )
}

fn mdx_message_card(i: usize) -> String {
    let checked = if i % 2 == 0 { "true" } else { "false" };
    format!(
        "<Card>\n<Heading>room {room} digest {i}</Heading>\n<Body>message batch {batch} is ready for review by user-{owner}</Body>\n<VStack gap={gap}>\n<ChecklistItem name=\"confirm_{i}\" checked={checked} />\n<SubmitButton action=\"approve_{i}\" variant=\"primary\" />\n</VStack>\n</Card>\n",
        room = i % 21,
        batch = (i % 8) + 1,
        owner = i % 16,
        gap = (i % 4) + 4,
        checked = checked,
    )
}

#[test]
fn parse_then_serialize_chat_corpus_many_times() {
    let corpus = build_chat_corpus();
    assert_eq!(512, corpus.len(), "chat workload corpus should stay stable");

    let mut validated_json = 0usize;
    let mut corpus_submit_actions = 0usize;

    for message in &corpus {
        let ast = hypernote_mdx::parse(message);
        assert!(
            !ast.nodes.is_empty(),
            "parser returned an empty AST for message: {message}"
        );
        assert!(
            ast.errors.len() <= 16,
            "unexpected error volume for message: {message}\nerrors={:?}",
            ast.errors.iter().map(|e| e.tag.name()).collect::<Vec<_>>()
        );

        let json = hypernote_mdx::serialize_tree(&ast);
        let root: serde_json::Value =
            serde_json::from_str(&json).expect("chat workload JSON should be valid");
        assert_eq!("root", root["type"]);
        assert!(
            root["children"].is_array(),
            "root children should be an array"
        );
        assert!(root["errors"].is_array(), "root errors should be an array");

        if json.contains("\"submit_button\"") || json.contains("SubmitButton") {
            corpus_submit_actions += 1;
        }
        validated_json += 1;
    }

    assert_eq!(corpus.len(), validated_json);
    assert!(
        corpus_submit_actions > 0,
        "workload should include hypernote actions"
    );

    let rounds = 64usize;
    let mut total_messages = 0usize;
    let mut total_nodes = 0usize;
    let mut total_errors = 0usize;
    let mut total_json_bytes = 0usize;

    for _ in 0..rounds {
        for message in &corpus {
            let ast = hypernote_mdx::parse(message);
            let json = hypernote_mdx::serialize_tree(&ast);

            total_messages += 1;
            total_nodes += ast.nodes.len();
            total_errors += ast.errors.len();
            total_json_bytes += json.len();
        }
    }

    black_box(total_messages);
    black_box(total_nodes);
    black_box(total_errors);
    black_box(total_json_bytes);

    assert_eq!(rounds * corpus.len(), total_messages);
    assert!(total_nodes > total_messages * 4);
    assert!(total_json_bytes > total_messages * 64);
    assert!(total_errors < total_messages);
}
