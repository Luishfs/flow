use super::{collection, indexed, reference, schema, storage_mapping, Error};
use itertools::Itertools;
use json::schema::types;
use models::{build, tables};
use protocol::flow;
use superslice::Ext;

pub fn walk_all_derivations(
    build_config: &flow::build_api::Config,
    built_collections: &[tables::BuiltCollection],
    derivations: &[tables::Derivation],
    imports: &[tables::Import],
    schema_index: &doc::SchemaIndex<'_>,
    schema_shapes: &[schema::Shape],
    storage_mappings: &[tables::StorageMapping],
    transforms: &[tables::Transform],
    errors: &mut tables::Errors,
) -> tables::BuiltDerivations {
    let mut built_derivations = tables::BuiltDerivations::new();

    for derivation in derivations {
        // Transforms are already ordered on (derivation, transform).
        let transforms =
            &transforms[transforms.equal_range_by_key(&&derivation.derivation, |t| &t.derivation)];

        built_derivations.insert_row(
            &derivation.scope,
            &derivation.derivation,
            walk_derivation(
                build_config,
                built_collections,
                derivation,
                imports,
                schema_index,
                schema_shapes,
                storage_mappings,
                transforms,
                errors,
            ),
        );
    }

    built_derivations
}

fn walk_derivation(
    build_config: &flow::build_api::Config,
    built_collections: &[tables::BuiltCollection],
    derivation: &tables::Derivation,
    imports: &[tables::Import],
    schema_index: &doc::SchemaIndex<'_>,
    schema_shapes: &[schema::Shape],
    storage_mappings: &[tables::StorageMapping],
    transforms: &[tables::Transform],
    errors: &mut tables::Errors,
) -> flow::DerivationSpec {
    let tables::Derivation {
        scope,
        derivation: collection,
        register_schema,
        register_initial,
        shards: _,
    } = derivation;

    // Pluck the BuiltCollection and register schema Shape of this Derivation.
    // Both of these must exist, though the |register_schema| may be a
    // placeholder if the schema does not exist.
    let built_collection = &built_collections
        [built_collections.equal_range_by_key(&collection.as_str(), |c| &c.collection)][0];
    let register_schema =
        &schema_shapes[schema_shapes.equal_range_by_key(&register_schema, |s| &s.schema)][0];

    // Verify that the register's initial value conforms to its schema.
    if schema_index.fetch(&register_schema.schema).is_none() {
        // Referential integrity error, which we've already reported.
    } else if let Err(err) = doc::Validation::validate(
        &mut doc::Validator::new(schema_index),
        &register_schema.schema,
        register_initial.clone(),
    )
    .unwrap()
    .ok()
    {
        Error::RegisterInitialInvalid(err).push(scope, errors);
    }

    // We'll collect TransformSpecs and types of each transform's shuffle key (if known).
    let mut built_transforms = Vec::new();
    let mut shuffle_types: Vec<(Vec<types::Set>, &tables::Transform)> = Vec::new();
    let mut strict_shuffle = false;

    // Walk transforms of this derivation.
    for transform in transforms {
        if let Some(type_set) = walk_transform(
            built_collections,
            imports,
            schema_shapes,
            transform,
            &mut built_transforms,
            errors,
        ) {
            shuffle_types.push((type_set, transform));
        }

        // In the trivial case of a publish-only derivation which has no shuffles,
        // we don't require that shuffle keys align on types and length.
        // This is because it doesn't matter for correctness, and the user probably
        // wants the default behavior of shuffling each collection on it's native key
        // (which minimizes data movement).
        // If the derivation is stateful, or if the user showed intent towards a
        // particular means of shuffling, then we *do* require that shuffle keys match.
        if transform.update_lambda.is_some()
            || transform.shuffle_key.is_some()
            || transform.shuffle_lambda.is_some()
        {
            strict_shuffle = true;
        }
    }

    indexed::walk_duplicates(
        transforms
            .iter()
            .map(|t| ("transform", t.transform.as_str(), &t.scope)),
        errors,
    );

    // Verify that shuffle key types & lengths align.
    for ((l_types, l_transform), (r_types, r_transform)) in shuffle_types.iter().tuple_windows() {
        if strict_shuffle && l_types != r_types {
            Error::ShuffleKeyMismatch {
                lhs_name: l_transform.transform.to_string(),
                lhs_types: l_types.clone(),
                rhs_name: r_transform.transform.to_string(),
                rhs_types: r_types.clone(),
            }
            .push(&l_transform.scope, errors);
        }
    }

    let recovery_stores = storage_mapping::mapped_stores(
        scope,
        "derivation",
        imports,
        &format!("recovery/{}", collection.as_str()),
        storage_mappings,
        errors,
    );

    build::derivation_spec(
        build_config,
        derivation,
        built_collection,
        built_transforms,
        recovery_stores,
        &register_schema.bundle,
    )
}

