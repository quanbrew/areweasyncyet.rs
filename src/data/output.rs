use super::input::InputData;
use super::{Issue, IssueId, RFC_REPO, RUSTC_REPO};
use crate::fetcher::IssueData;
use crate::query::Repo;
use serde::Serialize;
use std::collections::HashMap;

pub type OutputData = HashMap<String, Vec<Item>>;

#[derive(Debug, Serialize)]
pub struct Item {
    pub title: String,
    pub rfc: Option<Rfc>,
    pub tracking: Option<Issue>,
    pub issue_label: Option<String>,
    pub issues: Option<Vec<Issue>>,
    pub stabilized: Option<Stabilization>,
    pub unresolved: Option<Rfc>,
}

#[derive(Debug, Serialize)]
pub struct Rfc {
    issue: Issue,
    url: String,
    merged: bool,
}

#[derive(Debug, Serialize)]
pub struct Stabilization {
    pub version: String,
    pub pr: Issue,
}

pub fn generate(input: InputData, issue_data: &IssueData) -> OutputData {
    let builder = Builder { issue_data };
    builder.build(input)
}

struct Builder<'a> {
    issue_data: &'a IssueData,
}

impl Builder<'_> {
    fn build(&self, input: InputData) -> OutputData {
        input
            .into_iter()
            .map(|(key, items)| {
                let items = items
                    .into_iter()
                    .map(|item| {
                        Item {
                            title: item.title,
                            rfc: self.convert_rfc(item.rfc),
                            tracking: self.get_optional_issue(&*RUSTC_REPO, item.tracking),
                            issues: item.issue_label.as_ref().map(|label| {
                                self.issue_data
                                    // TODO Don't clone?
                                    .labels[&(RUSTC_REPO.clone(), label.clone())]
                                    .iter()
                                    .map(|id| self.get_issue(&*RUSTC_REPO, *id))
                                    .collect()
                            }),
                            issue_label: item.issue_label,
                            stabilized: item.stabilized.map(|stabilized| Stabilization {
                                version: stabilized.version,
                                pr: self.get_issue(&*RUSTC_REPO, stabilized.pr),
                            }),
                            unresolved: self.convert_rfc(item.unresolved),
                        }
                    })
                    .collect();
                (key, items)
            })
            .collect()
    }

    fn convert_rfc(&self, rfc: Option<String>) -> Option<Rfc> {
        let rfc = rfc?;
        let dash = rfc.find('-');
        let number = rfc[..dash.unwrap_or_else(|| rfc.len())]
            .parse()
            .expect("unexpected rfc number");
        let (url, merged) = if dash.is_none() {
            (
                format!("https://github.com/rust-lang/rfcs/pull/{}", rfc),
                false,
            )
        } else {
            let hash = rfc.find('#').unwrap_or_else(|| rfc.len());
            let (page, frag) = rfc.split_at(hash);
            (
                format!("https://rust-lang.github.io/rfcs/{}.html{}", page, frag),
                true,
            )
        };
        let issue = self.get_issue(&*RFC_REPO, number);
        Some(Rfc { issue, url, merged })
    }

    fn get_optional_issue(&self, repo: &Repo, id: Option<IssueId>) -> Option<Issue> {
        id.map(|id| self.get_issue(repo, id))
    }

    fn get_issue(&self, repo: &Repo, id: IssueId) -> Issue {
        // TODO Don't clone?
        self.issue_data.issues[&(repo.clone(), id)].clone()
    }
}
