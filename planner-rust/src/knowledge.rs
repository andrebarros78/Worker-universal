use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeQuery {
    pub query_id: String,
    pub question: String,
    pub max_sources: usize,
    pub deadline_ms: u64,
    pub cost_budget_microunits: u64,
}

impl KnowledgeQuery {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.query_id.trim().is_empty() || self.question.trim().is_empty() {
            return Err("query id and question are required");
        }
        if self.max_sources == 0 {
            return Err("max_sources must be positive");
        }
        if self.deadline_ms == 0 {
            return Err("deadline_ms must be positive");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeSource {
    pub source_id: String,
    pub title: String,
    pub excerpt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeResult {
    pub provider_id: String,
    pub sources: Vec<KnowledgeSource>,
    pub cost_microunits: u64,
}

pub trait ResearchProvider {
    fn provider_id(&self) -> &str;
    fn research(&self, query: &KnowledgeQuery) -> Result<KnowledgeResult, String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReasonerRequest {
    pub request_id: String,
    pub instruction: String,
    pub evidence: Vec<KnowledgeSource>,
    pub deadline_ms: u64,
    pub cost_budget_microunits: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReasonerOutput {
    pub provider_id: String,
    pub text: String,
    pub evidence_ids: Vec<String>,
    pub cost_microunits: u64,
}

pub trait ReasonerProvider {
    fn provider_id(&self) -> &str;
    fn reason(&self, request: &ReasonerRequest) -> Result<ReasonerOutput, String>;
}

#[derive(Debug, Default, Clone)]
pub struct LocalResearchProvider {
    entries: BTreeMap<String, KnowledgeSource>,
}

impl LocalResearchProvider {
    pub fn insert(&mut self, source: KnowledgeSource) {
        self.entries.insert(source.source_id.clone(), source);
    }
}

impl ResearchProvider for LocalResearchProvider {
    fn provider_id(&self) -> &str {
        "research.local"
    }

    fn research(&self, query: &KnowledgeQuery) -> Result<KnowledgeResult, String> {
        query.validate().map_err(str::to_string)?;
        let terms = query
            .question
            .to_lowercase()
            .split_whitespace()
            .filter(|term| term.len() >= 3)
            .map(str::to_string)
            .collect::<Vec<_>>();

        let mut matches = self
            .entries
            .values()
            .filter(|source| {
                let haystack = format!("{} {}", source.title, source.excerpt).to_lowercase();
                terms.iter().any(|term| haystack.contains(term))
            })
            .cloned()
            .collect::<Vec<_>>();
        matches.sort_by(|left, right| left.source_id.cmp(&right.source_id));
        matches.truncate(query.max_sources);

        Ok(KnowledgeResult {
            provider_id: self.provider_id().to_string(),
            sources: matches,
            cost_microunits: 0,
        })
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LocalRuleReasoner;

impl ReasonerProvider for LocalRuleReasoner {
    fn provider_id(&self) -> &str {
        "reasoner.local.rules"
    }

    fn reason(&self, request: &ReasonerRequest) -> Result<ReasonerOutput, String> {
        if request.request_id.trim().is_empty()
            || request.instruction.trim().is_empty()
            || request.deadline_ms == 0
        {
            return Err("invalid reasoner request".to_string());
        }
        let mut evidence = request.evidence.clone();
        evidence.sort_by(|left, right| left.source_id.cmp(&right.source_id));
        let evidence_ids = evidence
            .iter()
            .map(|item| item.source_id.clone())
            .collect::<Vec<_>>();
        let text = evidence
            .iter()
            .map(|item| item.excerpt.as_str())
            .collect::<Vec<_>>()
            .join(" | ");

        Ok(ReasonerOutput {
            provider_id: self.provider_id().to_string(),
            text,
            evidence_ids,
            cost_microunits: 0,
        })
    }
}

pub fn validate_reasoner_output(
    request: &ReasonerRequest,
    output: &ReasonerOutput,
) -> Result<(), String> {
    if output.text.trim().is_empty() {
        return Err("empty_reasoning".to_string());
    }
    if output.cost_microunits > request.cost_budget_microunits {
        return Err("reasoner_cost_budget_exceeded".to_string());
    }
    let allowed = request
        .evidence
        .iter()
        .map(|source| source.source_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for evidence_id in &output.evidence_ids {
        if !allowed.contains(evidence_id.as_str()) {
            return Err(format!("unknown_evidence:{evidence_id}"));
        }
    }
    Ok(())
}
