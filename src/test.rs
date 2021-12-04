#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        fs::{copy, create_dir, create_dir_all, remove_dir_all, File},
        io::{self, Write},
        path::Path,
    };

    use tempfile::TempDir;

    use crate::{
        asset_config::AssetConfig,
        asset_filter::{AssetFilter, AssetFilterOption, AssetFilterRegistry},
        assets::{AssetError, AssetErrorType, AssetFilterError},
        load_cache_manifest, pack,
    };

    #[derive(Debug)]
    struct DummyError {}

    impl AssetFilterError for DummyError {}

    struct TestCatFilter {}

    impl AssetFilter<DummyError> for TestCatFilter {
        fn process_asset_file(
            &self,
            input_file_paths: &[std::path::PathBuf],
            output_file_path: &Path,
            options: &HashMap<String, AssetFilterOption>,
        ) -> Result<(), AssetError<DummyError>> {
            let additional_text: Option<String>;
            let additional_text_option = options.get("additional_text");
            if let Some(AssetFilterOption::String(additional_text_real)) = additional_text_option {
                additional_text = Some(additional_text_real.clone());
            } else {
                additional_text = None;
            }

            if let Some(output_file_path_parent) = output_file_path.parent() {
                create_dir_all(output_file_path_parent)?;
            }
            let mut output_file = File::create(output_file_path)?;

            for input_file_path in input_file_paths {
                let mut input_file = File::open(input_file_path)?;
                io::copy(&mut input_file, &mut output_file)?;
            }
            if let Some(additional_text_real) = additional_text {
                write!(output_file, "{}", additional_text_real)?;
            }

            Ok(())
        }
    }

    #[test]
    fn test1() {
        let test_directory_path = Path::new("test_files");
        let temp_directory = TempDir::new().unwrap();
        let temp_directory_path = temp_directory.path();

        let resource_source_directory_path = test_directory_path.join("source");
        let resource_output_directory_path = test_directory_path.join("correct_output");
        let resource_manifest_directory_path = test_directory_path.join("assets.json");

        let source_directory_path = temp_directory_path.join("source");
        let internal_directory_path = temp_directory_path.join("internal");
        let target_directory_path = temp_directory_path.join("target");
        let cache_manifest_path = temp_directory_path.join("cache.json");
        let manifest_path = temp_directory_path.join("assets.json");

        create_dir(&source_directory_path).unwrap();
        create_dir(&internal_directory_path).unwrap();
        create_dir(&target_directory_path).unwrap();

        copy(
            resource_source_directory_path.join("a1.txt"),
            &source_directory_path.join("a.txt"),
        )
        .unwrap();
        copy(
            resource_source_directory_path.join("b.txt"),
            source_directory_path.join("b.txt"),
        )
        .unwrap();
        copy(&resource_manifest_directory_path, &manifest_path).unwrap();

        let config = AssetConfig {
            target_directory_path: target_directory_path.clone(),
            internal_directory_path: internal_directory_path.clone(),
            source_directory_path: source_directory_path.clone(),
        };

        let mut filters_map: HashMap<String, Box<dyn AssetFilter<DummyError>>> = HashMap::new();
        filters_map.insert("TestCat".to_string(), Box::new(TestCatFilter {}));

        let filter_registry = AssetFilterRegistry::new(filters_map);

        println!("Run pass #1");
        pack(
            &manifest_path,
            &cache_manifest_path,
            &config,
            &filter_registry,
        )
        .unwrap();

        let correct_output1 =
            std::fs::read_to_string(resource_output_directory_path.join("out1.txt")).unwrap();

        let cache_manifest1 = load_cache_manifest::<DummyError>(&cache_manifest_path).unwrap();
        let output1_path =
            target_directory_path.join(cache_manifest1.get_entry("out").unwrap().path);

        let output1 = std::fs::read_to_string(&output1_path).unwrap();
        assert_eq!(output1.trim(), correct_output1.trim());
        assert!(output1_path.starts_with(target_directory_path.join("out_text")));

        copy(
            resource_source_directory_path.join("a2.txt"),
            source_directory_path.join("a.txt"),
        )
        .unwrap();

        println!("Run pass #2");
        pack(
            &manifest_path,
            &cache_manifest_path,
            &config,
            &filter_registry,
        )
        .unwrap();

        let correct_output2 =
            std::fs::read_to_string(resource_output_directory_path.join("out2.txt")).unwrap();

        let cache_manifest2 = load_cache_manifest::<DummyError>(&cache_manifest_path).unwrap();
        let output2_path =
            target_directory_path.join(cache_manifest2.get_entry("out").unwrap().path);

        let output2 = std::fs::read_to_string(&output2_path).unwrap();
        assert_eq!(output2.trim(), correct_output2.trim());
        assert!(output1_path.starts_with(target_directory_path.join("out_text")));

        assert_ne!(output1_path, output2_path);
    }

    #[test]
    fn test2() {
        let test_directory_path = Path::new("test_files");
        let temp_directory = TempDir::new().unwrap();
        let temp_directory_path = temp_directory.path();

        let resource_source_directory_path = test_directory_path.join("source");
        let resource_output_directory_path = test_directory_path.join("correct_output");
        let resource_manifest_directory_path = test_directory_path.join("assets.json");

        let source_directory_path = temp_directory_path.join("source");
        let internal_directory_path = temp_directory_path.join("internal");
        let target_directory_path = temp_directory_path.join("target");
        let cache_manifest_path = temp_directory_path.join("cache.json");
        let manifest_path = temp_directory_path.join("assets.json");

        create_dir(&source_directory_path).unwrap();
        create_dir(&internal_directory_path).unwrap();
        create_dir(&target_directory_path).unwrap();

        copy(
            resource_source_directory_path.join("a1.txt"),
            source_directory_path.join("a.txt"),
        )
        .unwrap();
        copy(
            resource_source_directory_path.join("b.txt"),
            source_directory_path.join("b.txt"),
        )
        .unwrap();
        copy(&resource_manifest_directory_path, &manifest_path).unwrap();

        let config = AssetConfig {
            target_directory_path: target_directory_path.clone(),
            internal_directory_path: internal_directory_path.clone(),
            source_directory_path: source_directory_path.clone(),
        };

        let mut filters_map: HashMap<String, Box<dyn AssetFilter<DummyError>>> = HashMap::new();
        filters_map.insert("TestCat".to_string(), Box::new(TestCatFilter {}));

        let filter_registry = AssetFilterRegistry::new(filters_map);

        println!("Run pass #1");
        pack(
            &manifest_path,
            &cache_manifest_path,
            &config,
            &filter_registry,
        )
        .unwrap();

        let correct_output1 =
            std::fs::read_to_string(resource_output_directory_path.join("out1.txt")).unwrap();

        let cache_manifest1 = load_cache_manifest::<DummyError>(&cache_manifest_path).unwrap();
        let output1_path =
            target_directory_path.join(cache_manifest1.get_entry("out").unwrap().path);

        let output1 = std::fs::read_to_string(output1_path.clone()).unwrap();
        assert_eq!(output1.trim(), correct_output1.trim());
        assert!(output1_path.starts_with(target_directory_path.join("out_text")));

        remove_dir_all(&internal_directory_path).unwrap();
        create_dir(&internal_directory_path).unwrap();

        copy(
            resource_source_directory_path.join("a2.txt"),
            source_directory_path.join("a.txt"),
        )
        .unwrap();

        println!("Run pass #2");
        pack(
            &manifest_path,
            &cache_manifest_path,
            &config,
            &filter_registry,
        )
        .unwrap();

        let correct_output2 =
            std::fs::read_to_string(resource_output_directory_path.join("out2.txt")).unwrap();

        let cache_manifest2 = load_cache_manifest::<DummyError>(&cache_manifest_path).unwrap();
        let output2_path =
            target_directory_path.join(cache_manifest2.get_entry("out").unwrap().path);

        let output2 = std::fs::read_to_string(output2_path.clone()).unwrap();
        assert_eq!(output2.trim(), correct_output2.trim());
        assert!(output2_path.starts_with(target_directory_path.join("out_text")));

        assert_ne!(output1_path, output2_path);
    }

    #[test]
    fn test_invalid() {
        let test_directory_path = Path::new("test_files");
        let temp_directory = TempDir::new().unwrap();
        let temp_directory_path = temp_directory.path();

        let resource_source_directory_path = test_directory_path.join("source");
        let resource_manifest_directory_path = test_directory_path.join("assets_invalid.json");

        let source_directory_path = temp_directory_path.join("source");
        let internal_directory_path = temp_directory_path.join("internal");
        let target_directory_path = temp_directory_path.join("target");
        let cache_manifest_path = temp_directory_path.join("cache.json");
        let manifest_path = temp_directory_path.join("assets.json");

        create_dir(&source_directory_path).unwrap();
        create_dir(&internal_directory_path).unwrap();
        create_dir(&target_directory_path).unwrap();

        copy(
            resource_source_directory_path.join("a1.txt"),
            source_directory_path.join("a.txt"),
        )
        .unwrap();
        copy(
            resource_source_directory_path.join("b.txt"),
            source_directory_path.join("b.txt"),
        )
        .unwrap();
        copy(&resource_manifest_directory_path, &manifest_path).unwrap();

        let config = AssetConfig {
            target_directory_path: target_directory_path.clone(),
            internal_directory_path: internal_directory_path.clone(),
            source_directory_path: source_directory_path.clone(),
        };

        let mut filters_map: HashMap<String, Box<dyn AssetFilter<DummyError>>> = HashMap::new();
        filters_map.insert("TestCat".to_string(), Box::new(TestCatFilter {}));

        let filter_registry = AssetFilterRegistry::new(filters_map);

        let result = pack(
            &manifest_path,
            &cache_manifest_path,
            &config,
            &filter_registry,
        );

        if let Err(err) = result {
            match err.error_type {
                AssetErrorType::AssetNotFoundInManifestError(asset_name) => {
                    assert_eq!(asset_name, "c")
                }
                _ => panic!("{:?}", err.error_type),
            }
        } else {
            panic!();
        }
    }
}
