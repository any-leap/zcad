//! Rules Engine - Condition-based entity validation and automation

use crate::entity::{Entity, EntityId, PropertyValue};
use crate::entity_store::EntityStore;
use std::collections::HashMap;

/// Rule condition
#[derive(Debug, Clone)]
pub enum Condition {
    PropertyEquals { key: String, value: PropertyValue },
    PropertyGreater { key: String, value: f64 },
    PropertyLess { key: String, value: f64 },
    HasTag(String),
    HasRelation { relation_type: String },
    And(Vec<Condition>),
    Or(Vec<Condition>),
    Not(Box<Condition>),
}

impl Condition {
    pub fn evaluate(&self, entity: &Entity) -> bool {
        match self {
            Condition::PropertyEquals { key, value } => {
                entity.get_property(key).map(|v| v == value).unwrap_or(false)
            }
            Condition::PropertyGreater { key, value } => {
                entity.get_property(key)
                    .and_then(|v| v.as_number())
                    .map(|n| n > *value)
                    .unwrap_or(false)
            }
            Condition::PropertyLess { key, value } => {
                entity.get_property(key)
                    .and_then(|v| v.as_number())
                    .map(|n| n < *value)
                    .unwrap_or(false)
            }
            Condition::HasTag(tag) => entity.has_tag(tag),
            Condition::HasRelation { relation_type } => {
                entity.relations.iter().any(|r| r.relation_type == *relation_type)
            }
            Condition::And(conditions) => conditions.iter().all(|c| c.evaluate(entity)),
            Condition::Or(conditions) => conditions.iter().any(|c| c.evaluate(entity)),
            Condition::Not(condition) => !condition.evaluate(entity),
        }
    }
}

/// Rule action
#[derive(Debug, Clone)]
pub enum RuleAction {
    SetProperty { key: String, value: PropertyValue },
    AddTag(String),
    RemoveTag(String),
    Warn(String),
    Error(String),
}

/// Rule result
#[derive(Debug, Clone)]
pub struct RuleResult {
    pub rule_name: String,
    pub entity_id: EntityId,
    pub action: RuleAction,
    pub message: Option<String>,
}

/// A rule definition
#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub description: String,
    pub condition: Condition,
    pub actions: Vec<RuleAction>,
    pub enabled: bool,
}

impl Rule {
    pub fn new(name: impl Into<String>, condition: Condition) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            condition,
            actions: Vec::new(),
            enabled: true,
        }
    }

    pub fn with_action(mut self, action: RuleAction) -> Self {
        self.actions.push(action);
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
}

/// Rules engine
pub struct RuleEngine {
    rules: Vec<Rule>,
}

impl Default for RuleEngine {
    fn default() -> Self { Self::new() }
}

impl RuleEngine {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// Evaluate rules for a single entity
    pub fn evaluate(&self, entity: &Entity) -> Vec<RuleResult> {
        let mut results = Vec::new();
        
        for rule in &self.rules {
            if !rule.enabled { continue; }
            
            if rule.condition.evaluate(entity) {
                for action in &rule.actions {
                    results.push(RuleResult {
                        rule_name: rule.name.clone(),
                        entity_id: entity.id,
                        action: action.clone(),
                        message: match action {
                            RuleAction::Warn(msg) | RuleAction::Error(msg) => Some(msg.clone()),
                            _ => None,
                        },
                    });
                }
            }
        }
        
        results
    }

    /// Evaluate rules for all entities in store
    pub fn evaluate_all(&self, store: &EntityStore) -> Vec<RuleResult> {
        store.iter().flat_map(|e| self.evaluate(e)).collect()
    }

    /// Get warnings only
    pub fn get_warnings(&self, store: &EntityStore) -> Vec<RuleResult> {
        self.evaluate_all(store)
            .into_iter()
            .filter(|r| matches!(r.action, RuleAction::Warn(_)))
            .collect()
    }

    /// Get errors only
    pub fn get_errors(&self, store: &EntityStore) -> Vec<RuleResult> {
        self.evaluate_all(store)
            .into_iter()
            .filter(|r| matches!(r.action, RuleAction::Error(_)))
            .collect()
    }
}
