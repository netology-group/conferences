use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use diesel::{pg::PgConnection, result::Error};
use serde_derive::{Deserialize, Serialize};
use svc_agent::AgentId;
use uuid::Uuid;

use super::room::Object as Room;
use crate::schema::agent;

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, DbEnum, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[PgType = "agent_status"]
#[DieselType = "Agent_status"]
pub(crate) enum Status {
    #[serde(rename = "in_progress")]
    InProgress,
    Ready,
}

#[derive(Debug, Serialize, Deserialize, Identifiable, Queryable, QueryableByName, Associations)]
#[belongs_to(Room, foreign_key = "room_id")]
#[table_name = "agent"]
pub(crate) struct Object {
    id: Uuid,
    agent_id: AgentId,
    room_id: Uuid,
    #[serde(with = "ts_seconds")]
    created_at: DateTime<Utc>,
    status: Status,
}

impl Object {
    pub(crate) fn id(&self) -> Uuid {
        self.id
    }

    pub(crate) fn agent_id(&self) -> &AgentId {
        &self.agent_id
    }

    pub(crate) fn room_id(&self) -> Uuid {
        self.room_id
    }

    pub(crate) fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub(crate) fn status(&self) -> &Status {
        &self.status
    }
}

////////////////////////////////////////////////////////////////////////////////

pub(crate) struct ListQuery<'a> {
    agent_id: Option<&'a AgentId>,
    room_id: Option<Uuid>,
    status: Option<Status>,
    offset: Option<i64>,
    limit: Option<i64>,
}

impl<'a> ListQuery<'a> {
    pub(crate) fn new() -> Self {
        Self {
            agent_id: None,
            room_id: None,
            status: None,
            offset: None,
            limit: None,
        }
    }

    pub(crate) fn agent_id(self, agent_id: &'a AgentId) -> Self {
        Self {
            agent_id: Some(agent_id),
            ..self
        }
    }

    pub(crate) fn room_id(self, room_id: Uuid) -> Self {
        Self {
            room_id: Some(room_id),
            ..self
        }
    }

    pub(crate) fn status(self, status: Status) -> Self {
        Self {
            status: Some(status),
            ..self
        }
    }

    pub(crate) fn offset(self, offset: i64) -> Self {
        Self {
            offset: Some(offset),
            ..self
        }
    }

    pub(crate) fn limit(self, limit: i64) -> Self {
        Self {
            limit: Some(limit),
            ..self
        }
    }

    pub(crate) fn execute(&self, conn: &PgConnection) -> Result<Vec<Object>, Error> {
        use diesel::prelude::*;

        let mut q = agent::table.into_boxed();

        if let Some(agent_id) = self.agent_id {
            q = q.filter(agent::agent_id.eq(agent_id));
        }

        if let Some(room_id) = self.room_id {
            q = q.filter(agent::room_id.eq(room_id));
        }

        if let Some(status) = self.status {
            q = q.filter(agent::status.eq(status));
        }

        if let Some(offset) = self.offset {
            q = q.offset(offset);
        }

        if let Some(limit) = self.limit {
            q = q.limit(limit);
        }

        q.order_by(agent::created_at.desc()).get_results(conn)
    }
}

impl<'a> From<(Option<Uuid>, Option<i64>, Option<i64>, Option<Status>)> for ListQuery<'a> {
    fn from(value: (Option<Uuid>, Option<i64>, Option<i64>, Option<Status>)) -> Self {
        Self {
            agent_id: None,
            room_id: value.0,
            offset: value.1,
            limit: value.2,
            status: value.3,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Insertable)]
#[table_name = "agent"]
pub(crate) struct InsertQuery<'a> {
    id: Option<Uuid>,
    agent_id: &'a AgentId,
    room_id: Uuid,
    status: Status,
}

impl<'a> InsertQuery<'a> {
    pub(crate) fn new(agent_id: &'a AgentId, room_id: Uuid) -> Self {
        Self {
            id: None,
            agent_id,
            room_id,
            status: Status::InProgress,
        }
    }

    pub(crate) fn status(self, status: Status) -> Self {
        Self { status, ..self }
    }

    pub(crate) fn execute(&self, conn: &PgConnection) -> Result<Object, Error> {
        use crate::schema::agent::dsl::*;
        use diesel::RunQueryDsl;

        diesel::insert_into(agent)
            .values(self)
            .on_conflict((agent_id, room_id))
            .do_nothing()
            .get_result(conn)
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, AsChangeset)]
#[table_name = "agent"]
pub(crate) struct UpdateQuery<'a> {
    agent_id: &'a AgentId,
    room_id: Uuid,
    status: Option<Status>,
}

impl<'a> UpdateQuery<'a> {
    pub(crate) fn new(agent_id: &'a AgentId, room_id: Uuid) -> Self {
        Self {
            agent_id,
            room_id,
            status: None,
        }
    }

    pub(crate) fn status(self, status: Status) -> Self {
        Self {
            status: Some(status),
            ..self
        }
    }

    pub(crate) fn execute(&self, conn: &PgConnection) -> Result<Option<Object>, Error> {
        use diesel::prelude::*;

        let query = agent::table
            .filter(agent::agent_id.eq(self.agent_id))
            .filter(agent::room_id.eq(self.room_id));

        diesel::update(query).set(self).get_result(conn).optional()
    }
}

///////////////////////////////////////////////////////////////////////////////

pub(crate) struct DeleteQuery<'a> {
    agent_id: &'a AgentId,
    room_id: Uuid,
}

impl<'a> DeleteQuery<'a> {
    pub(crate) fn new(agent_id: &'a AgentId, room_id: Uuid) -> Self {
        Self { agent_id, room_id }
    }

    pub(crate) fn execute(&self, conn: &PgConnection) -> Result<usize, Error> {
        use diesel::prelude::*;

        let query = agent::table
            .filter(agent::agent_id.eq(self.agent_id))
            .filter(agent::room_id.eq(self.room_id));

        diesel::delete(query).execute(conn)
    }
}