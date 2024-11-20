use anyhow::Context;
use ar_entity::delegation_evidence::ResourceRule;
use ishare::delegation_evidence::{
    DelegationEvidence, DelegationEvidenceContainer, DelegationTarget, PolicySetTarget,
    PolicySetTargetEnvironment, Resource, ResourceRules, ResourceTarget,
};
use ishare::delegation_request::{DelegationRequest, Policy, PolicySet};
use sea_orm::DatabaseConnection;

use crate::db::policy::{self as policy_store, DelegationEvidencePolicy, MatchingPolicySetRow};
use crate::error::AppError;
use crate::TimeProvider;

pub fn is_contained_by<T: PartialEq>(vec_a: &Vec<T>, vec_b: &Vec<T>) -> bool {
    vec_a.iter().all(|x| vec_b.contains(x))
}

// returns true if either the first element of vec_a is a star: ['*']
// or if all the elements of vec_a ar present in vec_b
pub fn star_or_contained_by(vec_a: &Vec<String>, vec_b: &Vec<String>) -> bool {
    vec_b.get(0).is_some_and(|i| i == "*") || is_contained_by(vec_a, vec_b)
}

pub fn is_matching_policy(dr_policy: &Policy, de_policy_set: &DelegationEvidencePolicy) -> bool {
    return star_or_contained_by(
        &dr_policy.target.resource.identifiers,
        &de_policy_set.identifiers,
    ) && star_or_contained_by(
        &dr_policy.target.resource.attributes,
        &de_policy_set.attributes,
    ) && star_or_contained_by(&dr_policy.target.actions, &de_policy_set.actions)
        && dr_policy.target.resource.resource_type == de_policy_set.resource_type
        && is_contained_by(
            &dr_policy.target.environment.service_providers,
            &de_policy_set.service_providers,
        );
}

pub fn mask_matching_policy_sets<'a>(
    policy_set: &PolicySet,
    de_policy_sets: &'a Vec<MatchingPolicySetRow>,
) -> Vec<&'a MatchingPolicySetRow> {
    let filtered: Vec<&MatchingPolicySetRow> = de_policy_sets
        .into_iter()
        .filter(|ps| {
            policy_set
                .policies
                .iter()
                .all(|p1| ps.policies.iter().any(|p2| is_matching_policy(p1, p2)))
        })
        .collect();

    filtered
}

pub fn is_permit(policy: &Policy, matching_row: &MatchingPolicySetRow) -> bool {
    let mut matching_policies = matching_row
        .policies
        .iter()
        .filter(|mp| is_matching_policy(policy, mp));

    let permit = matching_policies.all(|matching_policy| {
        matching_policy.rules.iter().all(|r| match r {
            ResourceRule::Permit => true,
            ResourceRule::Deny(t) => {
                !(star_or_contained_by(
                    &policy.target.resource.identifiers,
                    &t.target.resource.identifiers,
                ) && star_or_contained_by(
                    &policy.target.resource.attributes,
                    &t.target.resource.attributes,
                ) && star_or_contained_by(&policy.target.actions, &t.target.actions)
                    && &policy.target.resource.resource_type == &t.target.resource.resource_type)
            }
        })
    });

    return permit;
}

