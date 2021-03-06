use std::{error::Error as StdError, fmt};

use enum_iterator::IntoEnumIterator;
use slog::{warn, Logger};
use svc_agent::mqtt::ResponseStatus;
use svc_error::{extension::sentry, Error as SvcError};
////////////////////////////////////////////////////////////////////////////////

struct ErrorKindProperties {
    status: ResponseStatus,
    kind: &'static str,
    title: &'static str,
    is_notify_sentry: bool,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, IntoEnumIterator)]
pub enum ErrorKind {
    AccessDenied,
    AgentNotConnected,
    AgentNotEnteredTheRoom,
    AuthorizationFailed,
    BackendRecordingMissing,
    BackendRequestFailed,
    BackendClientCreationFailed,
    _BackendRequestTimedOut,
    BackendNotFound,
    BrokerRequestFailed,
    CapacityExceeded,
    ConfigKeyMissing,
    DbConnAcquisitionFailed,
    DbQueryFailed,
    InvalidHandleId,
    InvalidJsepFormat,
    InvalidRoomTime,
    InvalidSdpType,
    InvalidSubscriptionObject,
    InvalidPayload,
    MessageBuildingFailed,
    MessageHandlingFailed,
    MessageParsingFailed,
    NoAvailableBackends,
    NotImplemented,
    PublishFailed,
    ResubscriptionFailed,
    RoomClosed,
    RoomNotFound,
    RoomTimeChangingForbidden,
    RtcNotFound,
}

impl ErrorKind {
    pub fn status(self) -> ResponseStatus {
        let properties: ErrorKindProperties = self.into();
        properties.status
    }

    pub fn kind(self) -> &'static str {
        let properties: ErrorKindProperties = self.into();
        properties.kind
    }

    pub fn title(self) -> &'static str {
        let properties: ErrorKindProperties = self.into();
        properties.title
    }

    pub fn is_notify_sentry(self) -> bool {
        let properties: ErrorKindProperties = self.into();
        properties.is_notify_sentry
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let properties: ErrorKindProperties = self.to_owned().into();
        write!(f, "{}", properties.title)
    }
}

