//LICENSE Portions Copyright 2019-2021 ZomboDB, LLC.
//LICENSE
//LICENSE Portions Copyright 2021-2023 Technology Concepts & Design, Inc.
//LICENSE
//LICENSE Portions Copyright 2023-2023 PgCentral Foundation, Inc. <contact@pgcentral.org>
//LICENSE
//LICENSE All rights reserved.
//LICENSE
//LICENSE Use of this source code is governed by the MIT license that can be found in the LICENSE file.

use std::ffi::CStr;

use crate::command_tag::PgCommandTag;

struct PgEventTriggerData {
    tag: PgCommandTag,
    event: PgEventTriggerEvent,
}

impl PgEventTriggerData {
    fn from(raw: &pgrx_pg_sys::EventTriggerData) -> Result<Self, PgEventTriggerError> {
        let tag = PgCommandTag::try_from(raw.tag)
            .map_err(|_| PgEventTriggerError::ExtractEventTriggerData)?;

        if raw.event.is_null() {
            return Err(PgEventTriggerError::ExtractEventTriggerData);
        }

        let event = match unsafe { CStr::from_ptr(raw.event).to_str() } {
            Ok("ddl_command_start") => PgEventTriggerEvent::DdlCommandStart,
            Ok("ddl_command_end") => PgEventTriggerEvent::DdlCommandEnd,
            Ok("sql_drop") => PgEventTriggerEvent::SqlDrop,
            Ok("table_rewrite") => PgEventTriggerEvent::TableRewrite,
            _ => return Err(PgEventTriggerError::ExtractEventTriggerData),
        };

        Ok(Self { tag, event })
    }
}

pub struct PgEventTrigger<'a> {
    event_trigger_data: PgEventTriggerData,
    raw_event_trigger_data: &'a pgrx_pg_sys::EventTriggerData,
}

impl<'a> PgEventTrigger<'a> {
    #[doc(hidden)]
    pub unsafe fn from_fcinfo(
        fcinfo: &'a pgrx_pg_sys::FunctionCallInfoBaseData,
    ) -> Result<Self, PgEventTriggerError> {
        if !called_as_event_trigger(fcinfo as *const _ as *mut _) {
            return Err(PgEventTriggerError::WrongContext);
        }

        let raw_event_trigger_data = (fcinfo.context as *mut pgrx_pg_sys::EventTriggerData)
            .as_ref()
            .ok_or(PgEventTriggerError::NullEventTriggerData)?;

        let event_trigger_data = PgEventTriggerData::from(&raw_event_trigger_data)?;

        Ok(Self { raw_event_trigger_data, event_trigger_data })
    }

    pub fn tag(&self) -> PgCommandTag {
        self.event_trigger_data.tag
    }

    pub fn event(&self) -> PgEventTriggerEvent {
        self.event_trigger_data.event
    }

    pub fn raw_data(&self) -> &'a pgrx_pg_sys::EventTriggerData {
        self.raw_event_trigger_data
    }
}

#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum PgEventTriggerError {
    #[error("Not fired by event trigger manager")]
    WrongContext,
    #[error("The `pgrx::pg_sys::FunctionCallInfo`'s `context` field was a NULL pointer")]
    NullEventTriggerData,
    #[error("Failed to extract")]
    ExtractEventTriggerData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PgEventTriggerEvent {
    DdlCommandStart,
    DdlCommandEnd,
    SqlDrop,
    TableRewrite,
}

#[inline]
pub unsafe fn called_as_event_trigger(fcinfo: pgrx_pg_sys::FunctionCallInfo) -> bool {
    let fcinfo = fcinfo.as_ref().expect("fcinfo was null");
    !fcinfo.context.is_null()
        && crate::is_a(fcinfo.context, pgrx_pg_sys::NodeTag::T_EventTriggerData)
}
