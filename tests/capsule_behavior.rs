use proptest::prelude::*;

use nexum::capsule::{Capsule, CapsuleMode, normalize_slug};

#[test]
fn slug_is_normalized_for_domain_identity() {
    let capsule = Capsule::new("cap-1", "Alpha Project_01!", CapsuleMode::HostDefault, 3);
    assert_eq!(capsule.slug, "alpha-project-01");
    assert_eq!(capsule.domain(), "alpha-project-01.nexum.local");
}

#[test]
fn capsule_slug_is_immutable_after_rename() {
    let mut capsule = Capsule::new("cap-2", "Billing API", CapsuleMode::IsolatedNixShell, 5);
    assert_eq!(capsule.slug, "billing-api");

    capsule.rename_display_name("Billing API V2");

    assert_eq!(capsule.display_name, "Billing API V2");
    assert_eq!(capsule.slug, "billing-api");
    assert_eq!(capsule.domain(), "billing-api.nexum.local");
}

proptest! {
    #[test]
    fn normalized_slugs_are_dns_safe(input in "[A-Za-z0-9 _./-]{1,48}") {
        let slug = normalize_slug(&input);
        prop_assert!(!slug.is_empty());
        prop_assert!(slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'));
        prop_assert!(!slug.starts_with('-'));
        prop_assert!(!slug.ends_with('-'));
    }
}
