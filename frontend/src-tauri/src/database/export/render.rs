use super::{DocumentSelection, ExportError, ExportNotes, ExportRequest, MeetingRecord};
use crate::audio::source::AudioSource;
use serde_json::Value;
use std::fmt::Write;

pub(super) fn channel(source: AudioSource) -> &'static str {
    match source {
        AudioSource::Microphone => "Microphone",
        AudioSource::System => "System audio",
        AudioSource::Mixed => "Mixed audio",
        AudioSource::Unknown => "Unknown",
    }
}

pub(super) fn filename(title: &str, extension: &str) -> String {
    let mut stem = String::new();
    let mut utf16_len = 0;
    for ch in title.trim().chars() {
        let ch = if ch.is_control() || "<>:\"/\\|?*".contains(ch) {
            '-'
        } else {
            ch
        };
        if stem.len() + ch.len_utf8() > 180 || utf16_len + ch.len_utf16() > 120 {
            break;
        }
        stem.push(ch);
        utf16_len += ch.len_utf16();
    }
    let stem = stem.trim_matches(|c: char| c.is_whitespace() || c == '.');
    let stem = if stem.is_empty() { "meeting" } else { stem };
    let device = stem
        .split('.')
        .next()
        .unwrap_or(stem)
        .trim_end()
        .to_uppercase();
    let reserved = matches!(
        device.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CLOCK$" | "CONIN$" | "CONOUT$"
    ) || ["COM", "LPT"].iter().any(|prefix| {
        device.strip_prefix(prefix).is_some_and(|n| {
            matches!(
                n,
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        })
    });
    format!("{}{stem}.{extension}", if reserved { "_" } else { "" })
}

fn inline(text: &str) -> String {
    let mut output = String::new();
    for ch in text.chars() {
        if "\\`*_{}[]<>!#|".contains(ch) {
            output.push('\\');
        }
        output.push(if ch == '\n' || ch == '\r' { ' ' } else { ch });
    }
    output
}

fn quote(text: &str) -> String {
    text.lines()
        .map(|line| format!("> {}", inline(line)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(inline)
        .unwrap_or_else(|| "Unknown".into())
}
fn time(seconds: Option<f64>) -> String {
    match seconds.filter(|s| s.is_finite() && *s >= 0.0) {
        Some(s) => {
            let s = s.floor() as u64;
            format!("{:02}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
        }
        None => "Time unavailable".into(),
    }
}

fn original_markdown(notes: &ExportNotes) -> Result<String, ExportError> {
    let empty = notes.notes_json.is_none()
        || notes
            .blocks
            .as_ref()
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty);
    if let Some(markdown) = &notes.notes_markdown {
        // Revision zero belongs to the unchanged legacy-note migration. Current
        // saves always set ready/unavailable together with their projection.
        if notes.conversion_status == "ready" || notes.revision == 0 {
            return Ok(markdown.clone());
        }
    }
    if empty && notes.notes_markdown.as_ref().is_none_or(|s| s.is_empty()) {
        return Ok("No original notes were saved.".into());
    }
    Err(ExportError::new("conversion", "Original notes have no current Markdown projection. Export JSON to preserve all saved blocks."))
}

pub(super) fn markdown(
    record: &MeetingRecord,
    request: &ExportRequest,
) -> Result<String, ExportError> {
    let mut output = format!("# {}\n\n", inline(&record.meeting.title));
    writeln!(output, "- Meeting ID: {}", inline(&record.meeting.id)).unwrap();
    writeln!(output, "- Status: {}", inline(&record.meeting.status)).unwrap();
    writeln!(output, "- Created: {}", inline(&record.meeting.created_at)).unwrap();
    writeln!(
        output,
        "- Capture start: {}",
        record
            .meeting
            .started_at
            .as_deref()
            .map(inline)
            .unwrap_or_else(|| "Unknown; creation date is not a capture time".into())
    )
    .unwrap();
    writeln!(
        output,
        "- Capture end: {}",
        record
            .meeting
            .ended_at
            .as_deref()
            .map(inline)
            .unwrap_or_else(|| "Unknown".into())
    )
    .unwrap();
    writeln!(
        output,
        "- Timezone: {}",
        record
            .meeting
            .timezone
            .as_deref()
            .map(inline)
            .unwrap_or_else(|| "Unknown".into())
    )
    .unwrap();
    writeln!(output, "- Snapshot: {}", inline(&record.exported_at)).unwrap();
    writeln!(
        output,
        "- Transcript coverage: {}",
        record.transcript_coverage
    )
    .unwrap();
    if !record.meeting.participants.is_empty() {
        writeln!(
            output,
            "- Manual participants: {}",
            record
                .meeting
                .participants
                .iter()
                .map(|s| inline(s))
                .collect::<Vec<_>>()
                .join(", ")
        )
        .unwrap();
    }
    if !record.meeting.tags.is_empty() {
        writeln!(
            output,
            "- Tags: {}",
            record
                .meeting
                .tags
                .iter()
                .map(|s| inline(s))
                .collect::<Vec<_>>()
                .join(", ")
        )
        .unwrap();
    }
    output.push('\n');
    match record.transcript_coverage.as_str() {
        "incomplete_active" => output.push_str("> Incomplete snapshot: recording or transcript finalization is still in progress. Only committed passages available at the snapshot time are included.\n\n"),
        "incomplete_interrupted" => output.push_str("> Incomplete snapshot: this meeting was interrupted. Only available committed passages are included.\n\n"),
        _ => {}
    }
    let include_original = request.document != DocumentSelection::Approved;
    let mut referenced_markers = std::collections::HashSet::new();
    if include_original {
        output.push_str("## My notes\n\n");
        if let Some(notes) = &record.original_notes {
            writeln!(output, "Saved revision: {}\n", notes.revision).unwrap();
            let mut body = original_markdown(notes)?;
            for (index, marker) in notes.timestamp_links.iter().enumerate() {
                referenced_markers.insert(index);
                body = body.replace(
                    &format!("meetily-note://{}", marker.link.id),
                    &format!("#note-timestamp-{}", index + 1),
                );
            }
            output.push_str(&body);
            output.push_str("\n\n");
        } else {
            output.push_str("No original notes were saved.\n\n");
        }
    }
    if request.document != DocumentSelection::Original {
        let version = record.approved().ok_or_else(|| ExportError::new("no_approval", "No approved notes are available. Select original notes or export the complete JSON record."))?;
        let body = version.document.as_ref().and_then(|d| d.get("markdown")).and_then(Value::as_str)
            .ok_or_else(|| ExportError::new("conversion", "Approved notes have no current Markdown projection. Export JSON to preserve the saved document."))?;
        writeln!(
            output,
            "## Approved enhanced notes\n\nVersion: {} · Saved revision: {}\n",
            inline(&version.id),
            version.revision
        )
        .unwrap();
        match version.source_stale {
            Some(true) => output.push_str("> Uses an earlier or unavailable source revision. Review the retained evidence against current sources.\n\n"),
            None => output.push_str("> Source freshness is unknown. Review this document and its evidence.\n\n"),
            _ => {}
        }
        let mut body = body.to_owned();
        if let Some(notes) = &record.original_notes {
            for (index, marker) in notes.timestamp_links.iter().enumerate() {
                let href = format!("meetily-note://{}", marker.link.id);
                if body.contains(&href) {
                    referenced_markers.insert(index);
                    body = body.replace(&href, &format!("#note-timestamp-{}", index + 1));
                }
            }
        }
        output.push_str(&body);
        output.push_str("\n\n");
        if let Some(claims) = version
            .document
            .as_ref()
            .and_then(|d| d.get("claims"))
            .and_then(Value::as_array)
            .filter(|c| !c.is_empty())
        {
            output.push_str("## Supporting evidence\n\nExcerpts identify saved source text. Review whether they support each claim; channel labels do not identify participants.\n\n");
            if version
                .document
                .as_ref()
                .and_then(|d| d.get("user_edited"))
                .and_then(Value::as_bool)
                == Some(true)
            {
                output.push_str("> This document was edited. Review retained structured claims against the edited notes.\n\n");
            }
            let sources = version
                .source_snapshot
                .as_ref()
                .and_then(|s| s.get("sources"))
                .and_then(Value::as_array);
            for (index, claim) in claims.iter().enumerate() {
                writeln!(
                    output,
                    "### Claim {}: {}\n\n- ID: {}\n- Kind: {}\n- Review status: {}",
                    index + 1,
                    text(claim, "text"),
                    text(claim, "id"),
                    text(claim, "kind"),
                    text(claim, "status")
                )
                .unwrap();
                for key in ["owner", "deadline"] {
                    if let Some(value) = claim.get(key).and_then(Value::as_str) {
                        writeln!(output, "- {key}: {}", inline(value)).unwrap();
                    }
                }
                output.push('\n');
                for citation in claim
                    .get("sources")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    let id = citation
                        .get("source_id")
                        .and_then(Value::as_str)
                        .unwrap_or("Unknown source");
                    let saved = sources.and_then(|items| {
                        items
                            .iter()
                            .find(|s| s.get("source_id").and_then(Value::as_str) == Some(id))
                    });
                    writeln!(output, "Source: {}\n", inline(id)).unwrap();
                    if let Some(saved) = saved {
                        if saved.get("kind").and_then(Value::as_str) == Some("transcript") {
                            writeln!(
                                output,
                                "{} – {} · {}\n",
                                time(saved.get("audio_start_time").and_then(Value::as_f64)),
                                time(saved.get("audio_end_time").and_then(Value::as_f64)),
                                text(saved, "audio_source_label")
                            )
                            .unwrap();
                            let current = record
                                .transcripts
                                .iter()
                                .find(|p| id.strip_prefix("transcript:") == Some(p.id.as_str()));
                            if current.is_none() {
                                output.push_str(
                                    "The saved passage is no longer in the current transcript.\n\n",
                                );
                            } else if saved.get("text").and_then(Value::as_str)
                                != current.map(|p| p.text.as_str())
                            {
                                output.push_str(
                                    "The saved passage differs from the current transcript.\n\n",
                                );
                            }
                        } else {
                            output.push_str("My notes · frozen source revision\n\n");
                        }
                    } else {
                        output.push_str("Saved source details and timing unavailable; the retained excerpt follows.\n\n");
                    }
                    output.push_str(&quote(
                        citation
                            .get("excerpt")
                            .and_then(Value::as_str)
                            .unwrap_or("Excerpt unavailable"),
                    ));
                    output.push_str("\n\n");
                }
            }
        }
    }
    if !referenced_markers.is_empty() {
        if let Some(notes) = &record.original_notes {
            if !notes.timestamp_links.is_empty() {
                output.push_str("## Note timestamps\n\n");
            }
            for (index, marker) in notes.timestamp_links.iter().enumerate() {
                if !referenced_markers.contains(&index) {
                    continue;
                }
                writeln!(
                    output,
                    "<a id=\"note-timestamp-{}\"></a>\n\n### {}\n\nMarker: {} · Block: {}\n",
                    index + 1,
                    time(Some(marker.link.recording_offset)),
                    inline(&marker.link.id),
                    inline(&marker.link.block_id)
                )
                .unwrap();
                if let Some(passage) = &marker.passage {
                    writeln!(
                        output,
                        "Source: {} · {} – {} · {}\n\n{}\n",
                        text(passage, "id"),
                        time(passage.get("audio_start_time").and_then(Value::as_f64)),
                        time(passage.get("audio_end_time").and_then(Value::as_f64)),
                        text(passage, "audio_source_label"),
                        quote(passage.get("text").and_then(Value::as_str).unwrap_or(""))
                    )
                    .unwrap();
                } else {
                    output.push_str("Transcript passage unavailable in this snapshot.\n\n");
                }
            }
        }
    }
    if request.include_transcript {
        output.push_str("## Full transcript\n\n");
        if record.transcripts.is_empty() {
            output.push_str("No committed transcript passages are available.\n\n");
        }
        for p in &record.transcripts {
            writeln!(
                output,
                "### {} – {} · {}\n\nPassage: {}\n",
                time(p.audio_start_time.map(|t|t+p.timeline_offset)),
                time(p.audio_end_time.map(|t|t+p.timeline_offset)),
                p.audio_source_label,
                inline(&p.id)
            )
            .unwrap();
            if p.audio_start_time.is_none() {
                writeln!(
                    output,
                    "Saved display timestamp: {} (recording-relative time unknown)\n",
                    inline(&p.timestamp)
                )
                .unwrap();
            }
            output.push_str(&quote(&p.text));
            output.push_str("\n\n");
        }
    }
    if !record.warnings.is_empty() {
        output.push_str("## Export notices\n\n");
        for warning in &record.warnings {
            writeln!(output, "- {}", inline(warning)).unwrap();
        }
    }
    Ok(output)
}
