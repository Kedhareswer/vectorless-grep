use crate::reasoner::planner::PlannerInput;

pub fn planner_prompt(input: &PlannerInput) -> String {
    let mut text = String::new();
    text.push_str("You are the reasoning planner for a document QA agent.\n");
    text.push_str("Pick exactly one next action. Be concise and strategic.\n");
    text.push_str("Return ONLY JSON with keys: stepType, objective, reasoning, decision.\n");
    text.push_str("Allowed stepType: search, inspect, synthesize, self_check, finish.\n");
    text.push_str("Allowed decision: continue, backtrack, stop.\n\n");
    text.push_str("STATE:\n");
    text.push_str(&format!("query: {}\n", input.query));
    text.push_str(&format!("stepCount: {}\n", input.step_count));
    text.push_str(&format!("backtrackCount: {}\n", input.backtrack_count));
    text.push_str(&format!("hasEvidence: {}\n", input.has_evidence));
    text.push_str(&format!(
        "lastConfidence: {}\n",
        input
            .last_confidence
            .map(|value| format!("{value:.2}"))
            .unwrap_or_else(|| "none".to_string())
    ));
    if !input.explored_sections.is_empty() {
        text.push_str("exploredSections:\n");
        for section in &input.explored_sections {
            text.push_str(&format!("- {section}\n"));
        }
    }

    text.push_str("\nStrategy hints:\n");
    text.push_str("- Use search before inspect when evidence is weak.\n");
    text.push_str("- Use synthesize only after evidence exists.\n");
    text.push_str("- Use self_check after synthesis.\n");
    text.push_str("- Use finish only when answer quality is sufficient.\n");
    text
}

pub fn synthesis_prompt(query: &str, evidence: &[String]) -> String {
    let mut text = String::new();
    text.push_str("You are a retrieval reasoner. Answer only from the provided evidence.\n");
    text.push_str(
        "If evidence is insufficient, explicitly say what is missing instead of guessing.\n",
    );
    text.push_str("Do not paste raw node ids in prose except inside citations.\n\n");
    text.push_str("USER QUERY:\n");
    text.push_str(query);
    text.push_str("\n\nEVIDENCE:\n");
    for (idx, item) in evidence.iter().enumerate() {
        text.push_str(&format!("{}. {item}\n", idx + 1));
    }
    text.push_str("\nOutput rules:\n");
    text.push_str("- If the query compares or relates files/documents, structure answer_markdown with headings:\n");
    text.push_str(
        "  1) What each file is about\n  2) How they are related\n  3) Gaps or uncertainty\n",
    );
    text.push_str("- Every substantive claim must be grounded by at least one citation id.\n");
    text.push_str("- citations must only contain ids that appear in evidence ([citation:...]).\n");
    text.push_str("\nReturn ONLY valid JSON with this exact shape:\n");
    text.push_str("{\"answer_markdown\":\"...\",\"confidence\":0.0,\"citations\":[\"node-id\"]}\n");
    text
}
