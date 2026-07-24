//! In-memory Authorization Registry fake for consumer test suites.
//!
//! Decisions are made by the real `ar_delegation` matching engine, so tests
//! exercise the same logic as a live AR without a server or database. This is
//! the only crate a consumer needs as a dev-dependency: the engine and the
//! delegation-evidence types are re-exported here.
//!
//! Typical use is a thin adapter implementing the consumer's own
//! authorization-provider trait:
//!
//! ```ignore
//! let ar = TestAuthorizationRegistry::new()
//!     .permit("EU.EORI.OWNER", "EU.EORI.REQUESTER", "DMI.DataAccess", "Read", "file-1");
//!
//! #[async_trait]
//! impl AuthorizationProvider for MyTestProvider {
//!     async fn authorize(&self, /* ... */) -> Result<(), MyError> {
//!         if self.ar.is_permitted(subject, issuer, resource_type, action, identifiers, None) {
//!             Ok(())
//!         } else {
//!             Err(MyError::Unauthorized)
//!         }
//!     }
//! }
//! ```

use std::sync::Mutex;

use ishare::delegation_request::{
    DelegationRequest, DelegationTarget, Environment, Policy, PolicySet, Resource, ResourceRules,
    ResourceTarget,
};
use uuid::Uuid;

pub use ar_delegation;
pub use ar_entity;
pub use ar_entity::delegation_evidence::{
    DelegationEvidencePolicy, MatchingPolicySetRow, ResourceRule,
};

/// An in-memory stand-in for the Authorization Registry's policy store and
/// delegation endpoint. Rows are kept behind a `Mutex` so the registry can be
/// shared as `Arc<TestAuthorizationRegistry>` and mutated through `&self`,
/// matching how provider traits are injected into app state.
#[derive(Default)]
pub struct TestAuthorizationRegistry {
    rows: Mutex<Vec<MatchingPolicySetRow>>,
}

