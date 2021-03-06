use chrono::{serde::ts_seconds, DateTime, Utc};
use diesel::{pg::PgConnection, result::Error};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use svc_agent::AgentId;
use uuid::Uuid;

use super::room::Object as Room;
use crate::db;
use crate::schema::agent;
use derive_more::Display;
use diesel_derive_newtype::DieselNewType;

////////////////////////////////////////////////////////////////////////////////
#[derive(
    Debug, Deserialize, Serialize, Display, Copy, Clone, DieselNewType, Hash, PartialEq, Eq,
)]
pub struct Id(Uuid);

#[derive(Clone, Copy, Debug, DbEnum, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[PgType = "agent_status"]
#[DieselType = "Agent_status"]
pub enum Status {
    #[serde(rename = "in_progress")]
    InProgress,
    Ready,
}

#[derive(Debug, Serialize, Deserialize, Identifiable, Queryable, QueryableByName, Associations)]
#[belongs_to(Room, foreign_key = "room_id")]
#[table_name = "agent"]
pub struct Object {
    id: Id,
    agent_id: AgentId,
    room_id: db::room::Id,
    #[serde(with = "ts_seconds")]
    created_at: DateTime<Utc>,
    status: Status,
}

#[cfg(test)]
impl Object {
    pub fn status(&self) -> Status {
        self.status
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct ListQuery<'a> {
    agent_id: Option<&'a AgentId>,
    room_id: Option<db::room::Id>,
    status: Option<Status>,
    offset: Option<i64>,
    limit: Option<i64>,
}

impl<'a> ListQuery<'a> {
    pub fn new() -> Self {
        Self {
            agent_id: None,
            room_id: None,
            status: None,
            offset: None,
            limit: None,
        }
    }

    pub fn agent_id(self, agent_id: &'a AgentId) -> Self {
        Self {
            agent_id: Some(agent_id),
            ..self
        }
    }

    pub fn room_id(self, room_id: db::room::Id) -> Self {
        Self {
            room_id: Some(room_id),
            ..self
        }
    }

    pub fn status(self, status: Status) -> Self {
        Self {
            status: Some(status),
            ..self
        }
    }

    pub fn offset(self, offset: i64) -> Self {
        Self {
            offset: Some(offset),
            ..self
        }
    }

    pub fn limit(self, limit: i64) -> Self {
        Self {
            limit: Some(limit),
            ..self
        }
    }

    pub fn execute(&self, conn: &PgConnection) -> Result<Vec<Object>, Error> {
        use diesel::prelude::*;

        let mut q = agent::table
            .into_boxed()
            .filter(agent::status.eq(Status::Ready));

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

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Insertable)]
#[table_name = "agent"]
pub struct InsertQuery<'a> {
    id: Option<Id>,
    agent_id: &'a AgentId,
    room_id: db::room::Id,
    status: Status,
}

impl<'a> InsertQuery<'a> {
    pub fn new(agent_id: &'a AgentId, room_id: db::room::Id) -> Self {
        Self {
            id: None,
            agent_id,
            room_id,
            status: Status::InProgress,
        }
    }

    #[cfg(test)]
    pub fn status(self, status: Status) -> Self {
        Self { status, ..self }
    }

    pub fn execute(&self, conn: &PgConnection) -> Result<Object, Error> {
        use crate::schema::agent::dsl::*;
        use diesel::{ExpressionMethods, RunQueryDsl};

        diesel::insert_into(agent)
            .values(self)
            .on_conflict((agent_id, room_id))
            .do_update()
            .set(status.eq(Status::InProgress))
            .get_result(conn)
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, AsChangeset)]
#[table_name = "agent"]
pub struct UpdateQuery<'a> {
    agent_id: &'a AgentId,
    room_id: db::room::Id,
    status: Option<Status>,
}

impl<'a> UpdateQuery<'a> {
    pub fn new(agent_id: &'a AgentId, room_id: db::room::Id) -> Self {
        Self {
            agent_id,
            room_id,
            status: None,
        }
    }

    pub fn status(self, status: Status) -> Self {
        Self {
            status: Some(status),
            ..self
        }
    }

    pub fn execute(&self, conn: &PgConnection) -> Result<Option<Object>, Error> {
        use diesel::prelude::*;

        let query = agent::table
            .filter(agent::agent_id.eq(self.agent_id))
            .filter(agent::room_id.eq(self.room_id));

        diesel::update(query).set(self).get_result(conn).optional()
    }
}

///////////////////////////////////////////////////////////////////////////////

pub struct DeleteQuery<'a> {
    agent_id: Option<&'a AgentId>,
    room_id: Option<db::room::Id>,
}

impl<'a> DeleteQuery<'a> {
    pub fn new() -> Self {
        Self {
            agent_id: None,
            room_id: None,
        }
    }

    pub fn agent_id(self, agent_id: &'a AgentId) -> Self {
        Self {
            agent_id: Some(agent_id),
            ..self
        }
    }

    pub fn room_id(self, room_id: db::room::Id) -> Self {
        Self {
            room_id: Some(room_id),
            ..self
        }
    }

    pub fn execute(&self, conn: &PgConnection) -> Result<usize, Error> {
        use diesel::prelude::*;

        let mut query = diesel::delete(agent::table).into_boxed();

        if let Some(agent_id) = self.agent_id {
            query = query.filter(agent::agent_id.eq(agent_id));
        }

        if let Some(room_id) = self.room_id {
            query = query.filter(agent::room_id.eq(room_id));
        }

        query.execute(conn)
    }
}
