#[cfg(test)]
pub mod fixtures {
    #[derive(Deserialize)]
    pub struct PolicySetFixture {
        pub policy_set: ar_entity::policy_set::Model,
        policies: Vec<ar_entity::policy::Model>,
    }

    use sea_orm::{DatabaseConnection, EntityTrait, IntoActiveModel};
    use serde::Deserialize;

    // fn load_policy_fixture(path: &str) -> ar_entity::policy::Model {
    //     let content = std::fs::read_to_string(path).expect("Can't load policy fixture");
    //     return serde_json::from_str::<ar_entity::policy::Model>(&content)
    //         .expect(format!("can't deserialize policy in {path}").as_str());
    // }

    pub fn load_company_fixture(path: &str) -> ar_entity::company::Model {
        let content = std::fs::read_to_string(path).expect("Can't load company fixture");
        return serde_json::from_str::<ar_entity::company::Model>(&content).unwrap();
    }

    pub fn load_user_fixture(path: &str) -> ar_entity::ishare_user::Model {
        let content = std::fs::read_to_string(path).expect("Can't load user fixture");
        return serde_json::from_str::<ar_entity::ishare_user::Model>(&content).unwrap();
    }

    pub fn load_policy_set_fixture(path: &str) -> PolicySetFixture {
        let content = std::fs::read_to_string(path).expect("Can't load policy set fixture");
        return serde_json::from_str::<PolicySetFixture>(&content)
            .expect(format!("can't deserialize policy set in {path}").as_str());
    }

    pub async fn insert_policy_set_fixture(path: &str, db: &DatabaseConnection) {
        let policy_set = load_policy_set_fixture(path);
        let active_policy_set = policy_set.policy_set.into_active_model();

        ar_entity::policy_set::Entity::insert(active_policy_set)
            .exec(db)
            .await
            .unwrap();

        for p in policy_set.policies.iter() {
            let active_p = p.clone().into_active_model();

            ar_entity::policy::Entity::insert(active_p)
                .exec(db)
                .await
                .unwrap();
        }
    }

    async fn insert_company_fixture(path: &str, db: &DatabaseConnection) {
        let policy_template = load_company_fixture(path);
        let active_company = policy_template.into_active_model();

        ar_entity::company::Entity::insert(active_company)
            .exec(db)
            .await
            .unwrap();
    }

    async fn insert_user_fixture(path: &str, db: &DatabaseConnection) {
        let policy_template = load_user_fixture(path);
        let active_user = policy_template.into_active_model();

        ar_entity::ishare_user::Entity::insert(active_user)
            .exec(db)
            .await
            .unwrap();
    }

    pub async fn apply(db: &DatabaseConnection) {
        insert_company_fixture("./fixtures/company1.json", db).await;
        insert_company_fixture("./fixtures/company2.json", db).await;
        insert_company_fixture("./fixtures/company3.json", db).await;

        insert_user_fixture("./fixtures/user1.json", db).await;
        insert_user_fixture("./fixtures/user2.json", db).await;
    }
}