impl TestAuthorizationRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Chainable builder: `policy_issuer` permits `access_subject` to perform
    /// `action` on the resource `identifier` of `resource_type`.
    pub fn permit(
        self,
        policy_issuer: &str,
        access_subject: &str,
        resource_type: &str,
        action: &str,
        identifier: &str,
    ) -> Self {
        self.insert_permit(
            policy_issuer,
            access_subject,
            None,
            resource_type,
            vec![action.to_owned()],
            Some(vec![identifier.to_owned()]),
        );
        self
    }

    /// Insert a permit policy set and return its id, mirroring the AR's
    /// policy-set insertion. Use this from adapters that implement
    /// append-style APIs, or when a permit needs multiple actions, wildcard
    /// identifiers (`None`), or a service provider restriction.
    pub fn insert_permit(
        &self,
        policy_issuer: &str,
        access_subject: &str,
        service_provider: Option<&str>,
        resource_type: &str,
        actions: Vec<String>,
        identifiers: Option<Vec<String>>,
    ) -> Uuid {
        let policy_set_id = Uuid::new_v4();
        self.rows.lock().unwrap().push(MatchingPolicySetRow {
            policy_set_id,
            access_subject: access_subject.to_owned(),
            policy_issuer: policy_issuer.to_owned(),
            licenses: vec![],
            max_delegation_depth: 0,
            policies: vec![DelegationEvidencePolicy {
                id: Uuid::new_v4(),
                identifiers: identifiers.unwrap_or_else(|| vec!["*".to_owned()]),
                resource_type: resource_type.to_owned(),
                attributes: vec!["*".to_owned()],
                actions,
                service_providers: service_provider
                    .map(|sp| vec![sp.to_owned()])
                    .unwrap_or_default(),
                rules: vec![ResourceRule::Permit],
            }],
        });
        policy_set_id
    }

    /// Remove the policy set with the given id, mirroring policy-set deletion.
    pub fn remove(&self, policy_set_id: &str) {
        self.rows
            .lock()
            .unwrap()
            .retain(|r| r.policy_set_id.to_string() != policy_set_id);
    }

    pub fn rows(&self) -> Vec<MatchingPolicySetRow> {
        self.rows.lock().unwrap().clone()
    }

    /// Convenience decision check: builds a single-policy delegation request
    /// (attributes `["*"]`) and returns true when the stored rows yield
    /// evidence that permits it. `identifiers: None` matches rows regardless
    /// of their identifiers.
    ///
    /// `service_provider: Some(eori)` asks as that service provider (the way
    /// the ishare PDP client sends its own EORI as the request environment),
    /// so only rows naming that provider match — there is no wildcard for
    /// service providers in the matching engine. `None` sends no environment
    /// and skips service-provider matching entirely.
    pub fn is_permitted(
        &self,
        access_subject: &str,
        policy_issuer: &str,
        resource_type: &str,
        action: &str,
        identifiers: Option<Vec<String>>,
        service_provider: Option<&str>,
    ) -> bool {
        let request = DelegationRequest {
            policy_issuer: policy_issuer.to_owned(),
            target: DelegationTarget {
                access_subject: access_subject.to_owned(),
            },
            policy_sets: vec![PolicySet {
                policies: vec![Policy {
                    target: ResourceTarget {
                        resource: Resource {
                            resource_type: resource_type.to_owned(),
                            identifiers: identifiers.unwrap_or_default(),
                            attributes: vec!["*".to_owned()],
                        },
                        actions: vec![action.to_owned()],
                        environment: service_provider.map(|sp| Environment {
                            service_providers: vec![sp.to_owned()],
                        }),
                    },
                    rules: vec![ResourceRules {
                        effect: "Permit".to_owned(),
                    }],
                }],
            }],
        };

        let evidence = self.evaluate(&request);
        !evidence.is_empty()
            && evidence.iter().all(|ps| {
                ps.policies
                    .iter()
                    .all(|p| p.rules.iter().all(|r| r.effect == "Permit"))
            })
    }

    /// Escape hatch for full control over the delegation request: filters the
    /// stored rows by the request's policy issuer and access subject (as the
    /// AR does in its database query) and runs the real matching engine.
    pub fn evaluate(
        &self,
        request: &DelegationRequest,
    ) -> Vec<ishare::delegation_evidence::PolicySet> {
        let rows: Vec<MatchingPolicySetRow> = self
            .rows
            .lock()
            .unwrap()
            .iter()
            .filter(|r| {
                r.policy_issuer == request.policy_issuer
                    && r.access_subject == request.target.access_subject
            })
            .cloned()
            .collect();

        ar_delegation::get_delegation_evidence_policy_sets(request, &rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWNER: &str = "EU.EORI.NLOWNER";
    const REQUESTER: &str = "EU.EORI.NLREQUESTER";
    const RESOURCE_TYPE: &str = "DMI.DataAccess";

    fn ids(values: &[&str]) -> Option<Vec<String>> {
        Some(values.iter().map(|v| v.to_string()).collect())
    }

    #[test]
    fn permit_grants_access() {
        let ar = TestAuthorizationRegistry::new().permit(
            OWNER,
            REQUESTER,
            RESOURCE_TYPE,
            "Read",
            "file-1",
        );

        assert!(ar.is_permitted(
            REQUESTER,
            OWNER,
            RESOURCE_TYPE,
            "Read",
            ids(&["file-1"]),
            None
        ));
    }

    #[test]
    fn no_matching_row_denies_access() {
        let ar = TestAuthorizationRegistry::new();

        assert!(!ar.is_permitted(
            REQUESTER,
            OWNER,
            RESOURCE_TYPE,
            "Read",
            ids(&["file-1"]),
            None
        ));
    }

    #[test]
    fn different_identifier_action_or_type_denies_access() {
        let ar = TestAuthorizationRegistry::new().permit(
            OWNER,
            REQUESTER,
            RESOURCE_TYPE,
            "Read",
            "file-1",
        );

        assert!(!ar.is_permitted(
            REQUESTER,
            OWNER,
            RESOURCE_TYPE,
            "Read",
            ids(&["file-2"]),
            None
        ));
        assert!(!ar.is_permitted(
            REQUESTER,
            OWNER,
            RESOURCE_TYPE,
            "Delete",
            ids(&["file-1"]),
            None
        ));
        assert!(!ar.is_permitted(
            REQUESTER,
            OWNER,
            "Other.Type",
            "Read",
            ids(&["file-1"]),
            None
        ));
    }

    #[test]
    fn issuer_and_subject_must_match() {
        let ar = TestAuthorizationRegistry::new().permit(
            OWNER,
            REQUESTER,
            RESOURCE_TYPE,
            "Read",
            "file-1",
        );

        assert!(!ar.is_permitted(
            OWNER,
            REQUESTER,
            RESOURCE_TYPE,
            "Read",
            ids(&["file-1"]),
            None
        ));
        assert!(!ar.is_permitted(
            "EU.EORI.NLOTHER",
            OWNER,
            RESOURCE_TYPE,
            "Read",
            ids(&["file-1"]),
            None
        ));
    }

    #[test]
    fn service_provider_restriction_is_matched() {
        const DATASTATION: &str = "EU.EORI.NLDATASTATION";

        let ar = TestAuthorizationRegistry::new();
        ar.insert_permit(
            OWNER,
            REQUESTER,
            Some(DATASTATION),
            RESOURCE_TYPE,
            vec!["Read".to_owned()],
            Some(vec!["file-1".to_owned()]),
        );

        // asking as the service provider the permit names
        assert!(ar.is_permitted(
            REQUESTER,
            OWNER,
            RESOURCE_TYPE,
            "Read",
            ids(&["file-1"]),
            Some(DATASTATION)
        ));
        // asking as another service provider
        assert!(!ar.is_permitted(
            REQUESTER,
            OWNER,
            RESOURCE_TYPE,
            "Read",
            ids(&["file-1"]),
            Some("EU.EORI.NLOTHERSTATION")
        ));
        // no environment: service-provider matching is skipped
        assert!(ar.is_permitted(
            REQUESTER,
            OWNER,
            RESOURCE_TYPE,
            "Read",
            ids(&["file-1"]),
            None
        ));
    }

    #[test]
    fn permit_without_service_provider_denies_environment_requests() {
        let ar = TestAuthorizationRegistry::new().permit(
            OWNER,
            REQUESTER,
            RESOURCE_TYPE,
            "Read",
            "file-1",
        );

        // rows without service providers can never match a request that
        // carries an environment -- there is no wildcard for service providers
        assert!(!ar.is_permitted(
            REQUESTER,
            OWNER,
            RESOURCE_TYPE,
            "Read",
            ids(&["file-1"]),
            Some("EU.EORI.NLDATASTATION")
        ));
    }

    #[test]
    fn wildcard_identifier_permits_any_identifier() {
        let ar = TestAuthorizationRegistry::new();
        ar.insert_permit(
            OWNER,
            REQUESTER,
            None,
            RESOURCE_TYPE,
            vec!["Read".to_owned()],
            None,
        );

        assert!(ar.is_permitted(
            REQUESTER,
            OWNER,
            RESOURCE_TYPE,
            "Read",
            ids(&["anything"]),
            None
        ));
    }

    #[test]
    fn removed_policy_set_no_longer_permits() {
        let ar = TestAuthorizationRegistry::new();
        let policy_set_id = ar.insert_permit(
            OWNER,
            REQUESTER,
            None,
            RESOURCE_TYPE,
            vec!["Read".to_owned()],
            Some(vec!["file-1".to_owned()]),
        );

        assert!(ar.is_permitted(
            REQUESTER,
            OWNER,
            RESOURCE_TYPE,
            "Read",
            ids(&["file-1"]),
            None
        ));

        ar.remove(&policy_set_id.to_string());

        assert!(!ar.is_permitted(
            REQUESTER,
            OWNER,
            RESOURCE_TYPE,
            "Read",
            ids(&["file-1"]),
            None
        ));
        assert!(ar.rows().is_empty());
    }
}
