//LICENSE Portions Copyright 2019-2021 ZomboDB, LLC.
//LICENSE
//LICENSE Portions Copyright 2021-2023 Technology Concepts & Design, Inc.
//LICENSE
//LICENSE Portions Copyright 2023-2023 PgCentral Foundation, Inc. <contact@pgcentral.org>
//LICENSE
//LICENSE All rights reserved.
//LICENSE
//LICENSE Use of this source code is governed by the MIT license that can be found in the LICENSE file.

use crate::{PgrxSql, SqlGraphEntity, SqlGraphIdentifier, ToSql, ToSqlConfigEntity};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PgEventTriggerEntity {
    pub function_name: &'static str,
    pub to_sql_config: ToSqlConfigEntity,
    pub file: &'static str,
    pub line: u32,
    pub module_path: &'static str,
    pub full_path: &'static str,
}

impl PgEventTriggerEntity {
    fn wrapper_function_name(&self) -> String {
        self.function_name.to_string() + "_wrapper"
    }
}

impl From<PgEventTriggerEntity> for SqlGraphEntity {
    fn from(val: PgEventTriggerEntity) -> Self {
        SqlGraphEntity::EventTrigger(val)
    }
}

impl ToSql for PgEventTriggerEntity {
    fn to_sql(&self, context: &PgrxSql) -> eyre::Result<String> {
        let self_index = context.event_triggers[self];
        let schema = context.schema_prefix_for(&self_index);

        let PgEventTriggerEntity { file, line, full_path, function_name, .. } = self;
        let sql = format!(
            "\n\
            -- {file}:{line}\n\
            -- {full_path}\n\
            CREATE FUNCTION {schema}\"{function_name}\"()\n\
                \tRETURNS event_trigger\n\
                \tLANGUAGE c\n\
                \tAS 'MODULE_PATHNAME', '{wrapper_function_name}';",
            wrapper_function_name = self.wrapper_function_name(),
        );
        Ok(sql)
    }
}

impl SqlGraphIdentifier for PgEventTriggerEntity {
    fn dot_identifier(&self) -> String {
        format!("event trigger fn {}", self.full_path)
    }
    fn rust_identifier(&self) -> String {
        self.full_path.to_string()
    }

    fn file(&self) -> Option<&'static str> {
        Some(self.file)
    }

    fn line(&self) -> Option<u32> {
        Some(self.line)
    }
}
