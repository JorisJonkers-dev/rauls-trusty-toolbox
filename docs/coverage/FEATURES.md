# Feature coverage

Generated on main by tally from `catalogue/` (docs/COVERAGE.md, "Feature catalogue"). Do not edit.

| | Built | Has a test | Tested on some OSes | Tested on all OSes | Not built yet | Needs the game to test | Not needed |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| **Total** | 175/177 | 169 | 169 | 143 | 2 | 3 | 1 |
| **Benchmarks** | 5/6 | 5 | 5 | 5 | 1 | 0 | 0 |
| raul bench (`bench.suite`) | 5/6 | 5 | 5 | 5 | 1 | 0 | 0 |
| **Command line** | 7/7 | 7 | 7 | 7 | 0 | 0 | 0 |
| The raul command (`cli.commands`) | 7/7 | 7 | 7 | 7 | 0 | 0 | 0 |
| **Contracts** | 7/7 | 7 | 7 | 7 | 0 | 0 | 0 |
| The documents raul prints (`contracts.documents`) | 7/7 | 7 | 7 | 7 | 0 | 0 | 0 |
| **Game detection and configuration** | 19/19 | 18 | 18 | 18 | 0 | 0 | 0 |
| The configuration file (`detect.config`) | 10/10 | 10 | 10 | 10 | 0 | 0 | 0 |
| Finding game copies (`detect.games`) | 9/9 | 8 | 8 | 8 | 0 | 0 | 0 |
| **What raul promises about how it works** | 12/12 | 11 | 11 | 5 | 0 | 0 | 0 |
| How raul is built (`guarantees.build`) | 12/12 | 11 | 11 | 5 | 0 | 0 | 0 |
| **The window** | 15/15 | 15 | 15 | 10 | 0 | 0 | 0 |
| The pages (`gui.pages`) | 8/8 | 8 | 8 | 4 | 0 | 0 | 0 |
| The shell (`gui.shell`) | 7/7 | 7 | 7 | 6 | 0 | 0 | 0 |
| **Distant view (LOD)** | 26/26 | 25 | 25 | 25 | 0 | 1 | 0 |
| Generating the distant view (`lod.generate`) | 13/13 | 12 | 12 | 12 | 0 | 1 | 0 |
| The LOD settings (`lod.settings`) | 9/9 | 9 | 9 | 9 | 0 | 0 | 0 |
| Knowing when the distant view is out of date (`lod.stale`) | 4/4 | 4 | 4 | 4 | 0 | 0 | 0 |
| **Load order, mods and add-ons** | 23/23 | 23 | 23 | 20 | 0 | 0 | 0 |
| Add-ons for the mods you have (`mods.addons`) | 8/8 | 8 | 8 | 7 | 0 | 0 | 0 |
| The Instance (`mods.instance`) | 2/2 | 2 | 2 | 1 | 0 | 0 | 0 |
| Mod lists (`mods.modlist`) | 8/8 | 8 | 8 | 8 | 0 | 0 | 0 |
| Sorting the load order (`mods.sort`) | 5/5 | 5 | 5 | 4 | 0 | 0 | 0 |
| **Play and pictures** | 12/12 | 10 | 10 | 7 | 0 | 1 | 0 |
| Starting the game (`play.launch`) | 8/8 | 7 | 7 | 7 | 0 | 1 | 0 |
| Pictures and benchmark runs (`play.scene`) | 4/4 | 3 | 3 | 0 | 0 | 0 | 0 |
| **Packages** | 0/1 | 0 | 0 | 0 | 1 | 0 | 1 |
| Packages (`release.packages`) | 0/1 | 0 | 0 | 0 | 1 | 0 | 1 |
| **First-run setup** | 19/19 | 19 | 19 | 19 | 0 | 0 | 0 |
| The setup journal (`setup.journal`) | 8/8 | 8 | 8 | 8 | 0 | 0 | 0 |
| The setup pipeline (`setup.pipeline`) | 11/11 | 11 | 11 | 11 | 0 | 0 | 0 |
| **Tale of Two Wastelands** | 12/12 | 11 | 11 | 3 | 0 | 1 | 0 |
| Installing Tale of Two Wastelands (`ttw.install`) | 12/12 | 11 | 11 | 3 | 0 | 1 | 0 |
| **Updating raul** | 18/18 | 18 | 18 | 17 | 0 | 0 | 0 |
| Checking for a new release (`update.check`) | 6/6 | 6 | 6 | 5 | 0 | 0 | 0 |
| Updating and going back (`update.install`) | 12/12 | 12 | 12 | 12 | 0 | 0 | 0 |

### raul bench (`bench.suite`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| The quick suite measures sorting, plugin reading and mod install | `bench.suite.quick_suite` | tested on all OSes | `crates/raul-bench/tests/bench.rs::the_quick_suite_measures_sorting_plugin_reading_and_mod_install` |
| A tool that is missing is skipped with a reason | `bench.suite.skip_reason` | tested on all OSes | `crates/raul-bench/tests/bench.rs::lod_and_frame_time_are_skipped_with_a_reason_when_their_tools_are_missing` |
| The Markdown report has a row for every result and skip | `bench.suite.markdown_report` | tested on all OSes | `crates/raul-bench/tests/bench.rs::the_markdown_report_has_a_row_for_every_result_and_every_skip` |
| A result slower than the threshold is a regression | `bench.suite.regression_threshold` | tested on all OSes | `crates/raul-bench/tests/bench.rs::a_result_slower_than_the_threshold_is_a_regression` |
| A regression warns and does not fail | `bench.suite.warns_not_fails` | tested on all OSes | `crates/raul-cli/tests/cli.rs::bench_compare_warns_but_succeeds_on_a_regression` |
| Frame-time capture along a fixed camera path | `bench.suite.frame_time` | not built yet |  |

