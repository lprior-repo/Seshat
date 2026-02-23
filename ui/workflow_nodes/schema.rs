use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WorkflowNode {
    HttpTrigger(HttpTriggerConfig),
    ScheduleTrigger(ScheduleTriggerConfig),
    ServiceCall(ServiceCallConfig),
    SendMessage(SendMessageConfig),
    DelayedMessage(DelayedMessageConfig),
    SaveToMemory(SaveToMemoryConfig),
    LoadFromMemory(LoadFromMemoryConfig),
    Delay(DelayConfig),
    Router(RouterConfig),
    WaitForWebhook(WaitForWebhookConfig),
    WaitForSignal(WaitForSignalConfig),
    RunCode(RunCodeConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpTriggerConfig {
    pub handler_name: String,
    pub method: HttpMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTriggerConfig {
    pub cron_expression: String,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCallConfig {
    pub service_name: String,
    pub handler_name: String,
    pub input: serde_json::Value,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageConfig {
    pub target_type: TargetType,
    pub service_name: String,
    pub key: Option<String>,
    pub handler_name: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelayedMessageConfig {
    pub target_type: TargetType,
    pub service_name: String,
    pub key: Option<String>,
    pub handler_name: String,
    pub input: serde_json::Value,
    pub delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetType {
    Service,
    VirtualObject,
    Workflow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveToMemoryConfig {
    pub key: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadFromMemoryConfig {
    pub key: String,
    pub default: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelayConfig {
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    pub branches: Vec<RouterBranch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterBranch {
    pub name: String,
    pub condition: String,
    pub next_node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitForWebhookConfig {
    pub webhook_name: String,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitForSignalConfig {
    pub signal_name: String,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunCodeConfig {
    pub code: String,
    pub language: CodeLanguage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodeLanguage {
    JavaScript,
    Python,
    Expression,
}

impl Default for WorkflowNode {
    fn default() -> Self {
        WorkflowNode::HttpTrigger(HttpTriggerConfig {
            handler_name: String::new(),
            method: HttpMethod::POST,
        })
    }
}