pub async fn create_delegation_evidence(
    delegation_request: &DelegationRequest,
    time_provider: std::sync::Arc<dyn TimeProvider>,
    de_expiry_seconds: i64,
    db: &DatabaseConnection,
) -> Result<DelegationEvidenceContainer, AppError> {
    tracing::info!(
        "Retrieving policy sets for access subject '{}' and policy issuer '{}'",
        &delegation_request.target.access_subject,
        &delegation_request.policy_issuer
    );

    let de_policy_sets = policy_store::get_policy_sets_with_policies(
        Some(delegation_request.target.access_subject.to_owned()),
        Some(delegation_request.policy_issuer.to_owned()),
        &db,
    )
    .await
    .context("Error getting policy sets")?;

    let mut policy_sets = vec![];
    for ps in delegation_request.policy_sets.iter() {
        let matching_policy_sets = mask_matching_policy_sets(ps, &de_policy_sets);

        for matching in matching_policy_sets {
            let policies: Vec<ishare::delegation_evidence::Policy> = ps
                .policies
                .iter()
                .map(|p| {
                    let permit = is_permit(p, matching);

                    ishare::delegation_evidence::Policy {
                        target: ResourceTarget {
                            actions: p.target.actions.clone(),
                            environment: ishare::delegation_evidence::Environment {
                                service_providers: p.target.environment.service_providers.clone(),
                            },
                            resource: Resource {
                                resource_type: p.target.resource.resource_type.clone(),
                                identifiers: p.target.resource.identifiers.clone(),
                                attributes: p.target.resource.attributes.clone(),
                            },
                        },
                        rules: vec![ResourceRules {
                            effect: if permit {
                                "Permit".to_string()
                            } else {
                                "Deny".to_string()
                            },
                        }],
                    }
                })
                .collect();

            let new_policy_set = ishare::delegation_evidence::PolicySet {
                max_delegation_depth: matching.max_delegation_depth,
                policies,
                target: PolicySetTarget {
                    environment: PolicySetTargetEnvironment {
                        licenses: matching.licenses.clone(),
                    },
                },
            };

            policy_sets.push(new_policy_set);
        }
    }

    let now = time_provider.now().timestamp();
    let de_container = DelegationEvidenceContainer {
        delegation_evidence: DelegationEvidence {
            not_before: now,
            not_on_or_after: now + de_expiry_seconds,
            policy_issuer: delegation_request.policy_issuer.clone(),
            target: DelegationTarget {
                access_subject: delegation_request.target.access_subject.clone(),
            },
            policy_sets,
        },
    };

    Ok(de_container)
}

#[cfg(test)]
mod tests {
    use ar_entity::delegation_evidence::{Deny, Resource, ResourceRule, Target};
    use ishare::delegation_request::{
        Environment, Resource as DRResource, ResourceRules, ResourceTarget,
    };
    use uuid::Uuid;

    use super::*;

    #[test]
    fn test_is_contained_by() {
        assert_eq!(is_contained_by::<i32>(&vec![], &vec![]), true);
        assert_eq!(is_contained_by(&vec![1, 2], &vec![1]), false);
        assert_eq!(is_contained_by(&vec![1, 2], &vec![1, 2]), true);
        assert_eq!(is_contained_by(&vec![4, 3, 2], &vec![2, 3, 4, 5]), true);
    }

    #[test]
    fn test_star_or_contained_by() {
        assert_eq!(star_or_contained_by(&vec![], &vec!["*".to_owned()]), true);
        assert_eq!(
            star_or_contained_by(&vec!["chicken".to_owned()], &vec!["*".to_owned()]),
            true
        );
        assert_eq!(star_or_contained_by(&vec![], &vec![]), true);
        assert_eq!(
            star_or_contained_by(&vec![], &vec!["fish".to_owned()]),
            true
        );
        assert_eq!(
            star_or_contained_by(&vec!["a".to_owned()], &vec!["a".to_owned()]),
            true
        );
        assert_eq!(
            star_or_contained_by(&vec!["a".to_owned(), "b".to_owned()], &vec!["b".to_owned()]),
            false
        );
        assert_eq!(
            star_or_contained_by(
                &vec!["fish".to_owned()],
                &vec!["chicken".to_owned(), "*".to_owned()]
            ),
            false
        );
    }

