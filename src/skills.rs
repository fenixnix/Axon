//! Skills Module
//!
//! Handles loading and managing Claude Code format skills.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SkillError {
    #[error("技能文件不存在: {0}")]
    FileNotFound(String),

    #[error("技能解析失败: {0}")]
    ParseError(String),

    #[error("技能加载失败: {0}")]
    LoadError(String),
}

/// Skill manifest (skill.json)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SkillManifest {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub allowed_tools: Vec<String>,
}

/// A loaded skill
#[derive(Debug)]
pub struct Skill {
    pub manifest: SkillManifest,
    pub instructions: String,
}

/// Skill loader
pub struct SkillLoader {
    skills_dir: PathBuf,
}

impl SkillLoader {
    pub fn new(skills_dir: PathBuf) -> Self {
        Self { skills_dir }
    }

    /// Load all skills from the skills directory
    pub async fn load_all(&self) -> Result<Vec<Skill>, SkillError> {
        let mut skills = Vec::new();

        if !self.skills_dir.exists() {
            return Ok(skills);
        }

        let mut entries = tokio::fs::read_dir(&self.skills_dir)
            .await
            .map_err(|e| SkillError::LoadError(e.to_string()))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| SkillError::LoadError(e.to_string()))?
        {
            let path = entry.path();

            if path.is_dir() {
                if let Ok(skill) = self.load_skill_dir(&path).await {
                    skills.push(skill);
                }
            } else if path.extension().is_some_and(|e| e == "md")
                && let Ok(skill) = self.load_skill_file(&path).await
            {
                skills.push(skill);
            }
        }

        Ok(skills)
    }

    /// Load skill from directory (skill.json + SKILL.md)
    async fn load_skill_dir(&self, dir: &Path) -> Result<Skill, SkillError> {
        let manifest_path = dir.join("skill.json");
        let md_path = dir.join("SKILL.md");

        let manifest = if manifest_path.exists() {
            let content = tokio::fs::read_to_string(&manifest_path)
                .await
                .map_err(|e| SkillError::LoadError(e.to_string()))?;
            serde_json::from_str(&content).map_err(|e| SkillError::ParseError(e.to_string()))?
        } else {
            return Err(SkillError::FileNotFound("Missing skill.json".to_string()));
        };

        let instructions = if md_path.exists() {
            tokio::fs::read_to_string(&md_path)
                .await
                .map_err(|e| SkillError::LoadError(e.to_string()))?
        } else {
            String::new()
        };

        Ok(Skill {
            manifest,
            instructions,
        })
    }

    /// Load skill from single .md file (with YAML frontmatter)
    async fn load_skill_file(&self, path: &Path) -> Result<Skill, SkillError> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| SkillError::LoadError(e.to_string()))?;

        if let Some(frontmatter) = Self::parse_frontmatter(&content) {
            Ok(Skill {
                manifest: frontmatter,
                instructions: content,
            })
        } else {
            Err(SkillError::ParseError("Invalid skill format".to_string()))
        }
    }

    /// Parse YAML frontmatter from markdown
    fn parse_frontmatter(content: &str) -> Option<SkillManifest> {
        if let Some(stripped) = content.strip_prefix("---")
            && let Some(end) = stripped.find("---")
        {
            let yaml = &stripped[..end];
            return serde_yaml::from_str(yaml).ok();
        }
        None
    }
}

/// Skill registry - manages loaded skills
pub struct SkillRegistry {
    skills: Vec<Skill>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self { skills: Vec::new() }
    }

    pub fn add(&mut self, skill: Skill) {
        self.skills.push(skill);
    }

    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.iter().find(|s| s.manifest.name == name)
    }

    pub fn list(&self) -> Vec<(&str, &str)> {
        self.skills
            .iter()
            .map(|s| (s.manifest.name.as_str(), s.manifest.description.as_str()))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.skills.len()
    }

    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = "---
name: test-skill
description: A test skill
allowed_tools:
  - read
  - write
---

# Test Skill
";

        let manifest = SkillLoader::parse_frontmatter(content);
        assert!(manifest.is_some());
        assert_eq!(manifest.unwrap().name, "test-skill");
    }

    #[test]
    fn test_skill_registry() {
        let mut registry = SkillRegistry::new();

        let skill = Skill {
            manifest: SkillManifest {
                name: "test".to_string(),
                description: "Test skill".to_string(),
                version: "1.0.0".to_string(),
                author: "".to_string(),
                allowed_tools: vec![],
            },
            instructions: "# Test".to_string(),
        };

        registry.add(skill);

        assert_eq!(registry.len(), 1);
        assert!(registry.get("test").is_some());
    }
}
