//! One bounded, persistent conversation per career, shared by both interfaces.
use crate::{codex::ChatMessage, Error, Result, Store};
use rusqlite::{params, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};

pub const MAX_PROMPT: usize = 8_000;
pub const MAX_ANSWER: usize = 128 * 1024;
pub const MAX_STORED_TEXT: usize = 256 * 1024;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Client {
    Desktop,
    Game,
}

impl Client {
    fn name(self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Game => "game",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: i64,
    pub role: String,
    pub text: String,
    pub state: String,
    pub client: Client,
}

pub fn validate_prompt(prompt: &str) -> Result<()> {
    if prompt.trim().is_empty() || prompt.len() > MAX_PROMPT {
        return Err(Error::Invalid(
            "Mesaj boş olamaz ve en fazla 8.000 bayt olabilir.".into(),
        ));
    }
    Ok(())
}

impl Store {
    pub fn conversation_model(&self) -> Result<Option<String>> {
        Ok(self
            .connection
            .query_row(
                "SELECT model FROM assistant_conversations WHERE scope=?",
                [self.scope()],
                |r| r.get::<_, Option<String>>(0),
            )
            .optional()?
            .flatten())
    }

    pub fn set_conversation_model(&mut self, model: Option<String>) -> Result<()> {
        let model = model.filter(|m| !m.is_empty());
        if model
            .as_ref()
            .is_some_and(|m| m.len() > 160 || m.chars().any(char::is_control))
        {
            return Err(Error::Invalid("Geçersiz model seçimi.".into()));
        }
        self.connection.execute(
            "INSERT INTO assistant_conversations(scope,model) VALUES (?,?) ON CONFLICT(scope) DO UPDATE SET model=excluded.model",
            params![self.scope(), model],
        )?;
        Ok(())
    }

    pub fn conversation(&self) -> Result<Vec<Message>> {
        let scope = self.scope();
        let (count, bytes): (usize, usize) = self.connection.query_row(
            "SELECT count(*),coalesce(sum(length(CAST(text AS BLOB))),0) FROM assistant_messages WHERE scope=?",
            [&scope], |r| Ok((r.get(0)?,r.get(1)?)),
        )?;
        if count > 40 || bytes > MAX_STORED_TEXT {
            return Err(Error::Invalid(
                "Kayıtlı konuşma izin verilen boyutu aşıyor.".into(),
            ));
        }
        let mut statement = self.connection.prepare(
            "SELECT id,role,text,state,client FROM assistant_messages WHERE scope=? ORDER BY id",
        )?;
        let messages = statement
            .query_map([scope], |r| {
                let client: String = r.get(4)?;
                Ok(Message {
                    id: r.get(0)?,
                    role: r.get(1)?,
                    text: r.get(2)?,
                    state: r.get(3)?,
                    client: if client == "game" {
                        Client::Game
                    } else {
                        Client::Desktop
                    },
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        for message in &messages {
            if !matches!(message.role.as_str(), "user" | "assistant")
                || !matches!(
                    message.state.as_str(),
                    "complete" | "pending" | "failed" | "interrupted"
                )
                || message.text.len() > MAX_ANSWER
            {
                return Err(Error::Invalid("Kayıtlı konuşma geçersiz.".into()));
            }
        }
        Ok(messages)
    }

    pub fn conversation_history(&self) -> Result<Vec<ChatMessage>> {
        let messages = self.conversation()?;
        let completed: Vec<_> = messages
            .as_chunks::<2>()
            .0
            .iter()
            .filter(|pair| {
                pair[0].role == "user"
                    && pair[1].role == "assistant"
                    && pair.iter().all(|m| m.state == "complete")
            })
            .collect();
        Ok(completed
            .iter()
            .rev()
            .take(8)
            .rev()
            .flat_map(|pair| pair.iter())
            .map(|m| ChatMessage {
                role: m.role.clone(),
                text: bounded_text(&m.text, 3500),
            })
            .collect())
    }

    pub fn begin_conversation_turn(&mut self, prompt: &str, client: Client) -> Result<i64> {
        validate_prompt(prompt)?;
        let scope = self.scope();
        let transaction = self.connection.transaction()?;
        transaction.execute("INSERT INTO assistant_messages(scope,role,text,state,client) VALUES (?,'user',?,'complete',?)",params![scope,prompt.trim(),client.name()])?;
        transaction.execute("INSERT INTO assistant_messages(scope,role,text,state,client) VALUES (?,'assistant','','pending',?)",params![scope,client.name()])?;
        let id = transaction.last_insert_rowid();
        trim(&transaction, &scope)?;
        transaction.commit()?;
        Ok(id)
    }

    pub fn finish_conversation_turn(
        &mut self,
        scope: &str,
        id: i64,
        text: &str,
        state: &str,
    ) -> Result<()> {
        if text.len() > MAX_ANSWER || !matches!(state, "complete" | "failed" | "interrupted") {
            return Err(Error::Invalid("Geçersiz asistan yanıtı.".into()));
        }
        let transaction = self.connection.transaction()?;
        let changed = transaction.execute("UPDATE assistant_messages SET text=?,state=? WHERE scope=? AND id=? AND state='pending' AND role='assistant'",params![text,state,scope,id])?;
        if changed != 1 {
            return Err(Error::Invalid("Konuşma yanıtı artık güncel değil.".into()));
        }
        trim(&transaction, scope)?;
        transaction.commit()?;
        Ok(())
    }

    pub fn game_bridge_enabled(&self) -> Result<bool> {
        Ok(self
            .connection
            .query_row(
                "SELECT value FROM settings WHERE key='game_bridge_enabled'",
                [],
                |r| r.get::<_, String>(0),
            )
            .optional()?
            .as_deref()
            == Some("true"))
    }

    pub fn set_game_bridge_enabled(&mut self, enabled: bool) -> Result<()> {
        self.connection.execute("INSERT INTO settings(key,value) VALUES ('game_bridge_enabled',?) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[if enabled { "true" } else { "false" }])?;
        Ok(())
    }
}

fn trim(transaction: &Transaction<'_>, scope: &str) -> Result<()> {
    loop {
        let (count,bytes):(usize,usize) = transaction.query_row("SELECT count(*),coalesce(sum(length(CAST(text AS BLOB))),0) FROM assistant_messages WHERE scope=?",[scope],|r|Ok((r.get(0)?,r.get(1)?)))?;
        if count <= 40 && bytes <= MAX_STORED_TEXT {
            return Ok(());
        }
        transaction.execute("DELETE FROM assistant_messages WHERE id IN (SELECT id FROM assistant_messages WHERE scope=? ORDER BY id LIMIT 2)",[scope])?;
    }
}

pub(crate) fn bounded_text(text: &str, max: usize) -> String {
    let mut end = text.len().min(max);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_string()
}