    #[test]
    fn test_is_matching_policy_match_stars() {
        let is_match = is_matching_policy(
            &Policy {
                target: ResourceTarget {
                    actions: vec!["Read".to_owned()],
                    resource: DRResource {
                        resource_type: "nice-resource".to_owned(),
                        identifiers: vec!["chicken".to_owned()],
                        attributes: vec!["chicken".to_owned()],
                    },
                    environment: Environment {
                        service_providers: vec!["fishery".to_owned()],
                    },
                },
                rules: vec![ResourceRules {
                    effect: "Effect".to_owned(),
                }],
            },
            &DelegationEvidencePolicy {
                id: Uuid::new_v4(),
                actions: vec!["*".to_owned()],
                identifiers: vec!["*".to_owned()],
                attributes: vec!["*".to_owned()],
                resource_type: "nice-resource".to_owned(),
                rules: vec![ResourceRule::Permit],
                service_providers: vec!["fishery".to_owned()],
            },
        );

        assert_eq!(is_match, true);
    }

    #[test]
    fn test_is_matching_policy_match_literal() {
        let is_match = is_matching_policy(
            &Policy {
                target: ResourceTarget {
                    actions: vec!["Read".to_owned()],
                    resource: DRResource {
                        resource_type: "nice-resource".to_owned(),
                        identifiers: vec!["id1".to_owned()],
                        attributes: vec!["att1".to_owned()],
                    },
                    environment: Environment {
                        service_providers: vec!["fishery".to_owned()],
                    },
                },
                rules: vec![ResourceRules {
                    effect: "Effect".to_owned(),
                }],
            },
            &DelegationEvidencePolicy {
                id: Uuid::new_v4(),
                actions: vec!["Read".to_owned()],
                identifiers: vec!["id1".to_owned()],
                attributes: vec!["att1".to_owned()],
                resource_type: "nice-resource".to_owned(),
                rules: vec![ResourceRule::Permit],
                service_providers: vec!["fishery".to_owned()],
            },
        );

        assert_eq!(is_match, true);
    }

    #[test]
    fn test_is_matching_policy_no_resource_type() {
        let is_match = is_matching_policy(
            &Policy {
                target: ResourceTarget {
                    actions: vec!["Read".to_owned()],
                    resource: DRResource {
                        resource_type: "nice-resource-2".to_owned(),
                        identifiers: vec!["chicken".to_owned()],
                        attributes: vec!["chicken".to_owned()],
                    },
                    environment: Environment {
                        service_providers: vec!["fishery".to_owned()],
                    },
                },
                rules: vec![ResourceRules {
                    effect: "Effect".to_owned(),
                }],
            },
            &DelegationEvidencePolicy {
                id: Uuid::new_v4(),
                actions: vec!["*".to_owned()],
                identifiers: vec!["*".to_owned()],
                attributes: vec!["*".to_owned()],
                resource_type: "nice-resource".to_owned(),
                rules: vec![ResourceRule::Permit],
                service_providers: vec!["fishery".to_owned()],
            },
        );

        assert_eq!(is_match, false);
    }

    #[test]
    fn test_is_matching_policy_no_id() {
        let is_match = is_matching_policy(
            &Policy {
                target: ResourceTarget {
                    actions: vec!["Read".to_owned()],
                    resource: DRResource {
                        resource_type: "nice-resource".to_owned(),
                        identifiers: vec!["chicken".to_owned()],
                        attributes: vec!["chicken".to_owned()],
                    },
                    environment: Environment {
                        service_providers: vec!["fishery".to_owned()],
                    },
                },
                rules: vec![ResourceRules {
                    effect: "Effect".to_owned(),
                }],
            },
            &DelegationEvidencePolicy {
                id: Uuid::new_v4(),
                actions: vec!["*".to_owned()],
                identifiers: vec!["fish".to_owned()],
                attributes: vec!["*".to_owned()],
                resource_type: "nice-resource".to_owned(),
                rules: vec![ResourceRule::Permit],
                service_providers: vec!["fishery".to_owned()],
            },
        );

        assert_eq!(is_match, false);
    }

