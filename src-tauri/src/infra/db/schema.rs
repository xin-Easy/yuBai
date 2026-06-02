diesel::table! {
    browser_profiles (profile_id) {
        profile_id -> Text,
        profile_name -> Text,
        user_data_dir -> Text,
        core_id -> Text,
        fingerprint_args -> Text,
        fingerprint_json -> Text,
        proxy_id -> Text,
        proxy_config -> Text,
        launch_args -> Text,
        tags -> Text,
        keywords -> Text,
        group_id -> Nullable<Text>,
        proxy_bind_source_id -> Nullable<Text>,
        proxy_bind_source_url -> Nullable<Text>,
        proxy_bind_name -> Nullable<Text>,
        proxy_bind_updated_at -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    browser_proxies (proxy_id) {
        proxy_id -> Text,
        proxy_name -> Text,
        proxy_config -> Text,
        dns_servers -> Nullable<Text>,
        sort_order -> Integer,
        group_name -> Nullable<Text>,
        last_latency_ms -> BigInt,
        last_test_ok -> Integer,
        last_tested_at -> Nullable<Text>,
        last_ip_health_json -> Text,
        source_id -> Nullable<Text>,
        source_url -> Nullable<Text>,
        source_name_prefix -> Nullable<Text>,
        source_auto_refresh -> Integer,
        source_refresh_interval_m -> Integer,
        source_last_refresh_at -> Nullable<Text>,
        created_at -> Text,
    }
}

diesel::table! {
    browser_cores (core_id) {
        core_id -> Text,
        core_name -> Text,
        core_path -> Text,
        is_default -> Integer,
        sort_order -> Integer,
        created_at -> Text,
    }
}

diesel::table! {
    browser_bookmarks (id) {
        id -> Integer,
        name -> Text,
        url -> Text,
        open_on_start -> Integer,
        sort_order -> Integer,
    }
}

diesel::table! {
    browser_groups (group_id) {
        group_id -> Text,
        group_name -> Text,
        parent_id -> Nullable<Text>,
        sort_order -> Integer,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    browser_profiles,
    browser_proxies,
    browser_cores,
    browser_bookmarks,
    browser_groups,
);
