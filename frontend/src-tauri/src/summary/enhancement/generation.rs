use super::work::{Store, Work};
use super::*;
use grounding::{Claim, Source};

fn subdivide(sources: &[Source]) -> Option<(Vec<Source>, Vec<Source>)> {
    if sources.len() > 1 {
        let mid = sources.len() / 2;
        return Some((sources[..mid].to_vec(), sources[mid..].to_vec()));
    }
    let source = sources.first()?;
    let mid = source
        .text
        .char_indices()
        .nth(source.text.chars().count() / 2)?
        .0;
    if mid == 0 {
        return None;
    }
    let mut left = source.clone();
    let mut right = source.clone();
    left.text = source.text[..mid].into();
    right.text = source.text[mid..].into();
    Some((vec![left], vec![right]))
}

async fn save(store: Option<&Store<'_>>, key: &str, work: &Work) -> Result<(), String> {
    if let Some(store) = store {
        store.save(key, work).await?;
    }
    Ok(())
}

pub(super) async fn run(
    config: &ProviderConfig,
    snapshot: &SourceSnapshot,
    template: &Template,
    instructions: &str,
    app_data_dir: Option<&PathBuf>,
    token: &CancellationToken,
    mut audit: Option<&mut Vec<Value>>,
    store: Option<&Store<'_>>,
) -> Result<Value, String> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .resolve(
            "localhost",
            std::net::SocketAddr::from((
                [127, 0, 0, 1],
                config
                    .ollama_endpoint
                    .as_deref()
                    .or(config.custom_endpoint.as_deref())
                    .and_then(|s| Url::parse(s).ok())
                    .and_then(|u| u.port_or_known_default())
                    .unwrap_or(80),
            )),
        )
        .build()
        .map_err(|_| "Cannot initialize inference client")?;
    let section_rules: Vec<_> = template
        .sections
        .iter()
        .map(|s| serde_json::json!({"title":s.title,"instruction":s.instruction}))
        .collect();
    let header=serde_json::json!({"template_sections":section_rules,"instructions":instructions,"coverage":snapshot.transcript_coverage,"summary_language":config.summary_language}).to_string();
    let system = format!(
        "{}\n{}",
        wire::SYSTEM,
        language_instruction(config.summary_language.as_deref())
    );
    let output_budget = if config.provider == LLMProvider::BuiltInAI {
        4096
    } else {
        config.max_tokens.unwrap_or(1024)
    };
    // Reserve room for a bounded, privacy-safe repair instruction as well.
    let budget = config
        .context
        .saturating_sub(system.len() + header.len() + 512 + output_budget as usize)
        .min(8192);
    let spans = wire::passages(&snapshot.sources);
    let chunks = grounding::source_chunks(&spans, budget)?;
    let count = chunks.len();
    let builtin_profile = if config.provider == LLMProvider::BuiltInAI {
        crate::summary::summary_engine::models::get_model_by_name(&config.model).map(|model| {
            let file = app_data_dir
                .and_then(|dir| {
                    crate::summary::summary_engine::models::get_model_path(dir, &config.model).ok()
                })
                .and_then(|path| std::fs::metadata(path).ok());
            serde_json::json!({"sampling":model.sampling,"chat_template":model.template,
                "file_bytes":file.as_ref().map(|f|f.len()),
                "file_modified":file.and_then(|f|f.modified().ok()).and_then(|t|
                    t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d|d.as_nanos().to_string())})
        })
    } else {
        None
    };
    let key=grounding::fingerprint(&serde_json::json!({"format":wire::FORMAT_VERSION,"source":snapshot.source_fingerprint,
        "system":system,"header":header,"template":template,"provider":format!("{:?}",config.provider),"model":config.model,
        "endpoint":config.destination.destination,"context":config.context,"output":output_budget,
        "temperature":config.temperature,"top_p":config.top_p,"builtin_profile":builtin_profile,
        "budget":budget,"planner":1}).to_string());
    let mut work = if let Some(store) = store {
        store.load(&key, count).await?
    } else {
        Work {
            total: count,
            ..Default::default()
        }
    };
    let mut claims = Vec::<Claim>::new();
    for (index, chunk) in chunks.into_iter().enumerate() {
        let mut stack = vec![(index.to_string(), chunk, 0usize)];
        let mut calls = 0;
        while let Some((part, sources, depth)) = stack.pop() {
            if token.is_cancelled() {
                return Err("Generation cancelled".into());
            }
            if let Some(cached) = work.accepted.get(&part) {
                // Revalidate persisted, canonical claims against precisely this partition.
                let valid = grounding::validate_claims(
                    grounding::GeneratedClaims {
                        claims: cached.clone(),
                    },
                    &sources,
                )
                .map_err(|_| "Stored generation checkpoint failed validation")?;
                claims.extend(valid);
                continue;
            }
            if work.split.contains(&part) {
                let (left, right) =
                    subdivide(&sources).ok_or("Invalid generation partition checkpoint")?;
                stack.push((format!("{part}R"), right, depth + 1));
                stack.push((format!("{part}L"), left, depth + 1));
                continue;
            }
            let prompt=format!("{header}\nChunk {}/{}. Extract relevant supported findings from these passages:\n{}",index+1,count,wire::payload(&sources));
            let grammar = wire::grammar(&sources, template);
            let mut accepted = None;
            let mut failure = "invalid_structure: response requires repair".to_string();
            for attempt in 0..2 {
                if token.is_cancelled() {
                    return Err("Generation cancelled".into());
                }
                calls += 1;
                work.attempts += 1;
                let request_system = if attempt == 0 {
                    system.clone()
                } else {
                    format!("{system}\nPrevious validation failure: {failure}. Correct that field. Return only the required JSON using supplied passage IDs.")
                };
                let mut wire_audit = Vec::new();
                let started = std::time::Instant::now();
                let response = llm_client::generate_summary_with_grammar(
                    &client,
                    &config.provider,
                    &config.model,
                    &config.api_key,
                    &request_system,
                    &prompt,
                    config.ollama_endpoint.as_deref(),
                    config.custom_endpoint.as_deref(),
                    Some(output_budget),
                    config.temperature,
                    config.top_p,
                    app_data_dir,
                    Some(token),
                    audit.as_ref().map(|_| &mut wire_audit),
                    Some(&grammar),
                )
                .await;
                let validation = match &response {
                    Ok(r) => {
                        work.prompt_tokens = r.prompt_tokens;
                        work.generated_tokens = r.generated_tokens;
                        if matches!(r.finish_reason.as_deref(), Some("length" | "context_limit")) {
                            Err("output_limit: model could not complete this part".into())
                        } else {
                            wire::resolve(&r.content, &sources)
                        }
                    }
                    Err(e) => Err(if e.contains("helper_incompatible:") {
                        "helper_incompatible: matching application helper required".into()
                    } else {
                        "inference_unavailable: selected inference provider failed".into()
                    }),
                };
                if let Some(audit) = audit.as_mut() {
                    audit.push(serde_json::json!({"chunk":index+1,"chunk_count":count,"partition":part,"attempt":attempt,
                        "system":request_system,"prompt":prompt,"source_budget_bytes":budget,"sources":sources,
                        "elapsed_ms":started.elapsed().as_millis(),"provider_responses":wire_audit,
                        "raw_response":response.as_ref().ok().map(|r|&r.content),
                        "finish_reason":response.as_ref().ok().and_then(|r|r.finish_reason.as_deref()),
                        "prompt_tokens":response.as_ref().ok().and_then(|r|r.prompt_tokens),
                        "generated_tokens":response.as_ref().ok().and_then(|r|r.generated_tokens),
                        "reasoning_stripped":response.as_ref().ok().map(|r|r.reasoning_stripped),
                        "validated_claims":validation.as_ref().ok(),"validation_error":validation.as_ref().err()}));
                }
                match validation {
                    Ok(valid) => {
                        accepted = Some(valid);
                        break;
                    }
                    Err(error) => {
                        failure = error;
                        if failure.starts_with("output_limit:")
                            || failure.starts_with("helper_incompatible:")
                            || failure.starts_with("inference_unavailable:")
                        {
                            break;
                        }
                    }
                }
            }
            if let Some(valid) = accepted {
                work.accepted.insert(part, valid.clone());
                claims.extend(valid);
                save(store, &key, &work).await?;
                continue;
            }
            if depth < 2
                && calls < 14
                && !failure.starts_with("helper_incompatible:")
                && !failure.starts_with("inference_unavailable:")
            {
                if let Some((left, right)) = subdivide(&sources) {
                    work.split.insert(part.clone());
                    save(store, &key, &work).await?;
                    stack.push((format!("{part}R"), right, depth + 1));
                    stack.push((format!("{part}L"), left, depth + 1));
                    continue;
                }
            }
            let code = failure
                .split(':')
                .next()
                .unwrap_or("invalid_structure")
                .to_string();
            work.error_code = Some(code.clone());
            work.failed_chunk = Some(index + 1);
            save(store, &key, &work).await?;
            let reason=match code.as_str() {
                "helper_incompatible"=>"The application helper is incompatible. Use the matching complete application bundle.",
                "inference_unavailable"=>"The selected model could not run. Check the model and provider settings.",
                "output_limit"=>"The model could not finish this part within its generation limit.",
                "invalid_reference"=>"The model could not produce valid source references.",
                _=>"The model could not produce the required structured result.",
            };
            return Err(format!("{reason} Part {} of {count} failed; {} completed parts can be reused on retry. Original and approved notes are preserved.",index+1,work.completed.len()));
        }
        work.completed.insert(index);
        save(store, &key, &work).await?;
    }
    if claims.is_empty() {
        work.accepted.clear();
        work.completed.clear();
        work.split.clear();
        work.error_code = Some("no_supported_content".into());
        save(store, &key, &work).await?;
        return Err("The model returned no supported content. Retry generation or select another model. Original and approved notes are preserved.".into());
    }
    let mut seen = std::collections::HashSet::new();
    claims.retain(|c| {
        seen.insert(format!(
            "{}:{}:{}",
            c.kind,
            c.text,
            serde_json::to_string(&c.sources).unwrap_or_default()
        ))
    });
    Ok(grounding::document(
        claims,
        snapshot,
        count,
        &template
            .sections
            .iter()
            .map(|s| s.title.clone())
            .collect::<Vec<_>>(),
    ))
}