    #[test]
    fn test_is_matching_policy_no_att() {
        let is_match = is_matching_policy(
            &Policy {
                target: ResourceTarget {
                    actions: vec!["Read".to_owned()],
                    resource: DRResource {
                        resource_type: "nice-resource".to_owned(),
                        identifiers: vec!["chicken".to_owned()],
                        attributes: vec!["chicken".to_owned()],
                    },
                    environment: Environment {
                        service_providers: vec!["fishery".to_owned()],
                    },
                },
                rules: vec![ResourceRules {
                    effect: "Effect".to_owned(),
                }],
            },
            &DelegationEvidencePolicy {
                id: Uuid::new_v4(),
                actions: vec!["*".to_owned()],
                identifiers: vec!["*".to_owned()],
                attributes: vec!["att".to_owned()],
                resource_type: "nice-resource".to_owned(),
                rules: vec![ResourceRule::Permit],
                service_providers: vec!["fishery".to_owned()],
            },
        );

        assert_eq!(is_match, false);
    }

    #[test]
    fn test_is_matching_policy_no_service_provider() {
        let is_match = is_matching_policy(
            &Policy {
                target: ResourceTarget {
                    actions: vec!["Read".to_owned()],
                    resource: DRResource {
                        resource_type: "nice-resource".to_owned(),
                        identifiers: vec!["chicken".to_owned()],
                        attributes: vec!["chicken".to_owned()],
                    },
                    environment: Environment {
                        service_providers: vec!["another-fishery".to_owned()],
                    },
                },
                rules: vec![ResourceRules {
                    effect: "Effect".to_owned(),
                }],
            },
            &DelegationEvidencePolicy {
                id: Uuid::new_v4(),
                actions: vec!["*".to_owned()],
                identifiers: vec!["*".to_owned()],
                attributes: vec!["*".to_owned()],
                resource_type: "nice-resource".to_owned(),
                rules: vec![ResourceRule::Permit],
                service_providers: vec!["fishery".to_owned()],
            },
        );

        assert_eq!(is_match, false);
    }

    #[test]
    fn test_mask_matching_policy_sets_match() {
        let matching_policy_set_rows = vec![MatchingPolicySetRow {
            access_subject: "as".to_owned(),
            licenses: vec!["ISHARE.001".to_owned()],
            policy_set_id: Uuid::new_v4(),
            policy_issuer: "issuer".to_owned(),
            max_delegation_depth: 1,
            policies: vec![DelegationEvidencePolicy {
                id: Uuid::new_v4(),
                identifiers: vec!["fish".to_owned()],
                resource_type: "nice-resource".to_owned(),
                attributes: vec!["chicken".to_owned()],
                actions: vec!["Read".to_owned()],
                service_providers: vec!["fishery".to_owned()],
                rules: vec![ResourceRule::Permit],
            }],
        }];

        let matching_rows = mask_matching_policy_sets(
            &PolicySet {
                policies: vec![Policy {
                    target: ResourceTarget {
                        actions: vec!["Read".to_owned()],
                        resource: DRResource {
                            resource_type: "nice-resource".to_owned(),
                            identifiers: vec!["fish".to_owned()],
                            attributes: vec!["chicken".to_owned()],
                        },
                        environment: Environment {
                            service_providers: vec!["fishery".to_owned()],
                        },
                    },
                    rules: vec![ResourceRules {
                        effect: "Effect".to_owned(),
                    }],
                }],
            },
            &matching_policy_set_rows,
        );

        assert_eq!(matching_rows.len(), 1);
    }

