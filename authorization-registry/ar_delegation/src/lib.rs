use ar_entity::delegation_evidence::ResourceRule;
use ishare::delegation_evidence::{
    PolicySetTarget, PolicySetTargetEnvironment, Resource, ResourceRules, ResourceTarget,
};
use ishare::delegation_request::{DelegationRequest, Policy, PolicySet};

pub use ar_entity::delegation_evidence::{DelegationEvidencePolicy, MatchingPolicySetRow};

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
        && dr_policy.target.environment.as_ref().is_none_or(|e| {
            is_contained_by(&e.service_providers, &de_policy_set.service_providers)
        });
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

pub fn get_delegation_evidence_policy_sets(
    delegation_request: &DelegationRequest,
    matching_policy_sets: &Vec<MatchingPolicySetRow>,
) -> Vec<ishare::delegation_evidence::PolicySet> {
    let mut policy_sets = vec![];
    for ps in delegation_request.policy_sets.iter() {
        let matching_policy_sets = mask_matching_policy_sets(ps, &matching_policy_sets);

        if matching_policy_sets.len() > 0 {
            for matching in matching_policy_sets.into_iter() {
                let policies: Vec<ishare::delegation_evidence::Policy> = ps
                    .policies
                    .iter()
                    .map(|p| {
                        let permit = is_permit(p, matching);

                        ishare::delegation_evidence::Policy {
                            target: ResourceTarget {
                                actions: p.target.actions.clone(),
                                environment: match p.target.environment.as_ref() {
                                    Some(e) => Some(ishare::delegation_evidence::Environment {
                                        service_providers: e.service_providers.clone(),
                                    }),
                                    None => None,
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
        } else {
            let policies: Vec<ishare::delegation_evidence::Policy> = ps
                .policies
                .iter()
                .map(|p| ishare::delegation_evidence::Policy {
                    target: ResourceTarget {
                        actions: p.target.actions.clone(),
                        environment: match p.target.environment.as_ref() {
                            Some(e) => Some(ishare::delegation_evidence::Environment {
                                service_providers: e.service_providers.clone(),
                            }),
                            None => None,
                        },
                        resource: Resource {
                            resource_type: p.target.resource.resource_type.clone(),
                            identifiers: p.target.resource.identifiers.clone(),
                            attributes: p.target.resource.attributes.clone(),
                        },
                    },
                    rules: vec![ResourceRules {
                        effect: "Deny".to_string(),
                    }],
                })
                .collect();

            let new_policy_set = ishare::delegation_evidence::PolicySet {
                max_delegation_depth: 0,
                policies,
                target: PolicySetTarget {
                    environment: PolicySetTargetEnvironment { licenses: vec![] },
                },
            };

            policy_sets.push(new_policy_set);
        }
    }

    return policy_sets;
}
