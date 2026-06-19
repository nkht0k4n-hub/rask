use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::client::Method;
use crate::project::Project;
use crate::{Error, Result};
use crate::{IdNameSet, RASK_CLIENT};

#[derive(Debug)]
pub struct Document;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRequest {
    content: String,
    description: String,
    project_id: usize,
    start_at: String,
    end_at: String,
    location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentResponse {
    pub id: usize,
    pub content: String,
    pub creator: IdNameSet,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub project: Option<IdNameSet>,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub location: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, ValueEnum, PartialEq, Display)]
pub enum DocType {
    New,
    GN,
    Other,
}

impl Document {
    pub fn list() -> Result<Vec<DocumentResponse>> {
        let client = RASK_CLIENT.get().ok_or(Error::NotInitialized)?;
        client
            .get("documents.json")?
            .json()
            .map_err(|e| Error::JsonDecode(e.to_string()))
    }

    pub fn save(data: DocumentRequest) -> Result<()> {
        let client = RASK_CLIENT.get().ok_or(Error::NotInitialized)?;
        let _ = client.send_request(Method::POST, "documents.json", Some(data))?;
        Ok(())
    }

    pub fn search(
        docs: &[DocumentResponse],
        id: Option<usize>,
        content: Option<&[String]>,
        creator_id: Option<usize>,
        creator_name: Option<&[String]>,
        description: Option<&[String]>,
        project_id: Option<usize>,
        project_name: Option<&[String]>,
        created_at: Option<DateTime<Utc>>,
        updated_at: Option<DateTime<Utc>>,
        start_at: Option<DateTime<Utc>>,
        end_at: Option<DateTime<Utc>>,
        term_day: Option<usize>,
    ) -> Vec<DocumentResponse> {
        // term_day が指定されない場合は当日(0日の範囲 = 完全一致相当)とする
        let term_duration = chrono::Duration::days(term_day.unwrap_or(0) as i64);

        // created_at / updated_at / start_at / end_at は String で保持されているため、
        // 範囲比較のためにその場で RFC3339 としてパースする。
        let parse_date = |s: &str| -> Option<DateTime<Utc>> {
            DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        };

        docs.iter()
            .filter(|doc| {
                let match_id = id.is_none() || id == Some(doc.id);

                let match_content = content.map_or(true, |c| {
                    c.is_empty() || c.iter().all(|kw| doc.content.contains(kw))
                });

                let match_creator_id = creator_id.is_none() || creator_id == Some(doc.creator.id);

                let match_creator_name = creator_name.map_or(true, |names| {
                    names.is_empty() || names.iter().all(|kw| doc.creator.name.contains(kw))
                });

                let match_description = description.map_or(true, |kws| {
                    kws.is_empty()
                        || kws
                            .iter()
                            .all(|kw| doc.description.as_ref().is_some_and(|d| d.contains(kw)))
                });

                let match_project_id = project_id.is_none()
                    || doc
                        .project
                        .as_ref()
                        .is_some_and(|p| project_id == Some(p.id));

                let match_project_name = project_name.map_or(true, |names| {
                    names.is_empty()
                        || names
                            .iter()
                            .all(|kw| doc.project.as_ref().is_some_and(|p| p.name.contains(kw)))
                });

                let within_created_at = created_at.is_none()
                    || created_at.is_some_and(|ca| {
                        parse_date(&doc.created_at).is_some_and(|doc_ca| {
                            let lower = ca - term_duration;
                            let upper = ca + term_duration;
                            lower <= doc_ca && doc_ca <= upper
                        })
                    });

                let within_updated_at = updated_at.is_none()
                    || updated_at.is_some_and(|ua| {
                        parse_date(&doc.updated_at).is_some_and(|doc_ua| {
                            let lower = ua - term_duration;
                            let upper = ua + term_duration;
                            lower <= doc_ua && doc_ua <= upper
                        })
                    });

                let within_start_at = start_at.is_none()
                    || start_at.is_some_and(|sa| {
                        doc.start_at
                            .as_deref()
                            .and_then(parse_date)
                            .is_some_and(|doc_start| {
                                let lower = sa - term_duration;
                                let upper = sa + term_duration;
                                lower <= doc_start && doc_start <= upper
                            })
                    });

                let within_end_at = end_at.is_none()
                    || end_at.is_some_and(|ea| {
                        doc.end_at
                            .as_deref()
                            .and_then(parse_date)
                            .is_some_and(|doc_end| {
                                let lower = ea - term_duration;
                                let upper = ea + term_duration;
                                lower <= doc_end && doc_end <= upper
                            })
                    });

                match_id
                    && match_content
                    && match_creator_id
                    && match_creator_name
                    && match_description
                    && match_project_id
                    && match_project_name
                    && within_created_at
                    && within_updated_at
                    && within_start_at
                    && within_end_at
            })
            .cloned()
            .collect()
    }
}

impl DocumentRequest {
    pub fn new<S>(
        content: String,
        description: String,
        project: S,
        start_at: String,
        end_at: String,
        location: String,
    ) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let project_id = Project::find_by_name(project)?.id;
        Ok(Self {
            content,
            description,
            project_id,
            start_at,
            end_at,
            location,
        })
    }
}
