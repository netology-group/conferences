use std::ops::Bound;

use chrono::{DateTime, Utc};
use failure::Error;
use serde_derive::Deserialize;

use crate::db::{room, ConnectionPool};
use crate::transport::util::mqtt::{
    compat::IntoEnvelope, IncomingRequest, OutgoingResponse, OutgoingResponseStatus, Publishable,
};

////////////////////////////////////////////////////////////////////////////////

pub(crate) type CreateRequest = IncomingRequest<CreateRequestData>;

#[derive(Debug, Deserialize)]
pub(crate) struct CreateRequestData {
    #[serde(with = "crate::serde::ts_seconds_bound_tuple")]
    time: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
    audience: String,
}

pub(crate) type ObjectResponse = OutgoingResponse<room::Object>;

////////////////////////////////////////////////////////////////////////////////

pub(crate) struct State {
    authz: authz::ClientMap,
    db: ConnectionPool,
}

impl State {
    pub(crate) fn new(authz: authz::ClientMap, db: ConnectionPool) -> Self {
        Self { authz, db }
    }
}

impl State {
    pub(crate) fn create(&self, inreq: &CreateRequest) -> Result<impl Publishable, Error> {
        // Authorization: future room's owner has to allow the action
        self.authz.authorize(
            &inreq.payload().audience,
            inreq.properties(),
            vec!["rooms"],
            "create",
        )?;

        // Creating a Room
        let object = {
            let conn = self.db.get()?;
            room::InsertQuery::new(inreq.payload().time, &inreq.payload().audience)
                .execute(&conn)?
        };

        let resp = inreq.to_response(object, &OutgoingResponseStatus::OK);
        resp.into_envelope()
    }
}
