pub fn synthesis_prompt(query: &str, evidence: &[String]) -> String {
    let mut text = String::new();
    text.push_str("You are a retrieval reasoner. Answer only from the provided evidence.\n");
    text.push_str("Return compact markdown with direct citations in a JSON block.\n\n");
    text.push_str("USER QUERY:\n");
    text.push_str(query);
    text.push_str("\n\nEVIDENCE:\n");
    for (idx, item) in evidence.iter().enumerate() {
        text.push_str(&format!("{}. {item}\n", idx + 1));
    }
    text.push_str("\nOutput format:\n");
    text.push_str("{\"answer_markdown\":\"...\",\"confidence\":0.0,\"citations\":[\"node-id\"]}\n");
    text
}
