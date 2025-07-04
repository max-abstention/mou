use async_trait::async_trait;
use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use swiftide::{
    agents::Agent,
    chat_completion::{self, ChatCompletionResponse, ToolCall},
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Response {
    /// Messages coming from an agent
    Chat(chat_completion::ChatMessage),
    /// Short activity updates
    Activity(String),
    /// A chat has been renamed
    RenameChat(String),
    /// A chat branch has been renamed
    RenameBranch(String),
    /// Backend system messages (kwaak currently just renders these as system chat like messages)
    BackendMessage(String),
    /// A command has been completed
    Completed,
    /// A streamed chunk of a chat message. Keeps it simple now to just return the updated string
    ChatChunk(chat_completion::ChatCompletionResponse),

    /// A request for feedback is requested
    ToolFeedbackRequested(ToolCall, Option<serde_json::Value>),
}

/// A responder reacts to updates from agents and other updates from commands
///
/// Backend defines the interface, frontend can define ways to handle the responses
///
/// Backend expects the responder to know where it should go (i.e. the chat id)
///
/// Responders are cloned often, so keep them small and cheap
///
/// TODO: Consider, perhaps with the new structure, less concrete methods are needed
/// and the frontend just uses a oneoff handler for each command
#[async_trait]
pub trait Responder: std::fmt::Debug + Send + Sync + DynClone + 'static {
    /// Generic handler for command responses
    async fn send(&self, response: Response);

    /// Messages from an agent
    async fn agent_message(&self, _agent: &Agent, message: chat_completion::ChatMessage) {
        self.send(Response::Chat(message)).await;
    }

    /// System messages from the backend
    async fn system_message(&self, message: &str) {
        self.send(Response::BackendMessage(message.to_string()))
            .await;
    }

    /// State updates with a message from the backend
    async fn update(&self, state: &str) {
        self.send(Response::Activity(state.to_string())).await;
    }

    /// A chat has been renamed
    async fn rename_chat(&self, name: &str) {
        self.send(Response::RenameChat(name.to_string())).await;
    }

    /// A git branch has been renamed
    async fn rename_branch(&self, branch_name: &str) {
        self.send(Response::RenameBranch(branch_name.to_string()))
            .await;
    }

    /// Streamed chunks from completions
    async fn completion_chunk(&self, completion: ChatCompletionResponse) {
        self.send(Response::ChatChunk(completion)).await;
    }

    async fn tool_feedback_requested(
        &self,
        tool_call: ToolCall,
        payload: Option<serde_json::Value>,
    ) {
        self.send(Response::ToolFeedbackRequested(tool_call, payload))
            .await;
    }
}
