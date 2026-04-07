use chrono::{FixedOffset, DateTime, NaiveDateTime};
use serde::{Serialize, Deserialize};
use sea_orm::{Iterable, entity::prelude::*, sea_query::ValueTuple};
use time::OffsetDateTime;

#[derive(Serialize, Deserialize, Debug)]
pub struct ChangeRecord {
    pub key: Uuid,
    pub changes: Vec<FieldValue>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FieldValue {
    pub column_id: u16,
    pub value: DbValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DbValue {
    String(String),
    TimeDateTimeWithTimeZone(OffsetDateTime),
    Uuid(uuid::Uuid),
    Null,
}

impl ChangeRecord {
    pub fn from_models<E>(changed_model: &E::Model, orig_model: &E::Model) -> ChangeRecord
    where
        E: EntityTrait,
        E::Model: ModelTrait<Entity = E>,
        E::Column: ColumnTrait + Iterable,
     {
        let primary_key = changed_model.get_primary_key_value();
        if let ValueTuple::One(Value::Uuid(Some(uuid))) = primary_key {
            let mut change_record = ChangeRecord {
                key: uuid,
                changes: Vec::new(),
            };
            for (col_index, col) in E::Column::iter().enumerate() {
                let v_changed = changed_model.get(col);
                let v_orig = orig_model.get(col);
                if v_changed != v_orig {
                    let field_value = FieldValue {
                        column_id: col_index as u16,
                        value: v_changed.into(),
                    };
                    change_record.changes.push(field_value);
                }
            }
            change_record
        } else {
            panic!("unsupported primary key type {:?}", primary_key);
        }
    }
    pub fn to_active_model<E>(&self) -> E::ActiveModel
    where
        E: EntityTrait,
        E::ActiveModel: ActiveModelTrait<Entity = E>,
        E::Column: ColumnTrait + Iterable,
        E::PrimaryKey: PrimaryKeyTrait + Iterable, 
     {
        let mut am = <E::ActiveModel as sea_orm::ActiveModelTrait>::default();
        for col in E::PrimaryKey::iter() {
            let _err = am.try_set(col.into_column(), Value::Uuid(Some(self.key)));
            break;
        }
        for fv in self.changes.iter() {
            for (c_id, c) in E::Column::iter().enumerate() {
                if c_id as u16 == fv.column_id {
                    let _err = am.try_set(c, (&fv.value).into());
                }
            }
        }
        am
    }
}

impl From<Value> for DbValue {
    fn from(value: Value) -> Self {
        match value {
            Value::String(string) => {
                match string {
                    Some(v) => {
                        DbValue::String(v)
                    }
                    None => {
                        DbValue::Null
                    }
                }
            },
            Value::Uuid(v) => {
                match v {
                    Some(v) => {
                        DbValue::Uuid(v)
                    }
                    None => {
                        DbValue::Null
                    }
                }
            }
            Value::TimeDateTimeWithTimeZone(v) => {
                match v {
                    Some(v) => {
                        DbValue::TimeDateTimeWithTimeZone(v)
                    }
                    None => {
                        DbValue::Null
                    }
                }
            }
            _ => {
                panic!("unsupported Value type TODO");
            }
        }
    }
} 


impl From<&DbValue> for Value {
    fn from(value: &DbValue) -> Self {
        match value {
            DbValue::String(v) => {
                Value::String(Some(v.clone()))
            }
            DbValue::Uuid(v) => {
                Value::Uuid(Some(*v))
            }
            DbValue::TimeDateTimeWithTimeZone(v) => {
                Value::TimeDateTimeWithTimeZone(Some(*v))
            }
            DbValue::Null => {
                panic!("can not be null")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap};

    use models::entity::articles::ActiveModel;
    use sea_orm::{Iden, Iterable, ModelTrait, Value, prelude::DateTimeWithTimeZone, sea_query::ValueTuple};
    use uuid::Uuid;

    use crate::dto::{ChangeRecord, FieldValue};

    #[test]
    fn test_record() {
        use models::entity::articles;
        
        println!("column literal is {}",articles::Column::Title.to_string());
        let mut column_id_map: HashMap<String, u16> = HashMap::new();
        for (id,col) in articles::Column::iter().enumerate() {
            column_id_map.insert(col.to_string(),id as u16);
        }
        let now: DateTimeWithTimeZone = chrono::Local::now().with_timezone(chrono::Local::now().offset());
        let a1 = articles::Model {
            id: Uuid::new_v4(),
            slug: "slug".into(),
            title: "tilte".into(),
            description: "description".into(),
            body: "body".into(),
            author_id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
        };
        let mut a2 = a1.clone();
        a2.title = "title2".into();
        a2.slug = "s1".into();
        let primary_key = a1.get_primary_key_value();
        if let ValueTuple::One(Value::Uuid(Some(uuid))) = primary_key {
            let mut change_record = ChangeRecord {
                key: uuid,
                changes: Vec::new(),
            };
            for (col_index, col) in articles::Column::iter().enumerate() {
                let v1 = a1.get(col);
                let v2 = a2.get(col);
                if v1 != v2 {
                    let field_value = FieldValue {
                        column_id: col_index as u16,
                        value: v2.into(),
                    };
                    change_record.changes.push(field_value);
                }
            }
            println!("change record {:?}", change_record);
            assert_eq!(2, change_record.changes.len());

            let active_model : ActiveModel = change_record.to_active_model::<articles::Entity>();
            println!("active model: {:?}", active_model);
            assert!(active_model.title.is_set());
            if let Some(Value::String(Some(v))) = active_model.title.into_value() {
                assert_eq!("title2",v);
            } else {
                panic!("wrong value");
            }
            assert!(active_model.body.is_not_set());

        } else {
            panic!("unsupported primary key type {:?}", primary_key);
        }

    }
}