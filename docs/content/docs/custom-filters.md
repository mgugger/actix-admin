---
title: "Custom Filters"
date: 2023-01-17T11:44:56+01:00
draft: false
weight: 6
---

# Custom Filters

You may add custom filters by implementing the ActixAdminModelFilterTrait for the Entity. The filters are separated from the actual values which might need to be loaded from the Db. For any filter requiring values for a dropdown, add a match for the filter name in the get_filter_values() method. 

```rust
#[async_trait]
impl ActixAdminModelFilterTrait<Entity> for Entity {
    fn get_filter() -> Vec<ActixAdminModelFilter<Entity>> {
        vec![
                        ActixAdminModelFilter::<Entity> {
                name: "User".to_string(),
                filter_type: ActixAdminModelFilterType::Text,
                filter: |q: sea_orm::Select<Entity>, v| -> sea_orm::Select<Entity> {
                    q.apply_if(v, |query, val: String| query.filter(Column::User.eq(val)))
                },
                values: None,
                foreign_key: None,
            },
            ActixAdminModelFilter::<Entity> {
                name: "Insert Date After".to_string(),
                filter_type: ActixAdminModelFilterType::DateTime,
                filter: |q: sea_orm::Select<Entity>, v| -> sea_orm::Select<Entity> {
                    q.apply_if(v, |query, val: String| {
                        let naive_dt =
                            NaiveDateTime::parse_from_str(&val, "%Y-%m-%dT%H:%M").unwrap();
                        let naive_utc = TimeZone::from_utc_datetime(&Utc, &naive_dt);
                        query.filter(Column::InsertDate.gte(naive_utc))
                    })
                },
                values: None,
                foreign_key: None,
            },
            ActixAdminModelFilter::<Entity> {
                name: "Is Visible".to_string(),
                filter_type: ActixAdminModelFilterType::Checkbox,
                filter: |q: sea_orm::Select<Entity>, v| -> sea_orm::Select<Entity> {
                    q.apply_if(v, |query, val: String| {
                        query.filter(Column::IsVisible.eq(val))
                    })
                },
                values: None,
                foreign_key: None,
            },
            ActixAdminModelFilter::<Entity> {
                name: "Post".to_string(),
                filter_type: ActixAdminModelFilterType::SelectList,
                filter: |q: sea_orm::Select<Entity>, v| -> sea_orm::Select<Entity> {
                    q.apply_if(v, |query, val: String| query.filter(Column::PostId.eq(val)))
                },
                values: None,
                foreign_key: None,
            },
            ActixAdminModelFilter::<Entity> {
                name: "Post with Tom Select".to_string(),
                filter_type: ActixAdminModelFilterType::TomSelectSearch,
                filter: |q: sea_orm::Select<Entity>, v| -> sea_orm::Select<Entity> {
                    q.apply_if(v, |query, val: String| query.filter(Column::PostId.eq(val)))
                },
                values: None,
                foreign_key: Some("post".to_string()),
            },
        ]
    }

    async fn get_filter_values(filter: &ActixAdminModelFilter<Entity>, db: &DatabaseConnection) -> Option<Vec<(String, String)>> { 
        match filter.name.as_str() {
            "Post" => Some({
                Post::find().order_by_asc(post::Column::Id).all(db).await.unwrap()
                    .iter().map(|p| (p.id.to_string(), p.title.to_string())).collect()
            }),
            _ => None
        }
    }
}
```