### The raul command (`cli.commands`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| version --json names the Toolbox version | `cli.commands.version_json` | tested on all OSes | `crates/raul-cli/tests/cli.rs::version_as_json_names_the_toolbox_version` |
| version prints one line | `cli.commands.version_text` | tested on all OSes | `crates/raul-cli/tests/cli.rs::version_as_text_is_one_line_with_the_toolbox_version` |
| An unknown command exits with the usage code | `cli.commands.unknown_command` | tested on all OSes | `crates/raul-cli/tests/cli.rs::an_unknown_command_exits_with_the_usage_code` |
| A command raul does not own is run by nvmod | `cli.commands.nvmod_commands` | tested on all OSes | `crates/raul-cli/tests/cli.rs::a_command_raul_does_not_own_is_run_by_nvmod` |
| Every own command is routed to the Toolbox | `cli.commands.own_commands` | tested on all OSes | `crates/raul-cli/src/lib.rs::every_own_command_is_routed_to_the_toolbox_and_the_rest_to_nvmod` |
| The program is called raul in what nvmod prints | `cli.commands.program_name` | tested on all OSes | `crates/raul-cli/src/program_name.rs::the_program_name_is_raul_at_the_start_after_usage_and_in_backticks` |
| The support bundle holds exactly its items and hides names on request | `cli.commands.support_bundle` | tested on all OSes | `crates/raul-cli/src/support_bundle.rs::hiding_names_leaves_no_plugin_name_and_the_home_folder_is_a_tilde`, `crates/raul-cli/src/support_bundle.rs::the_zip_holds_exactly_the_items_and_is_the_same_each_time` |

### The documents raul prints (`contracts.documents`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| Every schema file equals its generated schema | `contracts.documents.schemas_equal_types` | tested on all OSes | `crates/raul-core/tests/contracts.rs::every_schema_file_equals_the_generated_schema` |
| Every fixture is valid against its schema | `contracts.documents.fixtures_valid` | tested on all OSes | `crates/raul-core/tests/contracts.rs::every_fixture_is_valid_against_its_schema` |
| Every event stream is numbered from one and ends with finished | `contracts.documents.events_numbered` | tested on all OSes | `crates/raul-core/tests/contracts.rs::every_event_stream_is_numbered_from_one_and_ends_with_finished` |
| Every snapshot equals the pinned Component's schema | `contracts.documents.component_snapshots` | tested on all OSes | `crates/raul-core/tests/contracts.rs::every_snapshot_equals_the_schema_of_the_pinned_component` |
| A document of another version or contract is not valid | `contracts.documents.other_version_invalid` | tested on all OSes | `crates/raul-core/tests/contracts.rs::a_config_document_of_another_version_or_contract_is_not_valid` |
| The TTW result carries a valid ttwright result | `contracts.documents.ttw_result` | tested on all OSes | `crates/raul-core/tests/contracts.rs::ttw_result_fixtures_read_back_and_carry_valid_ttwright_results` |
| The LOD result carries a valid lodwright result | `contracts.documents.lod_result` | tested on all OSes | `crates/raul-core/tests/contracts.rs::lod_result_fixtures_read_back_and_carry_valid_lodwright_results` |

### The configuration file (`detect.config`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| Every key reads back from its name and only the Instance is read only | `detect.config.every_key` | tested on all OSes | `crates/raul-cli/tests/config.rs::list_prints_every_key_with_null_on_a_fresh_machine`, `crates/raul-core/src/config.rs::every_key_reads_back_from_its_name_and_only_the_instance_is_read_only` |
| set then get show the paths and the derived Instance | `detect.config.set_then_get` | tested on all OSes | `crates/raul-cli/tests/config.rs::set_then_get_and_list_show_the_paths_and_the_derived_instance` |
| A folder detect does not accept is refused and nothing is written | `detect.config.refuses_bad_folder` | tested on all OSes | `crates/raul-cli/tests/config.rs::a_folder_detect_does_not_accept_is_refused_with_exit_2_and_nothing_written` |
| A relative path is stored as an absolute one | `detect.config.absolute_paths` | tested on all OSes | `crates/raul-cli/tests/config.rs::a_relative_path_is_stored_as_an_absolute_one` |
| A newer schema version is refused and the file is left alone | `detect.config.newer_schema` | tested on all OSes | `crates/raul-cli/tests/config.rs::a_newer_schema_version_is_refused_and_the_file_is_left_alone` |
| Keys this build does not know survive a set | `detect.config.unknown_keys_survive` | tested on all OSes | `crates/raul-cli/tests/config.rs::unknown_keys_survive_a_set` |
| The TTW package is judged by content, not by name | `detect.config.ttw_package_by_content` | tested on all OSes | `crates/raul-cli/tests/config.rs::the_ttw_package_is_judged_by_content_not_by_name` |
| The free space of a folder is reported | `detect.config.free_space` | tested on all OSes | `crates/raul-cli/tests/cli.rs::space_reports_the_free_space_of_a_synthetic_folder_as_json` |
| A TTW profile gets its game settings and a plain one keeps the defaults | `detect.config.ttw_game_settings` | tested on all OSes | `crates/raul-core/src/game_ini.rs::a_ttw_profile_gets_the_values_and_a_plain_one_keeps_the_defaults` |
| A setting the person made is kept | `detect.config.person_setting_kept` | tested on all OSes | `crates/raul-core/src/game_ini.rs::a_setting_the_person_made_is_kept_when_the_master_leaves` |

