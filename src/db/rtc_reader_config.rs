use diesel::{pg::PgConnection, result::Error};
use svc_agent::AgentId;
use uuid::Uuid;

use crate::db::rtc::Object as Rtc;
use crate::schema::{rtc, rtc_reader_config};

////////////////////////////////////////////////////////////////////////////////

type AllColumns = (
    rtc_reader_config::rtc_id,
    rtc_reader_config::reader_id,
    rtc_reader_config::receive_video,
    rtc_reader_config::receive_audio,
);

const ALL_COLUMNS: AllColumns = (
    rtc_reader_config::rtc_id,
    rtc_reader_config::reader_id,
    rtc_reader_config::receive_video,
    rtc_reader_config::receive_audio,
);

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Identifiable, Queryable, QueryableByName, Associations)]
#[belongs_to(Rtc, foreign_key = "rtc_id")]
#[table_name = "rtc_reader_config"]
#[primary_key(rtc_id, reader_id)]
pub(crate) struct Object {
    rtc_id: Uuid,
    reader_id: AgentId,
    receive_video: bool,
    receive_audio: bool,
}

impl Object {
    pub(crate) fn reader_id(&self) -> &AgentId {
        &self.reader_id
    }

    pub(crate) fn receive_video(&self) -> bool {
        self.receive_video
    }

    pub(crate) fn receive_audio(&self) -> bool {
        self.receive_audio
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub(crate) struct ListWithRtcQuery<'a> {
    room_id: Uuid,
    reader_id: &'a AgentId,
}

impl<'a> ListWithRtcQuery<'a> {
    pub(crate) fn new(room_id: Uuid, reader_id: &'a AgentId) -> Self {
        Self { room_id, reader_id }
    }

    pub(crate) fn execute(&self, conn: &PgConnection) -> Result<Vec<(Object, Rtc)>, Error> {
        use diesel::prelude::*;

        rtc_reader_config::table
            .inner_join(rtc::table)
            .filter(rtc::room_id.eq(self.room_id))
            .filter(rtc_reader_config::reader_id.eq(self.reader_id))
            .select((ALL_COLUMNS, crate::db::rtc::ALL_COLUMNS))
            .get_results(conn)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Insertable, AsChangeset)]
#[table_name = "rtc_reader_config"]
pub(crate) struct UpsertQuery<'a> {
    rtc_id: Uuid,
    reader_id: &'a AgentId,
    receive_video: Option<bool>,
    receive_audio: Option<bool>,
}

impl<'a> UpsertQuery<'a> {
    pub(crate) fn new(rtc_id: Uuid, reader_id: &'a AgentId) -> Self {
        Self {
            rtc_id,
            reader_id,
            receive_video: None,
            receive_audio: None,
        }
    }

    pub(crate) fn receive_video(self, receive_video: bool) -> Self {
        Self {
            receive_video: Some(receive_video),
            ..self
        }
    }

    pub(crate) fn receive_audio(self, receive_audio: bool) -> Self {
        Self {
            receive_audio: Some(receive_audio),
            ..self
        }
    }

    pub(crate) fn execute(&self, conn: &PgConnection) -> Result<Object, Error> {
        use diesel::prelude::*;

        let mut insert_values = self.clone();

        if insert_values.receive_video.is_none() {
            insert_values.receive_video = Some(true);
        }

        if insert_values.receive_audio.is_none() {
            insert_values.receive_audio = Some(true);
        }

        diesel::insert_into(rtc_reader_config::table)
            .values(insert_values)
            .on_conflict((rtc_reader_config::rtc_id, rtc_reader_config::reader_id))
            .do_update()
            .set(self)
            .get_result(conn)
    }
}