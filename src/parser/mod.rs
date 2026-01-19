pub mod archive_review;
pub mod challenge;
pub mod frontmatter;
pub mod inline_yaml;
pub mod markdown;
pub mod proposal;
pub mod requirement;
pub mod review;
pub mod scenario;
pub mod xml;

pub use archive_review::parse_archive_review_verdict;
pub use challenge::{parse_challenge_verdict, ChallengeParser};
pub use proposal::{parse_affected_specs, parse_latest_review, ReviewBlock};
pub use frontmatter::{
    calculate_body_checksum, calculate_checksum, has_frontmatter, is_stale, normalize_content,
    parse_document, parse_frontmatter_value, split_frontmatter, ParsedDocument,
};
pub use inline_yaml::{
    extract_yaml_blocks, extract_yaml_blocks_with_lines, parse_issue_blocks,
    parse_requirement_blocks, parse_task_blocks, parse_typed_yaml_blocks, YamlBlock,
};
pub use markdown::extract_heading_section;
pub use requirement::RequirementParser;
pub use review::parse_review_verdict;
pub use scenario::ScenarioParser;
pub use xml::{extract_xml_block, extract_xml_blocks, parse_xml_attributes, update_xml_blocks, wrap_in_xml, UpdateMode, XmlBlock};