pub fn walk_transform(
    built_collections: &[tables::BuiltCollection],
    imports: &[tables::Import],
    schema_shapes: &[schema::Shape],
    transform: &tables::Transform,
    built_transforms: &mut Vec<flow::TransformSpec>,
    errors: &mut tables::Errors,
) -> Option<Vec<types::Set>> {
    let tables::Transform {
        scope,
        derivation: _,
        priority: _,
        publish_lambda,
        read_delay_seconds: _,
        shuffle_key,
        shuffle_lambda,
        source_collection: source,
        source_partitions,
        source_schema,
        transform: name,
        update_lambda,
    } = transform;

    indexed::walk_name(
        scope,
        "transform",
        name.as_ref(),
        models::Transform::regex(),
        errors,
    );

    if update_lambda.is_none() && publish_lambda.is_none() {
        Error::NoUpdateOrPublish {
            transform: name.to_string(),
        }
        .push(scope, errors);
    }

    // Dereference the transform's source. We can't continue without it.
    let source = match reference::walk_reference(
        scope,
        &format!("transform {}", name.as_str()),
        "collection",
        source,
        built_collections,
        |c| (&c.collection, &c.scope),
        imports,
        errors,
    ) {
        Some(s) => s,
        None => return None,
    };

    if let Some(selector) = source_partitions {
        // Note that the selector is deliberately checked against the
        // collection's schema shape, and not our own transform source schema.
        collection::walk_selector(scope, &source.spec, &selector, errors);
    }

    // Map to an effective source schema & shape.
    let source_schema = match source_schema {
        Some(url) => {
            // Was the collection locally defined using this same schema?
            if source.foreign_build_id.is_none() && url.as_str() == &source.spec.schema_uri {
                Error::SourceSchemaNotDifferent {
                    schema: url.clone(),
                    collection: source.collection.to_string(),
                }
                .push(scope, errors);
            }
            url.as_str()
        }
        None => &source.spec.schema_uri,
    };
    let source_shape =
        &schema_shapes[schema_shapes.equal_range_by_key(&source_schema, |s| s.schema.as_str())][0];

    // Project |source.spec.key| from Vec<String> => CompositeKey.
    let source_key = models::CompositeKey::new(
        source
            .spec
            .key_ptrs
            .iter()
            .map(|k| models::JsonPointer::new(k))
            .collect::<Vec<_>>(),
    );

    // Map to an effective shuffle key.
    let shuffle_key = match (shuffle_key, shuffle_lambda) {
        (Some(shuffle_key), None) => {
            if shuffle_key.iter().eq(source_key.iter()) {
                Error::ShuffleKeyNotDifferent {
                    transform: name.to_string(),
                    collection: source.collection.to_string(),
                }
                .push(scope, errors);
            }
            if shuffle_key.is_empty() {
                Error::ShuffleKeyEmpty {
                    transform: name.to_string(),
                }
                .push(scope, errors);
            }
            schema::walk_composite_key(scope, shuffle_key, source_shape, errors);

            Some(shuffle_key)
        }
        // If no shuffle_key is set, shuffle on the collection's key.
        (None, None) => Some(&source_key),
        // A shuffle lambda is an alternative to a shuffle key.
        (None, Some(_)) => None,

        (Some(_), Some(_)) => unreachable!("shuffle_key and shuffle_lambda cannot both be set"),
    };

    let shuffle_types = shuffle_key.and_then(|shuffle_key| {
        // Walk and collect key value types, so we can compare
        // with other transforms of this derivation later.
        schema::gather_key_types(shuffle_key, source_shape)
    });

    built_transforms.push(build::transform_spec(
        transform,
        &source.spec,
        &source_shape.bundle,
    ));

    shuffle_types
}