### Finding game copies (`detect.games`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| A New Vegas copy in a second Steam library is found | `detect.games.steam_libraries` | tested on all OSes | `crates/raul-core/tests/detect.rs::a_new_vegas_copy_in_a_second_steam_library_is_found` |
| An Epic manifest points at the install location | `detect.games.epic` | tested on all OSes | `crates/raul-core/tests/detect.rs::an_epic_manifest_points_at_its_install_location` |
| GOG copies are found | `detect.games.gog` | built |  |
| Fallout 3 GOTY is found for Tale of Two Wastelands | `detect.games.fallout3_for_ttw` | tested on all OSes | `crates/raul-core/tests/detect.rs::fallout_3_goty_is_found_for_tale_of_two_wastelands` |
| The executable version of a copy is read and judged supported or not | `detect.games.version_judged` | tested on all OSes | `crates/raul-core/tests/detect.rs::an_older_executable_version_is_judged_unsupported`, `crates/raul-core/tests/detect.rs::the_executable_version_of_a_copy_is_read_and_judged_supported` |
| Missing DLC are listed | `detect.games.missing_dlc` | tested on all OSes | `crates/raul-core/tests/detect.rs::missing_dlc_are_listed` |
| A copy reachable from two roots is reported once | `detect.games.reported_once` | tested on all OSes | `crates/raul-core/tests/detect.rs::a_copy_reachable_from_two_roots_is_reported_once` |
| A folder without the game is not a game copy | `detect.games.not_a_copy` | tested on all OSes | `crates/raul-cli/src/detect.rs::a_folder_without_a_game_has_no_copies`, `crates/raul-core/tests/detect.rs::a_folder_without_a_game_executable_is_not_a_game_copy` |
| Detect on a picked folder reports the copy as JSON | `detect.games.picked_folder` | tested on all OSes | `crates/raul-cli/tests/cli.rs::detect_on_a_picked_folder_reports_the_copy_as_json` |

### How raul is built (`guarantees.build`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| Only the runner crate starts a process | `guarantees.build.only_runner_starts_processes` | tested on all OSes | `crates/raul-runner/tests/no_command_elsewhere.rs::only_the_runner_crate_uses_process_command` |
| The crate graph keeps its layers | `guarantees.build.crate_graph` | tested on all OSes | `crates/raul-core/tests/crate_graph.rs::the_runner_and_the_supply_depend_on_no_other_toolbox_crate`, `crates/raul-core/tests/crate_graph.rs::the_view_model_depends_on_no_toolbox_crate_and_no_nvmod_crate` |
| No game data or game-style file enters the repository | `guarantees.build.no_game_files` | tested on all OSes | `crates/raul-core/tests/assets.rs::the_repository_has_no_unlisted_asset_and_no_game_style_file` |
| The window never starts a process | `guarantees.build.window_starts_no_process` | tested on all OSes | `crates/raul-gui/tests/no_process.rs::the_window_never_starts_a_process` |
| No page shows a command line | `guarantees.build.no_command_text` | tested on all OSes | `crates/raul-gui/tests/no_command_text.rs::no_page_fixture_shows_a_command_line` |
| Arguments are passed as words with no shell | `guarantees.build.no_shell` | tested on some OSes | `crates/raul-runner/src/runner.rs::arguments_are_passed_as_words_with_no_shell` |
| A started program gets a scrubbed environment | `guarantees.build.scrubbed_environment` | tested on some OSes | `crates/raul-runner/src/runner.rs::the_environment_is_scrubbed_and_extras_are_passed` |
| A program that runs too long is killed | `guarantees.build.timeout_kills` | tested on some OSes | `crates/raul-runner/src/runner.rs::a_tool_that_runs_too_long_is_killed_and_reported_as_failed` |
| A program that writes past the cap is killed | `guarantees.build.output_cap` | tested on some OSes | `crates/raul-runner/src/runner.rs::a_tool_that_writes_past_the_cap_is_killed` |
| A checksum mismatch stops the program before it starts | `guarantees.build.checksum_before_start` | tested on some OSes | `crates/raul-runner/src/runner.rs::a_checksum_mismatch_exits_3_and_the_tool_is_not_started` |
| A stop ends the whole process group politely | `guarantees.build.stop_group` | tested on some OSes | `crates/raul-runner/src/stream.rs::a_stop_ends_the_whole_group_politely` |
| raul never changes the game folder | `guarantees.build.game_folder_untouched` | built |  |

### The pages (`gui.pages`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| First run sorts the load order and sets the engine target | `gui.pages.first_run` | tested on some OSes | `crates/raul-gui/tests/first_run.rs::first_run_sorts_the_load_order_with_no_profile_action` |
| The Mods page lists, changes and removes mods | `gui.pages.mods_page` | tested on some OSes | `crates/raul-gui/tests/mods.rs::the_page_lists_changes_and_removes_mods_and_leaves_the_generated_lod_mod_to_the_lod_page` |
| The LOD page generates and finds a second run current | `gui.pages.lod_page` | tested on some OSes | `crates/raul-gui/tests/lod.rs::generate_keeps_one_enabled_mod_and_a_second_run_finds_it_current` |
| The TTW flow reviews with a dry run, then installs | `gui.pages.ttw_flow` | tested on some OSes | `crates/raul-gui/tests/ttw.rs::the_flow_reviews_with_a_dry_run_then_installs_the_mod` |
| A changed setting is saved through raul config set | `gui.pages.settings_save` | tested on all OSes | `crates/raul-gui/src/pages/settings.rs::leaving_a_changed_field_saves_it_through_raul_config_set` |
| The update button sits beside the version and checks for updates | `gui.pages.update_button` | tested on all OSes | `crates/raul-gui/src/pages/versions.rs::the_update_button_sits_beside_the_version_and_checks_for_updates` |
| A blocked Play explains itself and offers the fix | `gui.pages.play_blocked` | tested on all OSes | `crates/raul-gui/src/pages/home.rs::a_blocked_play_explains_and_its_fix_opens_the_owning_page` |
| Every action of the window is a command of the command line | `gui.pages.actions_are_commands` | tested on all OSes | `crates/raul-gui/crates/raul-ui-service/tests/parity.rs::every_action_is_a_command_of_the_real_command_line` |