impl From<ErrorKind> for ErrorKindProperties {
    fn from(val: ErrorKind) -> Self {
        match val {
            ErrorKind::AccessDenied => ErrorKindProperties {
                status: ResponseStatus::FORBIDDEN,
                kind: "access_denied",
                title: "Access denied",
                is_notify_sentry: false,
            },
            ErrorKind::AgentNotConnected => ErrorKindProperties {
                status: ResponseStatus::UNPROCESSABLE_ENTITY,
                kind: "agent_not_connected",
                title: "Agent not connected to the RTC",
                is_notify_sentry: false,
            },
            ErrorKind::AgentNotEnteredTheRoom => ErrorKindProperties {
                status: ResponseStatus::NOT_FOUND,
                kind: "agent_not_entered_the_room",
                title: "Agent not entered the room",
                is_notify_sentry: false,
            },
            ErrorKind::AuthorizationFailed => ErrorKindProperties {
                status: ResponseStatus::UNPROCESSABLE_ENTITY,
                kind: "authorization_failed",
                title: "Authorization failed",
                is_notify_sentry: false,
            },
            ErrorKind::BackendRecordingMissing => ErrorKindProperties {
                status: ResponseStatus::UNPROCESSABLE_ENTITY,
                kind: "backend_recording_missing",
                title: "Janus recording missing",
                is_notify_sentry: true,
            },
            ErrorKind::BackendRequestFailed => ErrorKindProperties {
                status: ResponseStatus::FAILED_DEPENDENCY,
                kind: "backend_request_failed",
                title: "Janus request failed",
                is_notify_sentry: true,
            },
            ErrorKind::BackendClientCreationFailed => ErrorKindProperties {
                status: ResponseStatus::FAILED_DEPENDENCY,
                kind: "backend_client_creation_failed",
                title: "Janus create client failed",
                is_notify_sentry: true,
            },
            ErrorKind::_BackendRequestTimedOut => ErrorKindProperties {
                status: ResponseStatus::FAILED_DEPENDENCY,
                kind: "backend_request_timed_out",
                title: "Janus request timed out",
                is_notify_sentry: true,
            },
            ErrorKind::BackendNotFound => ErrorKindProperties {
                status: ResponseStatus::NOT_FOUND,
                kind: "backend_not_found",
                title: "Backend not found",
                is_notify_sentry: true,
            },
            ErrorKind::BrokerRequestFailed => ErrorKindProperties {
                status: ResponseStatus::UNPROCESSABLE_ENTITY,
                kind: "broker_request_failed",
                title: "Broker request failed",
                is_notify_sentry: true,
            },
            ErrorKind::ConfigKeyMissing => ErrorKindProperties {
                status: ResponseStatus::UNPROCESSABLE_ENTITY,
                kind: "config_key_missing",
                title: "Config key missing",
                is_notify_sentry: true,
            },
            ErrorKind::CapacityExceeded => ErrorKindProperties {
                status: ResponseStatus::SERVICE_UNAVAILABLE,
                kind: "capacity_exceeded",
                title: "Capacity exceeded",
                is_notify_sentry: true,
            },
            ErrorKind::DbConnAcquisitionFailed => ErrorKindProperties {
                status: ResponseStatus::UNPROCESSABLE_ENTITY,
                kind: "database_connection_acquisition_failed",
                title: "Database connection acquisition failed",
                is_notify_sentry: true,
            },
            ErrorKind::DbQueryFailed => ErrorKindProperties {
                status: ResponseStatus::UNPROCESSABLE_ENTITY,
                kind: "database_query_failed",
                title: "Database query failed",
                is_notify_sentry: true,
            },
            ErrorKind::InvalidHandleId => ErrorKindProperties {
                status: ResponseStatus::BAD_REQUEST,
                kind: "invalid_handle_id",
                title: "Invalid handle ID",
                is_notify_sentry: false,
            },
            ErrorKind::InvalidJsepFormat => ErrorKindProperties {
                status: ResponseStatus::BAD_REQUEST,
                kind: "invalid_jsep_format",
                title: "Invalid JSEP format",
                is_notify_sentry: false,
            },
            ErrorKind::InvalidPayload => ErrorKindProperties {
                status: ResponseStatus::BAD_REQUEST,
                kind: "invalid_payload",
                title: "Invalid payload",
                is_notify_sentry: false,
            },
            ErrorKind::InvalidRoomTime => ErrorKindProperties {
                status: ResponseStatus::BAD_REQUEST,
                kind: "invalid_room_time",
                title: "Invalid room time",
                is_notify_sentry: true,
            },
            ErrorKind::InvalidSdpType => ErrorKindProperties {
                status: ResponseStatus::BAD_REQUEST,
                kind: "invalid_sdp_type",
                title: "Invalid SDP type",
                is_notify_sentry: false,
            },
            ErrorKind::InvalidSubscriptionObject => ErrorKindProperties {
                status: ResponseStatus::BAD_REQUEST,
                kind: "invalid_subscription_object",
                title: "Invalid subscription object",
                is_notify_sentry: true,
            },
            ErrorKind::MessageBuildingFailed => ErrorKindProperties {
                status: ResponseStatus::UNPROCESSABLE_ENTITY,
                kind: "message_building_failed",
                title: "Message building failed",
                is_notify_sentry: true,
            },
            ErrorKind::MessageHandlingFailed => ErrorKindProperties {
                status: ResponseStatus::UNPROCESSABLE_ENTITY,
                kind: "message_handling_failed",
                title: "Message handling failed",
                is_notify_sentry: true,
            },
            ErrorKind::MessageParsingFailed => ErrorKindProperties {
                status: ResponseStatus::BAD_REQUEST,
                kind: "message_parsing_failed",
                title: "Message parsing failed",
                is_notify_sentry: true,
            },
            ErrorKind::NoAvailableBackends => ErrorKindProperties {
                status: ResponseStatus::SERVICE_UNAVAILABLE,
                kind: "no_available_backends",
                title: "No available backends",
                is_notify_sentry: true,
            },
            ErrorKind::NotImplemented => ErrorKindProperties {
                status: ResponseStatus::INTERNAL_SERVER_ERROR,
                kind: "not_implemented",
                title: "Not implemented",
                is_notify_sentry: true,
            },
            ErrorKind::PublishFailed => ErrorKindProperties {
                status: ResponseStatus::UNPROCESSABLE_ENTITY,
                kind: "publish_failed",
                title: "Publish failed",
                is_notify_sentry: true,
            },
            ErrorKind::ResubscriptionFailed => ErrorKindProperties {
                status: ResponseStatus::INTERNAL_SERVER_ERROR,
                kind: "resubscription_failed",
                title: "Resubscription failed",
                is_notify_sentry: true,
            },
            ErrorKind::RoomClosed => ErrorKindProperties {
                status: ResponseStatus::NOT_FOUND,
                kind: "room_closed",
                title: "Room closed",
                is_notify_sentry: false,
            },
            ErrorKind::RoomNotFound => ErrorKindProperties {
                status: ResponseStatus::NOT_FOUND,
                kind: "room_not_found",
                title: "Room not found",
                is_notify_sentry: false,
            },
            ErrorKind::RoomTimeChangingForbidden => ErrorKindProperties {
                status: ResponseStatus::UNPROCESSABLE_ENTITY,
                kind: "room_time_changing_forbidden",
                title: "Room time changing forbidden",
                is_notify_sentry: false,
            },
            ErrorKind::RtcNotFound => ErrorKindProperties {
                status: ResponseStatus::NOT_FOUND,
                kind: "rtc_not_found",
                title: "RTC not found",
                is_notify_sentry: false,
            },
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Error {
    kind: ErrorKind,
    source: Box<dyn AsRef<dyn StdError + Send + Sync + 'static> + Send + Sync + 'static>,
}

impl Error {
    pub fn new<E>(kind: ErrorKind, source: E) -> Self
    where
        E: AsRef<dyn StdError + Send + Sync + 'static> + Send + Sync + 'static,
    {
        Self {
            kind,
            source: Box::new(source),
        }
    }

    pub fn status(&self) -> ResponseStatus {
        self.kind.status()
    }

    pub fn error_kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn kind(&self) -> &str {
        self.kind.kind()
    }

    pub fn title(&self) -> &str {
        self.kind.title()
    }

    pub fn source(&self) -> &(dyn StdError + Send + Sync + 'static) {
        self.source.as_ref().as_ref()
    }

    pub fn to_svc_error(&self) -> SvcError {
        let properties: ErrorKindProperties = self.kind.into();

        SvcError::builder()
            .status(properties.status)
            .kind(properties.kind, properties.title)
            .detail(&self.source.as_ref().as_ref().to_string())
            .build()
    }

    pub fn notify_sentry(&self, logger: &Logger) {
        if !self.kind.is_notify_sentry() {
            return;
        }

        sentry::send(self.to_svc_error()).unwrap_or_else(|err| {
            warn!(logger, "Error sending error to Sentry: {}", err);
        });
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.kind)
            .field("source", &self.source.as_ref().as_ref())
            .finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.source.as_ref().as_ref())
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(self.source.as_ref().as_ref())
    }
}

impl From<svc_authz::Error> for Error {
    fn from(source: svc_authz::Error) -> Self {
        let kind = match source.kind() {
            svc_authz::ErrorKind::Forbidden(_) => ErrorKind::AccessDenied,
            _ => ErrorKind::AuthorizationFailed,
        };

        Self {
            kind,
            source: Box::new(anyhow::Error::from(source)),
        }
    }
}

impl From<diesel::result::Error> for Error {
    fn from(source: diesel::result::Error) -> Self {
        Self {
            kind: ErrorKind::DbQueryFailed,
            source: Box::new(anyhow::Error::from(source)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait ErrorExt<T> {
    fn error(self, kind: ErrorKind) -> Result<T, Error>;
}

impl<T, E: AsRef<dyn StdError + Send + Sync + 'static> + Send + Sync + 'static> ErrorExt<T>
    for Result<T, E>
{
    fn error(self, kind: ErrorKind) -> Result<T, Error> {
        self.map_err(|source| Error::new(kind, source))
    }
}