    #[test]
    fn test_mask_matching_policy_sets_no_match() {
        let matching_policy_set_rows = vec![MatchingPolicySetRow {
            access_subject: "as".to_owned(),
            licenses: vec!["ISHARE.001".to_owned()],
            policy_set_id: Uuid::new_v4(),
            policy_issuer: "issuer".to_owned(),
            max_delegation_depth: 1,
            policies: vec![DelegationEvidencePolicy {
                id: Uuid::new_v4(),
                identifiers: vec!["fish".to_owned()],
                resource_type: "another-nice-resource".to_owned(),
                attributes: vec!["chicken".to_owned()],
                actions: vec!["Read".to_owned()],
                service_providers: vec!["fishery".to_owned()],
                rules: vec![ResourceRule::Permit],
            }],
        }];

        let matching_rows = mask_matching_policy_sets(
            &PolicySet {
                policies: vec![Policy {
                    target: ResourceTarget {
                        actions: vec!["Read".to_owned()],
                        resource: DRResource {
                            resource_type: "nice-resource".to_owned(),
                            identifiers: vec!["fish".to_owned()],
                            attributes: vec!["chicken".to_owned()],
                        },
                        environment: Environment {
                            service_providers: vec!["fishery".to_owned()],
                        },
                    },
                    rules: vec![ResourceRules {
                        effect: "Effect".to_owned(),
                    }],
                }],
            },
            &matching_policy_set_rows,
        );

        assert_eq!(matching_rows.len(), 0);
    }

    #[test]
    fn test_is_permit_permit() {
        let matching_policy_set_row = MatchingPolicySetRow {
            access_subject: "as".to_owned(),
            licenses: vec!["ISHARE.001".to_owned()],
            policy_set_id: Uuid::new_v4(),
            policy_issuer: "issuer".to_owned(),
            max_delegation_depth: 1,
            policies: vec![DelegationEvidencePolicy {
                id: Uuid::new_v4(),
                identifiers: vec!["fish".to_owned()],
                resource_type: "another-nice-resource".to_owned(),
                attributes: vec!["chicken".to_owned()],
                actions: vec!["Read".to_owned()],
                service_providers: vec!["fishery".to_owned()],
                rules: vec![ResourceRule::Permit],
            }],
        };

        let is_permit = is_permit(
            &Policy {
                target: ResourceTarget {
                    actions: vec!["Read".to_owned()],
                    resource: DRResource {
                        resource_type: "nice-resource".to_owned(),
                        identifiers: vec!["chicken".to_owned()],
                        attributes: vec!["chicken".to_owned()],
                    },
                    environment: Environment {
                        service_providers: vec!["fishery".to_owned()],
                    },
                },
                rules: vec![ResourceRules {
                    effect: "Effect".to_owned(),
                }],
            },
            &matching_policy_set_row,
        );

        assert_eq!(is_permit, true)
    }

    #[test]
    fn test_is_permit_deny() {
        let matching_policy_set_row = MatchingPolicySetRow {
            access_subject: "as".to_owned(),
            licenses: vec!["ISHARE.001".to_owned()],
            policy_set_id: Uuid::new_v4(),
            policy_issuer: "issuer".to_owned(),
            max_delegation_depth: 1,
            policies: vec![DelegationEvidencePolicy {
                id: Uuid::new_v4(),
                identifiers: vec!["*".to_owned()],
                resource_type: "nice-resource".to_owned(),
                attributes: vec!["*".to_owned()],
                actions: vec!["Read".to_owned()],
                service_providers: vec!["fishery".to_owned()],
                rules: vec![ResourceRule::Deny(Deny {
                    target: Target {
                        resource: Resource {
                            resource_type: "nice-resource".to_owned(),
                            identifiers: vec!["chicken".to_owned()],
                            attributes: vec!["chicken".to_owned()],
                        },
                        actions: vec!["Read".to_owned()],
                    },
                })],
            }],
        };

        let is_permit = is_permit(
            &Policy {
                target: ResourceTarget {
                    actions: vec!["Read".to_owned()],
                    resource: DRResource {
                        resource_type: "nice-resource".to_owned(),
                        identifiers: vec!["chicken".to_owned()],
                        attributes: vec!["chicken".to_owned()],
                    },
                    environment: Environment {
                        service_providers: vec!["fishery".to_owned()],
                    },
                },
                rules: vec![ResourceRules {
                    effect: "Effect".to_owned(),
                }],
            },
            &matching_policy_set_row,
        );

        assert_eq!(is_permit, false)
    }
}