### The shell (`gui.shell`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| The registry has every page in order | `gui.shell.pages_in_order` | tested on all OSes | `crates/raul-gui/src/pages/mod.rs::the_registry_has_every_page_in_order_then_the_flows` |
| Number keys open the pages in order | `gui.shell.number_keys` | tested on all OSes | `crates/raul-gui/tests/shell.rs::number_keys_open_the_pages_in_order` |
| Every page fits its window at the smallest and a large size | `gui.shell.fits_window` | tested on all OSes | `crates/raul-gui/tests/fits_window.rs::no_page_fixture_is_wider_or_taller_than_its_window` |
| Every page matches its picture | `gui.shell.snapshots` | tested on some OSes | `crates/raul-gui/tests/snapshots.rs::every_page_fixture_matches_its_snapshot` |
| Text reads on every surface | `gui.shell.theme_contrast` | tested on all OSes | `crates/raul-gui/src/theme.rs::every_text_colour_reads_on_every_surface` |
| The bundled font comes with its licence | `gui.shell.fonts_licensed` | tested on all OSes | `crates/raul-gui/src/fonts.rs::inter_is_bundled_with_its_licence` |
| Every error has a plain detail line | `gui.shell.plain_errors` | tested on all OSes | `crates/raul-gui/src/catalogue.rs::every_error_has_a_plain_detail_line` |

### Generating the distant view (`lod.generate`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| One generated LOD Mod per profile, current on a second run | `lod.generate.one_mod` | tested on all OSes | `crates/raul-cli/tests/lod_generate.rs::success_keeps_one_enabled_lod_mod_and_a_second_run_is_up_to_date` |
| A new run replaces the generated Mod | `lod.generate.replaces` | tested on all OSes | `crates/raul-cli/tests/lod_generate.rs::a_new_output_replaces_the_lod_mod_and_leaves_one` |
| Each profile has its own output folder | `lod.generate.per_profile` | tested on all OSes | `crates/raul-cli/tests/lod_generate.rs::each_profile_has_its_own_output_folder_and_the_shared_one_becomes_the_first_profiles` |
| The Mod is named for its profile and marked as generated | `lod.generate.named_and_marked` | tested on all OSes | `crates/raul-cli/tests/lod_generate.rs::the_lod_mod_is_named_for_its_profile_and_marked_as_generated` |
| A failed replacement puts the old Mod back | `lod.generate.failed_replacement` | tested on all OSes | `crates/raul-cli/tests/lod_generate.rs::a_failed_replacement_puts_the_old_mod_back` |
| A stop adds nothing and the next run resumes | `lod.generate.stop_resumes` | tested on all OSes | `crates/raul-cli/tests/lod_generate.rs::a_stop_during_the_generation_adds_nothing_and_the_next_run_resumes` |
| An output inside a git folder is refused before anything starts | `lod.generate.git_folder_refused` | tested on all OSes | `crates/raul-cli/tests/lod_generate.rs::a_generation_inside_a_git_folder_is_refused_before_anything_starts` |
| A second generate while one runs is refused | `lod.generate.second_run_refused` | tested on all OSes | `crates/raul-cli/tests/lod_generate.rs::a_second_generate_while_one_runs_is_refused_and_an_edit_waits_for_the_next_run` |
| A dry run or a bad input starts nothing | `lod.generate.dry_run` | tested on all OSes | `crates/raul-cli/tests/lod_generate.rs::a_dry_run_or_a_bad_input_starts_nothing` |
| Packing gives one archive and its stub | `lod.generate.packing` | tested on all OSes | `crates/raul-cli/tests/lod_generate.rs::packing_gives_a_mod_of_one_archive_and_its_stub_and_switching_it_either_way_marks_the_lod_out_of_date` |
| The generator's refusals keep their codes and leave the Instance unchanged | `lod.generate.refusals_exit_4` | tested on all OSes | `crates/raul-cli/tests/lod_generate.rs::lodwright_refusals_keep_their_codes_and_exit_4_and_leave_the_instance_unchanged` |
| A run that outlasts its timeout is stopped and leaves the Mod alone | `lod.generate.timeout` | tested on all OSes | `crates/raul-cli/tests/lod_stop.rs::a_run_that_outlasts_its_timeout_is_stopped_and_leaves_the_mod_alone` |
| A real generation for the player's load order | `lod.generate.real_generation` | needs the game to test |  |

