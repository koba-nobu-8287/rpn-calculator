# PC版 RPN電卓　製品受入試験対応表

2026-11-22。基準は [develop `37ea2ab`](https://github.com/koba-nobu-8287/rpn-calculator/commit/37ea2abe8c383da14d118ea1192228290eea98cd)。以下はそのソースに本変更の試験を追加した作業版の結果。

| 仕様 | 製品受入ID数 | 対象 |
|---|---:|---|
| TEST | 69 | Word、スタック、演算、フラグ、入力、CLIプロセス |
| SI-T | 76 | スタック・入力状態の遷移 |
| IA-L / IA-C / IA-G | 22 / 16 / 12 | CLI入力、Core公開API、GUI相当のキー入力 |
| 合計 | 195 | 下表の全IDに実行済みテストを対応付けた |

Windows、Rust 1.94.1。Debugの `cargo test --workspace --offline` とReleaseの `cargo test --workspace --release --offline` は各167件成功、失敗・無視0件。`cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --offline -- -D warnings` も成功。ケースは仕様ID単位、実行件数はRustのテスト関数単位で数えた。複数の入力ベクトルを含む関数、複数層の試験を合わせて確認するIDがある。

[全体設計の試験一覧](../rpn-calc-detailed-design.md)、[スタック・入力試験の初期条件と期待値](../rpn-calc-stack-input-test-cases.md)、[結合試験仕様](13-integration-and-compatibility.md)を基準とした。SI-Tのスタック表記はX/Y/Z/Tの順。SI-Tのリンク先には初期状態、入力、期待状態が記載されている。個別に再実行するには、リンクに示した関数名を `cargo test --workspace <関数名>` に渡す。

再点検では、TEST-048の末尾命令Push(9)とIA-C013のPush(7)をそれぞれ実行した。TEST-050はUnsigned・Twosの全8操作でGの両初期値を照合した。TEST-043はW1/3/4/8/13/63/64、全基数、F3の両値に加えてFloatの往復変換を検査した。SI-T011/025/026では、Cの初期値0/1を変えてABS・CHS後も同じ値になることを確認した。

## 全体設計

| ID | 確認内容 | 対応するテスト関数 |
|---|---|---|
| TEST-001 | WordSizeの値域。1..64は成功、0/65はInvalidWordSize | [`word_widths_and_signed_interpretations`](../../crates/rpn-core/tests/core.rs) |
| TEST-002 | 幅マスク。1/FF/FFFF/FFFFFFFF/FFFFFFFFFFFFFFFF、panicなし | [`word_widths_and_signed_interpretations`](../../crates/rpn-core/tests/core.rs) |
| TEST-003 | 明示マスク。bits=A5 | [`documented_word_and_stack_boundaries`](../../crates/rpn-core/tests/core.rs) |
| TEST-004 | 2補数解釈。数学値−128/−1/127 | [`documented_word_and_stack_boundaries`](../../crates/rpn-core/tests/core.rs) |
| TEST-005 | Unsigned最大値。to_signed=18446744073709551615 | [`word_widths_and_signed_interpretations`](../../crates/rpn-core/tests/core.rs) |
| TEST-006 | 1補数−0保持。rawFF、is_negative=true、is_negative_zero=true、数学値0 | [`word_widths_and_signed_interpretations`](../../crates/rpn-core/tests/core.rs) |
| TEST-007 | W1各形式。Unsigned=0/1、2補数=0/−1、1補数=+0/−0 | [`documented_word_and_stack_boundaries`](../../crates/rpn-core/tests/core.rs) |
| TEST-008 | 安全な生成。境界内成功、外側IntegerOutOfRange | [`test_008_exact_integer_bounds_for_every_width_and_sign_mode`](../../crates/rpn-core/src/word.rs) |
| TEST-009 | 他幅のWord取込み。IntegerOutOfRange、全状態保持 | [`errors_have_priority_and_single_commands_are_atomic`](../../crates/rpn-core/tests/core.rs) |
| TEST-010 | Lift/Drop。Lift=[1,1,2,3]、Drop=[2,3,4,4] | [`direct_stack_operations_preserve_history`](../../crates/rpn-core/src/stack.rs) |
| TEST-011 | Swap/循環。Swap=[2,1,3,4]、Up=[4,1,2,3]、Down=[2,3,4,1] | [`direct_stack_operations_preserve_history`](../../crates/rpn-core/src/stack.rs) |
| TEST-012 | LAST X保存。L=3、4段保持 | [`direct_stack_operations_preserve_history`](../../crates/rpn-core/src/stack.rs) |
| TEST-013 | 固定段数。X=0、underflowなし | [`documented_word_and_stack_boundaries`](../../crates/rpn-core/tests/core.rs) |
| TEST-014 | 回転逆操作。4段が元通り、L不変 | [`direct_stack_operations_preserve_history`](../../crates/rpn-core/src/stack.rs) |
| TEST-015 | Unsigned ADD。X=0,C=1,G=1 | [`eight_bit_arithmetic_result_flags_and_history`](../../crates/rpn-core/tests/core.rs)<br>[`word_boundary_addition_matches_signed_range_and_raw_carry`](../../crates/rpn-core/tests/core.rs) |
| TEST-016 | 符号付きoverflow。X=80,C=0,G=1 | [`eight_bit_arithmetic_result_flags_and_history`](../../crates/rpn-core/tests/core.rs)<br>[`word_boundary_addition_matches_signed_range_and_raw_carry`](../../crates/rpn-core/tests/core.rs) |
| TEST-017 | 負数加算。X=FE,C=1,G=0 | [`eight_bit_arithmetic_result_flags_and_history`](../../crates/rpn-core/tests/core.rs)<br>[`word_boundary_addition_matches_signed_range_and_raw_carry`](../../crates/rpn-core/tests/core.rs) |
| TEST-018 | 1補数循環Carry。X=2,C=1,G=0（SRC-03例との照合） | [`exhaustive_small_integer_binary_operations`](../../crates/rpn-core/tests/core.rs)<br>[`word_boundary_addition_matches_signed_range_and_raw_carry`](../../crates/rpn-core/tests/core.rs) |
| TEST-019 | Borrow。X=F,C=1,G=1 | [`exhaustive_small_integer_binary_operations`](../../crates/rpn-core/tests/core.rs) |
| TEST-020 | リセット後の入力と基本加算。[8,0,0,0],L=3,Idle/E | [`decimal_stack_sequences_from_reset`](../../crates/rpn-cli/src/line/tests.rs) |
| TEST-021 | 連続ENTER。[8,5,0,0],L=3,Idle/E | [`decimal_stack_sequences_from_reset`](../../crates/rpn-cli/src/line/tests.rs) |
| TEST-022 | CLx後上書き。[3,5,0,0],L=0,Editing/D | [`decimal_stack_sequences_from_reset`](../../crates/rpn-cli/src/line/tests.rs) |
| TEST-023 | 演算後自動Lift。X=16、L=2、Idle/E | [`decimal_stack_sequences_from_reset`](../../crates/rpn-cli/src/line/tests.rs) |
| TEST-024 | LAST X呼出し。[3,8,0,0],L=3,Idle/E | [`decimal_stack_sequences_from_reset`](../../crates/rpn-cli/src/line/tests.rs) |
| TEST-025 | 4段循環。[3,2,1,4]、L=0 | [`decimal_stack_sequences_from_reset`](../../crates/rpn-cli/src/line/tests.rs) |
| TEST-026 | 基数中立。X=8。基数変更前後Lift=D | [`decimal_stack_sequences_from_reset`](../../crates/rpn-cli/src/line/tests.rs)<br>[`integer_edit_completion_and_radix_neutrality`](../../crates/rpn-cli/src/line/tests.rs) |
| TEST-027 | 演算後CLx。X=7、Y=0、L=3、Editing/D | [`decimal_stack_sequences_from_reset`](../../crates/rpn-cli/src/line/tests.rs) |
| TEST-028 | Push連続。X=8、Idle/E。Pushごとに確定 | [`decimal_stack_sequences_from_reset`](../../crates/rpn-cli/src/line/tests.rs) |
| TEST-029 | EditingとPushの混在。InputInProgress、buffer/stack/Lift保持 | [`integer_edit_completion_and_radix_neutrality`](../../crates/rpn-cli/src/line/tests.rs) |
| TEST-030 | STO/RCL。X=5、R0=5、L=0、Idle/E | [`decimal_stack_sequences_from_reset`](../../crates/rpn-cli/src/line/tests.rs) |
| TEST-031 | レジスタ境界。15成功、16 InvalidRegister、状態保持 | [`stack_moves_and_entry_lift`](../../crates/rpn-core/tests/core.rs)<br>[`documented_word_and_stack_boundaries`](../../crates/rpn-core/tests/core.rs) |
| TEST-032 | 不正数字。InvalidDigit、不正開始でLiftしない | [`si_t045_invalid_first_digit`](../../crates/rpn-input/tests/stack_input_boundaries.rs)<br>[`invalid_tokens`](../../crates/rpn-cli/src/literal/tests.rs) |
| TEST-033 | 入力確定失敗の原子性。IncompleteEntry、buffer=1eと全値保持 | [`editing_rejections_preserve_the_common_fixture`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| TEST-034 | 整数編集中CHS。X=-5、L=5、buffer=None、Idle/E。続く2はLiftして[X=2,Y=-5] | [`integer_edit_completion_and_radix_neutrality`](../../crates/rpn-cli/src/line/tests.rs) |
| TEST-035 | Idle CHS。Integer X=-5,L=5、Float X=-5,L=7、両方Idle/E | [`signed_history_and_exponent_entry`](../../crates/rpn-cli/src/line/tests.rs) |
| TEST-036 | Backspace。buffer=0、X=0、Editing、追加Liftなし | [`decimal_stack_sequences_from_reset`](../../crates/rpn-cli/src/line/tests.rs) |
| TEST-037 | buffer上限。257文字目InputTooLong、256文字保持 | [`si_t051_accept_256th_digit`](../../crates/rpn-input/tests/stack_input_boundaries.rs)<br>[`si_t052_reject_257th_digit`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| TEST-038 | 指数入力。X=0.015、Y=0.015、Idle/D | [`signed_history_and_exponent_entry`](../../crates/rpn-cli/src/line/tests.rs) |
| TEST-039 | 幅拡大。各スタックraw00FF。LAST X/記憶領域はゼロ拡張、互換試験対象外 | [`widen_twos_words_without_sign_extension`](../../crates/rpn-core/tests/core.rs) |
| TEST-040 | 幅縮小。スタックraw34。元幅へ戻しても1234を復元しない。レジスタ番号と個数を保持 | [`si_t053_resize_all_words`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| TEST-041 | 補数変更。全raw保持、解釈はUnsigned255/ones負0/twos負1、flags/Lift保持、LAST X履歴保存なし | [`test_041_sign_transitions_preserve_raw_state_at_both_lift_settings`](../../crates/rpn-core/tests/core.rs) |
| TEST-042 | 整数→Float情報欠落。SetNumericMode(Float)はIntegerOutOfRange、全状態保持 | [`documented_word_and_stack_boundaries`](../../crates/rpn-core/tests/core.rs) |
| TEST-043 | 表示。DEC=-1/255/-0、HEX=1/01。端数幅・Floatの受入例は単位06参照 | [`sign_modes_control_decimal_display_and_zero_arithmetic`](../../crates/rpn-cli/tests/process.rs)<br>[`integer_format_width_radix_and_padding_matrix`](../../crates/rpn-core/tests/core.rs)<br>[`formatting_uses_raw_nondecimal_and_keeps_negative_zero`](../../crates/rpn-core/tests/core.rs) |
| TEST-044 | MULのKeep。X=6、Cは元の値、G=0 | [`eight_bit_arithmetic_result_flags_and_history`](../../crates/rpn-core/tests/core.rs) |
| TEST-045 | DIVとRMD。DIV=-2,C=1。RMD=-1、符号はSRC-07。RMDはC/G保持 | [`eight_bit_arithmetic_result_flags_and_history`](../../crates/rpn-core/tests/core.rs) |
| TEST-046 | 最小値/-1。DIV X=80,C=0,G=1、panicなし | [`eight_bit_arithmetic_result_flags_and_history`](../../crates/rpn-core/tests/core.rs) |
| TEST-047 | ゼロ除算の全状態保持。DivisionByZero、全フィールド完全一致 | [`editing_rejections_preserve_the_common_fixture`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| TEST-048 | 逐次バッチの失敗。5と0の成功は保持、Divide不変、9未実行 | [`core_entry_api_uses_the_common_acceptance_fixture`](../../crates/rpn-core/tests/core.rs)<br>[`si_t070_execute_all_stops`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| TEST-049 | 論理4種。AND=05、OR=AF、XOR=AA、NOT X=F0。C/G保持 | [`logic_shift_rotate_and_edit_command_effects`](../../crates/rpn-core/tests/core.rs) |
| TEST-050 | 単項シフト回転。SL02/SR40、ASR twos C0/Unsigned40、RL03/RRC40等。各C=1、G保持、L=81 | [`logic_shift_rotate_and_edit_command_effects`](../../crates/rpn-core/tests/core.rs) |
| TEST-051 | 回転0/負数/上限。0はDropし値/Carry保持、-1は1回と同じ。通常9はErr、Carry9成功/10 Err、65成功 | [`absolute_and_rotate_count_boundaries`](../../crates/rpn-core/tests/core.rs)<br>[`bitwise_masks_rotation_and_width_boundaries`](../../crates/rpn-core/tests/core.rs)<br>[`mask_edit_count_and_zero_rotation_keep_flags`](../../crates/rpn-core/tests/core.rs) |
| TEST-052 | マスク/bit。E0/07、該当bitのみ変更、testは不変、count=8 | [`logic_shift_rotate_and_edit_command_effects`](../../crates/rpn-core/tests/core.rs)<br>[`documented_word_and_stack_boundaries`](../../crates/rpn-core/tests/core.rs)<br>[`mask_edit_count_and_zero_rotation_keep_flags`](../../crates/rpn-core/tests/core.rs) |
| TEST-053 | ワンショットと途中基数。stdout=265と改行、stderr空、終了0 | [`batch_outputs_only_final_x`](../../crates/rpn-cli/tests/process.rs) |
| TEST-054 | 終了コード/復帰。0/1/2/3。REPL計算エラー後の正常exitは0 | [`batch_error_codes_and_diagnostics`](../../crates/rpn-cli/tests/process.rs)<br>[`repl_recovers_and_preserves_prior_values`](../../crates/rpn-cli/tests/process.rs)<br>[`invalid_utf8_reports_input_io_failure`](../../crates/rpn-cli/tests/process.rs) |
| TEST-055 | 番号と領域独立。F0..3不変、F4=1、F5保持、6はInvalidFlag | [`arithmetic_flag_update_preserves_user_bits`](../../crates/rpn-core/src/flags.rs) |
| TEST-056 | REPLキーと不完全入力。1eを編集中表示、CLxで復帰、終了0 | [`test_056_stack_query_shows_edit_buffer_on_x_only`](../../crates/rpn-cli/tests/process.rs)<br>[`repl_editing_and_eof`](../../crates/rpn-cli/tests/process.rs) |
| TEST-057 | u64最大乗算。raw1、G=1、Carry保持、中間overflowなし | [`integer_edge_arithmetic_and_flags`](../../crates/rpn-core/tests/core.rs) |
| TEST-058 | 1補数の負のゼロ。MULは既定00、oneszero raw後FF、oneszero canonical後00。SUBは両方式FE、除数FFはDivisionByZero | [`sign_modes_control_decimal_display_and_zero_arithmetic`](../../crates/rpn-cli/tests/process.rs) |
| TEST-059 | SQRT。4294967295/C1、4/C0、DomainError | [`integer_edge_arithmetic_and_flags`](../../crates/rpn-core/tests/core.rs)<br>[`unary_arithmetic_and_float_subnormals`](../../crates/rpn-core/tests/core.rs) |
| TEST-060 | 構文と未対応。未知語:行未実行/終了2。dblmul:5成功後未対応/終了1 | [`syntax_checks_the_entire_line`](../../crates/rpn-cli/src/line/tests.rs)<br>[`syntax_and_runtime_failures_have_different_commit_boundaries`](../../crates/rpn-cli/src/line/tests.rs)<br>[`batch_error_codes_and_diagnostics`](../../crates/rpn-cli/tests/process.rs) |
| TEST-061 | 設定・リセット。不正生成を拒否。resetは生成時設定/全値0/F0..5=0/I_D | [`default_reset_config_and_snapshot_ownership`](../../crates/rpn-core/tests/core.rs)<br>[`si_t072_restore_creation_config`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| TEST-062 | f64エラーと許容値。非有限拒否、overflow拒否、0.75、DomainError | [`nonfinite_float_push_preserves_state`](../../crates/rpn-core/tests/core.rs)<br>[`unary_arithmetic_and_float_subnormals`](../../crates/rpn-core/tests/core.rs)<br>[`floating_arithmetic_preserves_flags_and_chs_history`](../../crates/rpn-core/tests/core.rs) |
| TEST-063 | W1とW64ビット操作。SL0/C1、RL1/C1、mask=u64MAX、index64はInvalidBitIndex | [`logic_shift_rotate_and_edit_command_effects`](../../crates/rpn-core/tests/core.rs)<br>[`bitwise_masks_rotation_and_width_boundaries`](../../crates/rpn-core/tests/core.rs)<br>[`mask_edit_count_and_zero_rotation_keep_flags`](../../crates/rpn-core/tests/core.rs) |
| TEST-064 | 外部スクリプト自動化。同一プロセス内のみ状態継続、別起動は初期状態、stdout/stderrと終了コードを分離。内蔵プログラムAPIなし | [`batch_outputs_only_final_x`](../../crates/rpn-cli/tests/process.rs)<br>[`batch_error_codes_and_diagnostics`](../../crates/rpn-cli/tests/process.rs) |
| TEST-067 | f64の精度・範囲。順に0.30000000000000004、有限のsubnormal、FloatingPointOutOfRange | [`test_067_float_rounding_subnormal_and_overflow_preserve_flags`](../../crates/rpn-core/tests/core.rs) |
| TEST-068 | RMDフラグ保持。X=3、C/Gとも実行前と同じ、L=4、Drop | [`eight_bit_arithmetic_result_flags_and_history`](../../crates/rpn-core/tests/core.rs) |
| TEST-069 | Idle BSP。X=0、L=7、Idle/D。他段/flags保持 | [`idle_backspace_preserves_last_x_at_either_lift`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| TEST-070 | Float編集CHS。X=-5、編集中、L=7、Lift保持。続く2は-52 | [`signed_history_and_exponent_entry`](../../crates/rpn-cli/src/line/tests.rs) |
| TEST-073 | ABS。twos FB→05、80→80/G1、Unsigned FB保持。C保持、L=実行前X | [`absolute_and_rotate_count_boundaries`](../../crates/rpn-core/tests/core.rs) |

## スタック・入力

| ID | 確認内容 | 対応するテスト関数 |
|---|---|---|
| SI-T001 | ENTER | [`si_t001_enter`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T002 | 連続ENTER | [`si_t002_repeated_enter`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T003 | 編集ENTER | [`si_t003_editing_enter`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T004 | ENTER後数字 | [`si_t004_digit_after_enter`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T005 | Eから初回数字 | [`si_t005_start_with_lift`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T006 | Dから初回数字 | [`si_t006_start_without_lift`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T007 | 二項Add | [`si_t007_editing_add`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T008 | 非可換Subtract | [`si_t008_subtract`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T009 | Tの反復複製 | [`si_t009_repeated_drop`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T010 | 演算直後数字 | [`si_t010_digit_after_add`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T011 | 単項ABS | [`si_t011_absolute`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T012 | R↓ | [`si_t012_roll_down`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T013 | R↑ | [`si_t013_roll_up`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T014 | 交換 | [`si_t014_swap_xy`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T015 | 循環往復 | [`si_t015_rotation_cycles`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T016 | CLx編集中 | [`si_t016_clear_edit_then_digit`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T017 | CLx未完成 | [`si_t017_clear_unfinished_exponent`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T018 | LAST X有効Lift | [`si_t018_recall_last_x_enabled`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T019 | LAST X無効Lift | [`si_t019_recall_last_x_twice`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T020 | STO | [`si_t020_store`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T021 | RCL D | [`si_t021_recall_without_lift`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T022 | RCL E | [`si_t022_recall_with_lift`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T023 | Push区別 | [`si_t023_push_then_add`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T024 | Push編集中拒否 | [`si_t024_reject_push_during_edit`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T025 | Integer CHS | [`si_t025_digit_after_integer_chs`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T026 | Integer連続CHS | [`si_t026_repeated_integer_chs`](../../crates/rpn-input/tests/stack_operations.rs) |
| SI-T027 | Float編集CHS | [`si_t027_float_chs_then_digit`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T028 | Float連続CHS | [`si_t028_float_double_chs`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T029 | Float Idle CHS | [`si_t029_idle_float_chs`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T030 | Float負0 | [`si_t030_float_negative_zero`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T031 | 点から開始 | [`si_t031_start_decimal_point`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T032 | 重複点 | [`si_t032_duplicate_point`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T033 | Integer点 | [`si_t033_integer_point`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T034 | 指数と符号 | [`si_t034_exponent_sign`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T035 | 指数のBS | [`si_t035_delete_exponent`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T036 | 最終桁BS | [`si_t036_delete_last_digit`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T037 | Idle BS | [`si_t037_idle_backspace`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T038 | Idle EEX | [`si_t038_idle_exponent`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T039 | 不完全ENTER | [`si_t039_incomplete_enter`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T040 | 不完全操作 | [`si_t040_incomplete_commands`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T041 | 基数切替D | [`si_t041_change_base_disabled`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T042 | 基数切替E | [`si_t042_change_base_enabled`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T043 | 同基数指定 | [`si_t043_same_radix`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T044 | 全基数往復 | [`si_t044_radix_round_trip`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T045 | 初回不正桁 | [`si_t045_invalid_first_digit`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T046 | 編集不正桁 | [`si_t046_invalid_edit_digit`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T047 | HEX小文字 | [`si_t047_lowercase_hex`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T048 | W8最大raw | [`si_t048_maximum_raw`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T049 | W8超過 | [`si_t049_overflowing_digit`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T050 | W1初回幅外 | [`si_t050_one_bit_first_digit`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T051 | 256文字 | [`si_t051_accept_256th_digit`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T052 | 257文字拒否 | [`si_t052_reject_257th_digit`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T053 | 幅縮小／拡大 | [`si_t053_resize_all_words`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T054 | 符号形式変更 | [`si_t054_reinterpret_unsigned`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T055 | Integer→Float | [`si_t055_convert_to_float`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T056 | 変換全体拒否 | [`si_t056_inexact_register_blocks_conversion`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T057 | Float→Integer拒否 | [`si_t057_fraction_blocks_conversion`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T058 | Floatで基数 | [`si_t058_reject_hex_in_float`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T059 | Floatで同基数 | [`si_t059_decimal_in_float`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T060 | 同mode | [`si_t060_same_numeric_mode`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T061 | 除算失敗 | [`si_t061_zero_divisor`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T062 | エラー直後編集 | [`si_t062_resume_after_division_error`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T063 | 解除後新規入力 | [`si_t063_clear_after_division_error`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T064 | DomainError | [`si_t064_negative_square_root`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T065 | 無効レジスタ | [`si_t065_invalid_register`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T066 | ClearStack | [`si_t066_clear_stack`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T067 | ClearRegisters | [`si_t067_clear_registers`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T068 | 整数overflow成功 | [`si_t068_signed_add_overflow`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T069 | RMD保持 | [`si_t069_remainder_keeps_flags`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T070 | execute_all停止 | [`si_t070_execute_all_stops`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T071 | snapshot独立 | [`si_t071_independent_snapshot`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T072 | 初期化reset | [`si_t072_restore_creation_config`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T073 | Float指数編集 | [`si_t073_edit_large_exponent`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T074 | 不正文字集合 | [`si_t074_invalid_ascii_and_unicode_digits`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T075 | Float幅変更拒否 | [`si_t075_reject_word_settings_in_float`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| SI-T076 | 整数CHS最小 | [`si_t076_negate_minimum_integer`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |

## CLI入力

| ID | 確認内容 | 対応するテスト関数 |
|---|---|---|
| IA-L001 | 0x12,0x123,0x1FF,0x100を各独立parse → raw12,23,FF,00 | [`integer_vectors`](../../crates/rpn-cli/src/literal/tests.rs) |
| IA-L002 | Unsigned 0,255,256,-1,-256 → raw00,FF,00,FF,00（数学表示0,255,0,255,0） | [`integer_vectors`](../../crates/rpn-cli/src/literal/tests.rs)<br>[`decimal_literal_display_at_sign_boundaries`](../../crates/rpn-cli/tests/process.rs) |
| IA-L003 | Twos 127,128,-128,-129 → raw7F,80,80,7F、数学表示127,-128,-128,127 | [`integer_vectors`](../../crates/rpn-cli/src/literal/tests.rs)<br>[`decimal_literal_display_at_sign_boundaries`](../../crates/rpn-cli/tests/process.rs) |
| IA-L004 | Ones 127,128,-127,-128 → raw7F,80,80,7F、数学表示127,-127,-127,127 | [`integer_vectors`](../../crates/rpn-cli/src/literal/tests.rs)<br>[`decimal_literal_display_at_sign_boundaries`](../../crates/rpn-cli/tests/process.rs) |
| IA-L005 | -123、各SignedMode → Unsigned85/Twos85/Ones84 | [`integer_vectors`](../../crates/rpn-cli/src/literal/tests.rs) |
| IA-L006 | -0、各SignedModeとFloat → Unsigned00/Twos00/OnesFF/Float負0bit | [`integer_vectors`](../../crates/rpn-cli/src/literal/tests.rs)<br>[`float_grammar_and_bits`](../../crates/rpn-cli/src/literal/tests.rs) |
| IA-L007 | W1、各形式、0,1,2,-1 → 非負raw0,1,0。負1はUnsigned/Twos=1、Ones=0 | [`integer_vectors`](../../crates/rpn-cli/src/literal/tests.rs) |
| IA-L008 | W64 Unsigned 18446744073709551615,18446744073709551616 → FFFFFFFFFFFFFFFF,0、ホスト整数overflowなし | [`long_inputs`](../../crates/rpn-cli/src/literal/tests.rs) |
| IA-L009 | W64、0x＋Fを254桁、続いて255桁の別試行 → 256文字はFFFFFFFFFFFFFFFF、257文字はLiteral(InputTooLong) | [`long_inputs`](../../crates/rpn-cli/src/literal/tests.rs) |
| IA-L010 | 単独-、全SignedMode/Float → 正0。Ones負0やFloat負0にしない。トークン分類もLiteral | [`integer_vectors`](../../crates/rpn-cli/src/literal/tests.rs)<br>[`float_grammar_and_bits`](../../crates/rpn-cli/src/literal/tests.rs)<br>[`reserved_words_and_key_subtraction`](../../crates/rpn-cli/src/line/tests.rs) |
| IA-L011 | -1,-123、DEC → 通常負数。IA-L005/006と同じ表現式、符号単独規則を誤適用しない | [`integer_vectors`](../../crates/rpn-cli/src/literal/tests.rs) |
| IA-L012 | 0xG、0b2、0o8 → Literal(InvalidDigit)、最初の不正文字と接頭辞の基数 | [`invalid_tokens`](../../crates/rpn-cli/src/literal/tests.rs) |
| IA-L013 | 空、+、0x、--1、0d-1、1_0、全角１、内部空白 → parse単体でInvalidNumber（形不正）またはInvalidDigit（数字列不正）。1_0/全角１はInvalidDigit、他はInvalidNumber | [`invalid_tokens`](../../crates/rpn-cli/src/literal/tests.rs) |
| IA-L014 | Float 1e-,NaN,Inf,0x1 → InvalidNumber。Float 1e999はFloatingPointOutOfRange。Core未呼出 | [`float_grammar_and_bits`](../../crates/rpn-cli/src/literal/tests.rs) |
| IA-L015 | 257文字かつ末尾G → InputTooLong優先、桁検査より前 | [`long_inputs`](../../crates/rpn-cli/src/literal/tests.rs) |
| IA-L016 | HEX設定で-0d123、DECで0x123 → raw85,23、表示radixは各元設定を保持 | [`integer_vectors`](../../crates/rpn-cli/src/literal/tests.rs)<br>[`decimal_literal_display_at_sign_boundaries`](../../crates/rpn-cli/tests/process.rs) |
| IA-L017 | submit pending=0x123 → stack=[35,5,2,3],L9,E、flags/R0保持、pending消去 | [`pending_and_normal_execution_use_specification_fixture`](../../crates/rpn-cli/src/line/tests.rs) |
| IA-L018 | submit pending=0、next=Divide → Calculation(DivisionByZero)、Core全体とpending=0保持 | [`pending_and_normal_execution_use_specification_fixture`](../../crates/rpn-cli/src/line/tests.rs) |
| IA-L019 | submit pending=0xG、next=Divide → Literal InvalidDigit、Core呼出なし、全状態/pending保持 | [`pending_and_normal_execution_use_specification_fixture`](../../crates/rpn-cli/src/line/tests.rs) |
| IA-L020 | 通常トークン列5 0 / → [0,5,5,2],L9,E、除算失敗、先行2Push保持。初期状態は本節共通 | [`pending_and_normal_execution_use_specification_fixture`](../../crates/rpn-cli/src/line/tests.rs) |
| IA-L021 | -、sub、keys内-を分類 → Literal正0、Command Subtract、InputEvent Command Subtract。減算エイリアス衝突なし | [`reserved_words_and_key_subtraction`](../../crates/rpn-cli/src/line/tests.rs) |
| IA-L022 | Syntax/Literal/Input/Calculation/Io → 終了2/2/1/1/3、stderr診断、失敗Batchは最終Xをstdoutへ出さない | [`batch_error_codes_and_diagnostics`](../../crates/rpn-cli/tests/process.rs)<br>[`invalid_utf8_reports_input_io_failure`](../../crates/rpn-cli/tests/process.rs) |

## Core API

| ID | 確認内容 | 対応するテスト関数 |
|---|---|---|
| IA-C001 | 幅16で作ったWord raw123をPush → IntegerOutOfRange、全状態不変（自動マスク禁止） | [`core_entry_api_uses_the_common_acceptance_fixture`](../../crates/rpn-core/tests/core.rs) |
| IA-C002 | Push(3)、編集型を持たないCore → [3,5,2,3],L9,E。InputInProgressを返す経路なし | [`core_entry_api_uses_the_common_acceptance_fixture`](../../crates/rpn-core/tests/core.rs) |
| IA-C003 | Enter→Push(3) → [3,5,2,3],L9,E、Enter自身のlift後は上書き | [`core_entry_api_uses_the_common_acceptance_fixture`](../../crates/rpn-core/tests/core.rs) |
| IA-C004 | Start(1)→ReplaceX(12) → [12,5,2,3],L9,E、liftは一度 | [`core_entry_api_uses_the_common_acceptance_fixture`](../../crates/rpn-core/tests/core.rs) |
| IA-C005 | Enter→Start(1)→ReplaceX(12) → [12,5,2,3],L9,D、編集ではLiftを有効化しない | [`core_entry_api_uses_the_common_acceptance_fixture`](../../crates/rpn-core/tests/core.rs) |
| IA-C006 | IntegerでPush(Float(NaN)) → FloatingPointOutOfRangeがNumericModeMismatchに優先、不変 | [`core_entry_api_uses_the_common_acceptance_fixture`](../../crates/rpn-core/tests/core.rs) |
| IA-C007 | IntegerでPush(Float(1.0)) → NumericModeMismatch、不変 | [`core_entry_api_uses_the_common_acceptance_fixture`](../../crates/rpn-core/tests/core.rs) |
| IA-C008 | 内部fixtureのflags上位bit破損＋X0でDivide → InvariantViolation優先。演算せず破損を含め状態不変 | [`invalid_flags_precede_zero_divisor`](../../crates/rpn-core/src/calculator/tests.rs) |
| IA-C009 | FloatでEditBit index255 → InvalidBitIndex優先、mode不一致は次順位、無効bitシフトしない | [`errors_have_priority_and_single_commands_are_atomic`](../../crates/rpn-core/tests/core.rs) |
| IA-C010 | W64 Unsigned R0=9007199254740993→Float → IntegerOutOfRange、全領域一括取消、SI-T056と対応 | [`si_t056_inexact_register_blocks_conversion`](../../crates/rpn-input/tests/stack_input_boundaries.rs)<br>[`transitions_are_exact_and_include_stack_history_and_registers`](../../crates/rpn-core/tests/core.rs) |
| IA-C011 | X0,Y5でDivide/Remainder各独立 → DivisionByZero、stack/L/flags/設定/Ri不変 | [`core_entry_api_uses_the_common_acceptance_fixture`](../../crates/rpn-core/tests/core.rs) |
| IA-C012 | snapshotコピーのX/R0/flags変更 → 実Core不変、編集fieldなし | [`core_entry_api_uses_the_common_acceptance_fixture`](../../crates/rpn-core/tests/core.rs) |
| IA-C013 | execute_all Push5,Push0,Divide,Push7 → [0,5,5,2],L9,E、先行成功維持、Push7なし | [`core_entry_api_uses_the_common_acceptance_fixture`](../../crates/rpn-core/tests/core.rs) |
| IA-C014 | 順位選択関数に同時候補を与える内部単体試験 → IA §4の全順位・同列順を総当たり。対象順X/Y/Z/T/L/Riも固定 | [`ia_c014_uses_specification_order_and_keeps_first_target_on_ties`](../../crates/rpn-core/src/error/tests.rs)<br>[`conversion_visits_stack_history_then_registers_in_order`](../../crates/rpn-core/src/state.rs) |
| IA-C015 | 不正mode＋値依存条件のある命令 → mode適合前に演算しない。FloatのAndはNumericModeMismatch、結果範囲を評価しない | [`errors_have_priority_and_single_commands_are_atomic`](../../crates/rpn-core/tests/core.rs) |
| IA-C016 | 候補検証不整合を内部fault injection → InvariantViolation、実state不変。公開APIに注入機能を作らない | [`ia_c016_invalid_candidate_leaves_original_state_unchanged`](../../crates/rpn-core/src/calculator/tests.rs) |

## キー入力と状態保持

| ID | 確認内容 | 対応するテスト関数 |
|---|---|---|
| IA-G001 | 共通状態からDigit1,Digit2 → [12,5,2,3],L9,E、Editing("12")、part=Mantissa | [`common_fixture_digit_edits_preserve_unrelated_state`](../../crates/rpn-cli/src/line/tests.rs) |
| IA-G002 | Float共通状態、Digit1,Digit2,CHS,BS → [-1,5,2,3],L9,E、Editing("-1")、flags保持 | [`common_fixture_digit_edits_preserve_unrelated_state`](../../crates/rpn-cli/src/line/tests.rs) |
| IA-G003 | W8 HEX Editing("FF")→Digit F → Input経由Calculation(IntegerOutOfRange)、全セッション保持。CLIならFFF→FFを受理 | [`strict_keys_and_failed_literal_leave_edit_intact`](../../crates/rpn-cli/tests/literal_entry.rs) |
| IA-G004 | Float Editing("1e")、SetRadix HEX → Calculation(NumericModeMismatch)、全状態不変。旧SI-H004を確定移管 | [`editing_rejections_preserve_the_common_fixture`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| IA-G005 | Float Editing("1e")、Divide → Input(IncompleteEntry)、previewでゼロ判定しない、全保持 | [`editing_rejections_preserve_the_common_fixture`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| IA-G006 | Editing("12")、Push3 → Input(InputInProgress)、全保持。直接CoreのIA-C002と対比 | [`editing_rejections_preserve_the_common_fixture`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| IA-G007 | Float Editing("1.2")、点 → Input(InvalidNumber)、全保持。SI-T032のvariant確定 | [`si_t032_duplicate_point`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| IA-G008 | Editing("0")で除算失敗→Digit2 → SI-T062と同じEditing("02")、失敗後も編集継続 | [`si_t062_resume_after_division_error`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| IA-G009 | Enter→Digit3 → SI-T004の値、Disabled保持。CLI完成PushのIA-C003とLiftを区別 | [`si_t004_digit_after_enter`](../../crates/rpn-input/tests/stack_operations.rs) |
| IA-G010 | InputSnapshotのbuffer/part/Core snapshotを変更 → 実セッション全状態不変。SI-T071を拡張 | [`si_t071_independent_snapshot`](../../crates/rpn-input/tests/stack_input_boundaries.rs) |
| IA-G011 | HP Editing中にCLI pending入力への切替要求 → InputInProgress、Core・HP編集・pending不変、二重編集禁止 | [`editing_and_pending_cannot_commit_together`](../../crates/rpn-cli/src/line/tests.rs) |
| IA-G012 | Float Editing("-1")でBS → Editing("0")、X=+0.0、Mantissa、Lift保持。次の2でbuffer="02" | [`deleting_last_negative_digit_continues_with_positive_zero`](../../crates/rpn-input/tests/signed_zero.rs) |

## HP実機比較と旧ID

| ID | 扱い |
|---|---|
| TEST-065 / TEST-066 | 実機特殊状態およびHandbookのDIV/RMD比較。製品の演算結果・フラグはTEST-045/068で確認する。 |
| TEST-071 / SI-H006 | HP式WSIZEの記憶領域再配置は製品対象外。製品の個別レジスタ幅変換はSI-T053で確認する。 |
| TEST-072 / SI-H005 | HP FLOATのスタック変換は製品対象外。製品の数値モード変換はTEST-042、SI-T055～057で確認する。 |
| SI-H003 / SI-H004 | 製品判断をそれぞれIA-G012 / IA-G004へ移管済み。製品ID側に各1回計上した。 |
| SI-H001 / SI-H002 / SI-H007 / SI-H008 | HP実機の入力上限・符号・エラー解除動作を比較する参考ケース。実機未確認項目は製品合否の集計外。 |
