---
title: "Validation"
date: 2023-01-17T11:44:56+01:00
draft: false
weight: 5
---

# Validation

Models can be validated before writing to the database. In order to validate, the ValidationTrait needs to be implemented as in the following example.

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize, DeriveActixAdmin, DeriveActixAdminModel, DeriveActixAdminViewModel)]
#[sea_orm(table_name = "comment")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    #[actix_admin(primary_key)]
    pub id: i32,

    pub my_decimal: Decimal
}

impl ActixAdminModelValidationTrait<ActiveModel> for Entity {
    fn validate(model: &ActiveModel) -> HashMap<String, String> {
        let mut errors = HashMap::new();
        
        if model.my_decimal.clone().unwrap() < Decimal::from(100 as i16) {
            errors.insert("my_decimal".to_string(), "Must be larger than 100".to_string());
        }

        errors
    }
}
```