### The LOD settings (`lod.settings`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| Every option has a default and plain text | `lod.settings.defaults` | tested on all OSes | `crates/raul-drivers/src/lod/settings.rs::every_option_has_a_default_and_catalogue_text`, `crates/raul-drivers/src/lod/settings.rs::the_defaults_are_the_generators_level_defaults` |
| A set value reads back and the file holds only it | `lod.settings.set_reads_back` | tested on all OSes | `crates/raul-drivers/src/lod/settings.rs::a_set_value_reads_back_and_the_file_holds_only_it` |
| A refused value leaves the file as it was | `lod.settings.refused_value` | tested on all OSes | `crates/raul-drivers/src/lod/settings.rs::a_refused_value_leaves_the_file_as_it_was` |
| Restore defaults can be undone | `lod.settings.restore_and_undo` | tested on all OSes | `crates/raul-drivers/src/lod/settings.rs::restore_defaults_deletes_the_file_when_nothing_is_left_and_undo_puts_it_back` |
| A key that is not an option is refused by name | `lod.settings.unknown_key` | tested on all OSes | `crates/raul-drivers/src/lod/settings.rs::a_key_that_is_not_an_option_is_refused_by_name` |
| The ranges mirror the generator's own check | `lod.settings.ranges` | tested on all OSes | `crates/raul-drivers/src/lod/settings.rs::the_ranges_mirror_the_generators_own_check` |
| A far-shape rule applies only when its world is present and requested | `lod.settings.far_shape_rules` | tested on all OSes | `crates/raul-drivers/src/lod/far_shapes.rs::a_rule_applies_only_when_its_world_is_present_and_requested` |
| An inclusion rule applies only when its plugin is in the load order | `lod.settings.include_rules` | tested on all OSes | `crates/raul-drivers/src/lod/include_rules.rs::a_rule_applies_only_when_its_plugin_is_in_the_load_order` |
| A change during a run applies to the next run | `lod.settings.applies_next_run` | tested on all OSes | `crates/raul-cli/tests/lod_settings.rs::a_change_while_a_run_is_going_is_saved_and_says_it_applies_to_the_next_run` |

### Knowing when the distant view is out of date (`lod.stale`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| A settings change marks the LOD out of date until it runs again | `lod.stale.settings_change` | tested on all OSes | `crates/raul-cli/tests/lod_generate.rs::a_run_records_what_it_was_built_from_and_a_settings_change_marks_the_lod_out_of_date_until_it_runs_again` |
| A settings change must regenerate | `lod.stale.must_regenerate` | tested on all OSes | `crates/raul-drivers/src/lod/stale.rs::a_settings_change_must_regenerate` |
| Nothing changed means current | `lod.stale.nothing_changed` | tested on all OSes | `crates/raul-drivers/src/lod/stale.rs::nothing_changed_is_current` |
| A reorder only may regenerate | `lod.stale.reorder_may` | tested on all OSes | `crates/raul-drivers/src/lod/stale.rs::a_reorder_may` |

### Add-ons for the mods you have (`mods.addons`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| Every add-on in the list is complete | `mods.addons.list_complete` | tested on all OSes | `crates/raul-drivers/src/addons.rs::the_list_is_valid_and_every_add_on_is_complete` |
| The small lamp glow is the approved size | `mods.addons.lamp_glow_size` | tested on all OSes | `crates/raul-drivers/src/addons.rs::the_small_lamp_glow_is_the_approved_size` |
| The add-on goes right above its mod, its plugin after the mod's plugin | `mods.addons.placed_above_trigger` | tested on all OSes | `crates/raul-cli/tests/addons.rs::the_add_on_goes_right_above_street_lights_with_its_plugin_after_the_trigger` |
| A turned-off add-on stays off and a removed one is not added back | `mods.addons.off_stays_off` | tested on all OSes | `crates/raul-cli/tests/addons.rs::a_turned_off_add_on_stays_off_and_a_removed_one_is_not_added_back` |
| A profile without the mod is never touched | `mods.addons.untouched_without_trigger` | tested on all OSes | `crates/raul-cli/tests/addons.rs::a_profile_without_street_lights_is_never_touched` |
| The check before a game start never stops it | `mods.addons.check_never_stops_start` | tested on all OSes | `crates/raul-cli/tests/addons.rs::the_check_before_a_game_start_never_stops_it` |
| Setup adds the add-on and says so | `mods.addons.added_by_setup` | tested on all OSes | `crates/raul-core/tests/setup.rs::a_setup_with_street_lights_adds_the_lamp_glow_add_on_and_says_so` |
| The mesh is made from numbers, not from a shipped file | `mods.addons.mesh_from_numbers` | tested on some OSes | `crates/raul-drivers/src/addons/nif.rs::the_shipped_numbers_give_the_mesh_that_was_checked_in_game` |

### The Instance (`mods.instance`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| Two holders exclude each other until the first lets go | `mods.instance.lock_excludes` | tested on all OSes | `crates/raul-core/src/instance_lock.rs::two_holders_exclude_each_other_until_the_first_lets_go` |
| A lock left by a crashed process is taken over | `mods.instance.crashed_lock_taken_over` | tested on some OSes | `crates/raul-core/src/instance_lock.rs::a_lock_left_by_a_process_that_crashed_is_taken_over` |

### Mod lists (`mods.modlist`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| A Mod list is read and its Mods keep their order | `mods.modlist.list_read` | tested on all OSes | `crates/raul-modlist/tests/parse.rs::a_list_is_read_with_its_mods_in_order` |
| A list that breaks a rule is refused, with every broken rule named | `mods.modlist.broken_list_refused` | tested on all OSes | `crates/raul-modlist/tests/parse.rs::a_list_that_breaks_several_rules_names_each_one` |
| A list of another schema version is refused with a plain line | `mods.modlist.other_schema_refused` | tested on all OSes | `crates/raul-modlist/tests/parse.rs::another_schema_version_is_refused_with_a_plain_line` |
| Beautiful is carried by raul, passes the rules and holds the Lucky 38 Mods | `mods.modlist.beautiful_bundled` | tested on all OSes | `crates/raul-modlist/tests/beautiful.rs::beautiful_holds_both_lucky_38_mods_at_their_nexus_files`, `crates/raul-modlist/tests/beautiful.rs::every_bundled_list_passes_the_rules` |
| The plan says which Mods the Instance has, lacks and the list switches off | `mods.modlist.plan_present_missing` | tested on all OSes | `crates/raul-modlist/tests/plan.rs::present_missing_and_switched_off_mods_are_told_apart` |
| Every missing Mod comes with where to get it: a Nexus page, another link, or what to do by hand | `mods.modlist.plan_links` | tested on all OSes | `crates/raul-modlist/tests/plan.rs::a_nexus_file_with_an_id_links_the_file_and_one_without_links_the_files_tab`, `crates/raul-modlist/tests/plan.rs::other_links_and_hand_work_are_told_plainly` |
| A file id the list lacks is asked of a resolver, and the plan works without one | `mods.modlist.plan_resolver` | tested on all OSes | `crates/raul-modlist/tests/plan.rs::a_resolver_supplies_the_file_id_the_list_lacks_and_is_not_asked_when_the_list_has_it` |
| raul modlist plan shows the plan for an Instance and a Profile, as text or JSON, and writes nothing | `mods.modlist.command_plan` | tested on all OSes | `crates/raul-cli/tests/modlist.rs::the_plan_says_what_the_profile_has_and_where_to_get_the_rest_in_install_order`, `crates/raul-cli/tests/modlist.rs::the_plan_writes_nothing` |

### Sorting the load order (`mods.sort`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| The masterlist is pinned by checksum | `mods.sort.masterlist_pinned` | tested on all OSes | `crates/raul-core/src/pinned.rs::the_shipped_lock_pins_the_masterlist_by_checksum` |
| A file with a wrong checksum is never stored | `mods.sort.hash_checked` | tested on all OSes | `crates/raul-core/src/pinned.rs::a_wrong_checksum_stores_nothing_and_says_both_hashes` |
| --fetch-masterlist becomes a masterlist file for nvmod's sort | `mods.sort.fetch_option` | tested on all OSes | `crates/raul-cli/src/sort.rs::a_fetch_becomes_a_masterlist_file_and_the_source_options_are_taken_out` |
| A missing masterlist stops the sort step | `mods.sort.missing_masterlist` | tested on all OSes | `crates/raul-core/tests/setup.rs::a_missing_masterlist_stops_the_sort_step` |
| A sort is previewed, applied and undone byte for byte | `mods.sort.preview_apply_undo` | tested on some OSes | `crates/raul-gui/tests/load_order.rs::sort_preview_apply_then_undo_restores_the_lists_byte_for_byte` |

### Starting the game (`play.launch`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| The game starts through the launcher in the run folder and its output is logged | `play.launch.run_folder` | tested on all OSes | `crates/raul-cli/tests/run.rs::the_game_starts_through_the_launcher_in_the_run_folder_and_its_output_is_logged` |
| A game that ends badly ends the run badly | `play.launch.bad_exit` | tested on all OSes | `crates/raul-cli/tests/run.rs::a_game_that_ends_badly_ends_the_run_badly` |
| A missing viewer is refused and nothing starts | `play.launch.missing_viewer` | tested on all OSes | `crates/raul-cli/tests/run.rs::a_viewer_that_does_not_exist_is_refused_and_nothing_starts` |
| Exit codes have plain words | `play.launch.exit_words` | tested on all OSes | `crates/raul-cli/src/play.rs::exit_codes_have_plain_words_and_a_finished_game_has_none` |
| Stop asks every run folder or the named one | `play.launch.stop` | tested on all OSes | `crates/raul-cli/src/play.rs::stop_asks_every_run_folder_or_the_named_one` |
| A TTW profile gets its game settings before the game starts | `play.launch.ttw_settings` | tested on all OSes | `crates/raul-cli/src/play.rs::a_profile_with_tale_of_two_wastelands_gets_its_game_settings_before_the_game_starts` |
| The nv-rs target writes the engine's capabilities and vanilla writes none | `play.launch.engine_target` | tested on all OSes | `crates/raul-cli/tests/target.rs::the_nv_rs_target_writes_the_capabilities_the_engine_provided_and_vanilla_writes_none` |
| Playing the game on the built-in engine | `play.launch.real_game` | needs the game to test |  |

### Pictures and benchmark runs (`play.scene`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| A profile with a generated LOD gets the pair and the difference | `play.scene.lod_pair` | tested on some OSes | `crates/raul-cli/tests/scene.rs::a_profile_with_a_generated_lod_mod_gets_the_pair_and_the_difference` |
| A soak and a bench start once over the staging tree | `play.scene.soak_and_bench` | tested on some OSes | `crates/raul-cli/tests/scene.rs::a_soak_and_a_bench_start_once_over_the_staging_tree_and_the_bench_takes_the_lock` |
| Two bench summaries are compared | `play.scene.compare` | built |  |
| The place files raul carries are written and never overwritten | `play.scene.place_files` | tested on some OSes | `crates/raul-cli/tests/scene.rs::the_place_files_raul_carries_are_written_and_never_overwritten` |

### Packages (`release.packages`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| Signed Windows and macOS packages | `release.packages.signed` | not built yet |  |
| A full run on real Windows and macOS computers | `release.packages.real_hardware_check` | not needed |  |

### The setup journal (`setup.journal`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| The journal is written after each finished step | `setup.journal.written_each_step` | tested on all OSes | `crates/raul-core/tests/journal.rs::a_run_writes_the_journal_after_each_step_and_marks_it_complete` |
| An interrupted setup resumes at the first unfinished step | `setup.journal.resume` | tested on all OSes | `crates/raul-core/tests/journal.rs::an_interrupted_setup_resumes_at_the_first_unfinished_step` |
| A restart forgets the finished steps | `setup.journal.restart` | tested on all OSes | `crates/raul-core/tests/journal.rs::restart_forgets_the_finished_steps_and_keeps_the_file` |
| Remove deletes the journal and the next run starts afresh | `setup.journal.remove` | tested on all OSes | `crates/raul-core/tests/journal.rs::remove_deletes_the_journal_and_a_new_run_starts_from_the_first_step` |
| An unfinished nvmod setup is taken over once and resumes where it stopped | `setup.journal.nvmod_takeover` | tested on all OSes | `crates/raul-core/tests/journal.rs::an_unfinished_nvmod_setup_is_taken_over_once_and_resumes_at_the_same_step` |
| A journal of an older version is migrated after a copy is kept | `setup.journal.older_migrated` | tested on all OSes | `crates/raul-core/tests/journal.rs::a_journal_of_an_older_version_is_migrated_after_a_copy_is_kept` |
| A journal of a newer version is refused and left as it is | `setup.journal.newer_refused` | tested on all OSes | `crates/raul-core/tests/journal.rs::a_journal_of_a_newer_version_is_refused_and_left_as_it_is` |
| A half-written file never replaces the journal | `setup.journal.atomic_write` | tested on all OSes | `crates/raul-core/tests/journal.rs::a_partial_file_from_an_interrupted_write_does_not_replace_the_journal` |

### The setup pipeline (`setup.pipeline`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| A fresh setup runs every planned step in the order nvmod's setup used | `setup.pipeline.fresh_order` | tested on all OSes | `crates/raul-core/src/setup/plan.rs::a_fresh_instance_with_everything_has_the_order_of_nvmod_setup`, `crates/raul-core/tests/setup.rs::a_fresh_setup_runs_every_planned_step_in_order_and_reports_each` |
| The smallest plan is init, profile, stage, sort, verify | `setup.pipeline.smallest_plan` | tested on all OSes | `crates/raul-core/src/setup/plan.rs::the_smallest_plan_is_init_profile_stage_sort_verify` |
| An imported Instance keeps its load order and is stamped | `setup.pipeline.imported_keeps_order` | tested on all OSes | `crates/raul-core/tests/setup.rs::an_imported_instance_keeps_its_order_and_stamps_it` |
| A dry run plans and writes nothing | `setup.pipeline.dry_run` | tested on all OSes | `crates/raul-core/tests/setup_command.rs::a_dry_run_plans_and_writes_nothing` |
| A failing step stops the run with a typed error and no command line | `setup.pipeline.typed_failure` | tested on all OSes | `crates/raul-core/tests/setup.rs::a_failing_step_stops_the_run_with_its_typed_error_and_no_command_line` |
| A failing step removes what the run made | `setup.pipeline.failure_cleans_up` | tested on all OSes | `crates/raul-core/tests/setup_command.rs::a_failing_step_removes_what_the_run_made` |
| A stop request ends the run at the next step | `setup.pipeline.stop_request` | tested on all OSes | `crates/raul-core/tests/setup.rs::a_stop_asked_for_during_a_step_ends_the_run_at_the_next_step` |
| A RAR archive is refused in plain words and nothing is installed | `setup.pipeline.rar_refused` | tested on all OSes | `crates/raul-core/tests/setup.rs::a_rar_archive_is_refused_with_a_plain_message_and_nothing_is_installed` |
| The built-in engine's capabilities are written for the doctor | `setup.pipeline.engine_capabilities` | tested on all OSes | `crates/raul-core/tests/setup.rs::the_nv_rs_target_writes_the_linked_engines_capabilities_and_the_doctor_finds_them` |
| raul setup answers the transcripts nvmod's setup recorded | `setup.pipeline.cli_transcripts` | tested on all OSes | `crates/raul-cli/tests/setup.rs::raul_setup_answers_the_transcripts_nvmod_recorded` |
| The setup options make a typed request | `setup.pipeline.typed_request` | tested on all OSes | `crates/raul-core/src/setup/args.rs::the_options_of_a_setup_make_a_typed_request` |

### Installing Tale of Two Wastelands (`ttw.install`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| A file that starts with BSA is the package whatever it is called | `ttw.install.package_by_content` | tested on all OSes | `crates/raul-drivers/src/ttw/package.rs::a_file_that_starts_with_bsa_is_the_package_whatever_it_is_called` |
| The download archive is accepted and its package unpacked | `ttw.install.download_archive` | tested on some OSes | `crates/raul-cli/tests/ttw.rs::the_download_archive_is_accepted_and_its_package_is_unpacked_into_the_work_folder_and_removed` |
| A patch archive is refused before anything is written | `ttw.install.patch_archive_refused` | tested on some OSes | `crates/raul-cli/tests/ttw.rs::a_patch_archive_is_refused_with_exit_2_before_anything_is_written` |
| Too little space refuses before anything is unpacked | `ttw.install.space_check` | tested on all OSes | `crates/raul-drivers/src/ttw/driver.rs::too_little_space_refuses_before_anything_is_unpacked_and_says_how_much_is_missing` |
| Success adds one Mod and enables its plugins | `ttw.install.adds_one_mod` | tested on some OSes | `crates/raul-cli/tests/ttw.rs::success_adds_one_mod_and_enables_its_plugins` |
| Every failure maps to its exit code and leaves the Instance unchanged | `ttw.install.exit_codes` | tested on some OSes | `crates/raul-cli/tests/ttw.rs::every_failure_maps_to_its_exit_code_and_leaves_the_instance_unchanged` |
| A cancelled install resumes | `ttw.install.resume` | tested on some OSes | `crates/raul-cli/tests/ttw.rs::resume_is_passed_through_and_continues_a_cancelled_install` |
| A dry run reads and checks and writes nothing | `ttw.install.dry_run` | tested on some OSes | `crates/raul-cli/tests/ttw.rs::a_dry_run_reads_and_checks_and_writes_nothing` |
| A stop ends the install with nothing added | `ttw.install.stop` | tested on some OSes | `crates/raul-cli/tests/ttw.rs::sigint_stops_ttwright_and_exits_4_with_nothing_added` |
| The inputs come from the configuration when no flag names them | `ttw.install.inputs_from_config` | tested on some OSes | `crates/raul-cli/tests/ttw.rs::the_inputs_come_from_raul_config_when_no_flag_names_them` |
| The new plugins are in a valid order after the sort | `ttw.install.plugins_sorted` | tested on all OSes | `crates/raul-drivers/src/ttw/driver.rs::the_new_plugins_are_in_a_valid_order_after_the_sort` |
| A real install from the player's own packages and games | `ttw.install.real_install` | needs the game to test |  |

### Checking for a new release (`update.check`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| Update checks are off until the person turns them on | `update.check.off_by_default` | tested on all OSes | `crates/raul-core/tests/update.rs::update_checks_are_off_until_the_person_turns_them_on` |
| A newer release is offered | `update.check.newer_offered` | tested on all OSes | `crates/raul-core/tests/update.rs::a_newer_release_is_offered` |
| Drafts and pre-releases are never offered | `update.check.no_drafts` | tested on all OSes | `crates/raul-core/tests/update.rs::drafts_and_prereleases_are_never_offered` |
| The check asks for HTTPS only and ignores the person's curlrc | `update.check.https_only` | tested on all OSes | `crates/raul-core/tests/update.rs::curl_is_asked_for_https_only_without_the_persons_curlrc` |
| A failing check says why | `update.check.failure_says_why` | tested on some OSes | `crates/raul-cli/tests/cli.rs::a_failing_update_check_says_why` |
| Turning checks on is remembered | `update.check.remembered` | tested on all OSes | `crates/raul-cli/tests/cli.rs::turning_update_checks_on_is_remembered` |

### Updating and going back (`update.install`)

| Feature | Id | State | Tests |
| --- | --- | --- | --- |
| The pinned release's asset downloads and verifies | `update.install.download_verified` | tested on all OSes | `crates/raul-core/src/selfupdate/verify.rs::a_matching_asset_of_the_pinned_release_downloads_and_verifies` |
| A damaged download is refused and removed | `update.install.damaged_refused` | tested on all OSes | `crates/raul-core/src/selfupdate/verify.rs::a_damaged_download_is_refused_and_removed` |
| An older or equal version is refused unless asked for | `update.install.older_refused` | tested on all OSes | `crates/raul-core/src/selfupdate/verify.rs::an_older_or_equal_version_is_refused_unless_asked_for` |
| Paths that leave the folder are refused | `update.install.archive_paths` | tested on all OSes | `crates/raul-core/src/selfupdate/extract.rs::paths_that_leave_the_folder_are_refused_in_both_formats` |
| An update replaces the files and keeps the previous version | `update.install.keeps_previous` | tested on all OSes | `crates/raul-core/src/selfupdate/flow.rs::download_then_install_replaces_the_files_and_keeps_the_previous_version` |
| A new version that fails its check is taken out again | `update.install.failed_check_rolls_back` | tested on all OSes | `crates/raul-core/src/selfupdate/flow.rs::a_new_version_that_fails_its_check_is_taken_out_again` |
| The first start confirms a working new version | `update.install.first_start_confirms` | tested on all OSes | `crates/raul-core/src/selfupdate/install.rs::the_first_start_confirms_a_working_new_version` |
| A crash during the swap is repaired to the old version | `update.install.crash_repair` | tested on all OSes | `crates/raul-core/src/selfupdate/swap.rs::a_crash_after_any_number_of_renames_is_repaired_to_all_old` |
| A rollback brings back every old file | `update.install.rollback` | tested on all OSes | `crates/raul-core/src/selfupdate/swap.rs::a_rollback_brings_back_every_old_file_and_removes_the_added_one` |
| A development build says it cannot update itself | `update.install.development_build` | tested on all OSes | `crates/raul-cli/tests/update_flow.rs::status_says_a_development_build_cannot_update_itself_and_fetches_nothing` |
| An AppImage is staged as its one file and swapped | `update.install.appimage` | tested on all OSes | `crates/raul-core/src/selfupdate/flow.rs::an_appimage_is_staged_as_its_one_file_and_swapped` |
| An install that cannot update says why in plain words | `update.install.plain_refusal` | tested on all OSes | `crates/raul-core/src/selfupdate/flow.rs::an_install_that_cannot_be_updated_says_why_in_plain_words_and_asks_for_nothing` |
